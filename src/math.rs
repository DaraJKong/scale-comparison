use std::ops::{Div, Mul};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ENumber {
    significand: f64,
    exponent: i32,
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

impl Mul for ENumber {
    type Output = ENumber;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.significand * rhs.significand,
            self.exponent + rhs.exponent,
        )
    }
}

impl Div for ENumber {
    type Output = ENumber;
    fn div(self, rhs: Self) -> Self::Output {
        self.mul(Self::new(rhs.significand, -rhs.exponent))
    }
}

impl std::fmt::Display for ENumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.exponent {
            -6..=6 => write!(
                f,
                "{}",
                self.collapse().expect("Low exponents sould be collapsible")
            ),
            _ => write!(f, "{}e{}", self.significand, self.exponent),
        }
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
    pub fn new(significand: f64, exponent: i32) -> Self {
        if significand == 0. {
            return Self {
                significand,
                exponent: 0,
            };
        }
        let adjustment = significand.abs().log10().floor() as i32;
        Self {
            significand: significand / 10_f64.powi(adjustment),
            exponent: exponent + adjustment,
        }
    }

    pub fn significand(&self) -> f64 {
        self.significand
    }

    pub fn exponent(&self) -> i32 {
        self.exponent
    }

    pub fn collapse(&self) -> Option<f64> {
        let result = self.significand * 10_f64.powi(self.exponent);
        result.is_finite().then_some(result)
    }
}

pub fn precision(value: f64, significant: usize) -> usize {
    let a = value.abs();
    if a > 1. {
        let n = (1. + a.log10().floor()) as usize;
        if n <= significant { significant - n } else { 0 }
    } else if a > 0. {
        let n = -(1. + a.log10().floor()) as usize;
        significant + n
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumber_normalize() {
        assert_eq!(ENumber::new(12.0, 0), ENumber::new(1.2, 1));
        assert_eq!(ENumber::new(-12.0, 0), ENumber::new(-1.2, 1));
        assert_eq!(ENumber::new(0.012, -6), ENumber::new(1.2, -8));
        assert_eq!(ENumber::new(0.0, 0), ENumber::new(0.0, 0));
    }

    #[test]
    fn test_enumber_collapse() {
        assert_eq!(ENumber::new(3.4, 67).collapse(), Some(3.4e67));
        assert_eq!(ENumber::new(-3.4, 2).collapse(), Some(-3.4e2));
        assert_eq!(
            ENumber::new(3.4, -76).collapse(),
            Some(3.399999999999999e-76)
        );
        assert_eq!(ENumber::new(3.4, 309).collapse(), None);
    }
}
