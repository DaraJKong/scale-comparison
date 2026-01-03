use xilem::{
    EventLoop, TextAlign, WidgetView, WindowOptions, Xilem,
    core::Edit,
    masonry::{
        TextAlignOptions,
        core::render_text,
        parley::{FontFamily, FontStack, GenericFamily, StyleProperty},
        properties::types::AsUnit,
    },
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

const BAR_WIDTH: f64 = 75.0;
const BAR_HALF: f64 = BAR_WIDTH / 2.;
const BAR_GAP: f64 = 50.0;

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
            canvas(|state: &mut Self, ctx, scene, size| {
                let (fcx, lcx) = ctx.text_contexts();

                let half_size = size.to_vec2() / 2.;
                let text_view = Affine::translate(half_size);
                let world_view = text_view * Affine::FLIP_Y;
                let camera = state.camera.inverse();
                let text_camera = text_view * camera;
                let world_camera = world_view * camera;

                for (i, thing) in state.things.iter().enumerate() {
                    let x = -(BAR_WIDTH + BAR_GAP) * i as f64;
                    let value = (thing.value.inner() / ENumber::from_exp(state.scale))
                        .limit_collapse(1000.);
                    let rect = Rect::from_origin_size((x - BAR_HALF, 0.), (BAR_WIDTH, value));
                    scene.fill(Fill::NonZero, world_camera, css::WHITE, None, &rect);

                    let mut text_layout_builder = lcx.ranged_builder(fcx, &thing.name, 1., false);
                    text_layout_builder.push_default(StyleProperty::FontStack(FontStack::Single(
                        FontFamily::Generic(GenericFamily::SansSerif),
                    )));
                    text_layout_builder.push_default(StyleProperty::FontSize(14.));
                    let mut text_layout = text_layout_builder.build(&thing.name);
                    text_layout.break_all_lines(Some(BAR_WIDTH as f32 + BAR_GAP as f32));
                    text_layout.align(None, TextAlign::Center, TextAlignOptions::default());
                    render_text(
                        scene,
                        text_camera
                            .then_translate((x - text_layout.width() as f64 / 2., 0.).into()),
                        &text_layout,
                        &[css::WHITE.into()],
                        true,
                    );
                }

                // axes rendering
                let x_line = Line::new((-half_size.x, 0.), (half_size.x, 0.));
                let y_line = Line::new((0., -half_size.y), (0., half_size.y));
                let origin_dot = Circle::new(Point::ZERO, 2.);
                scene.stroke(
                    &Stroke::new(0.5),
                    world_view * ignore_x(camera),
                    css::RED,
                    None,
                    &x_line,
                );
                scene.stroke(
                    &Stroke::new(0.5),
                    world_view * ignore_y(camera),
                    css::BLUE,
                    None,
                    &y_line,
                );
                scene.fill(Fill::NonZero, world_camera, css::GREEN, None, &origin_dot);
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
