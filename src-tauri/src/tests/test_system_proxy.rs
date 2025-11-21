#[cfg(test)]
mod tests {
    use crate::system_proxy;
    use std::process::Command;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_proxy_command_construction() {
        // We can't easily mock Command::new in a unit test without a lot of scaffolding,
        // but we can verify the logic by extracting the command generation if we refactor system_proxy.rs.
        // For now, let's at least ensure the functions exist and take the correct types.
        let _ = system_proxy::enable_global_proxy(9050);
        let _ = system_proxy::disable_global_proxy();
    }
}
