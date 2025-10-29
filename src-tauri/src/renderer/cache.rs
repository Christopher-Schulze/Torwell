use crate::error::Result;
use blake3::Hasher;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

#[derive(Clone, Copy, Debug)]
pub struct ShaderSource {
    pub name: &'static str,
    pub source: &'static str,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct CacheEntry {
    hash: String,
    file: String,
    last_warmup: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct CacheIndex {
    entries: HashMap<String, CacheEntry>,
}

pub struct ShaderCache {
    cache_dir: PathBuf,
    index_path: PathBuf,
    index: CacheIndex,
}

impl ShaderCache {
    pub fn new() -> Result<Self> {
        let dir = std::env::var("TORWELL_SHADER_CACHE_DIR")
            .map(PathBuf::from)
            .or_else(|_| -> Result<PathBuf> {
                if let Some(proj) = ProjectDirs::from("", "", "Torwell") {
                    Ok(proj.data_dir().join("shader_cache"))
                } else {
                    Ok(PathBuf::from("./shader_cache"))
                }
            })?;
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let index_path = dir.join("index.json");
        let index = if index_path.exists() {
            fs::read_to_string(&index_path)
                .ok()
                .and_then(|content| serde_json::from_str::<CacheIndex>(&content).ok())
                .unwrap_or_default()
        } else {
            CacheIndex::default()
        };
        Ok(Self {
            cache_dir: dir,
            index_path,
            index,
        })
    }

    fn persist_index(&self) -> Result<()> {
        let tmp_path = self.index_path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(&self.index)?;
        fs::write(&tmp_path, data)?;
        fs::rename(tmp_path, &self.index_path)?;
        Ok(())
    }

    fn cache_entry_path(&self, name: &str, hash: &str) -> PathBuf {
        self.cache_dir
            .join(format!("{}-{}.wgsl", name, &hash[..16]))
    }

    pub fn warm_up(
        &mut self,
        device: &wgpu::Device,
        sources: &[ShaderSource],
    ) -> Result<Vec<wgpu::ShaderModule>> {
        let mut modules = Vec::with_capacity(sources.len());
        for shader in sources {
            let mut hasher = Hasher::new();
            hasher.update(shader.source.as_bytes());
            let hash = hasher.finalize().to_hex().to_string();
            let entry_path = self.cache_entry_path(shader.name, &hash);
            let needs_update = match self.index.entries.get(shader.name) {
                Some(entry) if entry.hash == hash && Path::new(&entry.file).exists() => false,
                _ => true,
            };
            if needs_update {
                fs::write(&entry_path, shader.source.as_bytes())?;
                self.index.entries.insert(
                    shader.name.to_string(),
                    CacheEntry {
                        hash: hash.clone(),
                        file: entry_path.to_string_lossy().into_owned(),
                        last_warmup: Utc::now().timestamp(),
                    },
                );
            }
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("shader::{}", shader.name)),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader.source)),
            });
            modules.push(module);
        }
        self.persist_index()?;
        Ok(modules)
    }
}
