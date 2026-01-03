use std::num::NonZeroUsize;

use lexical::{WriteFloatOptions, WriteFloatOptionsBuilder};
use xilem::vello::kurbo::{Affine, Vec2};

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
