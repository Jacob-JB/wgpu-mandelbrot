
use wgpu::*;
use winit::{event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, event_loop::ControlFlow};




fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let mut window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();


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


    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        module: &shader_module,
        entry_point: "compute_main",
        layout: Some(&compute_pipeline_layout),
    });



    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, ref event } => {
            if window_id != window.id() {return;}
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                },

                &WindowEvent::Resized(new_size) | &WindowEvent::ScaleFactorChanged { new_inner_size: &mut new_size, .. } => {
                    surface_config.width = new_size.width;
                    surface_config.height = new_size.height;
                    surface.configure(&device, &surface_config);
                },

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

                let output_texture = surface.get_current_texture()?;
                let output_texture_view = output_texture.texture.create_view(&TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Main Compute Pass"),
                });

                compute_pass.set_pipeline(&compute_pipeline);
                compute_pass.dispatch_workgroups(8, 8, 8);

                drop(compute_pass);

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
