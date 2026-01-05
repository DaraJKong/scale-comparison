use scale_comparison::State;
use xilem::winit::error::EventLoopError;
use xilem::{EventLoop, Xilem};

fn main() -> Result<(), EventLoopError> {
    let app_state = State::load().unwrap_or(State::new(Vec::new()));
    Xilem::new(app_state, State::view).run_in(EventLoop::with_user_event())
}
