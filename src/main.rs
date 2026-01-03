use std::time::Duration;

use xilem::{
    EventLoop, TextAlign, WidgetView, WindowOptions, Xilem,
    core::{Edit, fork, lens, one_of::Either},
    masonry::{
        TextAlignOptions,
        core::render_text,
        parley::{FontFamily, FontStack, GenericFamily, StyleProperty},
    },
    palette::css,
    style::Style,
    tokio::time,
    vello::{
        kurbo::{Affine, Circle, Line, Point, Rect, Stroke, Vec2},
        peniko::Fill,
    },
    view::{MainAxisAlignment, canvas, flex_col, sized_box, task, text_button, zstack},
    winit::error::EventLoopError,
};

use scale_comparison::{
    ignore_x, ignore_y, math::ENumber, units::TimeScale, y_flipped, y_flipped_translate,
};

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
struct Animation {
    active: bool,
    frame: u64,
}

impl Animation {
    const FRAME_DURATION: u64 = 32;
    const FPS: f64 = 1000. / Self::FRAME_DURATION as f64;

    fn controls_view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        if self.active {
            Either::A(text_button("Pause", |state: &mut Self| {
                state.active = false;
            }))
        } else {
            Either::B(text_button("Play", |state: &mut Self| {
                state.active = true;
            }))
        }
    }
}

struct AppState {
    animation: Animation,
    scale: f64,
    scale_speed: f64,
    camera: Affine,
    things: Vec<Thing>,
}

impl AppState {
    const SCALE_PADDING: f64 = 2.;
    const SCALE_ACCELERATION: f64 = 1.;
    const INITIAL_CAMERA_POSITION: Vec2 = Vec2::new(0., 200.);

    fn init(things: Vec<Thing>) -> Self {
        Self {
            animation: Animation::default(),
            scale: things[0].value.inner().erect().1 - Self::SCALE_PADDING,
            scale_speed: 0.,
            camera: Affine::translate(Self::INITIAL_CAMERA_POSITION),
            things,
        }
    }

    fn update_animation(&mut self) {
        self.animation.frame += 1;
        self.scale += self.scale_speed / Animation::FPS;
        self.scale_speed += Self::SCALE_ACCELERATION / Animation::FPS;
        let mut camera = self.camera.translation();
        camera.x -= 1.;
        camera.y -= 0.2;
        self.camera = self.camera.with_translation(camera);
    }

    fn view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        let canvas = canvas(|state: &mut Self, ctx, scene, size| {
            let (fcx, lcx) = ctx.text_contexts();

            let half_size = size.to_vec2() / 2.;
            let world = Affine::FLIP_Y.then_translate(half_size);
            let camera = state.camera.inverse();
            let world_view = world * camera;
            let text_view = (world * Affine::FLIP_Y) * y_flipped(camera);

            for (i, thing) in state.things.iter().enumerate() {
                let x = -(BAR_WIDTH + BAR_GAP) * i as f64;
                let value =
                    (thing.value.inner() / ENumber::from_exp(state.scale)).limit_collapse(1000.);
                let rect = Rect::from_origin_size((x - BAR_HALF, 0.), (BAR_WIDTH, value));
                scene.fill(Fill::NonZero, world_view, css::WHITE, None, &rect);

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
                    text_view * y_flipped_translate((x - text_layout.width() as f64 / 2., 0.)),
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
                world * ignore_x(camera),
                css::RED,
                None,
                &x_line,
            );
            scene.stroke(
                &Stroke::new(0.5),
                world * ignore_y(camera),
                css::BLUE,
                None,
                &y_line,
            );
            scene.fill(Fill::NonZero, world_view, css::GREEN, None, &origin_dot);
        });

        let animation_controls = lens(Animation::controls_view, move |state: &mut Self, ()| {
            &mut state.animation
        });
        let overlay =
            sized_box(flex_col(animation_controls).main_axis_alignment(MainAxisAlignment::End))
                .expand()
                .padding(15.);

        let animation = self.animation.active.then_some(task(
            |proxy, _| async move {
                let mut interval = time::interval(Duration::from_millis(Animation::FRAME_DURATION));
                loop {
                    interval.tick().await;
                    let Ok(()) = proxy.message(()) else {
                        break;
                    };
                }
            },
            |state: &mut Self, _| {
                state.update_animation();
            },
        ));

        fork(zstack((canvas, overlay)), animation)
    }
}

fn main() -> Result<(), EventLoopError> {
    let app_state = AppState::init(vec![
        Thing::new("Hydrogen-7 half-life", (2.3, -23)),
        Thing::new("Time for sunlight to reach earth", 8. * 60. + 20.),
        Thing::new("Week", (6.048, 5)),
        Thing::new("Sun's lifespan", (3.1556952, 17)),
    ]);
    Xilem::new_simple(
        app_state,
        AppState::view,
        WindowOptions::new("Scale Comparison"),
    )
    .run_in(EventLoop::with_user_event())
}
