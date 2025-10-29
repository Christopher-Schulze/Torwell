use super::cache::{ShaderCache, ShaderSource};
use super::metrics::{FrameMetrics, FrameMetricsSnapshot, RendererMetricsState};
use crate::error::{Error, Result};
use chrono::Utc;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use wgpu::util::DeviceExt;

const HISTORY_LIMIT: usize = 120;
const FRAME_SLOTS: usize = 3;
const SHADER_SOURCES: [ShaderSource; 1] = [ShaderSource {
    name: "fullscreen",
    source: include_str!("../../shaders/fullscreen.wgsl"),
}];

#[derive(Debug, Clone, Copy)]
pub struct RenderFrameDescriptor {
    pub clear_color: wgpu::Color,
}

impl Default for RenderFrameDescriptor {
    fn default() -> Self {
        Self {
            clear_color: wgpu::Color {
                r: 0.05,
                g: 0.08,
                b: 0.12,
                a: 1.0,
            },
        }
    }
}

#[derive(Clone)]
pub struct RendererService {
    inner: Arc<RendererInner>,
}

struct RendererInner {
    tx: Sender<RendererCommand>,
    metrics: Arc<RendererMetricsState>,
    available: AtomicBool,
    running: AtomicBool,
    render_loop: StdMutex<Option<std::thread::JoinHandle<()>>>,
    config: RwLock<RendererConfig>,
}

#[derive(Debug, Clone, Copy)]
struct RendererConfig {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    sample_count: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            sample_count: 1,
        }
    }
}

pub struct RenderTicket {
    receiver: oneshot::Receiver<Result<FrameMetrics>>,
}

impl RenderTicket {
    pub async fn resolve(self) -> Result<FrameMetrics> {
        let inner = self
            .receiver
            .await
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))?;
        inner
    }

    pub fn blocking_wait(self) -> Result<FrameMetrics> {
        let inner = self
            .receiver
            .blocking_recv()
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))?;
        inner
    }
}

pub struct CaptureTicket {
    receiver: oneshot::Receiver<Result<FrameCapture>>,
}

impl CaptureTicket {
    pub async fn resolve(self) -> Result<FrameCapture> {
        let inner = self
            .receiver
            .await
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))?;
        inner
    }

    pub fn blocking_wait(self) -> Result<FrameCapture> {
        let inner = self
            .receiver
            .blocking_recv()
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))?;
        inner
    }
}

#[derive(Clone)]
pub struct FrameCapture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub metrics: FrameMetrics,
}

enum RendererCommand {
    Render(RenderRequest),
    Resize(RendererConfig),
    Flush(oneshot::Sender<Result<Vec<FrameMetrics>>>),
    Shutdown,
}

struct RenderRequest {
    descriptor: RenderFrameDescriptor,
    respond_to: Option<oneshot::Sender<Result<FrameMetrics>>>,
    target: RenderTarget,
}

enum RenderTarget {
    Discard,
    Capture {
        responder: oneshot::Sender<Result<FrameCapture>>,
    },
}

struct RendererWorker {
    inner: Arc<RendererInner>,
    metrics: Arc<RendererMetricsState>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    cache: ShaderCache,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    config: RendererConfig,
    slots: Vec<FrameSlot>,
    next_slot: usize,
    frame_id: u64,
    last_frame_start: Option<Instant>,
    last_completed_frame: Option<u64>,
}

struct FrameSlot {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pending: Option<PendingFrame>,
}

struct PendingFrame {
    submission_index: wgpu::SubmissionIndex,
    submitted_at: Instant,
    encode_ns: u64,
    queue_ns: u64,
    frame_id: u64,
    frame_interval_ns: u64,
    triple_depth: u8,
    respond_to: Option<oneshot::Sender<Result<FrameMetrics>>>,
    capture: Option<CaptureState>,
}

struct CaptureState {
    buffer: wgpu::Buffer,
    bytes_per_row: u32,
    width: u32,
    height: u32,
    responder: oneshot::Sender<Result<FrameCapture>>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        }
    }
}

impl RendererService {
    pub fn new() -> Self {
        let metrics = Arc::new(RendererMetricsState::new(HISTORY_LIMIT));
        let (tx, rx) = crossbeam_channel::unbounded();
        let inner = Arc::new(RendererInner {
            tx: tx.clone(),
            metrics: metrics.clone(),
            available: AtomicBool::new(false),
            running: AtomicBool::new(true),
            render_loop: StdMutex::new(None),
            config: RwLock::new(RendererConfig::default()),
        });
        spawn_worker(inner.clone(), metrics, rx);
        Self { inner }
    }

    pub fn is_available(&self) -> bool {
        self.inner.available.load(Ordering::Relaxed)
    }

    pub fn wait_for_availability(&self, timeout: Duration) -> bool {
        if self.is_available() {
            return true;
        }
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.is_available() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    pub fn submit_frame(&self, descriptor: RenderFrameDescriptor) -> Result<RenderTicket> {
        let (tx, rx) = oneshot::channel();
        self.inner.send(RendererCommand::Render(RenderRequest {
            descriptor,
            respond_to: Some(tx),
            target: RenderTarget::Discard,
        }))?;
        Ok(RenderTicket { receiver: rx })
    }

    pub async fn render_frame(&self, descriptor: RenderFrameDescriptor) -> Result<FrameMetrics> {
        self.submit_frame(descriptor)?.resolve().await
    }

    pub fn capture_frame(&self, descriptor: RenderFrameDescriptor) -> Result<CaptureTicket> {
        let (tx, rx) = oneshot::channel();
        self.inner.send(RendererCommand::Render(RenderRequest {
            descriptor,
            respond_to: None,
            target: RenderTarget::Capture { responder: tx },
        }))?;
        Ok(CaptureTicket { receiver: rx })
    }

    pub async fn flush(&self) -> Result<Vec<FrameMetrics>> {
        let (tx, rx) = oneshot::channel();
        self.inner.send(RendererCommand::Flush(tx))?;
        rx.await
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))??
    }

    pub fn flush_blocking(&self) -> Result<Vec<FrameMetrics>> {
        let (tx, rx) = oneshot::channel();
        self.inner.send(RendererCommand::Flush(tx))?;
        rx.blocking_recv()
            .map_err(|_| Error::Gpu("renderer worker dropped".into()))??
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<()> {
        let mut cfg = self.inner.config.write().expect("renderer config poisoned");
        if cfg.width == width && cfg.height == height {
            return Ok(());
        }
        *cfg = RendererConfig {
            width,
            height,
            ..*cfg
        };
        self.inner.send(RendererCommand::Resize(*cfg))
    }

    pub fn metrics_snapshot(&self) -> FrameMetricsSnapshot {
        self.inner
            .metrics
            .snapshot(self.inner.available.load(Ordering::Relaxed))
    }

    pub fn attach_handle(&self, handle: tauri::AppHandle) {
        self.inner.metrics.set_handle(handle);
    }

    pub fn start_render_loop(&self) {
        let mut guard = self
            .inner
            .render_loop
            .lock()
            .expect("renderer loop guard poisoned");
        if guard.is_some() {
            return;
        }
        let service = self.clone();
        let handle = std::thread::Builder::new()
            .name("renderer-loop".into())
            .spawn(move || {
                let mut hue = 0.0f32;
                while service.inner.running.load(Ordering::Relaxed) {
                    if !service.is_available() {
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    let color = hue_to_color(hue);
                    if let Ok(ticket) =
                        service.submit_frame(RenderFrameDescriptor { clear_color: color })
                    {
                        let _ = ticket.blocking_wait();
                    }
                    hue = (hue + 0.015) % 1.0;
                    std::thread::sleep(Duration::from_millis(16));
                }
            })
            .expect("failed to spawn renderer loop");
        *guard = Some(handle);
    }

    pub fn shutdown(&self) {
        if self.inner.running.swap(false, Ordering::SeqCst) {
            let _ = self.inner.tx.send(RendererCommand::Shutdown);
        }
        if let Some(handle) = self
            .inner
            .render_loop
            .lock()
            .expect("renderer loop guard poisoned")
            .take()
        {
            let _ = handle.join();
        }
    }
}

impl RendererInner {
    fn send(&self, cmd: RendererCommand) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(Error::Gpu("renderer service stopped".into()));
        }
        self.tx
            .send(cmd)
            .map_err(|_| Error::Gpu("renderer worker stopped".into()))
    }
}

impl Drop for RendererInner {
    fn drop(&mut self) {
        if self.running.swap(false, Ordering::SeqCst) {
            let _ = self.tx.send(RendererCommand::Shutdown);
        }
        if let Some(handle) = self
            .render_loop
            .lock()
            .expect("renderer loop guard poisoned")
            .take()
        {
            let _ = handle.join();
        }
    }
}

fn spawn_worker(
    inner: Arc<RendererInner>,
    metrics: Arc<RendererMetricsState>,
    rx: Receiver<RendererCommand>,
) {
    std::thread::Builder::new()
        .name("renderer-worker".into())
        .spawn(
            move || match RendererWorker::initialize(inner.clone(), metrics.clone()) {
                Ok(mut worker) => {
                    inner.available.store(true, Ordering::SeqCst);
                    worker.run(rx);
                }
                Err(err) => {
                    log::error!("renderer initialization failed: {err}");
                    inner.available.store(false, Ordering::SeqCst);
                    inner.running.store(false, Ordering::SeqCst);
                    while let Ok(cmd) = rx.recv() {
                        match cmd {
                            RendererCommand::Render(request) => {
                                respond_with_error(request, err.clone());
                            }
                            RendererCommand::Flush(responder) => {
                                let _ = responder.send(Err(err.clone()));
                            }
                            RendererCommand::Shutdown => break,
                            RendererCommand::Resize(_) => {}
                        }
                    }
                }
            },
        )
        .expect("failed to spawn renderer worker");
}

fn respond_with_error(request: RenderRequest, err: Error) {
    if let RenderTarget::Capture { responder } = request.target {
        let _ = responder.send(Err(err.clone()));
    }
    if let Some(responder) = request.respond_to {
        let _ = responder.send(Err(err));
    }
}

impl RendererWorker {
    fn initialize(inner: Arc<RendererInner>, metrics: Arc<RendererMetricsState>) -> Result<Self> {
        let backends = if cfg!(target_os = "macos") {
            wgpu::Backends::METAL
        } else {
            wgpu::Backends::all()
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .or_else(|| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: true,
            }))
        })
        .ok_or_else(|| Error::Gpu("no compatible GPU adapter found".into()))?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("renderer.device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))?;
        let mut cache = ShaderCache::new()?;
        let modules = cache.warm_up(&device, &SHADER_SOURCES)?;
        let shader = modules
            .into_iter()
            .next()
            .ok_or_else(|| Error::Gpu("missing shader module".into()))?;
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("renderer.pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let config = *inner.config.read().expect("renderer config poisoned");
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("renderer.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let vertex_data = [
            Vertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
        ];
        let index_data: [u16; 6] = [0, 1, 2, 2, 1, 3];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("renderer.vertex"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("renderer.index"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });
        let slots = (0..FRAME_SLOTS)
            .map(|i| FrameSlot::new(&device, config, i))
            .collect();
        Ok(Self {
            inner,
            metrics,
            device,
            queue,
            cache,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: index_data.len() as u32,
            config,
            slots,
            next_slot: 0,
            frame_id: 0,
            last_frame_start: None,
            last_completed_frame: None,
        })
    }

    fn run(&mut self, rx: Receiver<RendererCommand>) {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                RendererCommand::Render(request) => {
                    if let Err(err) = self.render(request) {
                        log::error!("render submission failed: {err}");
                    }
                }
                RendererCommand::Resize(config) => {
                    if let Err(err) = self.resize(config) {
                        log::error!("resize failed: {err}");
                    }
                }
                RendererCommand::Flush(responder) => {
                    let result = self.flush_all();
                    let _ = responder.send(result);
                }
                RendererCommand::Shutdown => {
                    let _ = self.flush_all();
                    break;
                }
            }
        }
        self.inner.available.store(false, Ordering::SeqCst);
        self.inner.running.store(false, Ordering::SeqCst);
    }

    fn render(&mut self, mut request: RenderRequest) -> Result<()> {
        let slot_index = self.next_slot;
        self.next_slot = (self.next_slot + 1) % self.slots.len();
        if let Some(pending) = self.slots[slot_index].pending.take() {
            self.finalize_pending(pending)?;
        }
        let triple_depth = self.in_flight();
        let encode_start = Instant::now();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("renderer.encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("renderer.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.slots[slot_index].view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(request.descriptor.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        let capture_state = match request.target {
            RenderTarget::Capture { responder } => {
                let bytes_per_row = align_bytes(self.config.width * 4);
                let buffer_size = bytes_per_row as u64 * self.config.height as u64;
                let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("renderer.capture"),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTexture {
                        texture: &self.slots[slot_index].texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyBuffer {
                        buffer: &buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(bytes_per_row),
                            rows_per_image: Some(self.config.height),
                        },
                    },
                    wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                );
                Some(CaptureState {
                    buffer,
                    bytes_per_row,
                    width: self.config.width,
                    height: self.config.height,
                    responder,
                })
            }
            RenderTarget::Discard => None,
        };
        let encode_ns = encode_start.elapsed().as_nanos() as u64;
        let submit_start = Instant::now();
        let submission_index = self.queue.submit(Some(encoder.finish()));
        let queue_ns = submit_start.elapsed().as_nanos() as u64;
        let submitted_at = Instant::now();
        let interval_ns = self
            .last_frame_start
            .map(|last| submitted_at.duration_since(last).as_nanos() as u64)
            .unwrap_or(0);
        self.last_frame_start = Some(submitted_at);
        self.slots[slot_index].pending = Some(PendingFrame {
            submission_index,
            submitted_at,
            encode_ns,
            queue_ns,
            frame_id: self.frame_id,
            frame_interval_ns: interval_ns,
            triple_depth: (triple_depth + 1) as u8,
            respond_to: request.respond_to.take(),
            capture: capture_state,
        });
        self.frame_id += 1;
        Ok(())
    }

    fn resize(&mut self, config: RendererConfig) -> Result<()> {
        if self.config.width == config.width && self.config.height == config.height {
            return Ok(());
        }
        let _ = self.flush_all();
        self.config = config;
        self.slots = (0..FRAME_SLOTS)
            .map(|i| FrameSlot::new(&self.device, self.config, i))
            .collect();
        Ok(())
    }

    fn flush_all(&mut self) -> Result<Vec<FrameMetrics>> {
        let mut collected = Vec::new();
        for slot in &mut self.slots {
            if let Some(pending) = slot.pending.take() {
                match self.finalize_pending(pending) {
                    Ok(metrics) => collected.push(metrics),
                    Err(err) => log::error!("finalize failed: {err}"),
                }
            }
        }
        Ok(collected)
    }

    fn finalize_pending(&mut self, mut pending: PendingFrame) -> Result<FrameMetrics> {
        let wait_start = Instant::now();
        self.device.poll(wgpu::MaintainBase::WaitForSubmissionIndex(
            pending.submission_index,
        ));
        let queue_wait_ns = wait_start.elapsed().as_nanos() as u64;
        let gpu_time_ns = pending.submitted_at.elapsed().as_nanos() as u64;
        let dropped = self
            .last_completed_frame
            .map(|last| pending.frame_id.saturating_sub(last + 1))
            .unwrap_or(0);
        let metrics = FrameMetrics {
            frame_id: pending.frame_id,
            cpu_encode_ns: pending.encode_ns,
            queue_submit_ns: pending.queue_ns,
            gpu_time_ns,
            queue_wait_ns,
            frame_interval_ns: pending.frame_interval_ns,
            triple_buffer_depth: pending.triple_depth,
            dropped_frames: dropped,
            timestamp_ms: Utc::now().timestamp_millis(),
        };
        self.last_completed_frame = Some(pending.frame_id);
        if let Some(mut capture) = pending.capture.take() {
            if let Err(e) =
                pollster::block_on(capture.buffer.slice(..).map_async(wgpu::MapMode::Read))
            {
                let err = Error::Gpu(format!("failed to map capture buffer: {e:?}"));
                let _ = capture.responder.send(Err(err.clone()));
                if let Some(responder) = pending.respond_to.take() {
                    let _ = responder.send(Err(err.clone()));
                }
                return Err(err);
            }
            self.device.poll(wgpu::MaintainBase::Wait);
            let view = capture.buffer.slice(..).get_mapped_range();
            let mut pixels = vec![0u8; (capture.width * capture.height * 4) as usize];
            for y in 0..capture.height as usize {
                let src_offset = y * capture.bytes_per_row as usize;
                let dst_offset = y * (capture.width as usize * 4);
                let row = &view[src_offset..src_offset + capture.width as usize * 4];
                pixels[dst_offset..dst_offset + row.len()].copy_from_slice(row);
            }
            drop(view);
            capture.buffer.unmap();
            let result = FrameCapture {
                width: capture.width,
                height: capture.height,
                data: pixels,
                metrics: metrics.clone(),
            };
            let _ = capture.responder.send(Ok(result));
        }
        self.metrics.record(metrics.clone());
        if let Some(responder) = pending.respond_to.take() {
            let _ = responder.send(Ok(metrics.clone()));
        }
        Ok(metrics)
    }

    fn in_flight(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.pending.is_some())
            .count()
    }
}

impl FrameSlot {
    fn new(device: &wgpu::Device, config: RendererConfig, index: usize) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("renderer.frame.{index}")),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: config.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[config.format],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            pending: None,
        }
    }
}

fn align_bytes(value: u32) -> u32 {
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    ((value + alignment - 1) / alignment) * alignment
}

fn hue_to_color(hue: f32) -> wgpu::Color {
    let angle = hue * std::f32::consts::TAU;
    let r = 0.5 + 0.5 * angle.cos();
    let g = 0.5 + 0.5 * (angle + std::f32::consts::FRAC_PI_3).cos();
    let b = 0.5 + 0.5 * (angle + 2.0 * std::f32::consts::FRAC_PI_3).cos();
    wgpu::Color {
        r: r as f64,
        g: g as f64,
        b: b as f64,
        a: 1.0,
    }
}
