use std::time::Duration;

use xilem::{
    AppState, Color, EventLoop, TextAlign, WidgetView, WindowId, WindowView, Xilem,
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
        kurbo::{Affine, Axis, Rect, Vec2},
        peniko::Fill,
    },
    view::{MainAxisAlignment, canvas, flex_col, label, sized_box, task, text_button, zstack},
    window,
    winit::error::EventLoopError,
};

use scale_comparison::{
    ignore_x, math::ENumber, stroke_inf_line, stroke_inf_line_pad, text_layout, units::TimeScale,
    y_flipped, y_flipped_translate,
};

struct Thing {
    name: String,
    value: TimeScale,
}

impl Thing {
    const BAR_WIDTH: f64 = 40.0;
    const BAR_HALF: f64 = Self::BAR_WIDTH / 2.;
    const BAR_GAP: f64 = 80.0;
    const BAR_OFFSET: f64 = Self::BAR_WIDTH + Self::BAR_GAP;

    fn new(name: &str, value: impl Into<TimeScale>) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
        }
    }

    fn scale(&self) -> f64 {
        self.value.inner().erect().1
    }

    fn x_position(index: usize, half_size: Vec2) -> f64 {
        -half_size.x - Self::BAR_OFFSET * index as f64
    }

    fn y_position(&self, scale: f64) -> f64 {
        self.value.inner().to_scale(scale, Viewport::MAX_HEIGHT)
    }

    fn position(&self, index: usize, scale: f64, half_size: Vec2) -> Vec2 {
        Vec2::new(Self::x_position(index, half_size), self.y_position(scale))
    }

    fn render_bar(&self, position: Vec2, scene: &mut Scene, world_view: Affine) {
        let rect = Rect::from_origin_size(
            (position.x - Self::BAR_HALF, 0.),
            (Self::BAR_WIDTH, position.y),
        );
        scene.fill(Fill::NonZero, world_view, css::WHITE, None, &rect);
    }

    fn render_name(
        &self,
        position: Vec2,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        text_view: Affine,
    ) {
        let name_params = (
            self.name.as_str(),
            16.,
            GenericFamily::Serif,
            None,
            Some(Self::BAR_OFFSET as f32),
            TextAlign::Center,
        );
        let text_layout = text_layout(fcx, lcx, name_params);
        render_text(
            scene,
            text_view
                * y_flipped_translate((
                    position.x - text_layout.width() as f64 / 2.,
                    position.y + text_layout.height() as f64 + 10.,
                )),
            &text_layout,
            &[css::WHITE.into()],
            true,
        );
    }

    fn render_value(
        &self,
        position: Vec2,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        text_view: Affine,
    ) {
        let value = format!("{}", self.value);
        let name_params = (
            value.as_str(),
            18.,
            GenericFamily::SansSerif,
            Some(550.),
            Some(Self::BAR_OFFSET as f32),
            TextAlign::Center,
        );
        let text_layout = text_layout(fcx, lcx, name_params);
        render_text(
            scene,
            text_view * y_flipped_translate((position.x - text_layout.width() as f64 / 2., -10.)),
            &text_layout,
            &[css::TEAL.into()],
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
        half_size: Vec2,
        world_view: Affine,
        text_view: Affine,
    ) {
        let position = self.position(index, scale, half_size);
        self.render_bar(position, scene, world_view);
        self.render_name(position, fcx, lcx, scene, text_view);
        self.render_value(position, fcx, lcx, scene, text_view);
    }
}

#[derive(Debug)]
enum AnimStep {
    Idle(u64),
    Scaling,
    Pausing(u64),
    Shifting(u64),
}

impl Default for AnimStep {
    fn default() -> Self {
        Self::Shifting(Self::SHIFTING_FRAMES)
    }
}

impl AnimStep {
    const IDLE_TIME: f64 = 1.;
    const PAUSING_TIME: f64 = 3.;
    const SHIFTING_TIME: f64 = 2.;

    const IDLE_FRAMES: u64 = (Self::IDLE_TIME * Animation::FPS) as u64;
    const PAUSING_FRAMES: u64 = (Self::PAUSING_TIME * Animation::FPS) as u64;
    const SHIFTING_FRAMES: u64 = (Self::SHIFTING_TIME * Animation::FPS) as u64;

    fn next(&self) -> AnimStep {
        match self {
            AnimStep::Idle(_) => AnimStep::Scaling,
            AnimStep::Scaling => AnimStep::Pausing(Self::PAUSING_FRAMES),
            AnimStep::Pausing(_) => AnimStep::Shifting(Self::SHIFTING_FRAMES),
            AnimStep::Shifting(_) => AnimStep::Idle(Self::IDLE_FRAMES),
        }
    }

    fn advance(&mut self, current_done: bool) {
        match self {
            AnimStep::Idle(i) | AnimStep::Pausing(i) | AnimStep::Shifting(i) => {
                if *i > 0 {
                    *i -= 1;
                } else {
                    *self = self.next();
                }
            }
            AnimStep::Scaling => {
                if current_done {
                    *self = self.next();
                }
            }
        }
    }
}

#[derive(Default)]
struct Animation {
    active: bool,
    frame: u64,
    step: AnimStep,
}

impl Animation {
    const FRAME_DURATION: u64 = 16;
    const FPS: f64 = 1000. / Self::FRAME_DURATION as f64;

    fn tick(&mut self, current_done: bool) {
        self.frame += 1;
        self.step.advance(current_done);
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
    prev_shift: f64,
    shift: f64,
    camera: Affine,
}

impl Viewport {
    const MAX_HEIGHT: f64 = 1000.;
    const MINOR_LINES: usize = 3;
    const MINOR_OFFSET: f64 = (Self::MINOR_LINES as f64 + 1.).recip();
    const SCALE_PADDING: f64 = 2.75;
    const IDLE_SCALE_SPEED: f64 = 0.025;
    const SCALE_ACCELERATION: f64 = 0.5;
    const INITIAL_CAMERA_POSITION: Vec2 = Vec2::new(0., 350.);

    fn init(things: &Vec<Thing>) -> Self {
        Self {
            animation: Animation::default(),
            scale: things[0].scale() - Self::SCALE_PADDING,
            scale_speed: Self::IDLE_SCALE_SPEED,
            prev_shift: 0.,
            shift: 0.,
            camera: Affine::translate(Self::INITIAL_CAMERA_POSITION),
        }
    }

    fn update_animation(&mut self, things: &Vec<Thing>) {
        let current_done = match self.shift.floor() {
            ..=0. => true,
            i => {
                if let Some(thing) = things.get(dbg!(i as usize - 1)) {
                    thing.scale() - self.scale <= Self::SCALE_PADDING
                } else {
                    false
                }
            }
        };
        self.animation.tick(current_done);

        match self.animation.step {
            AnimStep::Idle(_) | AnimStep::Pausing(_) => {
                self.scale_speed = Self::IDLE_SCALE_SPEED;
            }
            AnimStep::Scaling => {
                self.scale_speed += Self::SCALE_ACCELERATION / Animation::FPS;
            }
            AnimStep::Shifting(i) => {
                self.scale_speed = Self::IDLE_SCALE_SPEED;
                if i > 0 {
                    let progress = 1. - (i as f64 / AnimStep::SHIFTING_FRAMES as f64);
                    self.shift = self.prev_shift + progress;
                } else {
                    self.prev_shift += 1.;
                    self.shift = self.prev_shift
                }
            }
        }
        self.scale += self.scale_speed / Animation::FPS;
        self.camera = self.camera.with_translation(
            Self::INITIAL_CAMERA_POSITION + Vec2::new(-Thing::BAR_OFFSET * self.shift, 0.),
        );
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
                let world_trans = Affine::FLIP_Y.then_translate(half_size);
                let text_trans = world_trans * Affine::FLIP_Y;
                let camera = viewport.camera.inverse();
                let world_camera = world_trans * camera;
                let text_camera = text_trans * y_flipped(camera);

                // visible logarithmic scale lines
                for offset in -1..=3 {
                    let scale = (viewport.scale + offset as f64).floor();
                    let major_pos =
                        ENumber::from_exp(scale).to_scale(viewport.scale, Self::MAX_HEIGHT);
                    let major_alpha = major_pos.clamp(0., 1.) as f32;

                    // major label
                    let major_label = TimeScale::from(ENumber::from_exp(scale)).fmt_secs();
                    let major_label_params = (
                        major_label.as_str(),
                        14.,
                        GenericFamily::SansSerif,
                        None,
                        None,
                        TextAlign::Start,
                    );
                    let major_text_layout = text_layout(fcx, lcx, major_label_params);
                    render_text(
                        scene,
                        text_trans
                            * y_flipped(ignore_x(camera))
                            * y_flipped_translate((
                                -half_size.x + 15.,
                                major_pos + major_text_layout.height() as f64 / 2.,
                            )),
                        &major_text_layout,
                        &[css::WHITE.with_alpha(major_alpha).into()],
                        true,
                    );

                    // major lines
                    let major_line_params = (
                        Axis::Horizontal,
                        major_pos,
                        css::LIGHT_GRAY.with_alpha(major_alpha),
                        0.8,
                    );
                    let major_line_padding = (major_text_layout.width() as f64 + 30., 0.);
                    stroke_inf_line_pad(
                        scene,
                        world_trans,
                        camera,
                        half_size,
                        major_line_params,
                        major_line_padding,
                    );

                    // minor lines
                    for i in 1..=Self::MINOR_LINES {
                        let minor_pos = ENumber::from_exp(scale + Self::MINOR_OFFSET * i as f64)
                            .to_scale(viewport.scale, Self::MAX_HEIGHT);
                        let minor_alpha = minor_pos.clamp(0., 1.) as f32;
                        let minor_line_params = (
                            Axis::Horizontal,
                            minor_pos,
                            Color::from_rgb8(85, 85, 85).with_alpha(minor_alpha),
                            0.2,
                        );
                        stroke_inf_line(scene, world_trans, camera, half_size, minor_line_params);
                    }
                }

                // things
                for (i, thing) in things.iter().enumerate() {
                    thing.render(
                        i,
                        viewport.scale,
                        fcx,
                        lcx,
                        scene,
                        half_size,
                        world_camera,
                        text_camera,
                    );
                }

                // axis line
                let x_line_params = (Axis::Horizontal, 0., css::TEAL, 1.);
                stroke_inf_line(scene, world_trans, camera, half_size, x_line_params);
            },
        );

        let debug = label(format!("{:?}", self.animation.step));
        let animation_controls = lens(Animation::controls_view, move |state: &mut State, ()| {
            &mut state.viewport.animation
        });
        let overlay = sized_box(
            flex_col((debug, animation_controls)).main_axis_alignment(MainAxisAlignment::End),
        )
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
                state.viewport.update_animation(&state.things);
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
