//! What a transition record refuses, and the one number it can check itself
//! against.
//!
//! The numbers are the worked case in `docs/decisions/transition-identity.md`,
//! two Fe I rows sixteen milliangstrom apart in the dense optical forest, so
//! that the arithmetic here is checkable against a decision a reader can open
//! rather than against constants invented in this file.
//!
//! Every refusal below arrives with the neighbour that is one change away from
//! it and is not refused. Two of those changes are the mistake somebody
//! actually makes: the two levels passed the wrong way round, and a scale name
//! spelled slightly differently on the two ends of one transition.

use linienbuch::register::provenance::{ClaimId, SourceId};
use linienbuch::register::uncertainty::Uncertainty;
use linienbuch::spectroscopy::levels::{Energy, Level, Parity, TotalAngularMomentum};
use linienbuch::spectroscopy::species::{Element, Species};
use linienbuch::spectroscopy::transitions::{
    Agreement, Kind, NotAPosition, Position, Refused, Transition, VacuumWavenumber,
};

fn element(symbol: &str) -> Element {
    Element::from_symbol(symbol).expect("a symbol in the table")
}

fn species(symbol: &str, charge: u8) -> Species {
    Species::atom(element(symbol), charge).expect("a species")
}

fn iron() -> Species {
    species("Fe", 0)
}

/// A level of one species, on a named scale, at a named energy.
fn level(species: Species, scale: &str, value: f64, doubled_j: u16) -> Level {
    Level {
        species,
        energy: Energy {
            value,
            uncertainty: Uncertainty::symmetric(0.005).expect("a width"),
            zero: SourceId::new(scale),
        },
        j: Some(TotalAngularMomentum::from_doubled(doubled_j)),
        parity: Some(Parity::Even),
        configuration: Some("3d6.4s2".to_owned()),
        term: Some("a5D".to_owned()),
    }
}

/// The lower and upper levels of row A1 of the worked case, both on one scale.
fn a1() -> (Level, Level) {
    (
        level(iron(), "nist", 0.0, 8),
        level(iron(), "nist", 19999.60, 6),
    )
}

fn position(cm_inverse: f64, uncertainty: Uncertainty, claim: &str) -> Position {
    Position {
        wavenumber: VacuumWavenumber::new(cm_inverse).expect("a position"),
        uncertainty,
        claim: ClaimId::new(claim),
    }
}

fn width(of: f64) -> Uncertainty {
    Uncertainty::symmetric(of).expect("a width")
}

/// A transition has both of its levels and both are readable.
///
/// The clause this is half of is held by the type rather than by this test: the
/// two levels are arguments to the only constructor and are not writable, so a
/// transition with one level is not a value that exists. What a test can show
/// is that neither was quietly dropped on the way in.
#[test]
fn a_transition_carries_both_of_its_levels() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower.clone(), upper.clone(), Kind::Component)
        .expect("two Fe I levels on one scale");

    assert_eq!(transition.lower(), &lower);
    assert_eq!(transition.upper(), &upper);
    assert_eq!(transition.species(), iron());
    assert_eq!(transition.kind(), &Kind::Component);
}

/// Two levels of different species are not a transition, and the neighbour is
/// the same pair with the charge that made them different put back.
#[test]
fn the_two_levels_belong_to_one_species() {
    let neutral = level(iron(), "nist", 0.0, 8);
    let ionised = level(species("Fe", 1), "nist", 19999.60, 6);

    assert_eq!(
        Transition::new(neutral.clone(), ionised, Kind::Component),
        Err(Refused::LevelsOfDifferentSpecies {
            lower: iron(),
            upper: species("Fe", 1),
        })
    );

    let same = level(iron(), "nist", 19999.60, 6);
    assert!(Transition::new(neutral, same, Kind::Component).is_ok());
}

/// Two energies measured from different zeros do not subtract, and the
/// neighbour is one scale name spelled the same on both ends.
#[test]
fn the_two_energies_are_on_one_scale() {
    let lower = level(iron(), "nist", 0.0, 8);
    let elsewhere = level(iron(), "nist-asd", 19999.60, 6);

    assert_eq!(
        Transition::new(lower.clone(), elsewhere, Kind::Component),
        Err(Refused::EnergiesOnDifferentScales {
            lower: SourceId::new("nist"),
            upper: SourceId::new("nist-asd"),
        })
    );

    let same_scale = level(iron(), "nist", 19999.60, 6);
    assert!(Transition::new(lower, same_scale, Kind::Component).is_ok());
}

/// The two levels the wrong way round is refused rather than sorted.
///
/// This is the near miss worth having. The two arguments have the same type, so
/// swapping them compiles, and a record that sorted them would take the swapped
/// row and produce a transition indistinguishable from a correct one.
#[test]
fn the_upper_level_is_above_the_lower_one() {
    let (lower, upper) = a1();

    assert_eq!(
        Transition::new(upper.clone(), lower.clone(), Kind::Component),
        Err(Refused::UpperNotAboveLower {
            lower: 19999.60,
            upper: 0.0,
        })
    );

    assert!(Transition::new(lower, upper, Kind::Component).is_ok());
}

/// Two levels at one energy are refused, and the neighbour is the pair a
/// hundredth of a wavenumber apart, which is not.
#[test]
fn two_levels_at_one_energy_are_not_a_transition() {
    let lower = level(iron(), "nist", 19999.60, 8);
    let same = level(iron(), "nist", 19999.60, 6);

    assert_eq!(
        Transition::new(lower.clone(), same, Kind::Component),
        Err(Refused::UpperNotAboveLower {
            lower: 19999.60,
            upper: 19999.60,
        })
    );

    let just_above = level(iron(), "nist", 19999.61, 6);
    assert!(Transition::new(lower, just_above, Kind::Component).is_ok());
}

/// An energy that is not a number does not compare, so the pair has no ordering
/// and is refused there rather than reaching the arithmetic.
#[test]
fn an_energy_that_is_not_a_number_is_refused() {
    let lower = level(iron(), "nist", 0.0, 8);
    let unordered = level(iron(), "nist", f64::NAN, 6);

    assert!(matches!(
        Transition::new(lower.clone(), unordered, Kind::Component),
        Err(Refused::UpperNotAboveLower { .. })
    ));

    let ordered = level(iron(), "nist", 19999.60, 6);
    assert!(Transition::new(lower, ordered, Kind::Component).is_ok());
}

/// Two finite energies whose difference is not finite are refused as a
/// position, which is the case the ordering above lets through.
#[test]
fn a_difference_that_is_not_a_position_is_refused() {
    let lower = level(iron(), "nist", -f64::MAX, 8);
    let upper = level(iron(), "nist", f64::MAX, 6);

    assert_eq!(
        Transition::new(lower, upper.clone(), Kind::Component),
        Err(Refused::NotAPosition(NotAPosition::NotFinite))
    );

    let representable = level(iron(), "nist", 0.0, 8);
    assert!(Transition::new(representable, upper, Kind::Component).is_ok());
}

/// A position is a finite number above zero, and nothing else is one.
#[test]
fn a_position_is_a_finite_wavenumber_above_zero() {
    assert_eq!(
        VacuumWavenumber::new(f64::INFINITY),
        Err(NotAPosition::NotFinite)
    );
    assert_eq!(VacuumWavenumber::new(0.0), Err(NotAPosition::NotAboveZero));
    assert_eq!(VacuumWavenumber::new(-1.0), Err(NotAPosition::NotAboveZero));

    let smallest = VacuumWavenumber::new(f64::MIN_POSITIVE).expect("above zero");
    assert_eq!(smallest.cm_inverse(), f64::MIN_POSITIVE);
}

/// A multiplet carries the components the source lists, and one of them may not
/// itself be a multiplet.
#[test]
fn a_multiplet_holds_components_and_not_multiplets() {
    let (lower, upper) = a1();
    let component =
        Transition::new(lower.clone(), upper.clone(), Kind::Component).expect("a component");
    let inner = Transition::new(
        lower.clone(),
        upper.clone(),
        Kind::Multiplet { components: None },
    )
    .expect("an unresolved row that lists nothing");

    assert_eq!(
        Transition::new(
            lower.clone(),
            upper.clone(),
            Kind::Multiplet {
                components: Some(vec![inner]),
            },
        ),
        Err(Refused::MultipletInsideAMultiplet)
    );

    assert!(
        Transition::new(
            lower,
            upper,
            Kind::Multiplet {
                components: Some(vec![component]),
            },
        )
        .is_ok()
    );
}

/// A multiplet's components belong to the multiplet's own species.
#[test]
fn a_multiplet_holds_components_of_its_own_species() {
    let (lower, upper) = a1();
    let foreign = Transition::new(
        level(species("Ni", 0), "nist", 0.0, 8),
        level(species("Ni", 0), "nist", 19999.60, 6),
        Kind::Component,
    )
    .expect("a Ni I component");

    assert_eq!(
        Transition::new(
            lower.clone(),
            upper.clone(),
            Kind::Multiplet {
                components: Some(vec![foreign]),
            },
        ),
        Err(Refused::ComponentOfAnotherSpecies {
            multiplet: iron(),
            component: species("Ni", 0),
        })
    );

    let own = Transition::new(lower.clone(), upper.clone(), Kind::Component).expect("a component");
    assert!(
        Transition::new(
            lower,
            upper,
            Kind::Multiplet {
                components: Some(vec![own]),
            },
        )
        .is_ok()
    );
}

/// An unresolved row that lists nothing is a different state from one that
/// lists an empty set.
#[test]
fn a_multiplet_listing_nothing_is_not_a_multiplet_listing_none() {
    let (lower, upper) = a1();

    let unlisted = Transition::new(
        lower.clone(),
        upper.clone(),
        Kind::Multiplet { components: None },
    )
    .expect("an unresolved row");
    let listed_none = Transition::new(
        lower,
        upper,
        Kind::Multiplet {
            components: Some(Vec::new()),
        },
    )
    .expect("an unresolved row listing no components");

    assert_ne!(unlisted.kind(), listed_none.kind());
}

/// The observed and the Ritz position are separate fields pointing at separate
/// claims, and neither is reachable through the other.
#[test]
fn observed_and_ritz_are_separate_fields_with_separate_provenance() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_observed(position(19999.58, width(0.02), "nist:observed:fe1-5000"))
        .with_ritz(position(19999.60, width(0.01), "nist:ritz:fe1-5000"));

    let observed = transition.observed().expect("an observed position");
    let ritz = transition.ritz().expect("a Ritz position");

    assert_ne!(observed.claim, ritz.claim);
    assert_ne!(observed.wavenumber, ritz.wavenumber);
    assert_ne!(observed.uncertainty, ritz.uncertainty);
    assert!(transition.position_not_stated().is_none());
}

/// A row that did not say which of the two it published is recorded as not
/// saying, and is filed under neither.
#[test]
fn a_position_the_source_did_not_describe_is_filed_under_neither() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_position_not_stated(position(19999.60, Uncertainty::Absent, "kurucz:fe1-5000"));

    assert!(transition.observed().is_none());
    assert!(transition.ritz().is_none());
    assert_eq!(
        transition
            .position_not_stated()
            .expect("a position")
            .wavenumber
            .cm_inverse(),
        19999.60
    );
}

/// A Ritz position recomputed from the stored level energies reproduces the
/// stored value inside the width the source stated on it.
///
/// Row A1 of the worked case: lower 0.000 cm^-1, upper 19999.60 cm^-1, so the
/// difference is the position the source published for it.
#[test]
fn a_recomputed_ritz_position_reproduces_the_stored_one() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_ritz(position(19999.60, width(0.002), "nist:ritz:fe1-5000"));

    assert_eq!(transition.ritz_from_levels().cm_inverse(), 19999.60);
    assert_eq!(
        transition.ritz_agrees(),
        Agreement::Within {
            difference: 0.0,
            stated: 0.002,
        }
    );
}

/// The second row of the worked case, where the two energies are not round
/// numbers and the recomputation is not exact in binary.
#[test]
fn the_recomputation_holds_where_the_arithmetic_is_not_exact() {
    let lower = level(iron(), "nist", 415.933, 6);
    let upper = level(iron(), "nist", 20415.55, 4);
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_ritz(position(19999.617, width(0.001), "nist:ritz:fe1-5000-116"));

    let recomputed = transition.ritz_from_levels().cm_inverse();
    assert!((recomputed - 19999.617).abs() < 1e-9);
    assert!(matches!(transition.ritz_agrees(), Agreement::Within { .. }));
}

/// A stored Ritz position further from the levels than its own stated width is
/// reported as beyond it rather than accepted.
///
/// The neighbour is the test above, whose only change is the stored number.
#[test]
fn a_ritz_position_outside_its_stated_width_is_reported_as_beyond() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_ritz(position(19999.65, width(0.002), "nist:ritz:fe1-5000"));

    match transition.ritz_agrees() {
        Agreement::Beyond { difference, stated } => {
            assert!((difference - 0.05).abs() < 1e-9);
            assert_eq!(stated, 0.002);
        }
        other => {
            panic!("a position 0.05 cm^-1 out with a width of 0.002 is beyond it, got {other:?}")
        }
    }
}

/// The difference is reported with its sign, so a comparison that subtracted
/// the two the other way round is visible rather than absorbed.
#[test]
fn the_difference_carries_its_sign() {
    let (lower, upper) = a1();
    let below = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_ritz(position(19999.55, width(0.002), "nist:ritz:fe1-5000"));

    match below.ritz_agrees() {
        Agreement::Beyond { difference, .. } => assert!(difference < 0.0),
        other => panic!("a stored position below the recomputed one is beyond, got {other:?}"),
    }
}

/// A source that stated no width on its Ritz position gets no verdict, and the
/// difference is reported anyway.
#[test]
fn a_ritz_position_with_no_stated_width_gets_no_verdict() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_ritz(position(
            19999.65,
            Uncertainty::Absent,
            "kurucz:ritz:fe1-5000",
        ));

    match transition.ritz_agrees() {
        Agreement::NoStatedWidth { difference } => assert!((difference - 0.05).abs() < 1e-9),
        other => panic!("an absent width is not a width of zero, got {other:?}"),
    }
}

/// A transition with no stored Ritz position says so rather than agreeing
/// perfectly with nothing.
#[test]
fn no_stored_ritz_position_is_its_own_answer() {
    let (lower, upper) = a1();
    let transition = Transition::new(lower, upper, Kind::Component)
        .expect("a component")
        .with_observed(position(19999.58, width(0.02), "nist:observed:fe1-5000"));

    assert_eq!(transition.ritz_agrees(), Agreement::NoStoredRitz);
}
