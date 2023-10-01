use camera::CameraUniform;
use cgmath::Rotation3;
use mesh::Instance;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalPosition,
};
use winit::window::Window;
use std::{borrow::Cow, fs};
use wgpu::{util::DeviceExt, BindGroupLayout};
use std::fs::File;
use std::path::Path;
use json;

mod mesh;
mod camera;
mod texture;

// Triangle
#[allow(unused)]
const TIANGLE_VERTICES: &[mesh::Vertex] = &[
    mesh::Vertex { position: [0., 0.5, 0.],    color: [ 1., 1., 1.], uv: [0.5, 0.0] },
    mesh::Vertex { position: [-0.5, -0.5, 0.], color: [ 1., 1., 1.], uv: [0.0, 1.0] },
    mesh::Vertex { position: [0.5, -0.5, 0.],  color: [ 1., 1., 1.], uv: [1.0, 1.0] },
];
#[allow(unused)]
const TRIANGLE_INDICES: &[u16] = &[
    0, 1, 2
];

// Trigonometic functions don't work in a const context :(
const PENTAGON_VERTICES: &[mesh::Vertex] = &[
    mesh::Vertex { position: [ 0.0    * 0.5,  1.0   * 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [( 0.0 + 1.0) * 0.5,    1. - 1.0] },
    mesh::Vertex { position: [ 0.951  * 0.5,  0.309 * 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [( 0.951 + 1.0) * 0.5,  1. - ( 0.309 + 1.0) * 0.5 ] },
    mesh::Vertex { position: [ 0.5878 * 0.5, -0.809 * 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [( 0.5878 + 1.0) * 0.5, 1. - (-0.809 + 1.0) * 0.5 ] },
    mesh::Vertex { position: [-0.5878 * 0.5, -0.809 * 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [(-0.5878 + 1.0) * 0.5, 1. - (-0.809 + 1.0) * 0.5 ] },
    mesh::Vertex { position: [-0.951  * 0.5,  0.309 * 0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [(-0.951 + 1.0) * 0.5,  1. - ( 0.309 + 1.0) * 0.5 ] },
];
const PENTAGON_INDICES: &[u16] = &[
    0, 4, 1,
    4, 2, 1,
    4, 3, 2,
];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    clear_color: wgpu::Color,

    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_2: wgpu::RenderPipeline,

    mesh: mesh::Mesh,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,

    orbit_camera: camera::OrbitCamera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    prev_mouse_pos: PhysicalPosition<f64>,

    depth_texture: texture::Texture,
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,

    space_pressed: bool,
    left_mouse_pressed: bool,
}

impl State {
    async fn new (window: Window) -> Self {
        let size = window.inner_size();

        // Create WGPU backend
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Create surface
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // Find suitable hardware
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // Create device interface and queue for hardware
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            }, None
        ).await.unwrap();

        // Configure backbuffer surface
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Load image data
        let diffuse_bytes = include_bytes!("uv_checker.jpg");
        let diffuse_texture = texture::Texture::from_memory(&device, &queue, diffuse_bytes, "uv_checker").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    }
                ],
            }
        );

        let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };

        let camera = camera::Camera {
            position: cgmath::point3(0., 0., 2.),
            forward: cgmath::vec3(0., 0., -1.),
            up: cgmath::vec3(0., 1., 0.),
            aspect_ratio: size.width as f32 / size.height as f32,
            fov_vertical: 45.,
            znear: 0.1,
            zfar: 100.,
        };

        let orbit_camera = camera::OrbitCamera::new(camera, cgmath::Point3 { x: 0., y: 0., z: 0. }, 2., 0., 0.);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_projection(orbit_camera.camera());

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
            }
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth texture");
        let render_pipeline = Self::create_render_pipeline(&device, &surface_config, include_str!("basic.wgsl").into(), "vs_main", "fs_main_2", &texture_bind_group_layout, &camera_bind_group_layout);
        let render_pipeline_2 = Self::create_render_pipeline(&device, &surface_config, include_str!("basic.wgsl").into(), "vs_main_2", "fs_main", &texture_bind_group_layout, &camera_bind_group_layout);

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("diffuse bind group"),
                layout: &texture_bind_group_layout.into(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ]
            }
        );

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ]
        });

        
        // Load glTF
        let mesh = mesh::Mesh::load_gltf(&std::path::Path::new("./res/monkey.gltf"));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&mesh.verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instances = (0..10).flat_map(|x| {
            (0..10).map(move |z| {
                let position = cgmath::Point3 { x: x as f32 - 5., y: 0., z: z as f32 - 5. };
                let rotation = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(x as f32 * z as f32));
                Instance { position, rotation }
            })
        }).collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_data).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            window,
            surface,
            device,
            queue,
            surface_config,
            size,
            clear_color,

            render_pipeline,
            render_pipeline_2,

            mesh,
            vertex_buffer,
            index_buffer,
            instances,
            instance_buffer,

            orbit_camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            prev_mouse_pos: PhysicalPosition { x: -1., y: -1. },

            depth_texture,
            diffuse_texture,
            diffuse_bind_group,
            space_pressed: false,
            left_mouse_pressed: false,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn create_render_pipeline(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration, shader: Cow<'_, str>, vs_entry: &str, fs_entry: &str, texture_bind_group_layout: &BindGroupLayout, camera_bind_group_layout: &BindGroupLayout) -> wgpu::RenderPipeline {
        // Real code to create a shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(shader),
        });
        // Shorthand helper for shader from file
        // let shader = device.create_shader_module(wgpu::include_wgsl!(shader_name));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: vs_entry,
                buffers: &[
                    mesh::Vertex::desc(), mesh::InstanceData::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: fs_entry,
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0, // Weird, or cool way to say -1 (invert all bits)
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        render_pipeline
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config)
        }
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.surface_config, "depth texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                if button == &MouseButton::Left {
                    if state == &ElementState::Pressed {
                        self.left_mouse_pressed = true;

                    } else {
                        self.left_mouse_pressed = false;
                    }
                }
                true
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_h, v) => {
                        self.orbit_camera.handle_scroll(*v)
                    }
                    _ => ()
                }
                true
            }
            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                if self.left_mouse_pressed {
                    let dx = position.x - self.prev_mouse_pos.x;
                    let dy = position.y - self.prev_mouse_pos.y;
                    self.orbit_camera.handle_mouse_drag(dx, dy);
                } else {
                    self.clear_color.r = position.x as f64 / self.size.width as f64;
                    self.clear_color.b = position.y as f64 / self.size.height as f64;
                }
                self.prev_mouse_pos = *position;
                true
            }
            WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                // Spacebar physical location
                if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Space) {
                    self.space_pressed = input.state == winit::event::ElementState::Pressed;
                    return true
                }
                false
            }
            _ => false
        }
    }

    fn update(&mut self) {
        self.camera_uniform.update_view_projection(&self.orbit_camera.camera());
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        // Extra scope so that _render_pass is dropped since it borrows mutable encoder
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if self.space_pressed == true {
                render_pass.set_pipeline(&self.render_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline_2);
            }
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.mesh.indices.len() as u32, 0, 0..1 as _);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => if !state.input(event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size)
                }
                WindowEvent::ScaleFactorChanged { scale_factor: _, new_inner_size } => {
                    state.resize(**new_inner_size)
                }
                _ => {}
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => {}
    });
}
