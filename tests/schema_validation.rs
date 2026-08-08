//! Every constraint the register can refuse has a fixture that trips exactly it,
//! and a neighbour one change away that it does not trip.
//!
//! A guard exercised only by a fixture violating five things at once cannot tell
//! you which of the five it catches, and one of them can quietly stop working.
//! So each case below is built to trip one constraint, the test asserts that it
//! trips that one and no other, and its neighbour is the same case with the one
//! violation repaired.
//!
//! The last test in this file is the one that keeps this true as the register
//! grows. It compares the constraints the type can refuse against the
//! constraints the cases above actually tripped, and reds the suite if a
//! constraint has no case. Adding a variant to `Refused` does not compile until
//! it is named, and naming it does not pass until a fixture reaches it.
//!
//! What this file does not cover is stated at the end and is not left to be
//! discovered.

use linienbuch::register::claims::{
    Ancestor, Calibration, Claim, Claims, Derivation, Edge, Method, QuantityId, Refused, SubjectId,
    Unit,
};
use linienbuch::register::provenance::{ClaimId, Digest, DigestAlgorithm, ReferenceId, SourceId};
use linienbuch::register::uncertainty::Uncertainty;
use std::collections::{BTreeMap, BTreeSet};

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
    BTreeMap::from([("effective-temperature-k".to_owned(), 5772.0)])
}

fn reference() -> ReferenceId {
    ReferenceId::Doi("10.1000/parameters".to_owned())
}

fn quotes(from: &str, to: &str) -> Edge {
    Edge {
        from: ClaimId::new(from),
        to: Ancestor::Claim(ClaimId::new(to)),
        derivation: Derivation::Quotation,
    }
}

/// One case: what it does, and what it is expected to trip.
struct Case {
    name: &'static str,
    /// Run the case and return every refusal it produced.
    run: fn() -> Vec<Refused>,
    /// The same case with the one violation repaired.
    neighbour: fn() -> Vec<Refused>,
    /// The one constraint this case is about.
    constraint: &'static str,
}

fn a_calibration_with_no_reference_object() -> Vec<Refused> {
    Calibration::new("   ", parameters(), reference())
        .err()
        .into_iter()
        .collect()
}

fn a_calibration_with_all_three() -> Vec<Refused> {
    Calibration::new("the Sun", parameters(), reference())
        .err()
        .into_iter()
        .collect()
}

fn a_compiled_claim_pointing_at_nothing() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("compiled", Method::Compiled))
        .expect("a claim is accepted");
    register.compiled_with_no_ancestor()
}

fn a_compiled_claim_that_quotes_something() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("compiled", Method::Compiled))
        .expect("a claim is accepted");
    register
        .add_edge(Edge {
            from: ClaimId::new("compiled"),
            to: Ancestor::Reference(reference()),
            derivation: Derivation::Quotation,
        })
        .expect("an edge to a reference is accepted");
    register.compiled_with_no_ancestor()
}

fn an_edge_from_a_claim_that_is_not_held() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("held", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    register
        .add_edge(quotes("absent", "held"))
        .err()
        .into_iter()
        .collect()
}

fn an_edge_from_a_claim_that_is_held() -> Vec<Refused> {
    let mut register = Claims::new();
    for id in ["held", "other"] {
        register
            .add(claim(id, Method::MeasuredInLaboratory))
            .expect("a claim is accepted");
    }
    register
        .add_edge(quotes("other", "held"))
        .err()
        .into_iter()
        .collect()
}

fn an_edge_to_a_claim_that_is_not_held() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("held", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    register
        .add_edge(quotes("held", "absent"))
        .err()
        .into_iter()
        .collect()
}

fn an_edge_to_a_reference_needs_no_claim() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("held", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    register
        .add_edge(Edge {
            from: ClaimId::new("held"),
            to: Ancestor::Reference(reference()),
            derivation: Derivation::Quotation,
        })
        .err()
        .into_iter()
        .collect()
}

fn one_identity_two_claims() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("one", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    let mut different = claim("one", Method::MeasuredInLaboratory);
    different.value = -0.29;
    register.add(different).err().into_iter().collect()
}

fn one_identity_added_twice_identically() -> Vec<Refused> {
    let mut register = Claims::new();
    register
        .add(claim("one", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    register
        .add(claim("one", Method::MeasuredInLaboratory))
        .err()
        .into_iter()
        .collect()
}

fn an_edge_that_closes_a_cycle() -> Vec<Refused> {
    let mut register = Claims::new();
    for id in ["a", "b"] {
        register
            .add(claim(id, Method::Compiled))
            .expect("a claim is accepted");
    }
    register
        .add_edge(quotes("a", "b"))
        .expect("a to b is accepted");
    register
        .add_edge(quotes("b", "a"))
        .err()
        .into_iter()
        .collect()
}

fn an_edge_that_closes_a_diamond() -> Vec<Refused> {
    let mut register = Claims::new();
    for id in ["top", "left", "right"] {
        register
            .add(claim(id, Method::Compiled))
            .expect("a claim is accepted");
    }
    register
        .add(claim("bottom", Method::MeasuredInLaboratory))
        .expect("a claim is accepted");
    for (from, to) in [("top", "left"), ("top", "right"), ("left", "bottom")] {
        register
            .add_edge(quotes(from, to))
            .unwrap_or_else(|e| panic!("{from} to {to} must be accepted, got {e}"));
    }
    // The edge that closes the diamond, which is the near miss for the cycle:
    // two paths meeting at one ancestor, which is what two sources quoting one
    // paper looks like.
    register
        .add_edge(quotes("right", "bottom"))
        .err()
        .into_iter()
        .collect()
}

/// One case per constraint, each with the neighbour that repairs it.
const CASES: [Case; 6] = [
    Case {
        name: "a calibration with no reference object",
        run: a_calibration_with_no_reference_object,
        neighbour: a_calibration_with_all_three,
        constraint: "a calibrated claim missing one of its three parts",
    },
    Case {
        name: "a compiled claim pointing at nothing",
        run: a_compiled_claim_pointing_at_nothing,
        neighbour: a_compiled_claim_that_quotes_something,
        constraint: "a compiled claim with no outgoing edge",
    },
    Case {
        name: "an edge from a claim that is not held",
        run: an_edge_from_a_claim_that_is_not_held,
        neighbour: an_edge_from_a_claim_that_is_held,
        constraint: "an edge from a claim the register does not hold",
    },
    Case {
        name: "an edge to a claim that is not held",
        run: an_edge_to_a_claim_that_is_not_held,
        neighbour: an_edge_to_a_reference_needs_no_claim,
        constraint: "an edge to a claim the register does not hold",
    },
    Case {
        name: "one identity holding two different claims",
        run: one_identity_two_claims,
        neighbour: one_identity_added_twice_identically,
        constraint: "one identity holding two different claims",
    },
    Case {
        name: "an edge that closes a cycle",
        run: an_edge_that_closes_a_cycle,
        neighbour: an_edge_that_closes_a_diamond,
        constraint: "a cycle in the provenance graph",
    },
];

/// Each case trips its own constraint and no other.
#[test]
fn every_case_refuses_exactly_the_constraint_it_is_about() {
    for case in &CASES {
        let refused: Vec<&'static str> = (case.run)().iter().map(Refused::constraint).collect();
        assert_eq!(
            refused,
            vec![case.constraint],
            "the case {:?} must refuse its own constraint and nothing else",
            case.name
        );
    }
}

/// Each neighbour is one change away and is refused by nothing.
#[test]
fn every_neighbour_is_refused_by_nothing() {
    for case in &CASES {
        let refused: Vec<&'static str> =
            (case.neighbour)().iter().map(Refused::constraint).collect();
        assert!(
            refused.is_empty(),
            "the neighbour of {:?} must not be refused, got {refused:?}",
            case.name
        );
    }
}

/// The one that keeps this file honest as the register grows.
///
/// A constraint added to `Refused` does not compile until it is named by
/// `constraint()`, and naming it does not pass this test until a case above
/// reaches it. Neither step is one somebody can skip quietly, which is the whole
/// of what this test buys.
#[test]
fn every_constraint_the_register_can_refuse_has_a_case() {
    let declared: BTreeSet<&'static str> = Refused::CONSTRAINTS.into_iter().collect();
    let covered: BTreeSet<&'static str> = CASES
        .iter()
        .flat_map(|case| (case.run)())
        .map(|refusal| refusal.constraint())
        .collect();

    let uncovered: Vec<&&'static str> = declared.difference(&covered).collect();
    assert!(
        uncovered.is_empty(),
        "constraints the register can refuse and no case reaches: {uncovered:?}"
    );

    // And the other direction, so a case naming a constraint that no longer
    // exists is caught rather than quietly passing over nothing.
    let unknown: Vec<&&'static str> = covered.difference(&declared).collect();
    assert!(
        unknown.is_empty(),
        "cases refusing something the register does not declare: {unknown:?}"
    );

    println!(
        "schema validation: {} constraint(s) declared, {} case(s), each with a neighbour",
        declared.len(),
        CASES.len()
    );
}

/// The declared list and the exhaustive match do not drift apart.
///
/// `CONSTRAINTS` is a list because Rust cannot enumerate an enum's variants, and
/// a list in a file drifts against the thing it describes. This is what stops it:
/// every case's refusal is named by the match, and every name is in the list.
#[test]
fn the_declared_list_holds_what_the_match_returns() {
    let declared: BTreeSet<&'static str> = Refused::CONSTRAINTS.into_iter().collect();
    assert_eq!(
        declared.len(),
        Refused::CONSTRAINTS.len(),
        "the declared list holds a name twice"
    );
    for case in &CASES {
        assert!(
            declared.contains(case.constraint),
            "the case {:?} names {:?}, which the register does not declare",
            case.name,
            case.constraint
        );
    }
}

/// What this file does not cover, printed by every run rather than left to be
/// discovered by whoever assumed a green run meant the whole list.
///
/// #25 names constraints across the whole record-model milestone. Three of them
/// have no record to be a constraint on yet, and a case for one of those would
/// be a case against a type nobody has written.
#[test]
fn the_constraints_that_have_no_record_yet_are_named() {
    let not_covered = [
        (
            "a transition missing a level",
            "there is no transition record, which is #22",
        ),
        (
            "a line position stored without its convention",
            "nothing stores a line position, which is #11's second half and #27",
        ),
        (
            "a species that does not parse into canonical form",
            "refused in the species parser and proved in tests/species_round_trip.rs, \
             which is a different file rather than a gap",
        ),
    ];

    println!("schema validation does not cover:");
    for (constraint, why) in not_covered {
        println!("  {constraint}: {why}");
    }

    // The list is prose and nothing reads it. What is refusable is that the
    // constraints this register does declare all have a case, which is the test
    // above.
    assert_eq!(not_covered.len(), 3);
}
