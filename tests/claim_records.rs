//! The claim, its method, and the two refusals the provenance graph rests on.
//!
//! Every fixture here trips exactly one refusal, and every refusal has a
//! neighbour one change away that is not refused. A guard exercised only by a
//! fixture violating several things at once cannot say which of them it catches,
//! and one of them can quietly stop working.

use linienbuch::register::claims::{
    Ancestor, Calibration, Claim, Claims, Derivation, Edge, Method, QuantityId, Refused, SubjectId,
    Unit,
};
use linienbuch::register::provenance::{ClaimId, Digest, DigestAlgorithm, ReferenceId, SourceId};
use linienbuch::register::uncertainty::Uncertainty;
use std::collections::BTreeMap;

fn digest() -> Digest {
    Digest::new(
        DigestAlgorithm::Sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )
    .expect("a well formed digest")
}

fn claim(id: &str, method: Method) -> Claim {
    Claim {
        id: ClaimId::new(id),
        quantity: QuantityId::new("log-gf"),
        about: SubjectId::new("fe-i-4045"),
        value: -0.28,
        unit: Unit::new("dex"),
        uncertainty: Uncertainty::symmetric(0.05).expect("a width"),
        method,
        year: Some(1988),
        source: SourceId::new("nist"),
        snapshot: digest(),
    }
}

fn parameters() -> BTreeMap<String, f64> {
    BTreeMap::from([
        ("effective-temperature-k".to_owned(), 5772.0),
        ("surface-gravity-log-cgs".to_owned(), 4.44),
    ])
}

/// The load bearing category. A compilation that points at nothing cannot be
/// told from an origin.
#[test]
fn a_compiled_claim_with_no_ancestor_is_refused() {
    let mut register = Claims::new();
    register
        .add(claim("compiled", Method::Compiled))
        .expect("a claim is accepted");

    assert_eq!(
        register.compiled_with_no_ancestor(),
        vec![Refused::CompiledWithNoAncestor(ClaimId::new("compiled"))],
        "a compiled claim pointing at nothing must be refused"
    );

    // The neighbour, one edge away and nothing else. The claim is unchanged.
    register
        .add_edge(Edge {
            from: ClaimId::new("compiled"),
            to: Ancestor::Reference(ReferenceId::Doi("10.1000/quoted".to_owned())),
            derivation: Derivation::Quotation,
        })
        .expect("an edge to a reference is accepted");
    assert!(
        register.compiled_with_no_ancestor().is_empty(),
        "the same claim with one edge must not be refused"
    );
}

/// The other neighbour. A measurement pointing at nothing is an origin, which is
/// what an origin looks like, and it is not refused.
#[test]
fn a_measurement_with_no_ancestor_is_not_refused() {
    let mut register = Claims::new();
    register
        .add(claim("measured", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    register
        .add(claim(
            "computed",
            Method::Computed {
                code: Some("cowan".to_owned()),
                approximation: None,
            },
        ))
        .expect("a claim is accepted");

    assert!(
        register.compiled_with_no_ancestor().is_empty(),
        "only the compiled category owes an ancestor"
    );
}

/// A number that came from itself came from nowhere.
#[test]
fn a_cycle_in_the_provenance_graph_is_refused() {
    let mut register = Claims::new();
    for id in ["a", "b", "c"] {
        register
            .add(claim(id, Method::Compiled))
            .expect("a claim is accepted");
    }

    register
        .add_edge(Edge {
            from: ClaimId::new("a"),
            to: Ancestor::Claim(ClaimId::new("b")),
            derivation: Derivation::Quotation,
        })
        .expect("a to b is accepted");
    register
        .add_edge(Edge {
            from: ClaimId::new("b"),
            to: Ancestor::Claim(ClaimId::new("c")),
            derivation: Derivation::UnitConversion,
        })
        .expect("b to c is accepted");

    // The edge that closes it.
    let closes = register.add_edge(Edge {
        from: ClaimId::new("c"),
        to: Ancestor::Claim(ClaimId::new("a")),
        derivation: Derivation::Quotation,
    });
    assert_eq!(
        closes,
        Err(Refused::Cycle(vec![
            ClaimId::new("c"),
            ClaimId::new("a"),
            ClaimId::new("b"),
            ClaimId::new("c"),
        ])),
        "the refusal names the claims on the cycle, in the order it walked them"
    );
}

/// The near miss for the cycle. A diamond is not a cycle, and a graph that
/// refused one would refuse the ordinary case of two claims quoting one paper.
#[test]
fn a_diamond_is_not_a_cycle() {
    let mut register = Claims::new();
    for id in ["top", "left", "right"] {
        register
            .add(claim(id, Method::Compiled))
            .expect("a claim is accepted");
    }
    // The bottom of a diamond is where the chain ends, so it is an origin and
    // owes no ancestor. A compiled claim there would be a leaf pointing at
    // nothing, which is the other refusal rather than this one.
    register
        .add(claim("bottom", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");

    for (from, to) in [
        ("top", "left"),
        ("top", "right"),
        ("left", "bottom"),
        ("right", "bottom"),
    ] {
        register
            .add_edge(Edge {
                from: ClaimId::new(from),
                to: Ancestor::Claim(ClaimId::new(to)),
                derivation: Derivation::Quotation,
            })
            .unwrap_or_else(|e| panic!("{from} to {to} must be accepted, got {e}"));
    }

    assert!(
        register.compiled_with_no_ancestor().is_empty(),
        "two paths meeting at one ancestor is the ordinary case of two sources quoting one paper"
    );
}

/// A claim deriving directly from itself is the shortest cycle and is refused
/// for the same reason.
#[test]
fn a_claim_that_derives_from_itself_is_refused() {
    let mut register = Claims::new();
    register
        .add(claim("alone", Method::Compiled))
        .expect("a claim is accepted");

    assert_eq!(
        register.add_edge(Edge {
            from: ClaimId::new("alone"),
            to: Ancestor::Claim(ClaimId::new("alone")),
            derivation: Derivation::Recalculation,
        }),
        Err(Refused::Cycle(vec![
            ClaimId::new("alone"),
            ClaimId::new("alone"),
        ]))
    );
}

/// An edge whose either end is not held is refused rather than stored and
/// resolved later.
#[test]
fn an_edge_to_or_from_a_claim_that_is_not_held_is_refused() {
    let mut register = Claims::new();
    register
        .add(claim("held", Method::Compiled))
        .expect("a claim is accepted");

    assert_eq!(
        register.add_edge(Edge {
            from: ClaimId::new("absent"),
            to: Ancestor::Claim(ClaimId::new("held")),
            derivation: Derivation::Quotation,
        }),
        Err(Refused::UnknownClaim(ClaimId::new("absent")))
    );
    assert_eq!(
        register.add_edge(Edge {
            from: ClaimId::new("held"),
            to: Ancestor::Claim(ClaimId::new("absent")),
            derivation: Derivation::Quotation,
        }),
        Err(Refused::UnknownAncestor(ClaimId::new("absent")))
    );

    // The neighbour: an edge to a reference needs no claim at the far end,
    // because a reference is where the chain leaves this register.
    assert!(
        register
            .add_edge(Edge {
                from: ClaimId::new("held"),
                to: Ancestor::Reference(ReferenceId::Bibcode("1988JPCRD..17S...1F".to_owned())),
                derivation: Derivation::Quotation,
            })
            .is_ok()
    );
}

/// The three parts of a calibration are required together, and each is refused
/// on its own.
#[test]
fn a_calibration_missing_any_of_its_three_parts_is_refused() {
    assert_eq!(
        Calibration::new(
            "  ",
            parameters(),
            ReferenceId::Doi("10.1000/parameters".to_owned())
        ),
        Err(Refused::CalibrationMissing("the reference object"))
    );
    assert_eq!(
        Calibration::new(
            "the Sun",
            BTreeMap::new(),
            ReferenceId::Doi("10.1000/parameters".to_owned())
        ),
        Err(Refused::CalibrationMissing(
            "the parameters assumed for the reference object"
        ))
    );
    assert_eq!(
        Calibration::new("the Sun", parameters(), ReferenceId::Local(String::new())),
        Err(Refused::CalibrationMissing(
            "where the assumed parameters came from"
        ))
    );

    // The neighbour, with all three. One field away from each of the above.
    let whole = Calibration::new(
        "the Sun",
        parameters(),
        ReferenceId::Doi("10.1000/parameters".to_owned()),
    )
    .expect("all three parts are given");
    assert_eq!(whole.reference_object(), "the Sun");
    assert_eq!(whole.assumed_parameters().len(), 2);
}

/// A calibrated claim carries the calibration in the method, so there is no
/// route to a marked claim that is missing it.
#[test]
fn a_calibrated_claim_carries_its_calibration() {
    let calibration = Calibration::new(
        "the Sun",
        parameters(),
        ReferenceId::Doi("10.1000/parameters".to_owned()),
    )
    .expect("all three parts are given");

    let mut register = Claims::new();
    register
        .add(claim("solar", Method::Calibrated(calibration.clone())))
        .expect("a claim is accepted");

    match &register
        .get(&ClaimId::new("solar"))
        .expect("the claim is held")
        .method
    {
        Method::Calibrated(held) => assert_eq!(held, &calibration),
        other => panic!("the mark must carry the calibration, got {other:?}"),
    }
}

/// A claim registered twice differently is refused rather than resolved.
#[test]
fn one_identity_may_not_hold_two_different_claims() {
    let mut register = Claims::new();
    register
        .add(claim("one", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");

    // The identical claim again changes nothing.
    register
        .add(claim("one", Method::MeasuredInLaboratory))
        .expect("the identical claim is accepted");
    assert_eq!(register.len(), 1);

    let mut different = claim("one", Method::MeasuredInLaboratory);
    different.value = -0.29;
    assert_eq!(
        register.add(different),
        Err(Refused::ClaimContradicted(ClaimId::new("one")))
    );
}

/// The derivation is typed, so a quotation and a recalculation are not one
/// thing.
#[test]
fn the_kind_of_derivation_survives() {
    let mut register = Claims::new();
    for id in ["quoted", "origin"] {
        register
            .add(claim(id, Method::Compiled))
            .expect("a claim is accepted");
    }
    register
        .add_edge(Edge {
            from: ClaimId::new("quoted"),
            to: Ancestor::Claim(ClaimId::new("origin")),
            derivation: Derivation::Renormalisation,
        })
        .expect("an edge is accepted");

    let ancestors = register.ancestors_of(&ClaimId::new("quoted"));
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].1, Derivation::Renormalisation);
    assert_ne!(ancestors[0].1, Derivation::Quotation);
}

/// A claim carries an absent uncertainty as a state, and no operation reads a
/// number out of it.
#[test]
fn a_claim_may_carry_no_uncertainty_and_it_is_not_a_number() {
    let mut without = claim("unquoted", Method::MeasuredInLaboratory);
    without.uncertainty = Uncertainty::Absent;

    assert!(without.uncertainty.widest().is_err());

    let mut register = Claims::new();
    register.add(without).expect("a claim is accepted");
    assert_eq!(register.len(), 1, "an absent uncertainty is not a refusal");
}
