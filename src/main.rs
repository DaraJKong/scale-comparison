use std::time::Duration;

use xilem::{
    AppState, EventLoop, TextAlign, WidgetView, WindowId, WindowView, Xilem,
    core::{Edit, fork, lens, one_of::Either},
    masonry::{
        core::{BrushIndex, render_text},
        parley::{FontContext, GenericFamily, LayoutContext},
    },
    palette::css,
    style::Style,
    tokio::time,
    vello::{
        Scene,
        kurbo::{Affine, Axis, Circle, Point, Rect, Vec2},
        peniko::Fill,
    },
    view::{MainAxisAlignment, canvas, flex_col, sized_box, task, text_button, zstack},
    window,
    winit::error::EventLoopError,
};

use scale_comparison::{
    math::ENumber, stroke_inf_line, text_layout, units::TimeScale, y_flipped, y_flipped_translate,
};

struct Thing {
    name: String,
    value: TimeScale,
}

impl Thing {
    const BAR_WIDTH: f64 = 75.0;
    const BAR_HALF: f64 = Self::BAR_WIDTH / 2.;
    const BAR_GAP: f64 = 50.0;

    fn new(name: &str, value: impl Into<TimeScale>) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
        }
    }

    fn scale(&self) -> f64 {
        self.value.inner().erect().1
    }

    fn x_position(index: usize) -> f64 {
        -(Self::BAR_WIDTH + Self::BAR_GAP) * index as f64
    }

    fn render_bar(&self, x_position: f64, scale: f64, scene: &mut Scene, world_view: Affine) {
        let value = self.value.inner().to_scale(scale, Viewport::MAX_HEIGHT);
        let rect =
            Rect::from_origin_size((x_position - Self::BAR_HALF, 0.), (Self::BAR_WIDTH, value));
        scene.fill(Fill::NonZero, world_view, css::WHITE, None, &rect);
    }

    fn render_text(
        &self,
        x_position: f64,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        text_view: Affine,
    ) {
        let name_params = (
            self.name.as_str(),
            16.,
            GenericFamily::SansSerif,
            Some(Self::BAR_WIDTH as f32 + Self::BAR_GAP as f32),
            TextAlign::Center,
        );
        let text_layout = text_layout(fcx, lcx, name_params);
        render_text(
            scene,
            text_view * y_flipped_translate((x_position - text_layout.width() as f64 / 2., 0.)),
            &text_layout,
            &[css::WHITE.into()],
            true,
        );
    }

    fn render(
        &self,
        index: usize,
        scale: f64,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        world_view: Affine,
        text_view: Affine,
    ) {
        let x = Self::x_position(index);
        self.render_bar(x, scale, scene, world_view);
        self.render_text(x, fcx, lcx, scene, text_view);
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

    fn tick(&mut self) {
        self.frame += 1;
    }

    fn secs(&self) -> f64 {
        self.frame as f64 / Self::FPS
    }

    fn info(&self) -> String {
        if self.frame > 0 {
            format!(
                " | frame: {}, time: {:.1} s{}",
                self.frame,
                self.secs(),
                if self.active { "" } else { " [paused]" }
            )
        } else {
            "".to_string()
        }
    }

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

struct Viewport {
    animation: Animation,
    scale: f64,
    scale_speed: f64,
    camera: Affine,
}

impl Viewport {
    const MAX_HEIGHT: f64 = 1000.;
    const SCALE_PADDING: f64 = 2.;
    const SCALE_ACCELERATION: f64 = 0.5;
    const INITIAL_CAMERA_POSITION: Vec2 = Vec2::new(0., 200.);

    fn init(things: &Vec<Thing>) -> Self {
        Self {
            animation: Animation::default(),
            scale: things[0].scale() - Self::SCALE_PADDING,
            scale_speed: 0.,
            camera: Affine::translate(Self::INITIAL_CAMERA_POSITION),
        }
    }

    fn update_animation(&mut self) {
        self.animation.tick();
        self.scale += self.scale_speed / Animation::FPS;
        self.scale_speed += Self::SCALE_ACCELERATION / Animation::FPS;
        // let mut camera = self.camera.translation();
        // camera -= (0.8, 0.2).into();
        // self.camera = self.camera.with_translation(camera);
    }

    fn view(&mut self) -> impl WidgetView<Edit<State>> + use<> {
        let canvas = canvas(
            |State {
                 things, viewport, ..
             }: &mut State,
             ctx,
             scene,
             size| {
                let (fcx, lcx) = ctx.text_contexts();

                let half_size = size.to_vec2() / 2.;
                let world = Affine::FLIP_Y.then_translate(half_size);
                let camera = viewport.camera.inverse();
                let world_view = world * camera;
                let text_view = (world * Affine::FLIP_Y) * y_flipped(camera);

                // visible logarithmic scale lines
                for offset in -1..=3 {
                    let scale = (viewport.scale + offset as f64).floor();
                    let log_line_pos =
                        ENumber::from_exp(scale).to_scale(viewport.scale, Self::MAX_HEIGHT);
                    let log_line_params = (Axis::Horizontal, log_line_pos, css::LIGHT_GRAY, 0.5);
                    stroke_inf_line(scene, world, camera, half_size, log_line_params);
                }

                // things rendering
                for (i, thing) in things.iter().enumerate() {
                    thing.render(i, viewport.scale, fcx, lcx, scene, world_view, text_view);
                }

                // axes rendering
                let x_line_params = (Axis::Horizontal, 0., css::RED, 0.5);
                let y_line_params = (Axis::Vertical, 0., css::BLUE, 0.5);
                let origin_dot = Circle::new(Point::ZERO, 2.);
                stroke_inf_line(scene, world, camera, half_size, x_line_params);
                stroke_inf_line(scene, world, camera, half_size, y_line_params);
                scene.fill(Fill::NonZero, world_view, css::GREEN, None, &origin_dot);
            },
        );

        let animation_controls = lens(Animation::controls_view, move |state: &mut State, ()| {
            &mut state.viewport.animation
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
            |state: &mut State, _| {
                state.viewport.update_animation();
            },
        ));

        fork(zstack((canvas, overlay)), animation)
    }
}

struct State {
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
    fn new(things: Vec<Thing>) -> Self {
        let viewport = Viewport::init(&things);
        Self {
            running: true,
            window_id: WindowId::next(),
            viewport,
            things,
        }
    }

    fn view(&mut self) -> impl Iterator<Item = WindowView<Self>> + use<> {
        std::iter::once(
            window(
                self.window_id,
                format!("Scale Comparison{}", self.viewport.animation.info()),
                self.viewport.view(),
            )
            .with_options(|options| options.on_close(|state: &mut State| state.running = false)),
        )
    }
}

fn main() -> Result<(), EventLoopError> {
    let app_state = State::new(vec![
        Thing::new("Hydrogen-7 half-life", (2.3, -23)),
        Thing::new("Time for sunlight to reach earth", 8. * 60. + 20.),
        Thing::new("Week", (6.048, 5)),
        Thing::new("Sun's lifespan", (3.1556952, 17)),
    ]);
    Xilem::new(app_state, State::view).run_in(EventLoop::with_user_event())
}
