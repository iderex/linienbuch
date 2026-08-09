//! The rendered uncertainty is never smaller than the one it was given.
//!
//! `docs/decisions/rounding.md` is the record. This file is the proof, and the
//! property it asserts is not that the rendering is close to the true number. It
//! is that the rendering is greater than or equal to it, for every input,
//! including the ones where that costs a digit. A test written as a tolerance
//! either side would pass a formatter that rounds down half the time, which is
//! the failure #40 exists to prevent.
//!
//! The property is asserted on the number a reader parses back out of the text
//! rather than on an intermediate, because the text is the whole of what leaves
//! this board.

use linienbuch::register::rounding::{FIGURES, render};
use linienbuch::register::uncertainty::Uncertainty;

/// Mantissas that between them reach every rounding decision two significant
/// figures can face: a digit below five, a digit at five, a digit above it, a
/// carry out of the leading digit, and a number already shorter than the
/// figures asked for.
const MANTISSAS: [f64; 18] = [
    1.0, 1.04, 1.05, 1.06, 1.234, 1.999, 2.5, 3.0, 4.44, 4.45, 5.0, 5.55, 6.789, 7.0, 9.0, 9.94,
    9.95, 9.999,
];

/// The decimal decades this sweep covers. Stated rather than implied: the
/// property is proven over these and not over the whole range of `f64`.
const DECADES: std::ops::RangeInclusive<i32> = -12..=12;

fn quoted(width: f64) -> Uncertainty {
    Uncertainty::symmetric(width).expect("a positive finite width is a legal uncertainty")
}

fn parse(text: &str) -> f64 {
    text.parse()
        .unwrap_or_else(|e| panic!("a rendering must read back as a number, got {text:?}: {e}"))
}

/// The whole of the rule, over every uncertainty in the swept range.
#[test]
fn a_rendered_uncertainty_is_never_smaller_than_the_one_it_was_given() {
    let mut examined = 0usize;
    for decade in DECADES {
        for mantissa in MANTISSAS {
            let width = mantissa * 10f64.powi(decade);
            for value in [0.0, width, -width, 41.37 * width, -7.62 * width] {
                let rendered = render(value, quoted(width)).expect("a finite value renders");
                let shown = parse(
                    rendered
                        .plus()
                        .expect("a quoted uncertainty has an upper half"),
                );
                assert!(
                    shown >= width,
                    "rendered {shown} for an uncertainty of {width}, \
                     which is smaller than what it was given"
                );
                assert_eq!(
                    rendered.minus(),
                    rendered.plus(),
                    "a symmetric uncertainty renders as the same number twice"
                );
                examined += 1;
            }
        }
    }
    assert!(examined > 0, "the sweep examined nothing");
    println!("rounding: {examined} value and uncertainty pair(s) examined");
}

/// The same, for an uncertainty whose two halves differ.
#[test]
fn both_halves_of_an_asymmetric_uncertainty_are_rounded_away_from_zero() {
    for decade in DECADES {
        for minus in MANTISSAS {
            for plus in MANTISSAS {
                let minus = minus * 10f64.powi(decade);
                let plus = plus * 10f64.powi(decade);
                let uncertainty = Uncertainty::asymmetric(minus, plus)
                    .expect("two positive finite widths are a legal uncertainty");
                let rendered = render(1.0, uncertainty).expect("a finite value renders");
                assert!(
                    parse(rendered.minus().expect("a lower half")) >= minus,
                    "the lower half was rounded toward zero"
                );
                assert!(
                    parse(rendered.plus().expect("an upper half")) >= plus,
                    "the upper half was rounded toward zero"
                );
            }
        }
    }
}

/// The rule differs from rounding to nearest, and here is where.
///
/// Without this, every assertion above would still pass under a formatter that
/// rounds to nearest, because most numbers round the same way either way. These
/// are the ones that do not, and the direction they move in is the issue.
#[test]
fn a_third_digit_that_nearest_would_discard_moves_the_second_one_up() {
    for (width, away, nearest) in [
        (0.0123_f64, "0.013", "0.012"),
        (1.201_f64, "1.3", "1.2"),
        (0.000_100_1_f64, "0.00011", "0.00010"),
        (99.01_f64, "100", "99"),
    ] {
        let rendered = render(0.0, quoted(width)).expect("a finite value renders");
        assert_eq!(
            rendered.plus(),
            Some(away),
            "an uncertainty of {width} must round away from zero"
        );
        assert_ne!(
            away, nearest,
            "the case for {width} does not separate the two rules"
        );
        assert!(
            parse(nearest) < width,
            "the neighbour {nearest} is only interesting if it is the smaller number"
        );
    }
}

/// A width that already has fewer digits than the policy asks for is not padded
/// upward. Rounding away from zero moves a digit that has something below it,
/// and there is nothing below this one.
#[test]
fn a_width_that_needs_no_rounding_is_not_moved() {
    for (width, expected) in [
        (0.013_f64, "0.013"),
        (2.0_f64, "2.0"),
        (150.0_f64, "150"),
        (0.5_f64, "0.50"),
    ] {
        let rendered = render(0.0, quoted(width)).expect("a finite value renders");
        assert_eq!(rendered.plus(), Some(expected), "for a width of {width}");
        assert_eq!(
            parse(rendered.plus().expect("an upper half")),
            width,
            "a width needing no rounding must render as itself"
        );
    }
}

/// A displayed value never implies more precision than its uncertainty supports.
#[test]
fn the_value_stops_where_the_uncertainty_stops() {
    for (value, width, expected) in [
        (1.234_567_f64, 0.0123_f64, "1.235"),
        (1.234_567_f64, 0.12_f64, "1.23"),
        (12_345.0_f64, 99.01_f64, "12350"),
        (-1.234_567_f64, 0.0123_f64, "-1.235"),
        (0.000_4_f64, 0.012_f64, "0.000"),
    ] {
        let rendered = render(value, quoted(width)).expect("a finite value renders");
        assert_eq!(
            rendered.value(),
            expected,
            "for a value of {value} against a width of {width}"
        );
    }
}

/// Over the whole sweep, and not only on the cases above: the value and the
/// uncertainty end at the same decimal place.
#[test]
fn the_value_and_the_uncertainty_end_at_one_place() {
    fn places(text: &str) -> Option<usize> {
        text.split_once('.').map(|(_, fraction)| fraction.len())
    }

    for decade in DECADES {
        for mantissa in MANTISSAS {
            let width = mantissa * 10f64.powi(decade);
            let rendered = render(1.0, quoted(width)).expect("a finite value renders");
            assert_eq!(
                places(rendered.value()),
                places(rendered.plus().expect("an upper half")),
                "value {} and uncertainty {} stop at different places",
                rendered.value(),
                rendered.plus().expect("an upper half")
            );
        }
    }
}

/// The value's own rounding is to nearest, in both directions.
///
/// Away from zero here would be a bias in the value, which is a different
/// defect rather than a safer version of the one this rule is about.
#[test]
fn the_value_is_rounded_to_nearest_rather_than_away_from_zero() {
    let width = quoted(0.12);
    assert_eq!(
        render(1.234, width).expect("renders").value(),
        "1.23",
        "a value with a small third digit rounds down"
    );
    assert_eq!(
        render(-1.234, width).expect("renders").value(),
        "-1.23",
        "and does so on the other side of zero too"
    );
    assert_eq!(
        render(1.236, width).expect("renders").value(),
        "1.24",
        "a value with a large third digit rounds up"
    );
}

/// An absent uncertainty is said rather than left blank.
#[test]
fn an_absent_uncertainty_renders_as_absent() {
    let rendered = render(1.234, Uncertainty::Absent).expect("a finite value renders");
    assert!(rendered.is_absent());
    assert_eq!(rendered.minus(), None);
    assert_eq!(rendered.plus(), None);
    assert_eq!(rendered.value(), "1.234", "with no place to round to");
    assert_eq!(rendered.to_string(), "1.234 (no uncertainty quoted)");
}

/// A width of exactly zero constrains no decimal place, so it does not silently
/// become one.
#[test]
fn a_width_of_zero_leaves_the_value_where_it_was() {
    let rendered = render(1.234_567, quoted(0.0)).expect("a finite value renders");
    assert_eq!(rendered.value(), "1.234567");
    assert_eq!(rendered.plus(), Some("0"));

    // One zero half and one that is not: the half that says something decides.
    let one_sided = Uncertainty::asymmetric(0.0, 0.012).expect("a legal uncertainty");
    let rendered = render(1.234_567, one_sided).expect("a finite value renders");
    assert_eq!(rendered.value(), "1.235");
    assert_eq!(rendered.minus(), Some("0.000"));
    assert_eq!(rendered.plus(), Some("0.012"));
}

/// A negative value keeps its sign.
///
/// The route with no uncertainty to stop at, which is the one nothing else in
/// this file reaches with a negative number. Dropping the sign here would
/// report a value on the other side of zero, and every assertion above would
/// still pass, because every value above is positive. Found by seeding faults
/// into the sign handling for #41.
#[test]
fn a_negative_value_with_no_uncertainty_keeps_its_sign() {
    let rendered = render(-1.5, Uncertainty::Absent).expect("a finite value renders");
    assert_eq!(rendered.value(), "-1.5");
    assert_eq!(rendered.to_string(), "-1.5 (no uncertainty quoted)");
}

/// A negative value that rounds to zero is shown as zero and not as minus zero.
///
/// The sign is suppressed when nothing of the number survives the rounding,
/// because a minus sign in front of a zero reads as a measurement that came out
/// just below zero rather than as one the uncertainty swallowed. This is the
/// only case in this file where the value and its sign disagree about whether
/// there is anything left.
#[test]
fn a_negative_value_that_rounds_away_is_not_shown_as_minus_zero() {
    let rendered = render(-0.006, quoted(10.0)).expect("a finite value renders");
    assert_eq!(rendered.value(), "0");
    assert_eq!(rendered.to_string(), "0 +/- 10");

    // The rendering carries the two halves, so it is not an absent one. Nothing
    // else here says so, and a rendering that reported itself absent would put
    // "no uncertainty quoted" in front of a reader who was given one.
    assert!(!rendered.is_absent());
    assert_eq!(rendered.minus(), Some("10"));
    assert_eq!(rendered.plus(), Some("10"));
}

/// A value lying entirely below the place it is rounded to goes to zero.
///
/// Nothing of the number reaches the last place that is kept, so there is no
/// digit to round on and the leading digit is not one. A rounding that read the
/// leading digit here would report `1` for a number two decades below the place
/// it was asked for, which is a plausible answer rather than an obviously wrong
/// one.
#[test]
fn a_value_below_the_place_it_is_rounded_to_is_zero() {
    let rendered = render(0.006, quoted(10.0)).expect("a finite value renders");
    assert_eq!(rendered.value(), "0");
}

/// A value one place below rounds to nearest on its own leading digit.
///
/// The neighbour of the case above, one decade higher, and the pair is the
/// point: `0.6` at the units place is `1` and `0.006` at the units place is
/// `0`. Both go through the branch where no digit is kept, and the two answers
/// differ, so a rounding that treats them alike is wrong on one of them
/// whichever way it decides.
#[test]
fn a_value_one_place_below_rounds_on_its_leading_digit() {
    let rendered = render(0.6, quoted(10.0)).expect("a finite value renders");
    assert_eq!(rendered.value(), "1");

    let below_the_half = render(0.4, quoted(10.0)).expect("a finite value renders");
    assert_eq!(below_the_half.value(), "0");
}

/// A value that is not a number has no rendering, rather than one that reads
/// like a measurement.
#[test]
fn a_value_that_is_not_finite_is_refused() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            render(value, quoted(0.1)).is_err(),
            "{value} must not render"
        );
    }
    // The neighbour, one value away: a finite value at the same magnitude does.
    assert!(render(f64::MAX, quoted(0.1)).is_ok());
}

/// The policy is two significant figures, and the tests above are written
/// against that number rather than against whatever the constant happens to say.
#[test]
fn the_figures_policy_is_the_one_the_record_states() {
    assert_eq!(FIGURES, 2);
}

/// The rendered forms, which #44 will assemble from the parts rather than
/// format again.
#[test]
fn the_default_rendering_distinguishes_the_three_cases() {
    let symmetric = Uncertainty::symmetric(0.012).expect("a legal uncertainty");
    assert_eq!(
        render(1.234_567, symmetric).expect("renders").to_string(),
        "1.235 +/- 0.012"
    );

    let asymmetric = Uncertainty::asymmetric(0.011, 0.0123).expect("a legal uncertainty");
    assert_eq!(
        render(1.234_567, asymmetric).expect("renders").to_string(),
        "1.235 +0.013 -0.011"
    );

    assert_eq!(
        render(1.234_567, Uncertainty::Absent)
            .expect("renders")
            .to_string(),
        "1.234567 (no uncertainty quoted)"
    );
}
