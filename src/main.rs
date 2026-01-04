use scale_comparison::State;
use scale_comparison::thing::Thing;
use xilem::winit::error::EventLoopError;
use xilem::{EventLoop, Xilem};

fn main() -> Result<(), EventLoopError> {
    let app_state = State::new(vec![
        Thing::new("Hydrogen-7 half-life", (2.3, -23)),
        Thing::new("Time for sunlight to reach earth", 8. * 60. + 20.),
        Thing::new("Week", (6.048, 5)),
        Thing::new("Sun's lifespan", (3.1556952, 17)),
    ]);
    Xilem::new(app_state, State::view).run_in(EventLoop::with_user_event())
}
