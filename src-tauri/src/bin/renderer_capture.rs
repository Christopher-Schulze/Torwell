use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use blake3::Hasher;
use png::Encoder;
use torwell84::renderer::{RenderFrameDescriptor, RendererService};

fn main() -> Result<()> {
    let mut output: Option<PathBuf> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--output requires a path"))?;
                output = Some(PathBuf::from(value));
            }
            "--width" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--width requires a value"))?;
                width = Some(value.parse::<u32>()?);
            }
            "--height" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--height requires a value"))?;
                height = Some(value.parse::<u32>()?);
            }
            unknown => return Err(anyhow!("unknown argument: {unknown}")),
        }
    }

    let service = RendererService::new();
    let desired_dims = match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        (Some(w), None) => Some((w, w)),
        (None, Some(h)) => Some((h, h)),
        _ => None,
    };

    if !service.wait_for_availability(Duration::from_secs(2)) {
        println!("SKIP: renderer unavailable");
        service.shutdown();
        return Ok(());
    }

    if let Some((w, h)) = desired_dims {
        service.resize(w, h)?;
    }

    let capture = service
        .capture_frame(RenderFrameDescriptor::default())?
        .blocking_wait()?;

    let reference = reference_frame(capture.width, capture.height);
    let mut expected_hasher = Hasher::new();
    expected_hasher.update(&reference);
    let expected_hash = expected_hasher.finalize();

    let mut actual_hasher = Hasher::new();
    actual_hasher.update(&capture.data);
    let actual_hash = actual_hasher.finalize();

    if actual_hash != expected_hash {
        service.shutdown();
        return Err(anyhow!(
            "frame hash mismatch: expected {}, got {}",
            expected_hash.to_hex(),
            actual_hash.to_hex()
        ));
    }

    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_png(&path, capture.width, capture.height, &capture.data)?;
    }

    println!("frame_hash={}", actual_hash.to_hex());
    println!(
        "metrics cpu_ns={} gpu_ns={} queue_wait_ns={} frame_interval_ns={}",
        capture.metrics.cpu_encode_ns,
        capture.metrics.gpu_time_ns,
        capture.metrics.queue_wait_ns,
        capture.metrics.frame_interval_ns
    );

    service.shutdown();
    Ok(())
}

fn reference_frame(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        let v = if height > 1 {
            y as f32 / (height - 1) as f32
        } else {
            0.0
        };
        for x in 0..width {
            let u = if width > 1 {
                x as f32 / (width - 1) as f32
            } else {
                0.0
            };
            let r = (u.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = ((1.0 - 0.5 * u).clamp(0.0, 1.0) * 255.0).round() as u8;
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }
    data
}

fn write_png(path: &PathBuf, width: u32, height: u32, data: &[u8]) -> Result<()> {
    let file = File::create(path).with_context(|| format!("failed to create {:?}", path))?;
    let mut encoder = Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    Ok(())
}
