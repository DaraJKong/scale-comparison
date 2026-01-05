use std::time::Duration;

use simple_easing::{cubic_in_out, cubic_out};
use xilem::core::{Edit, fork, lens};
use xilem::masonry::core::render_text;
use xilem::masonry::parley::GenericFamily;
use xilem::palette::css;
use xilem::style::Style;
use xilem::tokio::time;
use xilem::vello::kurbo::{Affine, Axis, Rect, Vec2};
use xilem::vello::peniko::Fill;
use xilem::view::{
    MainAxisAlignment, canvas, flex_col, flex_row, label, sized_box, task, text_button, zstack,
};
use xilem::{Color, TextAlign, WidgetView};

use crate::State;
use crate::animation::{AnimStep, Animation};
use crate::math::ENumber;
use crate::thing::Thing;
use crate::units::TimeScale;
use crate::utils::{
    ignore_x, stroke_inf_line, stroke_inf_line_pad, text_layout, y_flipped, y_flipped_translate,
};

pub struct Viewport {
    pub animation: Animation,
    pub scale: f64,
    pub scale_speed: f64,
    pub slow_scale_speed: f64,
    pub prev_shift: f64,
    pub shift: f64,
    pub camera: Affine,
}

impl Viewport {
    pub const FOOTER_AREA_COLOR: Color = Color::from_rgb8(25, 25, 25);
    pub const MAJOR_COLOR: Color = css::LIGHT_GRAY;
    pub const MINOR_LINE_COLOR: Color = Color::from_rgb8(85, 85, 85);

    pub const MAX_HEIGHT: f64 = 1000.;
    pub const MINOR_LINES: usize = 3;
    pub const MINOR_OFFSET: f64 = (Self::MINOR_LINES as f64 + 1.).recip();
    pub const SCALE_PADDING: f64 = 2.85;
    pub const IDLE_SCALE_SPEED: f64 = 0.025;
    pub const SCALE_ACCELERATION: f64 = 0.25;
    pub const INITIAL_SLOW_SCALE_SPEED: f64 = 3.;
    pub const INITIAL_CAMERA_POSITION: Vec2 = Vec2::new(0., 350.);

    pub fn init(things: &[Thing]) -> Self {
        let scale = things
            .get(0)
            .map(|thing| thing.scale() - Self::SCALE_PADDING)
            .unwrap_or(0.);
        Self {
            animation: Animation::default(),
            scale,
            scale_speed: Self::IDLE_SCALE_SPEED,
            slow_scale_speed: 0.,
            prev_shift: 0.,
            shift: 0.,
            camera: Affine::translate(Self::INITIAL_CAMERA_POSITION),
        }
    }

    fn update_animation(&mut self, things: &[Thing]) {
        let scaling_done = match self.shift.floor() {
            ..=0. => true,
            i => {
                if let Some(thing) = things.get(i as usize - 1) {
                    thing.scale() - self.scale <= Self::SCALE_PADDING
                } else {
                    false
                }
            }
        };
        let slowing_done = self.scale_speed <= Self::IDLE_SCALE_SPEED;

        self.animation.tick(scaling_done, slowing_done);

        match self.animation.step {
            AnimStep::Idle(_) | AnimStep::Pausing(_) => {
                self.scale_speed = Self::IDLE_SCALE_SPEED;
            }
            AnimStep::Scaling => {
                self.scale_speed += Self::SCALE_ACCELERATION / Animation::FPS;
            }
            AnimStep::Slowing(i) => {
                if i == AnimStep::SLOWING_FRAMES {
                    self.slow_scale_speed = self.scale_speed.min(Self::INITIAL_SLOW_SCALE_SPEED)
                }
                if i > 0 {
                    let progress = i as f32 / AnimStep::SLOWING_FRAMES as f32;
                    self.scale_speed = Self::IDLE_SCALE_SPEED
                        + (self.slow_scale_speed - Self::IDLE_SCALE_SPEED)
                            * cubic_out(progress) as f64;
                } else {
                    self.scale_speed = Self::IDLE_SCALE_SPEED;
                }
            }
            AnimStep::Shifting(i) => {
                if i > 0 {
                    let progress = 1. - (i as f32 / AnimStep::SHIFTING_FRAMES as f32);
                    self.shift = self.prev_shift + cubic_in_out(progress) as f64;
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

    pub fn view(&mut self) -> impl WidgetView<Edit<State>> + use<> {
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

                // things
                for (i, thing) in things.iter().enumerate() {
                    let position = thing.position(i, viewport.scale, half_size);
                    let alpha = Thing::alpha(i, viewport.shift);
                    thing.render_bar(position, alpha, scene, world_camera);
                    thing.render_name(position, alpha, fcx, lcx, scene, text_camera);
                }

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
                        &[Self::MAJOR_COLOR.with_alpha(major_alpha).into()],
                        true,
                    );

                    // major lines
                    let major_line_params = (
                        Axis::Horizontal,
                        major_pos,
                        Self::MAJOR_COLOR.with_alpha(major_alpha),
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
                            Self::MINOR_LINE_COLOR.with_alpha(minor_alpha),
                            0.2,
                        );
                        stroke_inf_line(scene, world_trans, camera, half_size, minor_line_params);
                    }
                }

                // area under axis line
                let rect = Rect::new(-half_size.x, 0., half_size.x, -half_size.y);
                scene.fill(
                    Fill::NonZero,
                    world_trans * ignore_x(camera),
                    Self::FOOTER_AREA_COLOR,
                    None,
                    &rect,
                );

                // axis line
                let x_line_params = (Axis::Horizontal, 0., Thing::VALUE_COLOR, 0.8);
                stroke_inf_line(scene, world_trans, camera, half_size, x_line_params);

                // thing values
                for (i, thing) in things.iter().enumerate() {
                    let position = thing.position(i, viewport.scale, half_size);
                    let alpha = Thing::alpha(i, viewport.shift);
                    thing.render_value(position, alpha, fcx, lcx, scene, text_camera);
                }
            },
        );

        let playback_btn = lens(Animation::playback_button, move |state: &mut State, ()| {
            &mut state.viewport.animation
        });
        let edit_btn = text_button("Edit", |state: &mut State| {
            state.viewport.animation.active = false;
            state.tab = crate::Tab::Data;
        });
        let controls = flex_row((playback_btn, edit_btn));
        let debug = label(format!("{:?}", self.animation.step));

        let overlay =
            sized_box(flex_col((debug, controls)).main_axis_alignment(MainAxisAlignment::End))
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
