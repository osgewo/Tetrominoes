use wgpu::SurfaceError;
use winit::event::KeyboardInput;

use crate::render::context::RenderContext;

/// Game scene.
pub trait Scene {
    /// Handles keyboard input.
    fn keyboard_input(&mut self, input: KeyboardInput) -> Action;

    /// Updates scene logic.
    fn tick(&mut self) -> Action;

    /// Renders scene.
    fn render(&mut self, ctx: &mut RenderContext) -> Result<(), SurfaceError>;
}

/// Action to be performed after a scene handler method returns.
#[must_use]
pub enum Action {
    /// Keep the game running.
    Continue,
    /// Switch to the specified scene.
    SwitchScene(Box<dyn Scene>),
    /// Exit the game.
    Exit,
}
