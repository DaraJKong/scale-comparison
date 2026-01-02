use xilem::{
    EventLoop, WidgetView, WindowOptions, Xilem,
    core::{Edit, Read, lens},
    view::{flex_col, label},
    winit::error::EventLoopError,
};

use scale_comparison::units::TimeScale;

struct Thing {
    name: String,
    value: TimeScale,
}

impl Thing {
    fn new(name: &str, value: impl Into<TimeScale>) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
        }
    }

    fn view(&self) -> impl WidgetView<Read<Self>> + use<> {
        label(format!("{}: {}", self.name, self.value))
    }
}

#[derive(Default)]
struct AppState {
    things: Vec<Thing>,
}

fn app_logic(state: &mut AppState) -> impl WidgetView<Edit<AppState>> + use<> {
    let things = state
        .things
        .iter()
        .enumerate()
        .map(|(i, _)| {
            lens(Thing::view, move |state: &mut AppState, ()| {
                state.things.get(i).unwrap()
            })
        })
        .collect::<Vec<_>>();
    flex_col(things)
}

fn main() -> Result<(), EventLoopError> {
    let app_state = AppState {
        things: vec![
            Thing::new("Hydrogen-7 half-life", (2.3, -23)),
            Thing::new("Time for sunlight to reach earth", 8. * 60. + 20.),
            Thing::new("Week", (6.048, 5)),
            Thing::new("Sun's lifespan", (3.15576, 17)),
        ],
    };
    Xilem::new_simple(app_state, app_logic, WindowOptions::new("Scale Comparison"))
        .run_in(EventLoop::with_user_event())
}
