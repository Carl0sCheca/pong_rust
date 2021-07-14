mod engine;
mod pong;
mod vertex;
mod window;

fn main() {
    let mut window = window::Window::new();
    window.run();
}
