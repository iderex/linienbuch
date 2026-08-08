//! The three paths through the disjointness rule, and the one that must not be
//! taken by mistake.
//!
//! `docs/decisions/shared-ancestry.md` carries a worked example built so that
//! the refused case and the permitted case differ by one edge and by nothing
//! else. That pair is the fixture and its neighbour here, spelled with the same
//! names the record uses, so a reader can hold the two side by side.
//!
//! The third path is the one the record calls the whole point of the decision.
//! An unresolved chain is not a disjoint one, and a register that assumed
//! otherwise would be wrong most often in the case that is most common. It gets
//! two tests rather than one: that it is refused, and that it is not refused for
//! the other reason.

use linienbuch::register::ancestry::{NotIndependent, Terminal, ancestry_of, may_marginalise};
use linienbuch::register::claims::{
    Ancestor, Claim, Claims, Derivation, Edge, Method, QuantityId, SubjectId, Unit,
};
use linienbuch::register::provenance::{ClaimId, Digest, DigestAlgorithm, ReferenceId, SourceId};
use linienbuch::register::uncertainty::Uncertainty;

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

fn id(name: &str) -> ClaimId {
    ClaimId::new(name)
}

/// A register holding the named claims, each with the given method.
fn register(of: &[(&str, Method)]) -> Claims {
    let mut held = Claims::new();
    for (name, method) in of {
        held.add(claim(name, method.clone()))
            .expect("a claim is accepted");
    }
    held
}

fn quotes(held: &mut Claims, from: &str, to: &str) {
    held.add_edge(Edge {
        from: id(from),
        to: Ancestor::Claim(id(to)),
        derivation: Derivation::Quotation,
    })
    .expect("an edge between two held claims is accepted");
}

fn quotes_literature(held: &mut Claims, from: &str, doi: &str) {
    held.add_edge(Edge {
        from: id(from),
        to: Ancestor::Reference(ReferenceId::Doi(doi.to_owned())),
        derivation: Derivation::Quotation,
    })
    .expect("an edge to a reference is accepted");
}

/// The record's worked example, resolved and refused.
///
/// Claim one is read from source A, which attributes it to compilation C of
/// 1988. Claim two is read from source B, which attributes it to compilation D
/// of 1995. Nothing visible connects the two: two databases, two references, two
/// years. D resolves through C, so both rest on measurement M.
#[test]
fn two_claims_resting_on_one_measurement_are_refused_with_it_named() {
    let mut held = register(&[
        ("claim-one", Method::Compiled),
        ("claim-two", Method::Compiled),
        ("compilation-c-1988", Method::Compiled),
        ("compilation-d-1995", Method::Compiled),
        ("measurement-m-1982", Method::MeasuredInLaboratory),
    ]);
    quotes(&mut held, "claim-one", "compilation-c-1988");
    quotes(&mut held, "compilation-c-1988", "measurement-m-1982");
    quotes(&mut held, "claim-two", "compilation-d-1995");
    quotes(&mut held, "compilation-d-1995", "compilation-c-1988");

    let refused = may_marginalise(&held, &[id("claim-one"), id("claim-two")])
        .expect_err("two claims resting on one measurement must be refused");

    let NotIndependent::SharedAncestor {
        left,
        right,
        ancestor,
        through,
    } = &refused
    else {
        panic!("expected a shared ancestor, got {refused:?}");
    };
    assert_eq!(left, &id("claim-one"));
    assert_eq!(right, &id("claim-two"));
    assert_eq!(
        ancestor,
        &Terminal::Origin(id("measurement-m-1982")),
        "the refusal must name the measurement both chains rest on"
    );
    assert_eq!(
        through,
        &vec![id("compilation-c-1988")],
        "the refusal must name the node both chains pass through"
    );
    assert!(
        refused.to_string().contains("measurement-m-1982")
            && refused.to_string().contains("compilation-c-1988"),
        "the message must carry both, got {refused}"
    );
}

/// The neighbour, one edge away and nothing else.
///
/// D attributes to a second measurement instead of to C. The record's point is
/// that the difference is invisible at the level a caller works at, so a guard
/// that refused this too would have proved only that it refuses claims with
/// ancestors.
#[test]
fn the_same_case_with_one_edge_moved_is_permitted() {
    let mut held = register(&[
        ("claim-one", Method::Compiled),
        ("claim-two", Method::Compiled),
        ("compilation-c-1988", Method::Compiled),
        ("compilation-d-1995", Method::Compiled),
        ("measurement-m-1982", Method::MeasuredInLaboratory),
        ("measurement-n-1993", Method::MeasuredInLaboratory),
    ]);
    quotes(&mut held, "claim-one", "compilation-c-1988");
    quotes(&mut held, "compilation-c-1988", "measurement-m-1982");
    quotes(&mut held, "claim-two", "compilation-d-1995");
    quotes(&mut held, "compilation-d-1995", "measurement-n-1993");

    assert_eq!(
        may_marginalise(&held, &[id("claim-one"), id("claim-two")]),
        Ok(()),
        "disjoint and resolved ancestry must proceed"
    );
}

/// A chain that leaves this register is refused, and not for the other reason.
///
/// The ends here are disjoint as far as anybody can see: one is a piece of
/// literature, the other is a measurement held here. A rule that only compared
/// the sets would let this through, which is the error the record says the
/// decision exists to prevent.
#[test]
fn an_unresolved_chain_is_refused_rather_than_read_as_disjoint() {
    let mut held = register(&[
        ("claim-one", Method::Compiled),
        ("claim-two", Method::Compiled),
        ("measurement-n-1993", Method::MeasuredInLaboratory),
    ]);
    quotes_literature(&mut held, "claim-one", "10.1000/private-communication");
    quotes(&mut held, "claim-two", "measurement-n-1993");

    let first = ancestry_of(&held, &id("claim-one")).expect("the claim is held");
    let second = ancestry_of(&held, &id("claim-two")).expect("the claim is held");
    assert!(
        first.ends().is_disjoint(second.ends()),
        "this fixture is only worth anything if the two sets look disjoint"
    );

    let refused = may_marginalise(&held, &[id("claim-one"), id("claim-two")])
        .expect_err("an unresolved chain must be refused");

    let NotIndependent::Unresolved { claim, stopped_at } = &refused else {
        panic!("expected an unresolved chain, got {refused:?}");
    };
    assert_eq!(claim, &id("claim-one"));
    assert_eq!(
        stopped_at,
        &Terminal::Unfollowed(ReferenceId::Doi("10.1000/private-communication".to_owned()))
    );
    assert!(
        refused.to_string().contains("unresolved"),
        "the message must say the chain is unresolved, got {refused}"
    );
}

/// The two refusals do not say the same thing.
///
/// A caller acting on them has different work to do. An overlap is a decision
/// about which claim to keep; an unresolved chain is resolution work. One
/// message for both would report the wrong repair.
#[test]
fn the_unresolved_message_is_not_the_overlap_message() {
    let overlap = NotIndependent::SharedAncestor {
        left: id("claim-one"),
        right: id("claim-two"),
        ancestor: Terminal::Origin(id("measurement-m-1982")),
        through: vec![id("compilation-c-1988")],
    };
    let unresolved = NotIndependent::Unresolved {
        claim: id("claim-one"),
        stopped_at: Terminal::Unfollowed(ReferenceId::Doi("10.1000/nothing".to_owned())),
    };

    assert_ne!(overlap, unresolved);
    assert_ne!(overlap.to_string(), unresolved.to_string());
    assert!(!overlap.to_string().contains("unresolved"));
}

/// A compilation that names nobody ends the chain without resolving it.
///
/// The register reports this claim separately as a compiled claim with no
/// outgoing edge. What matters here is that the walk does not read it as an
/// origin, which would make a compilation whose attribution nobody recorded look
/// like a bench measurement.
#[test]
fn a_compilation_that_names_nobody_is_not_an_origin() {
    let held = register(&[("claim-one", Method::Compiled)]);

    let ancestry = ancestry_of(&held, &id("claim-one")).expect("the claim is held");
    assert_eq!(
        ancestry.ends().iter().collect::<Vec<_>>(),
        vec![&Terminal::QuotesNobody(id("claim-one"))]
    );
    assert!(!Terminal::QuotesNobody(id("claim-one")).is_resolved());

    assert!(
        matches!(
            may_marginalise(&held, &[id("claim-one")]),
            Err(NotIndependent::Unresolved { .. })
        ),
        "a chain that stops at a compilation naming nobody must be refused"
    );
}

/// A measurement with nothing to follow is an origin, and one claim on its own
/// is not refused for having no pair.
#[test]
fn a_measurement_with_nothing_to_follow_resolves() {
    let held = register(&[("measurement-m-1982", Method::MeasuredInLaboratory)]);

    let ancestry = ancestry_of(&held, &id("measurement-m-1982")).expect("the claim is held");
    assert_eq!(
        ancestry.ends().iter().collect::<Vec<_>>(),
        vec![&Terminal::Origin(id("measurement-m-1982"))]
    );
    assert!(ancestry.unresolved().is_empty());
    assert_eq!(may_marginalise(&held, &[id("measurement-m-1982")]), Ok(()));
}

/// The same claim offered twice shares every ancestor with itself.
///
/// Worth a fixture because it is the cheapest way to produce the failure the
/// rule is about: a caller that assembled its list from two queries and got one
/// claim back from both would otherwise combine a number with itself.
#[test]
fn one_claim_offered_twice_is_refused() {
    let held = register(&[("measurement-m-1982", Method::MeasuredInLaboratory)]);

    let refused = may_marginalise(&held, &[id("measurement-m-1982"), id("measurement-m-1982")])
        .expect_err("a claim combined with itself must be refused");
    assert!(matches!(refused, NotIndependent::SharedAncestor { .. }));
}

/// A claim that is a converted copy of another shares its ancestor.
///
/// The walk follows edges rather than stopping at the first method that is not a
/// compilation. Without that, a value renormalised out of another one would look
/// like an independent computation, which is a copy wearing a different unit.
#[test]
fn a_renormalised_copy_shares_the_ancestor_it_was_derived_from() {
    let mut held = register(&[
        ("measurement-m-1982", Method::MeasuredInLaboratory),
        (
            "recomputed",
            Method::Computed {
                code: Some("cowan".to_owned()),
                approximation: None,
            },
        ),
    ]);
    held.add_edge(Edge {
        from: id("recomputed"),
        to: Ancestor::Claim(id("measurement-m-1982")),
        derivation: Derivation::Renormalisation,
    })
    .expect("an edge between two held claims is accepted");

    let refused = may_marginalise(&held, &[id("recomputed"), id("measurement-m-1982")])
        .expect_err("a renormalised copy is not independent of what it came from");
    assert!(matches!(
        refused,
        NotIndependent::SharedAncestor {
            ancestor: Terminal::Origin(_),
            ..
        }
    ));
}

/// A claim the register does not hold is reported rather than skipped.
#[test]
fn a_claim_the_register_does_not_hold_is_reported() {
    let held = register(&[("measurement-m-1982", Method::MeasuredInLaboratory)]);

    assert_eq!(
        may_marginalise(&held, &[id("measurement-m-1982"), id("not-here")]),
        Err(NotIndependent::UnknownClaim(id("not-here"))),
        "a caller naming a claim that is not held has asked about something else"
    );
}

/// A chain that fans out keeps every end, because an average of two measurements
/// shares an ancestor with each of them.
#[test]
fn a_compilation_of_two_measurements_rests_on_both() {
    let mut held = register(&[
        ("averaged", Method::Compiled),
        ("measurement-m-1982", Method::MeasuredInLaboratory),
        ("measurement-n-1993", Method::MeasuredInLaboratory),
        ("quotes-n-only", Method::Compiled),
    ]);
    quotes(&mut held, "averaged", "measurement-m-1982");
    quotes(&mut held, "averaged", "measurement-n-1993");
    quotes(&mut held, "quotes-n-only", "measurement-n-1993");

    let ancestry = ancestry_of(&held, &id("averaged")).expect("the claim is held");
    assert_eq!(ancestry.ends().len(), 2);

    let refused = may_marginalise(&held, &[id("averaged"), id("quotes-n-only")])
        .expect_err("an average shares an ancestor with each measurement in it");
    assert!(matches!(
        refused,
        NotIndependent::SharedAncestor {
            ancestor: Terminal::Origin(_),
            ..
        }
    ));
}
