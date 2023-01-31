use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use game::Game;
use render::context::RenderContext;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod board;
mod game;
#[allow(unused)]
mod grid;
mod render;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(300, 600))
        .build(&event_loop)
        .unwrap();

    let render_context = Arc::new(Mutex::new(pollster::block_on(RenderContext::new(&window))));
    let mut game = Game::new(render_context.clone());

    let start_time = Instant::now();
    let mut frames = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
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
                WindowEvent::Resized(new_size) => {
                    render_context.lock().unwrap().resize(*new_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    render_context.lock().unwrap().resize(**new_inner_size);
                }
                WindowEvent::KeyboardInput {
                    input: KeyboardInput { scancode: 28, .. },
                    ..
                } => {
                    // Restart on Enter press
                    game = Game::new(render_context.clone());
                }
                WindowEvent::KeyboardInput { input, .. } => game.keyboard_input(*input),
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                game.tick();
                match game.render() {
                    Ok(_) => {}
                    err => err.unwrap(),
                }
                frames += 1;
                // println!(
                //     "Average FPS: {}",
                //     frames as f32 / (Instant::now() - start_time).as_secs_f32()
                // );
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once unless manually requested
                window.request_redraw();
            }
            _ => {}
        }
    });
}
