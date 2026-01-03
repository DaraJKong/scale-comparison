use xilem::{
    EventLoop, WidgetView, WindowOptions, Xilem,
    core::Edit,
    masonry::properties::types::AsUnit,
    palette::css,
    style::Style,
    vello::{
        kurbo::{Affine, Circle, Line, Point, Rect, Stroke},
        peniko::Fill,
    },
    view::{MainAxisAlignment, canvas, flex_col, flex_row, label, sized_box, slider, zstack},
    winit::error::EventLoopError,
};

use scale_comparison::{ignore_x, ignore_y, math::ENumber, units::TimeScale};

const BAR_WIDTH: f64 = 50.0;
const BAR_HALF: f64 = BAR_WIDTH / 2.;
const BAR_GAP: f64 = 25.0;

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
}

#[derive(Default)]
struct AppState {
    camera: Affine,
    scale: f64,
    things: Vec<Thing>,
}

impl AppState {
    fn view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        zstack((
            canvas(|state: &mut Self, scene, size| {
                let half_size = size.to_vec2() / 2.;
                let world_view = Affine::FLIP_Y.then_translate(half_size);
                let camera_trans = state.camera.inverse();
                let camera_view = world_view * camera_trans;

                for (i, thing) in state.things.iter().enumerate() {
                    let x = -(BAR_WIDTH + BAR_GAP) * i as f64 - BAR_HALF;
                    let value = (thing.value.inner() / ENumber::from_exp(state.scale))
                        .limit_collapse(1000.);
                    let rect = Rect::from_origin_size((x, 0.), (BAR_WIDTH, value));
                    scene.fill(Fill::NonZero, camera_view, css::WHITE, None, &rect);
                }

                // axes rendering
                let x_line = Line::new((-half_size.x, 0.), (half_size.x, 0.));
                let y_line = Line::new((0., -half_size.y), (0., half_size.y));
                let origin_dot = Circle::new(Point::ZERO, 2.);
                scene.stroke(
                    &Stroke::new(0.5),
                    world_view * ignore_x(camera_trans),
                    css::RED,
                    None,
                    &x_line,
                );
                scene.stroke(
                    &Stroke::new(0.5),
                    world_view * ignore_y(camera_trans),
                    css::BLUE,
                    None,
                    &y_line,
                );
                scene.fill(Fill::NonZero, camera_view, css::GREEN, None, &origin_dot);
            }),
            sized_box(
                flex_col(flex_row((
                    sized_box(label(format!("{:.3}", self.scale))).width(60.px()),
                    slider(-24., 18., self.scale, |state: &mut Self, value| {
                        state.scale = value;
                    }),
                )))
                .main_axis_alignment(MainAxisAlignment::End),
            )
            .expand()
            .padding(5.),
        ))
    }
}

fn main() -> Result<(), EventLoopError> {
    let app_state = AppState {
        camera: Affine::IDENTITY,
        scale: 1.,
        things: vec![
            Thing::new("Hydrogen-7 half-life", (2.3, -23)),
            Thing::new("Time for sunlight to reach earth", 8. * 60. + 20.),
            Thing::new("Week", (6.048, 5)),
            Thing::new("Sun's lifespan", (3.1556952, 17)),
        ],
    };
    Xilem::new_simple(
        app_state,
        AppState::view,
        WindowOptions::new("Scale Comparison"),
    )
    .run_in(EventLoop::with_user_event())
}
