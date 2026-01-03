use std::num::NonZeroUsize;

use lexical::{WriteFloatOptions, WriteFloatOptionsBuilder};

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
