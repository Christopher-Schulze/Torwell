#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(header) = std::str::from_utf8(data) {
        let maybe_header = if header.is_empty() { None } else { Some(header) };
        let _ = torwell84::fuzz_parse_max_age(header);
        torwell84::fuzz_tls_version(maybe_header);
    }
});
