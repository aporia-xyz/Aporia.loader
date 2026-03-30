use wgpu::*;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    bloom_pipeline: RenderPipeline,
    blur_pipeline: RenderPipeline,
    rect_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    start_time: Instant,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let bloom_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("bloom_shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/bloom.wgsl"))),
        });

        // Создаём uniform буфер для времени
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniform_buffer"),
            size: 4, // f32 = 4 байта
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Создаём bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Создаём bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let bloom_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("bloom_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &bloom_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &bloom_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        // Blur пайплайн
        let blur_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("blur_shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/blur.wgsl"))),
        });

        let blur_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("blur_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("blur_pipeline"),
            layout: Some(&blur_pipeline_layout),
            vertex: VertexState {
                module: &blur_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &blur_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        // Rect пайплайн
        let rect_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("rect_shader"),
            source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/rect.wgsl"))),
        });

        let rect_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("rect_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let rect_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("rect_pipeline"),
            layout: Some(&rect_pipeline_layout),
            vertex: VertexState {
                module: &rect_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &rect_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            bloom_pipeline,
            blur_pipeline,
            rect_pipeline,
            uniform_buffer,
            bind_group,
            start_time: Instant::now(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&self) {
        // Обновляем uniform буфер с текущим временем
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[elapsed]));

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.024,
                            g: 0.024,
                            b: 0.055,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Рисуем bloom слой (фон, звёзды, кометы)
            render_pass.set_pipeline(&self.bloom_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..1);

            // Рисуем blur слой
            render_pass.set_pipeline(&self.blur_pipeline);
            render_pass.draw(0..6, 0..1);

            // Рисуем rect слой - 4 вертекса
            render_pass.set_pipeline(&self.rect_pipeline);
            render_pass.draw(0..4, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
