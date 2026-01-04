use xilem::{AppState, WindowId, WindowView, window};

pub mod animation;
pub mod math;
pub mod thing;
pub mod units;
pub mod utils;
pub mod viewport;

use crate::thing::Thing;
use crate::viewport::Viewport;

pub struct State {
    running: bool,
    window_id: WindowId,
    things: Vec<Thing>,
    viewport: Viewport,
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        self.running
    }
}

impl State {
    pub fn new(things: Vec<Thing>) -> Self {
        let viewport = Viewport::init(&things);
        Self {
            running: true,
            window_id: WindowId::next(),
            viewport,
            things,
        }
    }

    pub fn view(&mut self) -> impl Iterator<Item = WindowView<Self>> + use<> {
        std::iter::once(
            window(
                self.window_id,
                format!("Scale Comparison{}", self.viewport.animation.info()),
                self.viewport.view(),
            )
            .with_options(|options: xilem::WindowOptions<_>| {
                options.on_close(|state: &mut State| state.running = false)
            }),
        )
    }
}
