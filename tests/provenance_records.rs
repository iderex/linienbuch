//! The three records stay apart, and a reference stays reachable from claims in
//! more than one source.
//!
//! The constructed case is the one the board is built around. Two databases
//! quote one paper. Nothing about either database's row says so, and the only
//! thing that can say so is a register where the reference is its own record
//! rather than a string inside each source.

use linienbuch::provenance::{
    Attribution, ClaimId, Date, Digest, DigestAlgorithm, Reference, ReferenceId, Refused, Register,
    Snapshot, Source, SourceId, TermsId,
};

fn digest(fill: char) -> Digest {
    let hex: String = std::iter::repeat_n(fill, 64).collect();
    Digest::new(DigestAlgorithm::Sha256, hex).expect("64 lowercase hex characters")
}

fn source(id: &str) -> Source {
    Source {
        id: SourceId::new(id),
        name: format!("{id} line database"),
        home: format!("https://example.invalid/{id}"),
        maintainer: format!("{id} maintainer"),
        terms: TermsId::new(format!("terms/{id}")),
    }
}

fn snapshot(source_id: &str, fill: char, day: u8) -> Snapshot {
    Snapshot {
        source: SourceId::new(source_id),
        digest: digest(fill),
        retrieved: Date::new(2026, 1, day).expect("a real date"),
        request: format!("GET /{source_id}/lines?species=Fe+II"),
        upstream_version: Some("2025.2".to_owned()),
    }
}

fn paper() -> Reference {
    Reference {
        id: ReferenceId::Doi("10.0000/constructed.1982".to_owned()),
        citation: "The 1982 laboratory measurement".to_owned(),
    }
}

/// Two sources, one snapshot each, both attributing a claim to one paper. The
/// register can say that the two are not independent; neither source could.
#[test]
fn a_reference_is_reachable_from_claims_in_more_than_one_source() {
    let mut register = Register::new();
    register.add_source(source("alpha")).expect("new source");
    register.add_source(source("beta")).expect("new source");
    register.add_reference(paper());
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("source is known");
    register
        .add_snapshot(snapshot("beta", 'b', 15))
        .expect("source is known");

    for (fill, claim) in [('a', "claim/alpha/1"), ('b', "claim/beta/1")] {
        register
            .add_attribution(Attribution {
                claim: ClaimId::new(claim),
                snapshot: digest(fill),
                reference: paper().id,
            })
            .expect("snapshot and reference are known");
    }

    let claims = register.claims_citing(&paper().id);
    assert_eq!(claims.len(), 2, "both claims cite the paper");

    let sources = register.sources_citing(&paper().id);
    assert_eq!(sources.len(), 2, "the paper is reached from two sources");
    assert!(sources.contains(&SourceId::new("alpha")));
    assert!(sources.contains(&SourceId::new("beta")));
}

/// The neighbour of the case above, one edge away. The second source attributes
/// its claim to a different paper, so the reference is reached from one source
/// and the signal is absent. Without this, the test above would pass on a
/// register that reported every reference as shared.
#[test]
fn a_reference_cited_by_one_source_is_reached_from_one_source() {
    let other = Reference {
        id: ReferenceId::Bibcode("1993Constructed..1..1X".to_owned()),
        citation: "A different measurement".to_owned(),
    };

    let mut register = Register::new();
    register.add_source(source("alpha")).expect("new source");
    register.add_source(source("beta")).expect("new source");
    register.add_reference(paper());
    register.add_reference(other.clone());
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("source is known");
    register
        .add_snapshot(snapshot("beta", 'b', 15))
        .expect("source is known");

    register
        .add_attribution(Attribution {
            claim: ClaimId::new("claim/alpha/1"),
            snapshot: digest('a'),
            reference: paper().id,
        })
        .expect("known");
    register
        .add_attribution(Attribution {
            claim: ClaimId::new("claim/beta/1"),
            snapshot: digest('b'),
            reference: other.id.clone(),
        })
        .expect("known");

    assert_eq!(register.sources_citing(&paper().id).len(), 1);
    assert_eq!(register.sources_citing(&other.id).len(), 1);
}

/// A source and its snapshots are different records, so one source holds many
/// retrievals and each keeps its own digest, date, request and upstream version.
#[test]
fn one_source_holds_many_snapshots_and_each_keeps_its_own_identity() {
    let mut register = Register::new();
    register.add_source(source("alpha")).expect("new source");

    let first = snapshot("alpha", 'a', 14);
    let mut second = snapshot("alpha", 'c', 21);
    second.upstream_version = None;

    register
        .add_snapshot(first.clone())
        .expect("source is known");
    register
        .add_snapshot(second.clone())
        .expect("source is known");

    assert_eq!(register.snapshot(&digest('a')), Some(&first));
    assert_eq!(register.snapshot(&digest('c')), Some(&second));
    assert_eq!(
        register.snapshot(&digest('a')).map(|s| &s.source),
        register.snapshot(&digest('c')).map(|s| &s.source),
        "both retrievals are of one source"
    );
    assert_eq!(
        register
            .snapshot(&digest('c'))
            .and_then(|s| s.upstream_version.as_deref()),
        None,
        "a source that supplied no version is recorded as having supplied none"
    );
}

/// Re-registering a snapshot with identical contents changes nothing, because
/// the digest is the identity and identical bytes are one snapshot.
#[test]
fn an_identical_retrieval_is_the_same_snapshot() {
    let mut register = Register::new();
    register.add_source(source("alpha")).expect("new source");
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("source is known");
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("identical snapshot is accepted and changes nothing");

    assert_eq!(
        register.snapshot(&digest('a')),
        Some(&snapshot("alpha", 'a', 14))
    );
}

/// Every refusal, each asserted against the reason rather than against failure.
#[test]
fn the_register_refuses_a_dangling_or_contradicted_record() {
    let mut register = Register::new();

    let orphan = snapshot("nobody", 'a', 14);
    assert_eq!(
        register.add_snapshot(orphan),
        Err(Refused::UnknownSource(SourceId::new("nobody")))
    );

    register.add_source(source("alpha")).expect("new source");
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("source is known");

    assert_eq!(
        register.add_attribution(Attribution {
            claim: ClaimId::new("claim/alpha/1"),
            snapshot: digest('f'),
            reference: paper().id,
        }),
        Err(Refused::UnknownSnapshot(digest('f')))
    );

    assert_eq!(
        register.add_attribution(Attribution {
            claim: ClaimId::new("claim/alpha/1"),
            snapshot: digest('a'),
            reference: paper().id,
        }),
        Err(Refused::UnknownReference(paper().id))
    );

    let mut contradicting = snapshot("alpha", 'a', 14);
    contradicting.request = "GET /alpha/lines?species=Ca+II".to_owned();
    assert_eq!(
        register.add_snapshot(contradicting),
        Err(Refused::SnapshotContradicted(digest('a')))
    );

    let mut renamed = source("alpha");
    renamed.maintainer = "somebody else".to_owned();
    assert_eq!(
        register.add_source(renamed),
        Err(Refused::SourceContradicted(SourceId::new("alpha")))
    );
}

/// The neighbours of the refusals above. Each is one change from a refusal and
/// each must be accepted.
#[test]
fn the_neighbours_of_the_refusals_are_accepted() {
    let mut register = Register::new();
    register.add_source(source("alpha")).expect("new source");
    register.add_reference(paper());
    register
        .add_snapshot(snapshot("alpha", 'a', 14))
        .expect("source is known");
    register
        .add_attribution(Attribution {
            claim: ClaimId::new("claim/alpha/1"),
            snapshot: digest('a'),
            reference: paper().id,
        })
        .expect("snapshot and reference are known");
    register
        .add_source(source("alpha"))
        .expect("an identical source changes nothing");
}

/// A digest is refused unless it is the right length and lowercase hex, because
/// two spellings of one digest would be two identities for one snapshot.
#[test]
fn a_malformed_digest_is_refused() {
    let sixty_four_uppercase: String = std::iter::repeat_n('A', 64).collect();
    let sixty_three: String = std::iter::repeat_n('a', 63).collect();
    let with_a_g: String = std::iter::repeat_n('a', 63)
        .chain(std::iter::once('g'))
        .collect();

    for bad in [&sixty_four_uppercase, &sixty_three, &with_a_g] {
        assert_eq!(
            Digest::new(DigestAlgorithm::Sha256, bad.clone()),
            Err(Refused::MalformedDigest(bad.clone())),
            "{bad:?} must be refused"
        );
    }

    let sixty_four_lowercase: String = std::iter::repeat_n('a', 64).collect();
    Digest::new(DigestAlgorithm::Sha256, sixty_four_lowercase)
        .expect("the neighbour of all three, and it must be accepted");
}

/// A retrieval date is a real date. The near miss is the thirtieth of February
/// in a leap year, which a naive range check on the day accepts.
#[test]
fn a_malformed_date_is_refused() {
    for (year, month, day) in [
        (2026u16, 2u8, 29u8),
        (2024, 2, 30),
        (2026, 13, 1),
        (2026, 1, 0),
    ] {
        assert_eq!(
            Date::new(year, month, day),
            Err(Refused::MalformedDate { year, month, day }),
            "{year}-{month}-{day} must be refused"
        );
    }

    Date::new(2024, 2, 29).expect("2024 is a leap year, and this is the neighbour of the first");
    Date::new(2026, 2, 28).expect("the neighbour of the first in the other direction");
}

/// A reference with no persistent identifier is recorded as having none, which
/// is a different state from one whose identifier was never looked up. The
/// variants keep them apart, and two references are never equal across kinds.
#[test]
fn a_reference_without_a_persistent_identifier_is_its_own_state() {
    let doi = ReferenceId::Doi("10.0000/constructed.1982".to_owned());
    let local = ReferenceId::Local("10.0000/constructed.1982".to_owned());

    assert_ne!(doi, local);

    let mut register = Register::new();
    register.add_reference(Reference {
        id: local.clone(),
        citation: "A private communication, quoted by the source".to_owned(),
    });

    assert!(register.reference(&local).is_some());
    assert!(register.reference(&doi).is_none());
}
