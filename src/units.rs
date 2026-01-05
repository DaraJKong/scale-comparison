use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use xilem::WidgetView;
use xilem::core::one_of::Either;
use xilem::core::{Edit, lens};
use xilem::style::Style;
use xilem::view::{FlexExt, button, flex_row, label, text_button, text_input};

use crate::math::{ENumber, ENumberEditor};
use crate::thing::Thing;
use crate::utils::float_to_string;

pub const MINUTE: f64 = 60_f64;
pub const HOUR: f64 = 3600_f64;
pub const DAY: f64 = 86400_f64;
pub const YEAR: f64 = 31556952_f64;

pub const KILO: f64 = 1_000_f64;
pub const MEGA: f64 = 1_000_000_f64;
pub const GIGA: f64 = 1_000_000_000_f64;
pub const TERA: f64 = 1_000_000_000_000_f64;
pub const PETA: f64 = 1_000_000_000_000_000_f64;

#[derive(Default, Serialize, Deserialize)]
pub struct TimeScale(ENumber, #[serde(skip)] ENumberEditor);

impl std::fmt::Display for TimeScale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(collapsed) = self.0.collapse() {
            match collapsed {
                ..=MINUTE => return write!(f, "{} s", self.0.fmt_exp_break(6)),
                ..=HOUR => {
                    let mins = collapsed.div_euclid(MINUTE);
                    let secs = collapsed.rem_euclid(MINUTE);
                    write!(f, "{:.0} m", mins)?;
                    if secs != 0. {
                        write!(f, " {:.0} s", secs)?;
                    }
                    return Ok(());
                }
                ..=DAY => {
                    let hrs = collapsed.div_euclid(HOUR);
                    let mins = collapsed.rem_euclid(HOUR) / MINUTE;
                    write!(f, "{:.0} h", hrs)?;
                    if mins != 0. {
                        write!(f, " {:.0} m", mins)?;
                    }
                    return Ok(());
                }
                ..=YEAR => {
                    let days = collapsed / DAY;
                    return write!(f, "{} d", float_to_string(days));
                }
                _ => {
                    let yrs = collapsed / YEAR;
                    match yrs {
                        ..MEGA => {
                            return write!(f, "{} y", float_to_string(yrs));
                        }
                        ..GIGA => {
                            let mega = yrs / MEGA;
                            return write!(f, "{} My", float_to_string(mega));
                        }
                        ..TERA => {
                            let giga = yrs / GIGA;
                            return write!(f, "{} Gy", float_to_string(giga));
                        }
                        ..PETA => {
                            let tera = yrs / TERA;
                            return write!(f, "{} Ty", float_to_string(tera));
                        }
                        _ => (),
                    }
                }
            }
        }
        if self.0.exponent().signum() == 1. {
            let yrs = self.0 / YEAR;
            write!(f, "{} y", yrs.fmt_exp_break(6))
        } else {
            write!(f, "{} s", self.0.fmt_exp_break(6))
        }
    }
}

impl<T: Into<ENumber>> From<T> for TimeScale {
    fn from(value: T) -> Self {
        Self(value.into(), ENumberEditor::default())
    }
}

impl TimeScale {
    pub fn from_years(years: impl Into<ENumber>) -> Self {
        Self(years.into() * YEAR, ENumberEditor::default())
    }

    pub fn inner(&self) -> ENumber {
        self.0
    }

    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }

    pub fn fmt_secs(&self) -> String {
        format!("{} s", self.0.fmt_exp_break(3))
    }

    pub fn view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        if self.1.editing {
            Either::A(flex_row((
                button(label("Ok").color(Thing::VALUE_COLOR), |state: &mut Self| {
                    if let Ok(enumber) = state.1.clone().try_into() {
                        state.0 = enumber;
                    }
                    state.1.editing = false;
                }),
                lens(ENumberEditor::view, move |state: &mut Self, ()| {
                    &mut state.1
                })
                .flex(1.),
            )))
        } else {
            Either::B(flex_row((
                text_button("Edit", |state: &mut Self| {
                    state.1 = state.0.into();
                    state.1.editing = true;
                }),
                text_input(self.to_string(), |_, _| {})
                    .disabled(true)
                    .flex(1.),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_scale_format() {
        let tests = vec![
            ((1.23, -456).into(), "1.23e-456 s"),
            ((5.39, -44).into(), "5.39e-44 s"),
            (0.00086.into(), "0.00086 s"),
            (MINUTE.into(), "60 s"),
            ((8. * MINUTE + 20.).into(), "8 m 20 s"),
            (HOUR.into(), "60 m"),
            ((1. * HOUR + 32. * MINUTE).into(), "1 h 32 m"),
            (DAY.into(), "24 h"),
            ((7. * DAY).into(), "7 d"),
            ((30.4 * DAY).into(), "30.4 d"),
            (YEAR.into(), "365.24 d"),
            ((9.5 * YEAR).into(), "9.5 y"),
            ((MEGA * YEAR).into(), "1 My"),
            ((540. * MEGA * YEAR).into(), "540 My"),
            ((GIGA * YEAR).into(), "1 Gy"),
            ((2.5 * GIGA * YEAR).into(), "2.5 Gy"),
            ((TERA * YEAR).into(), "1 Ty"),
            ((10. * TERA * YEAR).into(), "10 Ty"),
            (TimeScale::from_years(1e161), "1e161 y"),
            (TimeScale::from_years((1., 32000)), "1e32000 y"),
        ];

        tests
            .iter()
            .for_each(|test| assert_eq!(format!("{}", test.0), test.1));
    }
}
