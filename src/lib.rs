use std::num::NonZeroUsize;

use lexical::{WriteFloatOptions, WriteFloatOptionsBuilder};
use xilem::{
    Color, FontWeight, TextAlign,
    masonry::{
        TextAlignOptions,
        core::BrushIndex,
        parley::{
            FontContext, FontFamily, FontStack, GenericFamily, Layout, LayoutContext, StyleProperty,
        },
    },
    vello::{
        Scene,
        kurbo::{Affine, Axis, Line, Stroke, Vec2},
    },
};

pub mod math;
pub mod units;

const FORMAT: u128 = lexical::format::STANDARD;
const WF_OPTIONS: WriteFloatOptions = WriteFloatOptionsBuilder::new()
    .trim_floats(true)
    .max_significant_digits(NonZeroUsize::new(5))
    .build_strict();

#[inline]
fn float_to_string(value: f64) -> String {
    lexical::to_string_with_options::<_, { FORMAT }>(value, &WF_OPTIONS)
}

pub fn y_flipped(trans: Affine) -> Affine {
    (Affine::FLIP_Y * trans) * Affine::FLIP_Y
}

pub fn y_flipped_translate<V: Into<Vec2>>(p: V) -> Affine {
    y_flipped(Affine::translate(p))
}

pub fn ignore_x(trans: Affine) -> Affine {
    let mut c = trans.as_coeffs();
    c[0] = 1.;
    c[2] = 0.;
    c[4] = 0.;
    Affine::new(c)
}

pub fn ignore_y(trans: Affine) -> Affine {
    let mut c = trans.as_coeffs();
    c[1] = 0.;
    c[3] = 1.;
    c[5] = 0.;
    Affine::new(c)
}

pub fn infinite_line(half_size: Vec2, axis: Axis, position: f64, padding: (f64, f64)) -> Line {
    match axis {
        Axis::Horizontal => Line::new(
            (-half_size.x + padding.0, position),
            (half_size.x - padding.1, position),
        ),
        Axis::Vertical => Line::new(
            (position, -half_size.y + padding.0),
            (position, half_size.y - padding.1),
        ),
    }
}

pub fn stroke_inf_line(
    scene: &mut Scene,
    world_trans: Affine,
    camera: Affine,
    half_size: Vec2,
    (axis, position, color, width): (Axis, f64, Color, f64),
) {
    let line = infinite_line(half_size, axis, position, (0., 0.));
    let transform = match axis {
        Axis::Horizontal => world_trans * ignore_x(camera),
        Axis::Vertical => world_trans * ignore_y(camera),
    };
    scene.stroke(&Stroke::new(width), transform, color, None, &line);
}

pub fn stroke_inf_line_pad(
    scene: &mut Scene,
    world_trans: Affine,
    camera: Affine,
    half_size: Vec2,
    (axis, position, color, width): (Axis, f64, Color, f64),
    padding: (f64, f64),
) {
    let line = infinite_line(half_size, axis, position, padding);
    let transform = match axis {
        Axis::Horizontal => world_trans * ignore_x(camera),
        Axis::Vertical => world_trans * ignore_y(camera),
    };
    scene.stroke(&Stroke::new(width), transform, color, None, &line);
}

pub fn text_layout(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<BrushIndex>,
    (text, size, generic_family, weight, max_advance, alignment): (
        &str,
        f32,
        GenericFamily,
        Option<f32>,
        Option<f32>,
        TextAlign,
    ),
) -> Layout<BrushIndex> {
    let mut text_layout_builder = lcx.ranged_builder(fcx, text, 1., false);
    text_layout_builder.push_default(StyleProperty::FontStack(FontStack::Single(
        FontFamily::Generic(generic_family),
    )));
    if let Some(weight) = weight {
        text_layout_builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
    }
    text_layout_builder.push_default(StyleProperty::FontSize(size));
    let mut text_layout = text_layout_builder.build(text);
    text_layout.break_all_lines(max_advance);
    text_layout.align(None, alignment, TextAlignOptions::default());
    text_layout
}
