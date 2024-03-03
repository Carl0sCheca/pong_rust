use wgpu::util::DeviceExt;

pub struct Engine {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub controller: crate::pong::Controller,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertices: Vec<crate::vertex::Vertex>,
    pub num_indices: u32,
}

impl Engine {
    pub fn screen_space_to_clip_space(
        pos: &[f32; 3],
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> [f32; 3] {
        [
            (2.0 * (pos[0] / size.width as f32)) - 1.0,
            (2.0 * (pos[1] / size.height as f32)) - 1.0,
            1.0,
        ]
    }

    #[allow(unused)]
    pub fn clip_space_to_screen_space(
        pos: &[f32; 3],
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> [f32; 3] {
        [
            (pos[0] + 1.0) / 2.0 * size.width as f32,
            (pos[1] + 1.0) / 2.0 * size.height as f32,
            1.0,
        ]
    }

    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
        };
        let surface = surface.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/default.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout Descriptor"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let controller = crate::pong::Controller::new(&size);

        let vertices = {
            let mut vertices: Vec<crate::vertex::Vertex> = Vec::new();

            for i in 0..12 {
                let index = {
                    match i {
                        0..=3 => 0,
                        4..=7 => 1,
                        _ => 2,
                    }
                };

                if index == 0 || index == 1 {
                    vertices.push(crate::vertex::Vertex {
                        position: controller.players[index].vertices[i % 4],
                        color: [1.0, 1.0, 1.0],
                    });
                } else {
                    vertices.push(crate::vertex::Vertex {
                        position: controller.ball.vertices[i % 4],
                        color: [1.0, 1.0, 1.0],
                    });
                }
            }
            vertices
        };

        let indices: Vec<u16> = {
            let mut indices: Vec<u16> = Vec::new();

            for i in 0..3 {
                indices.push(i * 4);
                indices.push((i * 4) + 1);
                indices.push((i * 4) + 2);
                indices.push(i * 4);
                indices.push((i * 4) + 2);
                indices.push((i * 4) + 3);
            }

            indices.append(&mut vec![0; indices.len() * 2 % 4]); // padding

            indices
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::vertex::Vertex>()
                        as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let num_indices = indices.len() as u32;

        Self {
            size,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            controller,
            vertex_buffer,
            index_buffer,
            vertices,
            num_indices,
        }
    }

    pub fn update(&mut self, _dt: &std::time::Duration) {
        let dt = _dt.as_secs_f32();

        self.controller.update(dt);

        // Update players positions
        for i in 0..=7 {
            let player = {
                if i & 1 << 2 != 0 {
                    1
                } else {
                    0
                }
            };

            self.vertices[i].position[1] = self.controller.players[player].vertices[i % 4][1];
        }

        // Update ball position
        for i in 8..=11 {
            self.vertices[i].position = self.controller.ball.vertices[i % 4];
        }

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass Descriptor"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        if let winit::event::WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(key),
                    state,
                    ..
                },
            ..
        } = event
        {
            match (key, state) {
                (winit::keyboard::KeyCode::KeyW, _) => match state {
                    winit::event::ElementState::Pressed => {
                        self.controller.players[0].input = crate::pong::Input::Up;
                    }
                    winit::event::ElementState::Released => {
                        self.controller.players[0].input = crate::pong::Input::None;
                    }
                },
                (winit::keyboard::KeyCode::KeyS, _) => match state {
                    winit::event::ElementState::Pressed => {
                        self.controller.players[0].input = crate::pong::Input::Down;
                    }
                    winit::event::ElementState::Released => {
                        self.controller.players[0].input = crate::pong::Input::None;
                    }
                },
                (winit::keyboard::KeyCode::ArrowUp, _) => match state {
                    winit::event::ElementState::Pressed => {
                        self.controller.players[1].input = crate::pong::Input::Up;
                    }
                    winit::event::ElementState::Released => {
                        self.controller.players[1].input = crate::pong::Input::None;
                    }
                },
                (winit::keyboard::KeyCode::ArrowDown, _) => match state {
                    winit::event::ElementState::Pressed => {
                        self.controller.players[1].input = crate::pong::Input::Down;
                    }
                    winit::event::ElementState::Released => {
                        self.controller.players[1].input = crate::pong::Input::None;
                    }
                },
                _ => (),
            }
        }
    }
}
