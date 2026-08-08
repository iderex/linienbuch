//! Where a value and its uncertainty become text, and the only place.
//!
//! `docs/decisions/rounding.md` carries the argument, the significant figures
//! policy and the reason for each of the three properties below. This file is
//! the implementation of that record and does not restate it.
//!
//! An uncertainty is rounded away from zero. The number of significant figures
//! is fixed here and applied to every uncertainty whatever it was derived from.
//! A value is shown to the place its uncertainty reaches and no further.
//!
//! Nothing else in this crate turns an uncertainty into text.
//! `tests/uncertainty_formatting.rs` refuses a second path that does, which is
//! the second half of #40's done condition and one of the invariants #50 names.
//!
//! What this file does not decide is what an answer looks like on the wire.
//! That is #44, and the parts of a rendering are readable here one at a time so
//! that a format can assemble them rather than round again.

use std::fmt;

use crate::register::uncertainty::Uncertainty;

/// How many significant figures an uncertainty is shown to.
///
/// Two, and the argument is in `docs/decisions/rounding.md`. It is a constant
/// rather than a parameter because a caller that may choose gets to choose the
/// coarser one, and the request to choose it arrives as a reasonable one about
/// display.
pub const FIGURES: usize = 2;

/// A value that cannot be rendered, because it is not a number.
///
/// `Claim::value` is a bare `f64` and nothing on the way in refuses an infinity
/// or a NaN, so this path meets one eventually. Printing `NaN` beside a rounded
/// uncertainty would read as a value with a known precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotANumber;

impl fmt::Display for NotANumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a value that is not finite has no rendering")
    }
}

/// A value and its uncertainty as text, with the parts kept separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    value: String,
    halves: Option<(String, String)>,
}

impl Rendered {
    /// The value, rounded to the place its uncertainty reaches.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// The lower half, rounded away from zero, where the source quoted one.
    pub fn minus(&self) -> Option<&str> {
        self.halves.as_ref().map(|(minus, _)| minus.as_str())
    }

    /// The upper half, rounded away from zero, where the source quoted one.
    pub fn plus(&self) -> Option<&str> {
        self.halves.as_ref().map(|(_, plus)| plus.as_str())
    }

    /// Whether the source quoted no uncertainty.
    ///
    /// A rendering of an absent uncertainty says so rather than leaving the
    /// place empty, because an empty place reads as a small number.
    pub fn is_absent(&self) -> bool {
        self.halves.is_none()
    }
}

impl fmt::Display for Rendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.halves {
            None => write!(f, "{} (no uncertainty quoted)", self.value),
            Some((minus, plus)) if minus == plus => write!(f, "{} +/- {}", self.value, plus),
            Some((minus, plus)) => write!(f, "{} +{} -{}", self.value, plus, minus),
        }
    }
}

/// The rule. A value and what a source said about how well it knows it, as text.
///
/// The uncertainty is rounded away from zero to [`FIGURES`] significant figures
/// and the value is rounded to nearest at the place the uncertainty ends. Where
/// the two halves end at different places the finer of the two is used for
/// both, so neither half is shown coarser than it is.
pub fn render(value: f64, uncertainty: Uncertainty) -> Result<Rendered, NotANumber> {
    if !value.is_finite() {
        return Err(NotANumber);
    }
    let Uncertainty::Quoted { minus, plus } = uncertainty else {
        return Ok(Rendered {
            value: shortest(value),
            halves: None,
        });
    };

    // A half of exactly zero says the value is exact in that direction. It
    // constrains no decimal place, so it does not get a vote on where the
    // rendering stops, and two zero halves leave the value unconstrained.
    let ends = [minus, plus]
        .into_iter()
        .filter(|half| *half > 0.0)
        .map(last_place)
        .min();
    let Some(at) = ends else {
        return Ok(Rendered {
            value: shortest(value),
            halves: Some(("0".to_owned(), "0".to_owned())),
        });
    };

    Ok(Rendered {
        value: place(value, at, Direction::Nearest),
        halves: Some((
            place(minus, at, Direction::AwayFromZero),
            place(plus, at, Direction::AwayFromZero),
        )),
    })
}

/// Which way a digit that has to move is allowed to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    /// For an uncertainty. Never toward zero, which is the whole of #40.
    AwayFromZero,
    /// For a value. Away from zero here would be a bias in the value itself,
    /// which is a different defect and not a safer version of this one.
    Nearest,
}

/// A finite decimal: the digits, and the power of ten the last one stands for.
///
/// The digits come from the shortest decimal that reads back as the same `f64`
/// rather than from an expansion of the binary value. That is what makes
/// rounding away from zero mean what a reader means by it: the expansion of
/// `0.0013` has a tail of digits below the two it was written with, and rounding
/// away from zero over that tail would report `0.0014` for a number nobody wrote
/// as anything but `0.0013`.
struct Decimal {
    digits: Vec<u8>,
    at: i32,
}

impl Decimal {
    /// The magnitude of a finite `f64`, exactly.
    fn of(magnitude: f64) -> Self {
        let text = format!("{magnitude:e}");
        let (mantissa, exponent) = text
            .split_once('e')
            .expect("exponential formatting of an f64 carries an exponent");
        let exponent: i32 = exponent
            .parse()
            .expect("the exponent of an f64 is a decimal integer");
        let digits: Vec<u8> = mantissa
            .bytes()
            .filter(u8::is_ascii_digit)
            .map(|digit| digit - b'0')
            .collect();
        let at = exponent - (digits.len() as i32 - 1);
        Decimal { digits, at }
    }

    /// The power of ten the leading digit stands for.
    fn leading_place(&self) -> i32 {
        self.at + self.digits.len() as i32 - 1
    }

    /// The same number with its last digit at `at`, moved in `direction`.
    fn rounded(&self, at: i32, direction: Direction) -> Decimal {
        if at <= self.at {
            let mut digits = self.digits.clone();
            digits.resize(digits.len() + (self.at - at) as usize, 0);
            return Decimal { digits, at };
        }

        let dropped = (at - self.at) as usize;
        let keep = self.digits.len().saturating_sub(dropped);
        let mut digits = if keep == 0 {
            vec![0]
        } else {
            self.digits[..keep].to_vec()
        };
        let up = match direction {
            // Any digit at all below the position asked for. Nothing is lost
            // downward, including the digit somebody would call negligible.
            Direction::AwayFromZero => self.digits[keep..].iter().any(|digit| *digit != 0),
            // The digit one place below, where the number reaches that far. A
            // number lying entirely below the position has a zero there.
            Direction::Nearest => {
                if keep > 0 {
                    self.digits[keep] >= 5
                } else if dropped == self.digits.len() {
                    self.digits[0] >= 5
                } else {
                    false
                }
            }
        };
        if up {
            increment(&mut digits);
        }
        Decimal { digits, at }
    }

    fn any_nonzero(&self) -> bool {
        self.digits.iter().any(|digit| *digit != 0)
    }

    fn is_zero(&self) -> bool {
        !self.any_nonzero()
    }

    /// Positional decimal text, with no sign.
    fn text(&self) -> String {
        let digits: String = self.digits.iter().map(|d| (d + b'0') as char).collect();
        if self.at >= 0 {
            return format!("{digits}{}", "0".repeat(self.at as usize));
        }
        let places = (-self.at) as usize;
        if digits.len() > places {
            let (whole, fraction) = digits.split_at(digits.len() - places);
            format!("{whole}.{fraction}")
        } else {
            format!("0.{}{digits}", "0".repeat(places - digits.len()))
        }
    }
}

/// One added at the last digit, carried leftward.
fn increment(digits: &mut Vec<u8>) {
    for digit in digits.iter_mut().rev() {
        if *digit < 9 {
            *digit += 1;
            return;
        }
        *digit = 0;
    }
    digits.insert(0, 1);
}

/// The power of ten a positive half's last significant digit stands for, after
/// the rounding rather than before it.
///
/// Rounding away from zero can carry out of the leading digit, and then the
/// figures the policy asks for sit one place higher than they did: 99.01 at two
/// figures is 100, whose second significant digit is at tens and not at units.
/// Reading the place off the number before it is rounded would leave the value
/// beside it showing a digit its uncertainty no longer reaches.
fn last_place(half: f64) -> i32 {
    let decimal = Decimal::of(half);
    let at = decimal.leading_place() - (FIGURES as i32 - 1);
    if decimal.rounded(at, Direction::AwayFromZero).digits.len() > FIGURES {
        at + 1
    } else {
        at
    }
}

/// `number` with its last digit at `at`.
fn place(number: f64, at: i32, direction: Direction) -> String {
    let rounded = Decimal::of(number.abs()).rounded(at, direction);
    let text = rounded.text();
    if number < 0.0 && !rounded.is_zero() {
        format!("-{text}")
    } else {
        text
    }
}

/// A number written out with no rounding, for the cases where no uncertainty
/// says where to stop.
fn shortest(number: f64) -> String {
    let decimal = Decimal::of(number.abs());
    let text = decimal.text();
    if number < 0.0 && !decimal.is_zero() {
        format!("-{text}")
    } else {
        text
    }
}
