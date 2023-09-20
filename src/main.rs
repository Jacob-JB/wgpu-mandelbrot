
use std::time::Instant;

use wgpu::{*, util::{DeviceExt, BufferInitDescriptor}};
use winit::{event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, event_loop::ControlFlow};


struct View {
    position: (f32, f32),
    size: (f32, f32),
    max_iterations: u32,
}

impl View {
    fn compute_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.append(&mut self.position.0.to_le_bytes().to_vec());
        bytes.append(&mut self.position.1.to_le_bytes().to_vec());

        bytes.append(&mut self.size.0.to_le_bytes().to_vec());
        bytes.append(&mut self.size.1.to_le_bytes().to_vec());

        bytes.append(&mut self.max_iterations.to_le_bytes().to_vec());

        // padding
        bytes.append(&mut vec![0; 4]);

        bytes
    }
}

impl Default for View {
    fn default() -> Self {
        View {
            position: (0., 0.),
            size: (2., 2.),
            max_iterations: 100,
        }
    }
}



fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();


    let wgpu_instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    // safety: `window` must live longer than `surface`
    let surface = unsafe { wgpu_instance.create_surface(&window) }.unwrap();


    let adapter = pollster::block_on(wgpu_instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        power_preference: PowerPreference::HighPerformance,
    })).expect("could not get an adapter");


    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: None,
        features: Features::empty(),
        limits: Limits::downlevel_defaults(),
    }, None)).unwrap();


    let surface_capabilities = surface.get_capabilities(&adapter);

    let surface_format = surface_capabilities.formats.iter().copied().find(
        |format| format.is_srgb()
    ).unwrap_or(surface_capabilities.formats[0]);

    let mut surface_config = SurfaceConfiguration {
        alpha_mode: surface_capabilities.alpha_modes[0],
        format: surface_format,
        height: window.inner_size().height,
        width: window.inner_size().width,
        present_mode: surface_capabilities.present_modes[0],
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: vec![],
    };

    surface.configure(&device, &surface_config);


    let indices = [
        0, 1, 2, 3, 2, 1,
    ].into_iter().fold(Vec::new(), |mut acc, i: u16| {
        for i in i.to_le_bytes() {acc.push(i)}
        acc
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: &indices,
        usage: BufferUsages::INDEX,
    });


    let mut view = View::default();

    let view_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("View Buffer"),
        contents: &view.compute_bytes(),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let view_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Vew Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
    });

    let view_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("View Bind Group"),
        layout: &view_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            },
        ],
    });


    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[
            &view_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &shader_module,
            entry_point: "fragment_main",
            targets: &[
                Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                })
            ]
        }),
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
        multiview: None,
    });


    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut right_pressed = false;
    let mut left_pressed = false;
    let mut in_pressed = false;
    let mut out_pressed = false;
    let mut increase_pressed = false;
    let mut decrease_pressed = false;

    let mut fov = 4.;

    let mut last_update = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, ref event } => {
            if window_id != window.id() {return;}
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                },

                &WindowEvent::Resized(new_size) | &WindowEvent::ScaleFactorChanged { new_inner_size: &mut new_size, .. } => {
                    if new_size.width != 0 && new_size.height != 0 {
                        surface_config.width = new_size.width;
                        surface_config.height = new_size.height;
                        surface.configure(&device, &surface_config);
                    }
                },

                WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(key), state, .. }, .. } => 'b: {
                    *match key {
                        VirtualKeyCode::W => &mut up_pressed,
                        VirtualKeyCode::S => &mut down_pressed,
                        VirtualKeyCode::D => &mut right_pressed,
                        VirtualKeyCode::A => &mut left_pressed,
                        VirtualKeyCode::Up => &mut in_pressed,
                        VirtualKeyCode::Down => &mut out_pressed,
                        VirtualKeyCode::Right => &mut increase_pressed,
                        VirtualKeyCode::Left => &mut decrease_pressed,
                        _ => break 'b,
                    } = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                }

                _ => (),
            }
        },

        Event::MainEventsCleared => {
            window.request_redraw();
        },

        Event::RedrawRequested(window_id) => {
            if window_id != window.id() {return;}

            // render

            match (|| {

                let delta = last_update.elapsed().as_secs_f32();

                view.position.1 += match (up_pressed, down_pressed) {
                    (true, false) => 1.,
                    (false, true) => -1.,
                    _ => 0.,
                } * delta * fov * 0.5;

                view.position.0 += match (right_pressed, left_pressed) {
                    (true, false) => 1.,
                    (false, true) => -1.,
                    _ => 0.,
                } * delta * fov * 0.5;


                fov += match (out_pressed, in_pressed) {
                    (true, false) => 1.,
                    (false, true) => -1.,
                    _ => 0.,
                } * delta * fov * 0.75;
                fov = fov.min(4.);

                let window_size = window.inner_size();
                let longest_axis = window_size.width.max(window_size.height) as f32;

                view.size.0 = fov * (window_size.width as f32 / longest_axis);
                view.size.1 = fov * (window_size.height as f32 / longest_axis);


                view.max_iterations = (view.max_iterations as i64 + match (increase_pressed, decrease_pressed) {
                    (true, false) => 1,
                    (false, true) => -1,
                    _ => 0,
                }).max(0) as u32;

                if increase_pressed || decrease_pressed {
                    println!("max iterations: {}", view.max_iterations);
                }


                last_update = Instant::now();

                queue.write_buffer(&view_buffer, 0, &view.compute_bytes());


                let output_texture = surface.get_current_texture()?;
                let output_texture_view = output_texture.texture.create_view(&TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(RenderPassColorAttachment {
                            view: &output_texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLACK),
                                store: true,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&pipeline);
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.set_bind_group(0, &view_bind_group, &[]);
                render_pass.draw_indexed(0..6, 0, 0..1);

                drop(render_pass);

                queue.submit(std::iter::once(encoder.finish()));
                output_texture.present();

                Ok(())
            })() {
                Ok(()) => (),
                Err(SurfaceError::Lost) => {
                    surface.configure(&device, &surface_config);
                },
                Err(SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit;
                },
                Err(e) => {
                    log::error!("error: {:?}", e);
                }
            }

        },

        _ => (),
    });
}
