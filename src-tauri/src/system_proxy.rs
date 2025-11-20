use std::process::Command;
use log::{info, error};

#[cfg(target_os = "linux")]
pub fn enable_global_proxy(port: u16) -> Result<(), String> {
    info!("Enabling global system proxy on port {}", port);

    let port_str = port.to_string();
    // GNOME / Unity settings
    let cmds = vec![
        vec!["gsettings", "set", "org.gnome.system.proxy", "mode", "manual"],
        vec!["gsettings", "set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"],
        vec!["gsettings", "set", "org.gnome.system.proxy.socks", "port", &port_str],
    ];

    for cmd in cmds {
        // We use the port string reference which is tricky in a loop of vecs
        // Let's construct the args cleanly
        let status = Command::new(cmd[0])
            .args(&cmd[1..])
            .status()
            .map_err(|e| format!("Failed to execute gsettings: {}", e))?;

        if !status.success() {
            error!("gsettings command failed: {:?}", cmd);
            // We don't abort hard here because not all desktops use gsettings,
            // but we log it.
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn disable_global_proxy() -> Result<(), String> {
    info!("Disabling global system proxy");

    let status = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "none"])
        .status()
        .map_err(|e| format!("Failed to execute gsettings: {}", e))?;

    if !status.success() {
        return Err("Failed to reset proxy mode to none".into());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn enable_global_proxy(port: u16) -> Result<(), String> {
    info!("Enabling global system proxy on Windows port {}", port);
    // Using registry modification via 'reg' command or powershell is common.
    // For simplicity/safety, we might use a crate in future, but here is a basic Powershell approach.

    let script = format!(
        "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings' -Name ProxyServer -Value 'socks=127.0.0.1:{}'; \
         Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings' -Name ProxyEnable -Value 1",
        port
    );

    let status = Command::new("powershell")
        .args(&["-Command", &script])
        .status()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;

    if !status.success() {
        return Err("Failed to set Windows proxy registry keys".into());
    }

    // Force refresh of system settings using WinInet via PowerShell P/Invoke
    let refresh_script = r#"
$signature = @'
[DllImport("wininet.dll", SetLastError = true, CharSet=CharSet.Auto)]
public static extern bool InternetSetOption(IntPtr hInternet, int dwOption, IntPtr lpBuffer, int dwBufferLength);
'@
$interop = Add-Type -MemberDefinition $signature -Name "WinInet" -Namespace Win32 -PassThru
$interop::InternetSetOption([IntPtr]::Zero, 39, [IntPtr]::Zero, 0) # INTERNET_OPTION_SETTINGS_CHANGED
$interop::InternetSetOption([IntPtr]::Zero, 37, [IntPtr]::Zero, 0) # INTERNET_OPTION_REFRESH
"#;

    let refresh_status = Command::new("powershell")
        .args(&["-Command", refresh_script])
        .status();

    if let Err(e) = refresh_status {
        error!("Failed to refresh Windows proxy settings: {}", e);
    } else {
        info!("Windows proxy settings refreshed successfully.");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn disable_global_proxy() -> Result<(), String> {
    info!("Disabling global system proxy on Windows");

    let script = "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings' -Name ProxyEnable -Value 0";

    let status = Command::new("powershell")
        .args(&["-Command", script])
        .status()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;

    if !status.success() {
        return Err("Failed to disable Windows proxy".into());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn enable_global_proxy(port: u16) -> Result<(), String> {
    info!("Enabling global system proxy on macOS port {}", port);

    let services = get_macos_network_services()?;
    let port_str = port.to_string();

    for service in services {
        // Set SOCKS proxy
        let _ = Command::new("networksetup")
            .args(&["-setsocksfirewallproxy", &service, "127.0.0.1", &port_str])
            .status();
        // Enable it
        let _ = Command::new("networksetup")
            .args(&["-setsocksfirewallproxystate", &service, "on"])
            .status();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable_global_proxy() -> Result<(), String> {
    info!("Disabling global system proxy on macOS");

    if let Ok(services) = get_macos_network_services() {
        for service in services {
            let _ = Command::new("networksetup")
                .args(&["-setsocksfirewallproxystate", &service, "off"])
                .status();
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn get_macos_network_services() -> Result<Vec<String>, String> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .map_err(|e| format!("Failed to list network services: {}", e))?;

    if !output.status.success() {
        return Err("Failed to list network services".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        if line.contains("An asterisk") { continue; }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            services.push(trimmed.to_string());
        }
    }
    Ok(services)
}

#[cfg(target_os = "android")]
pub fn enable_global_proxy(_port: u16) -> Result<(), String> {
    info!("Android system proxy must be handled by the Kotlin VpnService frontend");
    Ok(())
}

#[cfg(target_os = "android")]
pub fn disable_global_proxy() -> Result<(), String> {
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos", target_os = "android")))]
pub fn enable_global_proxy(_port: u16) -> Result<(), String> {
    info!("Global proxy not supported on this OS");
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos", target_os = "android")))]
pub fn disable_global_proxy() -> Result<(), String> {
    Ok(())
}
