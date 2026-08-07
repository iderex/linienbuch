//! The published scale maps as published, and a converted value never claims to
//! know more than the source said.

use linienbuch::accuracy::{Converted, Grade, Letters, NotAGrade};

/// The whole table, spelled out here rather than read from the code it checks.
///
/// A test that asked the implementation for the scale and then compared it with
/// itself would pass on any scale at all. These eleven rows are the published
/// ones as issue #28 records them.
const PUBLISHED: [(&str, f64); 11] = [
    ("AAA", 0.3),
    ("AA", 1.0),
    ("A+", 2.0),
    ("A", 3.0),
    ("B+", 7.0),
    ("B", 10.0),
    ("C+", 18.0),
    ("C", 25.0),
    ("D+", 40.0),
    ("D", 50.0),
    ("E", 50.0),
];

#[test]
fn the_full_table_maps_as_published() {
    for (spelling, percent) in PUBLISHED {
        let grade = Grade::parse(spelling).unwrap_or_else(|e| panic!("{spelling:?}: {e}"));
        assert_eq!(
            grade.letters().published_percent(),
            percent,
            "{spelling:?} has the wrong published bound"
        );
        assert_eq!(grade.letters().spelling(), spelling);
    }

    assert_eq!(
        Letters::all().count(),
        PUBLISHED.len(),
        "the scale has a letter the published table does not"
    );
}

/// The assertion the decision turns on. A converted value never reports a
/// smaller uncertainty than the bound it came from, so the choice of reading the
/// bound as one standard uncertainty is checked rather than only written down.
///
/// A uniform distribution over the bound would give the bound divided by the
/// square root of three, which is about 58 per cent of it, and would fail here.
#[test]
fn a_converted_value_never_reports_less_uncertainty_than_its_bound() {
    for (spelling, _) in PUBLISHED {
        let grade = Grade::parse(spelling).expect("on the scale");
        match grade.convert() {
            Converted::Standard {
                percent,
                bound_percent,
                ..
            } => assert!(
                percent >= bound_percent,
                "{spelling:?} converted to {percent} against a bound of {bound_percent}"
            ),
            Converted::Unusable { .. } => assert_eq!(spelling, "E"),
        }
    }
}

/// The last grade has no upper bound, so it converts to no number rather than to
/// a large one. Handled by name and not by a fallback.
#[test]
fn the_unbounded_grade_converts_to_no_number() {
    let e = Grade::parse("E").expect("on the scale");
    assert_eq!(
        e.convert(),
        Converted::Unusable {
            worse_than_percent: 50.0
        }
    );
    assert!(!e.convert().usable_for_weighting());

    // The neighbour. One letter better, and it does convert.
    let d = Grade::parse("D").expect("on the scale");
    assert!(matches!(d.convert(), Converted::Standard { .. }));
    assert!(d.convert().usable_for_weighting());
}

/// A primed grade converts, and the number it converts to is marked as a lower
/// bound rather than an estimate, because the source says the true accuracy may
/// be worse and does not say by how much.
#[test]
fn a_primed_grade_converts_to_a_lower_bound() {
    let primed = Grade::parse("B'").expect("on the scale");
    let plain = Grade::parse("B").expect("on the scale");

    assert!(primed.is_primed());
    assert!(!plain.is_primed());

    match (primed.convert(), plain.convert()) {
        (
            Converted::Standard {
                percent: a,
                is_lower_bound: primed_is_bound,
                ..
            },
            Converted::Standard {
                percent: b,
                is_lower_bound: plain_is_bound,
                ..
            },
        ) => {
            assert_eq!(a, b, "the prime does not invent a different number");
            assert!(primed_is_bound);
            assert!(!plain_is_bound);
        }
        other => panic!("both should convert to a standard uncertainty, got {other:?}"),
    }

    assert!(!primed.convert().usable_for_weighting());
    assert!(plain.convert().usable_for_weighting());
}

/// The original grade survives whatever the conversion does with it, including
/// the surrounding whitespace it arrived in, so revisiting the convention is a
/// recomputation rather than a re-ingest.
#[test]
fn the_original_grade_survives_verbatim() {
    for text in ["AAA", "B+", "B+'", " C ", "E"] {
        let grade = Grade::parse(text).unwrap_or_else(|e| panic!("{text:?}: {e}"));
        assert_eq!(grade.verbatim(), text);
    }
}

/// An unrecognised suffix is refused rather than dropped. This is the whole of
/// the prime safety: a spelling this parser does not know reds the ingest
/// instead of quietly producing an unprimed grade.
#[test]
fn a_suffix_this_parser_does_not_know_is_refused_rather_than_dropped() {
    let refused: [(&str, NotAGrade); 5] = [
        ("B*", NotAGrade::UnknownSuffix("*".to_owned())),
        ("B++", NotAGrade::UnknownSuffix("+".to_owned())),
        ("AAAA", NotAGrade::UnknownSuffix("A".to_owned())),
        ("F", NotAGrade::UnknownLetters("F".to_owned())),
        ("", NotAGrade::Empty),
    ];

    for (text, expected) in refused {
        match Grade::parse(text) {
            Ok(grade) => panic!("{text:?} was accepted as {grade:?}"),
            Err(reason) => assert_eq!(reason, expected, "{text:?} refused wrongly"),
        }
    }
}

/// The neighbours of the refusals above, each one change away and each accepted.
#[test]
fn the_neighbours_of_the_refusals_are_accepted() {
    for text in ["B'", "B+", "AAA", "D", "A"] {
        Grade::parse(text).unwrap_or_else(|e| panic!("the neighbour {text:?} was refused: {e}"));
    }
}

/// The scale is ordered, and a worse letter never converts to a smaller
/// uncertainty than a better one. The near miss is the pair either side of a
/// plus, where a lexical comparison would put `B+` after `B`.
#[test]
fn a_worse_letter_never_converts_to_less_uncertainty() {
    let bounded: Vec<f64> = PUBLISHED
        .iter()
        .filter(|(spelling, _)| *spelling != "E")
        .map(
            |(spelling, _)| match Grade::parse(spelling).expect("on the scale").convert() {
                Converted::Standard { percent, .. } => percent,
                Converted::Unusable { .. } => unreachable!("E was filtered out"),
            },
        )
        .collect();

    for pair in bounded.windows(2) {
        assert!(pair[1] > pair[0], "the scale is not increasing at {pair:?}");
    }
}
