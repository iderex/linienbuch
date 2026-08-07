//! Sources, snapshots and bibliographic references, kept apart.
//!
//! These three are routinely collapsed into one field and the collapse costs
//! exactly the two things this board exists to recover.
//!
//! Collapsing a source into a snapshot loses which version of a database a
//! number came from, so an answer can be reproduced only by whoever happened to
//! retrieve the same day.
//!
//! Collapsing a source into a reference makes it impossible to see that two
//! databases are quoting one paper, and two sources citing one reference is the
//! first signal that their values are not independent. That signal is what
//! `docs/decisions/shared-ancestry.md` builds the disjointness rule on, so
//! losing it here would leave that rule with nothing to read.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A database or a compilation as an ongoing thing.
///
/// Not a retrieval of it. A source persists across retrievals and is the thing
/// that has a maintainer and terms; what came back on a given day is a
/// [`Snapshot`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: SourceId,
    pub name: String,
    pub home: String,
    pub maintainer: String,
    /// The terms record for this source. The record itself is #54, which states
    /// what a terms record has to quote and from where; this is the handle that
    /// points at it, so that a source cannot exist without one being named.
    pub terms: TermsId,
}

/// One retrieval of a source.
///
/// The identity is the content digest of what came back, not the date and not a
/// serial number. Two retrievals returning identical bytes are one snapshot, and
/// a retrieval returning different bytes is a different snapshot however the
/// upstream labels it. Snapshots are immutable: a new retrieval makes a new
/// snapshot and never edits an existing one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub source: SourceId,
    pub digest: Digest,
    pub retrieved: Date,
    /// The exact request that produced it, so that the retrieval can be
    /// repeated rather than approximated.
    pub request: String,
    /// Whatever version string the upstream supplied, where it supplied one.
    /// `None` is a source that gave no version, which is a different state from
    /// a version this board did not record.
    pub upstream_version: Option<String>,
}

/// A piece of literature.
///
/// Shared between sources, which is the whole point of keeping it separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub id: ReferenceId,
    pub citation: String,
}

/// The edge from a claim to the literature the source attributes it to.
///
/// The claim record itself is #23. What this module needs from it is that it has
/// an identity, so the identity is here as an opaque handle and the record is
/// not duplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    pub claim: ClaimId,
    /// The snapshot the claim was read out of, so the attribution is anchored to
    /// bytes rather than to a database in general.
    pub snapshot: Digest,
    pub reference: ReferenceId,
}

macro_rules! opaque_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(text: impl Into<String>) -> Self {
                Self(text.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

opaque_id!(
    /// The identity of a source.
    SourceId
);
opaque_id!(
    /// The identity of a terms record. The record is #54.
    TermsId
);
opaque_id!(
    /// The identity of a claim. The record is #23.
    ClaimId
);

/// The persistent identifier of a piece of literature, where one exists.
///
/// `Local` is not a fourth kind of identifier. It is the recorded absence of
/// one, and it is a distinct variant so that a reference with no persistent
/// identifier cannot be mistaken for one whose identifier was not looked up.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceId {
    Doi(String),
    Bibcode(String),
    Local(String),
}

/// The algorithm a digest was computed with.
///
/// Recorded beside the digest rather than assumed, because a bare hex string
/// with no algorithm is a value nobody can recompute, and recomputing it is the
/// only thing it is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DigestAlgorithm {
    Sha256,
}

impl DigestAlgorithm {
    /// The number of hex characters a digest of this algorithm has.
    fn hex_length(self) -> usize {
        match self {
            DigestAlgorithm::Sha256 => 64,
        }
    }

    fn name(self) -> &'static str {
        match self {
            DigestAlgorithm::Sha256 => "sha256",
        }
    }
}

/// The content digest of a retrieval, which is a snapshot's identity.
///
/// This type holds and validates a digest. It does not compute one: the bytes a
/// digest is over arrive during ingest, which is #26, and the computation lives
/// there with the retrieval rather than here with the record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest {
    algorithm: DigestAlgorithm,
    hex: String,
}

impl Digest {
    /// A digest, refusing anything that is not the right length or not
    /// lowercase hex.
    ///
    /// Lowercase rather than either case, because two spellings of one digest
    /// would be two identities for one snapshot, which is the failure the digest
    /// is here to prevent.
    pub fn new(algorithm: DigestAlgorithm, hex: impl Into<String>) -> Result<Self, Refused> {
        let hex = hex.into();
        if hex.len() != algorithm.hex_length() {
            return Err(Refused::MalformedDigest(hex));
        }
        if !hex
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        {
            return Err(Refused::MalformedDigest(hex));
        }
        Ok(Digest { algorithm, hex })
    }

    pub fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }

    pub fn hex(&self) -> &str {
        &self.hex
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.name(), self.hex)
    }
}

/// A calendar date, with no time and no zone.
///
/// A retrieval date is a date. Carrying a time would suggest a precision the
/// upstream does not have and would make two records of one retrieval differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, Refused> {
        let valid = (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month);
        if !valid {
            return Err(Refused::MalformedDate { year, month, day });
        }
        Ok(Date { year, month, day })
    }

    pub fn year(self) -> u16 {
        self.year
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn day(self) -> u8 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

/// Why a record was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    MalformedDigest(String),
    MalformedDate {
        year: u16,
        month: u8,
        day: u8,
    },
    /// A snapshot names a source the register does not hold.
    UnknownSource(SourceId),
    /// An attribution names a snapshot the register does not hold.
    UnknownSnapshot(Digest),
    /// An attribution names a reference the register does not hold.
    UnknownReference(ReferenceId),
    /// A source is registered twice under one identity with different contents.
    SourceContradicted(SourceId),
    /// One digest is offered for two different retrievals. Since the digest is
    /// the identity, this says two different byte streams hashed the same, and
    /// it is refused rather than resolved.
    SnapshotContradicted(Digest),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::MalformedDigest(hex) => write!(f, "malformed digest {hex:?}"),
            Refused::MalformedDate { year, month, day } => {
                write!(f, "malformed date {year}-{month}-{day}")
            }
            Refused::UnknownSource(id) => write!(f, "unknown source {id}"),
            Refused::UnknownSnapshot(digest) => write!(f, "unknown snapshot {digest}"),
            Refused::UnknownReference(id) => write!(f, "unknown reference {id:?}"),
            Refused::SourceContradicted(id) => {
                write!(
                    f,
                    "source {id} is already registered with different contents"
                )
            }
            Refused::SnapshotContradicted(digest) => {
                write!(
                    f,
                    "snapshot {digest} is already registered with different contents"
                )
            }
        }
    }
}

/// The three records and the edges between them.
///
/// Adding a record whose referent is absent is refused rather than stored and
/// resolved later, because a dangling edge is discovered by whoever asks the
/// question the edge exists to answer, which is the worst moment to discover it.
#[derive(Debug, Default)]
pub struct Register {
    sources: BTreeMap<SourceId, Source>,
    snapshots: BTreeMap<Digest, Snapshot>,
    references: BTreeMap<ReferenceId, Reference>,
    attributions: Vec<Attribution>,
}

impl Register {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a source. Registering the identical source twice is accepted and
    /// changes nothing; registering a different source under one identity is
    /// refused.
    pub fn add_source(&mut self, source: Source) -> Result<(), Refused> {
        match self.sources.get(&source.id) {
            Some(existing) if *existing != source => {
                Err(Refused::SourceContradicted(source.id.clone()))
            }
            Some(_) => Ok(()),
            None => {
                self.sources.insert(source.id.clone(), source);
                Ok(())
            }
        }
    }

    pub fn add_reference(&mut self, reference: Reference) {
        self.references.insert(reference.id.clone(), reference);
    }

    /// Register a snapshot, refusing one whose source is not known and one that
    /// contradicts a snapshot already held under the same digest.
    pub fn add_snapshot(&mut self, snapshot: Snapshot) -> Result<(), Refused> {
        if !self.sources.contains_key(&snapshot.source) {
            return Err(Refused::UnknownSource(snapshot.source.clone()));
        }
        match self.snapshots.get(&snapshot.digest) {
            Some(existing) if *existing != snapshot => {
                Err(Refused::SnapshotContradicted(snapshot.digest.clone()))
            }
            Some(_) => Ok(()),
            None => {
                self.snapshots.insert(snapshot.digest.clone(), snapshot);
                Ok(())
            }
        }
    }

    /// Record that a claim's source attributes it to a reference.
    pub fn add_attribution(&mut self, attribution: Attribution) -> Result<(), Refused> {
        if !self.snapshots.contains_key(&attribution.snapshot) {
            return Err(Refused::UnknownSnapshot(attribution.snapshot.clone()));
        }
        if !self.references.contains_key(&attribution.reference) {
            return Err(Refused::UnknownReference(attribution.reference.clone()));
        }
        self.attributions.push(attribution);
        Ok(())
    }

    pub fn source(&self, id: &SourceId) -> Option<&Source> {
        self.sources.get(id)
    }

    pub fn snapshot(&self, digest: &Digest) -> Option<&Snapshot> {
        self.snapshots.get(digest)
    }

    pub fn reference(&self, id: &ReferenceId) -> Option<&Reference> {
        self.references.get(id)
    }

    /// Every claim that attributes a value to this reference.
    pub fn claims_citing(&self, reference: &ReferenceId) -> Vec<&ClaimId> {
        self.attributions
            .iter()
            .filter(|a| a.reference == *reference)
            .map(|a| &a.claim)
            .collect()
    }

    /// Every source whose claims attribute a value to this reference.
    ///
    /// A set of more than one is the signal that two databases are quoting one
    /// paper. It is not a conclusion about independence on its own, because the
    /// chain may go further, and following it is the resolution procedure in
    /// `docs/decisions/shared-ancestry.md`. It is where that procedure starts.
    pub fn sources_citing(&self, reference: &ReferenceId) -> BTreeSet<&SourceId> {
        self.attributions
            .iter()
            .filter(|a| a.reference == *reference)
            .filter_map(|a| self.snapshots.get(&a.snapshot))
            .map(|snapshot| &snapshot.source)
            .collect()
    }
}
