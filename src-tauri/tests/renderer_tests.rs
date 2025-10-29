use std::env;

use blake3::Hasher;
use torwell84::renderer::{RenderFrameDescriptor, RendererService};

fn setup_cache_env() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    env::set_var("TORWELL_SHADER_CACHE_DIR", dir.path());
    dir
}

#[test]
fn renderer_records_frame_metrics() {
    let _guard = setup_cache_env();
    let service = RendererService::new();
    if !service.is_available() {
        eprintln!("renderer unavailable, skipping test");
        return;
    }

    let mut tickets = Vec::new();
    for _ in 0..6 {
        let ticket = service
            .submit_frame(RenderFrameDescriptor::default())
            .expect("submit frame");
        tickets.push(ticket);
    }

    let metrics = service.flush_blocking().expect("flush");
    assert!(!metrics.is_empty());

    for ticket in tickets {
        let frame = ticket.blocking_wait().expect("ticket resolved");
        assert!(frame.triple_buffer_depth <= 3);
    }

    let snapshot = service.metrics_snapshot();
    assert!(snapshot.summary.frames_total >= metrics.len() as u64);
    service.shutdown();
}

#[test]
fn renderer_capture_matches_reference() {
    let _guard = setup_cache_env();
    let service = RendererService::new();
    if !service.is_available() {
        eprintln!("renderer unavailable, skipping test");
        return;
    }

    let ticket = service
        .capture_frame(RenderFrameDescriptor::default())
        .expect("capture submission");
    let metrics = service.flush_blocking().expect("flush capture");
    assert!(!metrics.is_empty());
    let capture = ticket.blocking_wait().expect("capture result");

    let reference = reference_frame(capture.width, capture.height);
    let mut hasher = Hasher::new();
    hasher.update(&reference);
    let expected = hasher.finalize();
    let mut actual_hasher = Hasher::new();
    actual_hasher.update(&capture.data);
    let actual = actual_hasher.finalize();
    assert_eq!(expected, actual, "captured frame hash mismatch");

    service.shutdown();
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
