#![allow(dead_code)]
#![allow(unused_mut)]

pub mod util {
    pub mod camera;
    pub mod constructors;
    pub mod image;
    pub mod shapes;
    pub mod vertex;
}

use cgmath::{Deg, Quaternion, Rotation3};
use std::default::Default;
use wgpu::{BufferBindingType, DynamicOffset};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use wgpu::util::DeviceExt;
use winit::window::CursorGrabMode;

use crate::util::camera::*;
use crate::util::constructors::*;
use crate::util::image::Video;
use crate::util::shapes::{Cube, Shape, ShapeManager};
use crate::util::vertex;
use crate::util::vertex::VERTICES;

struct State {
    // Device & Window config
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    // Vertex Config
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,

    // Shape Config
    shape_manager: ShapeManager,
    shape_bind_group: wgpu::BindGroup,
    shape_buffer: wgpu::Buffer,
    sphere_buffer: wgpu::Buffer,
    cube_buffer: wgpu::Buffer,

    // Camera Config
    camera: Camera,
    projection: Projection,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // Misc config
    mouse_pressed: bool,
    shader_params: ShaderParams,
    config_buffer: wgpu::Buffer,
    config_bind_group: wgpu::BindGroup,

    // Apple
    bad_apple: bool,
    bad_apple_timer: f32,
    bad_apple_size: (u32, u32),
    bad_apple_video: Video,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderParams {
    time: f32,
    width: u32,
    height: u32,

    // Array sizes
    shape_count: u32,
    sphere_count: u32,
    cube_count: u32,
}

impl ShaderParams {
    pub fn sin_t(&self) -> f32 {
        self.time.sin()
    }

    pub fn cos_t(&self) -> f32 {
        self.time.cos()
    }

    // scaled sin
    pub fn ssin_t(&self, scale: f32) -> f32 {
        (self.time * scale).sin()
    }

    // scaled cos
    pub fn scos_t(&self, scale: f32) -> f32 {
        (self.time * scale).cos()
    }
}

impl State {
    async fn new(window: &Window) -> State {
        //#region Device & Window Config
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = request_adapter(&instance, &surface).await;

        let (device, queue) = request_device(&adapter).await;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        //#endregion

        //#region Camera Config
        let camera = Camera::new((0.0, 0.0, 10.0), Deg::<f32>(0.0), Deg::<f32>(0.0));
        let projection = Projection::new(config.width, config.height, Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.25);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = create_bind_group_layout(
            &device,
            0,
            BufferBindingType::Uniform,
            false,
            None,
            "camera_bind_group_layout",
        );

        let camera_bind_group = create_bind_group(
            &device,
            &camera_bind_group_layout,
            0,
            camera_buffer.as_entire_binding(),
            "camera_bind_group",
        );
        //#endregion

        //#region config buffer
        let shader_params = ShaderParams {
            time: 0.0,
            width: size.width,
            height: size.height,
            shape_count: 0,
            sphere_count: 0,
            cube_count: 0,
        };

        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[shader_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let config_bind_group_layout = create_bind_group_layout(
            &device,
            1,
            BufferBindingType::Uniform,
            false,
            None,
            "config_bind_group_layout",
        );

        let config_bind_group = create_bind_group(
            &device,
            &config_bind_group_layout,
            1,
            config_buffer.as_entire_binding(),
            "config_bind_group",
        );
        //#endregion

        //#region shape buffers
        let mut shape_manager = ShapeManager::new();
        let apple_size = (20, 15);
        for x in 0..apple_size.0 {
            for y in 0..apple_size.1 {
                // shape_manager.new_sphere(
                //     ((x * 3) as f32, (y * 3) as f32, 0.0).into(),
                //     1.0,
                //     (0.2 + (x as f32 * 0.04), 0.2 + (y as f32 * 0.04), 0.2).into(),
                // );
                shape_manager.new_cube(
                    ((x * 2)  as f32, (y * 2) as f32, 0.0).into(),
                    (1.0, 1.0, 1.0).into(),
                    (0.2 + (x as f32 * 0.04), 0.2 + (y as f32 * 0.04), 0.2).into(),
                );
            }
        }

        let shape_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Buffer"),
            contents: &*shape_manager.serialize_shapes(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            contents: &*shape_manager.serialize_spheres(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let cube_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Buffer"),
            contents: &*shape_manager.serialize_cubes(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let shape_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("shape_bind_group_layout"),
            });

        let shape_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shape_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shape_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cube_buffer.as_entire_binding(),
                },
            ],
            label: Some("shape_bind_group"),
        });
        //#endregion

        //#region render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout, // group 0
                    &config_bind_group_layout, // group 1
                    &shape_bind_group_layout,  // group 2
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
                conservative: false, // based
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        //#endregion

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = VERTICES.len() as u32;

        Self {
            // GPU & Window config
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,

            // Vertex config
            vertex_buffer,
            num_vertices,

            // Shape Config
            shape_manager,
            shape_bind_group,
            shape_buffer,
            sphere_buffer,
            cube_buffer,

            // Camera config
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            // Misc
            mouse_pressed: false,
            shader_params,
            config_buffer,
            config_bind_group,

            // Apple
            bad_apple: false,
            bad_apple_timer: 0.0,
            bad_apple_size: apple_size,
            bad_apple_video: Video::new("./assets/apple", apple_size.0, apple_size.1),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.shader_params.width = new_size.width;
            self.shader_params.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => if self.camera_controller.process_keyboard(*key, *state) {
                true
            } else {
                match (key, state) {
                    (VirtualKeyCode::Q, ElementState::Pressed) => {
                        // Start / Stop bad apple
                        self.bad_apple = !self.bad_apple;
                        self.bad_apple_timer = 0.0;
                    }
                    _ => {}
                }
                false
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.shader_params.time += dt.as_secs_f32();

        if self.bad_apple {
            self.bad_apple_timer += dt.as_secs_f32();

            let frame = self.bad_apple_video.frame_index_from_time(self.bad_apple_timer, 30.0);
            for x in 0..self.bad_apple_size.0 {
                for y in 0..self.bad_apple_size.1 {
                    let p = self.bad_apple_video.get_pixel_value(frame, x, y);
                    self.shape_manager.get_cube_mut(x * self.bad_apple_size.1 + y).unwrap().set_bounds(
                        (p, p, p).into(),
                    );
                }
            }

        } else {
            self.bad_apple_timer = 0.0;
            self.shape_manager.iter_shapes_mut()
                .filter_map(|s| s.as_any_mut().downcast_mut::<Cube>())
                .for_each(|c| {
                    c.set_rotation(Quaternion::from_angle_x(Deg(0.0)));
                    c.set_bounds((1.0, 1.0, 1.0).into());
                });
        }


        self.shape_manager
            .update_shader_config(&mut self.shader_params);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.config_buffer,
            0,
            bytemuck::cast_slice(&[self.shader_params]),
        );
        self.queue.write_buffer(
            &self.shape_buffer,
            0,
            &*self.shape_manager.serialize_shapes(),
        );
        self.queue.write_buffer(
            &self.sphere_buffer,
            0,
            &*self.shape_manager.serialize_spheres(),
        );
        self.queue.write_buffer(
            &self.cube_buffer,
            0,
            &*self.shape_manager.serialize_cubes(),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_bind_group(1, &self.config_bind_group, &[]);

            render_pass.set_bind_group(
                2,
                &self.shape_bind_group,
                &[DynamicOffset::default(), DynamicOffset::default(), DynamicOffset::default()],
            );

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
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

    // window
    //     .set_cursor_grab(CursorGrabMode::Locked)
    //     .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
    //     .or_else(|_| window.set_cursor_grab(CursorGrabMode::None))
    //     .unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } =>
                if state.mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            // state.camera_controller.process_mouse(delta.0, delta.1),

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
