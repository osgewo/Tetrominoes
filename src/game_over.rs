use wgpu::SurfaceError;
use wgpu_glyph::{BuiltInLineBreaker, HorizontalAlign, Layout, Section, Text, VerticalAlign};
use winit::event::{ElementState, KeyboardInput};

use crate::{
    game::Game,
    render::context::RenderContext,
    scene::{Action, Scene},
};

// TODO Better game over screen (also show lines and level).
pub struct GameOver {
    score: u32,
}

impl GameOver {
    pub fn new(score: u32) -> Self {
        Self { score }
    }
}

impl Scene for GameOver {
    fn keyboard_input(&mut self, input: KeyboardInput) -> Action {
        match (input.scancode, input.state) {
            // Start new game [Enter]
            (28, ElementState::Pressed) => {
                return Action::SwitchScene(Box::new(Game::new()));
            }
            _ => (),
        }
        Action::Continue
    }

    fn tick(&mut self) -> Action {
        Action::Continue
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Result<(), SurfaceError> {
        ctx.glyph_brush.queue(Section {
            screen_position: (
                ctx.config.width as f32 / 2.0,
                ctx.config.height as f32 / 2.0,
            ),
            text: vec![Text::new(&format!("Game over!\n\nScore: {}", self.score))
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(30.0)],
            bounds: (ctx.config.width as f32, ctx.config.height as f32),
            layout: Layout::Wrap {
                line_breaker: BuiltInLineBreaker::default(),
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
            },
        });

        ctx.render_frame()
    }
}
