use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use wgpu::winit::{
    dpi::LogicalSize, ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode,
    WindowBuilder, WindowEvent,
};

fn main() {
    env_logger::init();

    //
    // Set up window and GPU
    //
    let instance = wgpu::Instance::new();

    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::HighPerformance,
    });

    let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let mut events_loop = EventsLoop::new();

    let version = env!("CARGO_PKG_VERSION");
    let window = WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: 1280.0,
            height: 720.0,
        })
        .with_title(format!("imgui-wgpu {}", version))
        .build(&events_loop)
        .unwrap();

    let surface = instance.create_surface(&window);

    let mut dpi_factor = window.get_hidpi_factor();
    let mut size = window.get_inner_size().unwrap().to_physical(dpi_factor);

    //
    // Set up swap chain
    //
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    //
    // Set up dear imgui
    //
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);

    let font_size = (13.0 * dpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / dpi_factor) as f32;

    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
    ]);

    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    //
    // Set up dear imgui wgpu renderer
    //
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    let mut renderer = Renderer::new(&mut imgui, &mut device, sc_desc.format, Some(clear_color))
        .expect("Failed to initialize renderer");

    let mut last_frame = Instant::now();
    let mut demo_open = true;

    //
    // Event loop
    //
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                // On resize
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    dpi_factor = window.get_hidpi_factor();
                    size = window.get_inner_size().unwrap().to_physical(dpi_factor);

                    sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        width: size.width as u32,
                        height: size.height as u32,
                        present_mode: wgpu::PresentMode::Vsync,
                    };

                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
                }
                // On ESC / close
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    running = false;
                }
                _ => (),
            }

            platform.handle_event(imgui.io_mut(), &window, &event);
        });

        let io = imgui.io_mut();
        platform.prepare_frame(io, &window).expect("Failed to start frame");
        last_frame = io.update_delta_time(last_frame);

        let frame = swap_chain.get_next_texture();
        let mut ui = imgui.frame();

        {
            Window::new(im_str!("Hello world"))
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("This...is...imgui-rs on WGPU!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));
                });

            Window::new(im_str!("Hello too"))
                .position([300.0, 300.0], Condition::FirstUseEver)
                .size([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                });

            ui.show_demo_window(&mut demo_open);
        }

        let mut encoder: wgpu::CommandEncoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let draw_data = ui.render();
        renderer
            .render(draw_data, &mut device, &mut encoder, &frame.view)
            .expect("Rendering failed");

        device.get_queue().submit(&[encoder.finish()]);
    }
}
