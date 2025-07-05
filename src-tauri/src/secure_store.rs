use crate::error::{Error, Result};
use std::process::{Command, Stdio};
use std::io::Write;

enum Backend {
    Keyring,
    Tpm,
}

fn backend() -> Backend {
    if let Ok(v) = std::env::var("TORWELL_KEY_BACKEND") {
        if v.eq_ignore_ascii_case("tpm") || v.eq_ignore_ascii_case("hsm") {
            return Backend::Tpm;
        }
    }
    Backend::Keyring
}

pub fn get_key() -> Result<Option<String>> {
    match backend() {
        Backend::Keyring => {
            let entry = keyring::Entry::new("torwell84", "aes-key")
                .map_err(|e| Error::Io(e.to_string()))?;
            match entry.get_password() {
                Ok(v) => Ok(Some(v)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(Error::Io(e.to_string())),
            }
        }
        Backend::Tpm => read_tpm(),
    }
}

pub fn set_key(value: &str) -> Result<()> {
    match backend() {
        Backend::Keyring => {
            let entry = keyring::Entry::new("torwell84", "aes-key")
                .map_err(|e| Error::Io(e.to_string()))?;
            entry.set_password(value).map_err(|e| Error::Io(e.to_string()))
        }
        Backend::Tpm => write_tpm(value),
    }
}

fn tpm_index() -> Result<String> {
    std::env::var("TORWELL_TPM_NV_INDEX")
        .map_err(|_| Error::Io("TORWELL_TPM_NV_INDEX not set".into()))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_tpm() -> Result<Option<String>> {
    let index = tpm_index()?;
    let output = Command::new("tpm2_nvread")
        .arg(&index)
        .output()
        .map_err(|e| Error::Io(e.to_string()))?;
    if !output.status.success() {
        return Err(Error::Io(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_string()))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_tpm(value: &str) -> Result<()> {
    let index = tpm_index()?;
    let mut child = Command::new("tpm2_nvwrite")
        .arg(&index)
        .arg("-i")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Io(e.to_string()))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(value.as_bytes())
            .map_err(|e| Error::Io(e.to_string()))?;
    }
    let output = child.wait_with_output().map_err(|e| Error::Io(e.to_string()))?;
    if !output.status.success() {
        return Err(Error::Io(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn read_tpm() -> Result<Option<String>> {
    Err(Error::Io("TPM backend not implemented on Windows".into()))
}

#[cfg(target_os = "windows")]
fn write_tpm(_value: &str) -> Result<()> {
    Err(Error::Io("TPM backend not implemented on Windows".into()))
}
