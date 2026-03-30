use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
    dpi::LogicalSize,
};
use std::sync::Arc;

mod renderer;
mod shaders;
mod ui;

use renderer::Renderer;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window_attrs = WindowAttributes::default()
        .with_inner_size(LogicalSize::new(1400.0, 800.0))
        .with_title("Aporia Loader v0.5.0");
    
    let window = event_loop.create_window(window_attrs).unwrap();
    let window = Arc::new(window);
    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                }
                WindowEvent::RedrawRequested => {
                    renderer.render();
                    window.request_redraw();
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
