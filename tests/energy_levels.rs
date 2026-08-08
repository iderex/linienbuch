//! What a level record keeps, and the one disagreement that is not many.
//!
//! Two sources can put every level of one species at a different number and
//! still agree completely about the spectrum, because the reference zero of the
//! energy scale is a property of the source. Reporting that as a mismatch per
//! level is the failure this file is mostly about: it buries one fact about a
//! scale under a list as long as the level set, and whoever reads the list
//! concludes the two sources disagree about everything.

use linienbuch::register::provenance::SourceId;
use linienbuch::register::uncertainty::{NoNumber, Refused, Uncertainty};
use linienbuch::spectroscopy::levels::{
    Comparison, Energy, EnergyZero, Level, Parity, Presence, TotalAngularMomentum, compare,
};
use linienbuch::spectroscopy::species::{Element, Species};

fn iron() -> Species {
    Species::atom(Element::from_symbol("Fe").expect("Fe is an element"), 0)
        .expect("a neutral atom is a species")
}

fn on(scale: &str, value: f64, uncertainty: Uncertainty) -> Energy {
    Energy {
        value,
        uncertainty,
        zero: SourceId::new(scale),
    }
}

/// A level with everything the sources supply, on a named scale.
fn full(scale: &str, value: f64, doubled_j: u16, configuration: &str) -> Level {
    Level {
        species: iron(),
        energy: on(scale, value, Uncertainty::symmetric(0.01).expect("a width")),
        j: Some(TotalAngularMomentum::from_doubled(doubled_j)),
        parity: Some(Parity::Even),
        configuration: Some(configuration.to_owned()),
        term: Some("a5D".to_owned()),
    }
}

/// Every optional field is separately present or absent, and what was present is
/// readable rather than inferable.
#[test]
fn each_field_is_separately_present_or_absent() {
    let everything = full("nist", 0.0, 8, "3d6.4s2");
    assert_eq!(
        everything.presence(),
        Presence {
            j: true,
            parity: true,
            configuration: true,
            term: true,
        }
    );

    // A source that gives the energy and J and nothing else. Not a level with
    // default values: a level about which four things are unknown, and the
    // record says which four.
    let sparse = Level {
        j: Some(TotalAngularMomentum::from_doubled(8)),
        parity: None,
        configuration: None,
        term: None,
        ..everything.clone()
    };
    assert_eq!(
        sparse.presence(),
        Presence {
            j: true,
            parity: false,
            configuration: false,
            term: false,
        }
    );

    // The difference is visible without comparing the levels themselves, which
    // is what lets a match record how strong it was.
    assert_ne!(everything.presence(), sparse.presence());
}

/// A level energy carries an uncertainty, and an absent one is not a number.
#[test]
fn an_energy_carries_its_uncertainty_and_an_absent_one_is_refused() {
    let measured = on(
        "nist",
        11976.2379,
        Uncertainty::symmetric(0.0004).expect("a width"),
    );
    assert_eq!(measured.uncertainty.widest(), Ok(0.0004));

    let unquoted = on("kurucz", 11976.24, Uncertainty::Absent);
    assert_eq!(
        unquoted.uncertainty.widest(),
        Err(NoNumber),
        "an operation that needs a number must be told there is none"
    );
    assert!(unquoted.uncertainty.is_absent());

    // The neighbour of the refusal: zero is a number a source can legitimately
    // quote and is not the same statement as quoting nothing.
    let exact = on("nist", 0.0, Uncertainty::symmetric(0.0).expect("a width"));
    assert_eq!(exact.uncertainty.widest(), Ok(0.0));
    assert!(!exact.uncertainty.is_absent());
}

/// An uncertainty that is not a width is refused before it exists.
#[test]
fn a_negative_or_infinite_uncertainty_is_refused() {
    assert_eq!(Uncertainty::symmetric(-0.1), Err(Refused::Negative));
    assert_eq!(
        Uncertainty::asymmetric(0.1, -0.1),
        Err(Refused::Negative),
        "either half is enough"
    );
    assert_eq!(Uncertainty::symmetric(f64::NAN), Err(Refused::NotFinite));
    assert_eq!(
        Uncertainty::symmetric(f64::INFINITY),
        Err(Refused::NotFinite)
    );

    // The neighbour, one sign away.
    assert!(Uncertainty::symmetric(0.1).is_ok());
}

/// An asymmetric uncertainty stays two numbers.
#[test]
fn an_asymmetric_uncertainty_is_not_collapsed() {
    let quoted = Uncertainty::asymmetric(0.02, 0.05).expect("two widths");
    assert_eq!(
        quoted,
        Uncertainty::Quoted {
            minus: 0.02,
            plus: 0.05
        }
    );
    assert_eq!(quoted.widest(), Ok(0.05));

    // Collapsing on the way in would have made this equal to the symmetric one,
    // and the direction the value is uncertain in would be gone.
    assert_ne!(quoted, Uncertainty::symmetric(0.05).expect("a width"));
}

/// The source's own zero is recorded, and it is not universal.
#[test]
fn the_energy_zero_is_a_property_of_the_source() {
    let states_a_limit = EnergyZero {
        source: SourceId::new("nist"),
        description: "the ground state of the neutral atom".to_owned(),
        ionisation_limit: Some(63737.704),
    };
    let states_none = EnergyZero {
        source: SourceId::new("kurucz"),
        description: "the ground state of the neutral atom".to_owned(),
        ionisation_limit: None,
    };

    assert!(states_a_limit.ionisation_limit.is_some());
    assert!(
        states_none.ionisation_limit.is_none(),
        "a source that stated no limit is a fact about the source, not a gap in \
         this record"
    );
    assert_ne!(states_a_limit.source, states_none.source);
}

/// The whole point. Two sources whose level sets differ only by a constant are
/// one finding rather than a list as long as the level set.
#[test]
fn a_constant_offset_is_detected_rather_than_reported_per_level() {
    let first = vec![
        full("nist", 0.0, 8, "3d6.4s2"),
        full("nist", 415.933, 6, "3d6.4s2"),
        full("nist", 704.007, 4, "3d6.4s2"),
        full("nist", 888.132, 2, "3d6.4s2"),
        full("nist", 978.074, 0, "3d6.4s2"),
    ];
    // The same spectrum on a scale whose zero sits 1000 cm^-1 lower.
    let second: Vec<Level> = first
        .iter()
        .map(|level| Level {
            energy: on(
                "other",
                level.energy.value + 1000.0,
                level.energy.uncertainty,
            ),
            ..level.clone()
        })
        .collect();

    let (found, unpaired) = compare(&first, &second, 0.001);
    match found {
        Comparison::ConstantOffset { offset, over_pairs } => {
            assert_eq!(over_pairs, 5, "five levels moved by one number");
            assert!(
                (offset - 1000.0).abs() < 1e-9,
                "the one finding must name the constant, got {offset}"
            );
        }
        other => panic!("five levels moved by one number is one finding, got {other:?}"),
    }
    assert_eq!(unpaired.only_in_first, 0);
    assert_eq!(unpaired.only_in_second, 0);
}

/// The near miss. One level off by more than the tolerance, and the answer is a
/// list of one rather than a list of five.
#[test]
fn one_level_that_does_not_fit_the_constant_is_the_only_one_reported() {
    let first = vec![
        full("nist", 0.0, 8, "3d6.4s2"),
        full("nist", 415.933, 6, "3d6.4s2"),
        full("nist", 704.007, 4, "3d6.4s2"),
        full("nist", 888.132, 2, "3d6.4s2"),
    ];
    let mut second: Vec<Level> = first
        .iter()
        .map(|level| Level {
            energy: on(
                "other",
                level.energy.value + 1000.0,
                level.energy.uncertainty,
            ),
            ..level.clone()
        })
        .collect();
    // One level moved by a further 4 cm^-1, which is a real disagreement about
    // that level rather than about the scale.
    second[2].energy.value += 4.0;

    let (found, _) = compare(&first, &second, 0.5);
    match found {
        Comparison::Disagreements {
            beyond_tolerance,
            over_pairs,
            ..
        } => {
            assert_eq!(over_pairs, 4);
            assert_eq!(
                beyond_tolerance.len(),
                1,
                "one level disagrees, so one level is reported, got {beyond_tolerance:?}"
            );
            assert_eq!(
                beyond_tolerance[0].j,
                Some(TotalAngularMomentum::from_doubled(4))
            );
        }
        other => panic!("expected a list of one, got {other:?}"),
    }
}

/// A level set that shares nothing says so rather than reporting agreement.
#[test]
fn nothing_paired_is_its_own_answer() {
    let first = vec![full("nist", 0.0, 8, "3d6.4s2")];
    let second = vec![full("other", 0.0, 8, "3d7.4s")];

    let (found, unpaired) = compare(&first, &second, 0.001);
    assert_eq!(
        found,
        Comparison::NothingPaired,
        "an empty list of mismatches over nothing reads exactly like agreement"
    );
    assert_eq!(unpaired.only_in_first, 1);
    assert_eq!(unpaired.only_in_second, 1);
}

/// A level present in one set only is counted rather than dropped.
#[test]
fn a_level_only_one_source_has_is_counted() {
    let first = vec![
        full("nist", 0.0, 8, "3d6.4s2"),
        full("nist", 415.933, 6, "3d6.4s2"),
    ];
    let second = vec![
        full("other", 1000.0, 8, "3d6.4s2"),
        full("other", 1415.933, 6, "3d6.4s2"),
        full("other", 2000.0, 2, "3d7.4s"),
    ];

    let (found, unpaired) = compare(&first, &second, 0.001);
    match found {
        Comparison::ConstantOffset { offset, over_pairs } => {
            assert_eq!(over_pairs, 2);
            assert!((offset - 1000.0).abs() < 1e-9, "got {offset}");
        }
        other => panic!("expected one constant, got {other:?}"),
    }
    assert_eq!(unpaired.only_in_first, 0);
    assert_eq!(
        unpaired.only_in_second, 1,
        "the level only the second source has is reported beside the comparison"
    );
}

/// The offset does not depend on the order the levels arrived in.
///
/// Taking the first difference would make two runs over one pair of sources
/// report different offsets for the same data, which is a reproducibility
/// failure in a board whose whole subject is reproducibility.
#[test]
fn the_offset_does_not_depend_on_the_order() {
    let first = vec![
        full("nist", 0.0, 8, "3d6.4s2"),
        full("nist", 415.933, 6, "3d6.4s2"),
        full("nist", 704.007, 4, "3d6.4s2"),
    ];
    let second: Vec<Level> = first
        .iter()
        .map(|level| Level {
            energy: on("other", level.energy.value + 12.5, level.energy.uncertainty),
            ..level.clone()
        })
        .collect();

    let (forwards, _) = compare(&first, &second, 0.001);
    let mut shuffled = second.clone();
    shuffled.reverse();
    let (backwards, _) = compare(&first, &shuffled, 0.001);
    assert_eq!(forwards, backwards);

    let mut first_shuffled = first.clone();
    first_shuffled.reverse();
    let (other_way, _) = compare(&first_shuffled, &second, 0.001);
    assert_eq!(forwards, other_way);
}

/// Pairing does not use the energy, which is what makes the constant visible.
///
/// A comparison that paired on the energy would find nothing in common between
/// two sources whose zeros differ, and would report every level as present in
/// one set only. That is the same failure wearing a different answer.
#[test]
fn pairing_does_not_use_the_energy() {
    let first = vec![full("nist", 0.0, 8, "3d6.4s2")];
    let second = vec![full("other", 99999.0, 8, "3d6.4s2")];

    let (found, unpaired) = compare(&first, &second, 0.001);
    assert_eq!(
        found,
        Comparison::ConstantOffset {
            offset: 99999.0,
            over_pairs: 1,
        },
        "a level far away on another scale is still the same level"
    );

    assert_eq!(unpaired.only_in_first, 0);
    assert_eq!(unpaired.only_in_second, 0);
}

/// Half integral J is exact, and prints the way a source writes it.
#[test]
fn a_half_integral_j_is_exact() {
    let five_halves = TotalAngularMomentum::from_doubled(5);
    let four = TotalAngularMomentum::from_doubled(8);
    assert_eq!(five_halves.to_string(), "5/2");
    assert_eq!(four.to_string(), "4");
    assert_eq!(five_halves.doubled(), 5);
    assert_ne!(five_halves, TotalAngularMomentum::from_doubled(6));
}
