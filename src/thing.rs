use simple_easing::cubic_in;
use xilem::core::{Edit, View, lens};
use xilem::masonry::core::{BrushIndex, render_text};
use xilem::masonry::parley::{FontContext, GenericFamily, LayoutContext};
use xilem::palette::css;
use xilem::style::Style;
use xilem::vello::Scene;
use xilem::vello::kurbo::{Affine, Rect, Vec2};
use xilem::vello::peniko::Fill;
use xilem::view::{
    CrossAxisAlignment, MainAxisAlignment, button, flex_col, flex_row, label, sized_box, text_input,
};
use xilem::{Color, FontWeight, TextAlign, WidgetView};

use crate::units::TimeScale;
use crate::utils::{text_layout, y_flipped_translate};
use crate::viewport::Viewport;

#[derive(Default)]
pub struct Thing {
    pub name: String,
    pub value: TimeScale,
}

impl Thing {
    pub const BAR_COLOR: Color = css::MEDIUM_SEA_GREEN;
    pub const NAME_COLOR: Color = css::WHITE;
    pub const VALUE_COLOR: Color = css::MEDIUM_SPRING_GREEN;

    pub const BAR_WIDTH: f64 = 40.0;
    pub const BAR_HALF: f64 = Self::BAR_WIDTH / 2.;
    pub const BAR_GAP: f64 = 100.0;
    pub const BAR_OFFSET: f64 = Self::BAR_WIDTH + Self::BAR_GAP;

    pub fn new(name: &str, value: impl Into<TimeScale>) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
        }
    }

    pub fn scale(&self) -> f64 {
        self.value.inner().erect().1
    }

    pub fn alpha(index: usize, shift: f64) -> f32 {
        cubic_in((shift - index as f64).clamp(0., 1.) as f32)
    }

    fn x_position(index: usize, half_size: Vec2) -> f64 {
        -half_size.x - Self::BAR_OFFSET * index as f64
    }

    fn y_position(&self, scale: f64) -> f64 {
        self.value.inner().to_scale(scale, Viewport::MAX_HEIGHT)
    }

    pub fn position(&self, index: usize, scale: f64, half_size: Vec2) -> Vec2 {
        Vec2::new(Self::x_position(index, half_size), self.y_position(scale))
    }

    pub fn render_bar(&self, position: Vec2, alpha: f32, scene: &mut Scene, world_camera: Affine) {
        let rect = Rect::from_origin_size(
            (position.x - Self::BAR_HALF, 0.),
            (Self::BAR_WIDTH, position.y),
        );
        scene.fill(
            Fill::NonZero,
            world_camera,
            Self::BAR_COLOR.with_alpha(alpha),
            None,
            &rect,
        );
    }

    pub fn render_name(
        &self,
        position: Vec2,
        alpha: f32,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        text_camera: Affine,
    ) {
        let name_params = (
            self.name.as_str(),
            16.,
            GenericFamily::Serif,
            None,
            Some(Self::BAR_HALF as f32 + Self::BAR_GAP as f32),
            TextAlign::Center,
        );
        let text_layout = text_layout(fcx, lcx, name_params);
        render_text(
            scene,
            text_camera
                * y_flipped_translate((
                    position.x - text_layout.width() as f64 / 2.,
                    position.y + text_layout.height() as f64 + 10.,
                )),
            &text_layout,
            &[Self::NAME_COLOR.with_alpha(alpha).into()],
            true,
        );
    }

    pub fn render_value(
        &self,
        position: Vec2,
        alpha: f32,
        fcx: &mut FontContext,
        lcx: &mut LayoutContext<BrushIndex>,
        scene: &mut Scene,
        text_camera: Affine,
    ) {
        let value = format!("{}", self.value);
        let name_params = (
            value.as_str(),
            18.,
            GenericFamily::Monospace,
            Some(500.),
            Some(Self::BAR_OFFSET as f32),
            TextAlign::Center,
        );
        let text_layout = text_layout(fcx, lcx, name_params);
        render_text(
            scene,
            text_camera * y_flipped_translate((position.x - text_layout.width() as f64 / 2., -10.)),
            &text_layout,
            &[Self::VALUE_COLOR.with_alpha(alpha).into()],
            true,
        );
    }

    pub fn view(&mut self) -> impl WidgetView<Edit<Self>, bool> + use<> {
        sized_box(
            flex_col((
                label("Name or description:")
                    .weight(FontWeight::SEMI_BOLD)
                    .color(Self::NAME_COLOR),
                text_input(self.name.clone(), |state: &mut Self, value| {
                    state.name = value;
                    false
                }),
                label("Value:")
                    .weight(FontWeight::SEMI_BOLD)
                    .color(Self::NAME_COLOR),
                lens(TimeScale::view, move |state: &mut Self, ()| {
                    &mut state.value
                })
                .map_action(|_, _| false),
                flex_row(button(label("Delete").color(css::RED), |_| true))
                    .must_fill_major_axis(true)
                    .main_axis_alignment(MainAxisAlignment::End),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .expand_width()
        .corner_radius(10.)
        .padding(10.)
        .border(Viewport::MINOR_LINE_COLOR, 1.)
        .background_color(Viewport::FOOTER_AREA_COLOR)
    }
}
