use std::num::ParseFloatError;
use std::ops::{Div, Mul};

use xilem::WidgetView;
use xilem::core::Edit;
use xilem::view::{FlexExt, flex_row, text_input};

use crate::utils::float_to_string;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct ENumber {
    significand: f64,
    exponent: f64,
}

// impl PartialOrd for ENumber {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.exponent
//             .partial_cmp(&other.exponent)
//             .and_then(|ord| match ord {
//                 std::cmp::Ordering::Equal => self.significand.partial_cmp(&other.significand),
//                 _ => Some(ord),
//             })
//     }
// }

impl Mul<ENumber> for ENumber {
    type Output = ENumber;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::normalize(
            self.significand * rhs.significand,
            self.exponent + rhs.exponent,
        )
    }
}

impl Div<ENumber> for ENumber {
    type Output = ENumber;
    fn div(self, rhs: Self) -> Self::Output {
        Self::normalize(
            self.significand / rhs.significand,
            self.exponent - rhs.exponent,
        )
    }
}

impl Mul<f64> for ENumber {
    type Output = ENumber;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::normalize(self.significand * rhs, self.exponent)
    }
}

impl Div<f64> for ENumber {
    type Output = ENumber;
    fn div(self, rhs: f64) -> Self::Output {
        Self::normalize(self.significand / rhs, self.exponent)
    }
}

impl std::fmt::Display for ENumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}e{}", float_to_string(self.significand), self.exponent)
    }
}

impl From<f64> for ENumber {
    fn from(value: f64) -> Self {
        ENumber::new(value, 0)
    }
}

impl From<(f64, i32)> for ENumber {
    fn from(value: (f64, i32)) -> Self {
        ENumber::new(value.0, value.1)
    }
}

impl ENumber {
    pub fn normalize(significand: f64, exponent: f64) -> Self {
        if significand == 0. {
            return Self {
                significand,
                exponent: 0.,
            };
        }
        let adjustment = significand.abs().log10().floor();
        Self {
            significand: significand / 10_f64.powf(adjustment),
            exponent: exponent + adjustment,
        }
    }

    pub fn new(significand: f64, exponent: i32) -> Self {
        Self::normalize(significand, exponent as f64)
    }

    pub fn from_exp(exponent: f64) -> Self {
        Self::normalize(1., exponent)
    }

    pub fn significand(&self) -> f64 {
        self.significand
    }

    pub fn exponent(&self) -> f64 {
        self.exponent
    }

    pub fn fmt_exp_break(&self, exp_break: u32) -> String {
        let break_range = -(exp_break as f64)..=(exp_break as f64);
        if break_range.contains(&self.exponent) {
            float_to_string(self.collapse().expect("Low exponents sould be collapsible"))
                .to_string()
        } else {
            format!("{}e{}", float_to_string(self.significand), self.exponent)
        }
    }

    pub fn erect(&self) -> (f64, f64) {
        (
            self.significand.signum(),
            self.exponent + self.significand.abs().log10(),
        )
    }

    pub fn collapse(&self) -> Option<f64> {
        let result = self.significand * 10_f64.powf(self.exponent);
        result.is_finite().then_some(result)
    }

    pub fn limit_collapse(&self, max: f64) -> f64 {
        let result = self.significand * 10_f64.powf(self.exponent);
        result.min(max)
    }

    pub fn to_scale(self, scale: f64, max: f64) -> f64 {
        (self / ENumber::from_exp(scale)).limit_collapse(max)
    }
}

#[derive(Default, Clone)]
pub struct ENumberEditor {
    pub editing: bool,
    pub significand: String,
    pub exponent: String,
}

impl From<ENumber> for ENumberEditor {
    fn from(value: ENumber) -> Self {
        Self {
            editing: true,
            significand: value.significand.to_string(),
            exponent: value.exponent.to_string(),
        }
    }
}

impl TryInto<ENumber> for ENumberEditor {
    type Error = ParseFloatError;
    fn try_into(self) -> Result<ENumber, Self::Error> {
        let significand = self.significand.parse()?;
        let exponent = self.exponent.parse()?;
        Ok(ENumber::normalize(significand, exponent))
    }
}

impl ENumberEditor {
    pub fn view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        flex_row((
            text_input(self.significand.clone(), |state: &mut Self, value| {
                state.significand = value;
            })
            .placeholder("significand")
            .flex(1.),
            text_input(self.exponent.clone(), |state: &mut Self, value| {
                state.exponent = value;
            })
            .placeholder("exponent")
            .flex(1.),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumber_creation() {
        assert_eq!(
            ENumber::from(0.),
            ENumber {
                significand: 0.,
                exponent: 0.
            }
        );
        assert_eq!(
            ENumber::from(1e161),
            ENumber {
                significand: 1.,
                exponent: 161.
            }
        );
    }

    #[test]
    fn test_enumber_mul_inverse_property() {
        let tests: Vec<(ENumber, ENumber)> = vec![
            ((1.23, -456).into(), 1e78.into()),
            ((-0.12, -34).into(), 1e56.into()),
            ((1.2, 34).into(), 1e-56.into()),
            ((0.1, 2345).into(), (-1., 678).into()),
        ];

        tests
            .iter()
            .for_each(|test| assert_eq!(test.0, (test.0 * test.1) / test.1));
    }

    #[test]
    fn test_enumber_normalize() {
        assert_eq!(ENumber::new(12.0, 0), ENumber::new(1.2, 1));
        assert_eq!(ENumber::new(-12.0, 0), ENumber::new(-1.2, 1));
        assert_eq!(ENumber::new(0.012, -6), ENumber::new(1.2, -8));
    }

    #[test]
    fn test_enumber_collapse() {
        assert_eq!(ENumber::new(3.4, 67).collapse(), Some(3.4e67));
        assert_eq!(ENumber::new(-3.4, 2).collapse(), Some(-3.4e2));
        assert_eq!(ENumber::new(3.4, -76).collapse(), Some(3.4e-76));
        assert_eq!(ENumber::new(3.4, 309).collapse(), None);
    }
}
