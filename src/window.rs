use winit::platform::pump_events::EventLoopExtPumpEvents;

pub struct Window {}

impl Window {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        let mut event_loop = winit::event_loop::EventLoop::new().unwrap();
        let window = winit::window::WindowBuilder::new()
            .with_title("pong")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let mut engine = futures::executor::block_on(crate::engine::Engine::new(&window));

        let mut last_update = std::time::Instant::now();

        'mainloop: loop {
            let status = event_loop.pump_events(None, |event, event_loop| match event {
                winit::event::Event::WindowEvent {
                    ref event,
                    window_id,
                    ..
                } => {
                    if window_id != window.id() {
                        return;
                    }

                    engine.input(event);
                    match event {
                        winit::event::WindowEvent::Resized(physical_size) => {
                            let winit::dpi::PhysicalSize { width, height, .. } = *physical_size;
                            if width > 0 && height > 0 {
                                engine.resize(*physical_size);
                            }
                        }
                        winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            let size = *scale_factor;
                            let new_inner_size @ winit::dpi::PhysicalSize { width, height } =
                                winit::dpi::PhysicalSize {
                                    width: (engine.config.width as f64 * size) as u32,
                                    height: (engine.config.width as f64 * size) as u32,
                                };
                            if width > 0 && height > 0 {
                                engine.resize(new_inner_size);
                            }
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }
                        winit::event::WindowEvent::KeyboardInput {
                            event:
                                winit::event::KeyEvent {
                                    physical_key: winit::keyboard::PhysicalKey::Code(key),
                                    state,
                                    ..
                                },
                            ..
                        } => {
                            if let (
                                winit::keyboard::KeyCode::Escape,
                                winit::event::ElementState::Released,
                            ) = (key, state)
                            {
                                event_loop.exit();
                            }
                        }
                        winit::event::WindowEvent::RedrawRequested => {
                            let now = std::time::Instant::now();
                            let dt = now - last_update;
                            last_update = now;

                            engine.update(&dt);
                            match engine.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => {
                                    engine.resize(engine.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    event_loop.exit();
                                }
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => (),
                    }
                }
                winit::event::Event::AboutToWait => window.request_redraw(),
                _ => (),
            });

            if let winit::platform::pump_events::PumpStatus::Exit(_) = status {
                break 'mainloop;
            }
        }
    }
}
