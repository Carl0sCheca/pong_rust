pub struct Window {}

impl Window {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) {
        let mut event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("pong")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let mut engine = futures::executor::block_on(crate::engine::Engine::new(&window));

        let mut last_update = std::time::Instant::now();

        winit::platform::run_return::EventLoopExtRunReturn::run_return(
            &mut event_loop,
            move |event, _, control_flow| match event {
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
                        winit::event::WindowEvent::ScaleFactorChanged {
                            new_inner_size, ..
                        } => {
                            let winit::dpi::PhysicalSize { width, height, .. } = **new_inner_size;
                            if width > 0 && height > 0 {
                                engine.resize(**new_inner_size);
                            }
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        winit::event::WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    virtual_keycode: Some(key),
                                    state,
                                    ..
                                },
                            ..
                        } => {
                            if let (
                                winit::event::VirtualKeyCode::Escape,
                                winit::event::ElementState::Released,
                            ) = (key, state)
                            {
                                *control_flow = winit::event_loop::ControlFlow::Exit;
                            }
                        }
                        _ => (),
                    }
                }
                winit::event::Event::RedrawRequested(_) => {
                    let now = std::time::Instant::now();
                    let dt = now - last_update;
                    last_update = now;

                    engine.update(&dt);
                    match engine.render() {
                        Ok(_) => {}
                        Err(wgpu::SwapChainError::Lost) => {
                            engine.resize(engine.size);
                        }
                        Err(wgpu::SwapChainError::OutOfMemory) => {
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                winit::event::Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => (),
            },
        );
    }
}
