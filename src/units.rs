use crate::math::{ENumber, precision};

pub struct TimeScale(ENumber);

impl std::fmt::Display for TimeScale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(collapsed) = self.0.collapse() {
            match collapsed {
                ..=60. => {
                    return write!(f, "{} s", self.0);
                }
                ..=3600. => {
                    let mins = collapsed.div_euclid(60.);
                    let secs = collapsed.rem_euclid(60.);
                    write!(f, "{:.0} m", mins)?;
                    if secs != 0. {
                        write!(f, " {:.0} s", secs)?;
                    }
                    return Ok(());
                }
                ..=86400. => {
                    let hrs = collapsed.div_euclid(3600.);
                    let mins = collapsed.rem_euclid(3600.) / 60.;
                    write!(f, "{:.0} h", hrs)?;
                    if mins != 0. {
                        write!(f, " {:.0} m", mins)?;
                    }
                    return Ok(());
                }
                ..=31556952. => {
                    let days = collapsed / 86400.;
                    return write!(f, "{:.1$} d", days, precision(days, 2));
                }
                _ => {
                    let yrs = collapsed / 31556952.;
                    match yrs {
                        ..1000000. => {
                            return write!(f, "{:.1$} y", yrs, precision(yrs, 2));
                        }
                        ..1000000000. => {
                            let mega = yrs / 1000000.;
                            return write!(f, "{:.1$} My", mega, precision(mega, 2));
                        }
                        ..1000000000000. => {
                            let giga = yrs / 1000000000.;
                            return write!(f, "{:.1$} Gy", giga, precision(giga, 2));
                        }
                        ..1000000000000000. => {
                            let tera = yrs / 1000000000000.;
                            return write!(f, "{:.1$} Ty", tera, precision(tera, 2));
                        }
                        _ => (),
                    }
                }
            }
        }
        if self.0.exponent() > 0 {
            let yrs = self.0 / 31556952_f64.into();
            write!(f, "{} y", yrs)
        } else {
            write!(f, "{} s", self.0)
        }
    }
}

impl<T: Into<ENumber>> From<T> for TimeScale {
    fn from(value: T) -> Self {
        TimeScale(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_scale_format() {
        let tests: Vec<(TimeScale, &str)> = vec![((5.39, -44).into(), "5.39e-44 s")];

        tests
            .iter()
            .for_each(|test| assert_eq!(format!("{}", test.0), test.1));
    }
}
