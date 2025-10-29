use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use hdrhistogram::Histogram;
use once_cell::sync::OnceCell;
use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

const PARK_TIMEOUT: Duration = Duration::from_micros(500);
const METRIC_BATCH: usize = 32;

/// Central task scheduler backed by a bounded work-stealing pool.
#[derive(Clone)]
pub struct TaskScheduler {
    inner: Arc<TaskSchedulerInner>,
}

struct TaskSchedulerInner {
    injector: Arc<Injector<ScheduledTask>>,
    stealers: Arc<Vec<Stealer<ScheduledTask>>>,
    sleeper: Arc<Sleeper>,
    shutdown: AtomicBool,
    queue_depth: AtomicUsize,
    metrics: Arc<SchedulerMetrics>,
    handles: Mutex<Vec<thread::JoinHandle<()>>>,
}

struct ScheduledTask {
    run: Box<dyn FnOnce() -> Option<Duration> + Send + 'static>,
}

impl ScheduledTask {
    fn run(self) -> Option<Duration> {
        (self.run)()
    }
}

struct Sleeper {
    lock: Mutex<()>,
    cond: Condvar,
}

impl Sleeper {
    fn new() -> Self {
        Self {
            lock: Mutex::new(()),
            cond: Condvar::new(),
        }
    }

    fn notify_one(&self) {
        self.cond.notify_one();
    }

    fn notify_all(&self) {
        self.cond.notify_all();
    }

    fn wait_timeout(&self, duration: Duration) {
        let guard = self.lock.lock().unwrap();
        let _ = self.cond.wait_timeout(guard, duration).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerSnapshot {
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub queue_depth: u64,
    pub total_tasks: u64,
}

#[derive(Debug, Clone)]
pub enum TaskError {
    Canceled,
    Panicked { task: &'static str, message: String },
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::Canceled => write!(f, "task canceled before completion"),
            TaskError::Panicked { task, message } => {
                write!(f, "task '{task}' panicked: {message}")
            }
        }
    }
}

impl std::error::Error for TaskError {}

pub struct TaskHandle<R> {
    rx: oneshot::Receiver<Result<R, TaskError>>,
}

impl<R> Future for TaskHandle<R> {
    type Output = Result<R, TaskError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut rx = unsafe { self.map_unchecked_mut(|s| &mut s.rx) };
        match rx.as_mut().poll(cx) {
            Poll::Ready(Ok(res)) => Poll::Ready(res),
            Poll::Ready(Err(_)) => Poll::Ready(Err(TaskError::Canceled)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<R> TaskHandle<R> {
    pub async fn wait(self) -> Result<R, TaskError> {
        self.await
    }
}

impl TaskScheduler {
    pub fn with_workers(worker_count: usize) -> Self {
        let threads = worker_count.max(1);
        let injector = Arc::new(Injector::new());
        let sleeper = Arc::new(Sleeper::new());
        let metrics = Arc::new(SchedulerMetrics::new());
        let mut workers = Vec::with_capacity(threads);
        let mut stealers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }
        let stealers = Arc::new(stealers);
        let inner = Arc::new(TaskSchedulerInner {
            injector: injector.clone(),
            stealers: stealers.clone(),
            sleeper: sleeper.clone(),
            shutdown: AtomicBool::new(false),
            queue_depth: AtomicUsize::new(0),
            metrics,
            handles: Mutex::new(Vec::with_capacity(threads)),
        });

        for (idx, worker) in workers.into_iter().enumerate() {
            let inner_clone = inner.clone();
            let injector = injector.clone();
            let stealers = stealers.clone();
            let sleeper = sleeper.clone();
            let handle = thread::Builder::new()
                .name(format!("torwell-scheduler-{idx}"))
                .spawn(move || worker_loop(inner_clone, injector, stealers, sleeper, worker, idx))
                .expect("failed to spawn scheduler worker");
            inner.handles.lock().unwrap().push(handle);
        }

        Self { inner }
    }

    pub fn spawn<F, R>(&self, task_name: &'static str, f: F) -> TaskHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if self.inner.shutdown.load(Ordering::Acquire) {
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(TaskError::Canceled));
            return TaskHandle { rx };
        }

        let (tx, rx) = oneshot::channel();
        let scheduled = ScheduledTask {
            run: Box::new(move || {
                let start = Instant::now();
                let result = std::panic::catch_unwind(AssertUnwindSafe(f));
                let elapsed = start.elapsed();
                match result {
                    Ok(value) => {
                        let _ = tx.send(Ok(value));
                        Some(elapsed)
                    }
                    Err(panic) => {
                        let message = panic_message(panic);
                        log::error!("scheduler task {task_name} panicked: {message}");
                        let _ = tx.send(Err(TaskError::Panicked {
                            task: task_name,
                            message: message.clone(),
                        }));
                        Some(elapsed)
                    }
                }
            }),
        };

        self.inner.queue_depth.fetch_add(1, Ordering::AcqRel);
        self.inner.injector.push(scheduled);
        self.inner.sleeper.notify_one();
        TaskHandle { rx }
    }

    pub fn snapshot(&self) -> SchedulerSnapshot {
        let mut snapshot = self.inner.metrics.snapshot();
        snapshot.queue_depth = self.inner.queue_depth.load(Ordering::Acquire) as u64;
        snapshot
    }

    pub fn shutdown(&self) {
        if self.inner.shutdown.swap(true, Ordering::SeqCst) {
            return;
        }
        self.inner.sleeper.notify_all();
        let mut handles = self.inner.handles.lock().unwrap();
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }

    pub fn global() -> TaskScheduler {
        static GLOBAL: OnceCell<TaskScheduler> = OnceCell::new();
        GLOBAL
            .get_or_init(|| TaskScheduler::with_workers(num_cpus::get().max(2)))
            .clone()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::global()
    }
}

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            self.shutdown();
        }
    }
}

fn worker_loop(
    inner: Arc<TaskSchedulerInner>,
    injector: Arc<Injector<ScheduledTask>>,
    stealers: Arc<Vec<Stealer<ScheduledTask>>>,
    sleeper: Arc<Sleeper>,
    mut worker: Worker<ScheduledTask>,
    index: usize,
) {
    let mut batch = Vec::with_capacity(METRIC_BATCH);
    loop {
        if let Some(task) = fetch_task(&injector, &stealers, &mut worker, index) {
            inner.queue_depth.fetch_sub(1, Ordering::AcqRel);
            if let Some(duration) = task.run() {
                batch.push(duration);
                if batch.len() >= METRIC_BATCH {
                    inner.metrics.record_batch(&batch);
                    batch.clear();
                }
            }
            continue;
        }

        if inner.shutdown.load(Ordering::Acquire) {
            break;
        }

        sleeper.wait_timeout(PARK_TIMEOUT);
    }

    while let Some(task) = worker.pop() {
        inner.queue_depth.fetch_sub(1, Ordering::AcqRel);
        if let Some(duration) = task.run() {
            batch.push(duration);
        }
    }

    if !batch.is_empty() {
        inner.metrics.record_batch(&batch);
    }
}

fn fetch_task(
    injector: &Arc<Injector<ScheduledTask>>,
    stealers: &Arc<Vec<Stealer<ScheduledTask>>>,
    worker: &mut Worker<ScheduledTask>,
    index: usize,
) -> Option<ScheduledTask> {
    if let Some(task) = worker.pop() {
        return Some(task);
    }

    if let Some(task) = injector.steal_batch_and_pop(worker) {
        return Some(task);
    }

    for (i, stealer) in stealers.iter().enumerate() {
        if i == index {
            continue;
        }
        loop {
            match stealer.steal() {
                Steal::Success(task) => return Some(task),
                Steal::Retry => {
                    thread::yield_now();
                    continue;
                }
                Steal::Empty => break,
            }
        }
    }

    None
}

fn panic_message(panic: Box<dyn Any + Send>) -> String {
    if let Some(msg) = panic.downcast_ref::<&str>() {
        (*msg).to_string()
    } else if let Some(msg) = panic.downcast_ref::<String>() {
        msg.clone()
    } else {
        "unknown panic".into()
    }
}

#[cfg(loom)]
type MetricsMutex<T> = loom::sync::Mutex<T>;

#[cfg(loom)]
fn lock_metrics<'a, T>(mutex: &'a MetricsMutex<T>) -> loom::sync::MutexGuard<'a, T> {
    mutex.lock()
}

#[cfg(not(loom))]
type MetricsMutex<T> = Mutex<T>;

#[cfg(not(loom))]
fn lock_metrics<'a, T>(mutex: &'a MetricsMutex<T>) -> std::sync::MutexGuard<'a, T> {
    mutex.lock().unwrap()
}

struct SchedulerMetrics {
    histogram: MetricsMutex<Histogram<u64>>,
    total_tasks: AtomicU64,
}

impl SchedulerMetrics {
    fn new() -> Self {
        Self {
            histogram: MetricsMutex::new(
                Histogram::new_with_bounds(1, 60_000_000, 3)
                    .expect("failed to initialise scheduler histogram"),
            ),
            total_tasks: AtomicU64::new(0),
        }
    }

    fn record_batch(&self, samples: &[Duration]) {
        if samples.is_empty() {
            return;
        }
        let mut hist = lock_metrics(&self.histogram);
        for sample in samples {
            let micros = sample.as_micros().max(1) as u64;
            let _ = hist.record(micros);
        }
        self.total_tasks
            .fetch_add(samples.len() as u64, Ordering::Relaxed);
    }

    fn snapshot(&self) -> SchedulerSnapshot {
        let total = self.total_tasks.load(Ordering::Relaxed);
        if total == 0 {
            return SchedulerSnapshot {
                p50_us: 0,
                p95_us: 0,
                p99_us: 0,
                queue_depth: 0,
                total_tasks: 0,
            };
        }
        let hist = lock_metrics(&self.histogram);
        SchedulerSnapshot {
            p50_us: hist.value_at_quantile(0.50),
            p95_us: hist.value_at_quantile(0.95),
            p99_us: hist.value_at_quantile(0.99),
            queue_depth: 0,
            total_tasks: total,
        }
    }
}

#[cfg(all(test, loom))]
mod loom_tests {
    use super::*;
    use loom::model;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn metrics_batch_is_linearizable() {
        model(|| {
            let metrics = Arc::new(SchedulerMetrics::new());
            let m1 = metrics.clone();
            let t1 = thread::spawn(move || {
                m1.record_batch(&[Duration::from_micros(10)]);
            });
            let m2 = metrics.clone();
            let t2 = thread::spawn(move || {
                m2.record_batch(&[Duration::from_micros(30), Duration::from_micros(40)]);
            });
            t1.join().unwrap();
            t2.join().unwrap();
            let snapshot = metrics.snapshot();
            assert!(snapshot.total_tasks >= 2);
            assert!(snapshot.p95_us >= snapshot.p50_us);
        });
    }
}
