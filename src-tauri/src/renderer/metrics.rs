use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Percentiles {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FrameMetricsSummary {
    pub frames_total: u64,
    pub dropped_frames_total: u64,
    pub cpu_encode_ns: Percentiles,
    pub gpu_time_ns: Percentiles,
    pub queue_wait_ns: Percentiles,
    pub frame_interval_ns: Percentiles,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameMetrics {
    pub frame_id: u64,
    pub cpu_encode_ns: u64,
    pub queue_submit_ns: u64,
    pub gpu_time_ns: u64,
    pub queue_wait_ns: u64,
    pub frame_interval_ns: u64,
    pub triple_buffer_depth: u8,
    pub dropped_frames: u64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameMetricsSnapshot {
    pub available: bool,
    pub history: Vec<FrameMetrics>,
    pub summary: FrameMetricsSummary,
}

pub struct RendererMetricsState {
    history: Mutex<VecDeque<FrameMetrics>>,
    history_limit: usize,
    total_frames: AtomicU64,
    total_dropped: AtomicU64,
    emitter: Mutex<Option<AppHandle>>,
}

impl RendererMetricsState {
    pub fn new(history_limit: usize) -> Self {
        Self {
            history: Mutex::new(VecDeque::with_capacity(history_limit)),
            history_limit,
            total_frames: AtomicU64::new(0),
            total_dropped: AtomicU64::new(0),
            emitter: Mutex::new(None),
        }
    }

    fn history_guard(&self) -> MutexGuard<'_, VecDeque<FrameMetrics>> {
        self.history.lock().expect("metrics history poisoned")
    }

    pub fn set_handle(&self, handle: AppHandle) {
        let mut guard = self.emitter.lock().expect("metrics emitter poisoned");
        *guard = Some(handle);
    }

    pub fn record(&self, metrics: FrameMetrics) {
        {
            let mut guard = self.history_guard();
            if guard.len() == self.history_limit {
                guard.pop_front();
            }
            guard.push_back(metrics.clone());
        }
        self.total_frames.fetch_add(1, Ordering::Relaxed);
        self.total_dropped
            .fetch_add(metrics.dropped_frames, Ordering::Relaxed);
        let maybe_handle = {
            let guard = self.emitter.lock().expect("metrics emitter poisoned");
            guard.clone()
        };
        if let Some(handle) = maybe_handle {
            let _ = handle.emit_all("frame-metrics", &metrics);
        }
    }

    pub fn snapshot(&self, available: bool) -> FrameMetricsSnapshot {
        let history = {
            let guard = self.history_guard();
            guard.iter().cloned().collect::<Vec<_>>()
        };
        let summary = self.build_summary(&history);
        FrameMetricsSnapshot {
            available,
            history,
            summary,
        }
    }

    fn build_summary(&self, history: &[FrameMetrics]) -> FrameMetricsSummary {
        let mut summary = FrameMetricsSummary::default();
        summary.frames_total = self.total_frames.load(Ordering::Relaxed);
        summary.dropped_frames_total = self.total_dropped.load(Ordering::Relaxed);
        summary.cpu_encode_ns = Self::calculate_percentiles(history, |m| m.cpu_encode_ns);
        summary.gpu_time_ns = Self::calculate_percentiles(history, |m| m.gpu_time_ns);
        summary.queue_wait_ns = Self::calculate_percentiles(history, |m| m.queue_wait_ns);
        summary.frame_interval_ns = Self::calculate_percentiles(history, |m| m.frame_interval_ns);
        summary
    }

    fn calculate_percentiles<F>(history: &[FrameMetrics], accessor: F) -> Percentiles
    where
        F: Fn(&FrameMetrics) -> u64,
    {
        if history.is_empty() {
            return Percentiles::default();
        }
        let mut values: Vec<u64> = history.iter().map(accessor).collect();
        let latest = values.last().cloned();
        values.sort_unstable();
        Percentiles {
            p50: Self::percentile(&values, 0.50),
            p95: Self::percentile(&values, 0.95),
            p99: Self::percentile(&values, 0.99),
            latest,
        }
    }

    fn percentile(values: &[u64], percentile: f64) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let idx = ((values.len() - 1) as f64 * percentile).round() as usize;
        values[idx.min(values.len() - 1)]
    }

    pub fn clear_handle(&self) {
        let mut guard = self.emitter.lock().expect("metrics emitter poisoned");
        *guard = None;
    }
}
