use std::time::Instant;

use game::Game;
use render::context::RenderContext;
use scene::{Action, Scene};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod board;
mod game;
#[allow(unused)]
mod grid;
mod render;
mod scene;
mod tetromino;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(580, 650))
        .build(&event_loop)
        .unwrap();

    let mut run_loop = RunLoop::new(window);

    event_loop.run(move |event, _, control_flow| {
        run_loop.handle_event(event, control_flow);
    });
}

struct RunLoop {
    window: Window,
    render_context: RenderContext,
    scene: Box<dyn Scene>,

    // Profiling:
    start_time: Instant,
    frames: usize,
}

impl RunLoop {
    fn new(window: Window) -> Self {
        let render_context = pollster::block_on(RenderContext::new(&window));
        Self {
            window,
            render_context,
            scene: Box::new(Game::new()),
            start_time: Instant::now(),
            frames: 0,
        }
    }

    fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => self.handle_window_event(event, control_flow),
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                let action = self.scene.tick();
                self.handle_action(action, control_flow);
                self.scene.render(&mut self.render_context).unwrap();

                self.frames += 1;
                let elapsed = Instant::now() - self.start_time;
                if elapsed.as_millis() >= 1000 {
                    self.start_time = Instant::now();
                    println!("FPS: {}", self.frames as f32 / elapsed.as_secs_f32());
                    self.frames = 0;
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once unless manually requested
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => {
                self.render_context.resize(*new_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.render_context.resize(**new_inner_size);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                let action = self.scene.keyboard_input(*input);
                self.handle_action(action, control_flow);
            }
            _ => {}
        }
    }

    fn handle_action(&mut self, action: Action, control_flow: &mut ControlFlow) {
        match action {
            Action::Continue => (),
            Action::SwitchScene(scene) => self.scene = scene,
            Action::Exit => *control_flow = ControlFlow::Exit,
        }
    }
}
