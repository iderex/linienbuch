//! Claims, the method that produced each one, and the edges between them.
//!
//! The centre of the register. Everything else exists so that this table can be
//! trusted.
//!
//! Nothing here names a quantity, a unit or a subject, and that is the layout
//! boundary rather than an omission. A claim is a value with an uncertainty,
//! produced by a method, read out of a snapshot, about something. What the
//! something is, and what quantities exist, is the domain's business. A register
//! of material parameters would use this file unchanged, which is the test
//! `docs/decisions/layout.md` draws the line by.
//!
//! Two properties are refused here rather than checked by whoever remembers.
//!
//! A claim whose method is compiled and which carries no outgoing edge. A
//! compilation is a source quoting somebody else, so a compiled claim that
//! points at nothing has lost the only thing that distinguishes it from a
//! primary measurement. Without the distinction every compilation looks like an
//! origin, and the ancestry work that `docs/decisions/shared-ancestry.md` rests
//! on has nothing to read.
//!
//! A cycle. Provenance is a claim about where a number came from, and a number
//! that came from itself came from nowhere. A cycle also makes every traversal a
//! question about termination rather than about ancestry.

use crate::register::provenance::{ClaimId, Digest, ReferenceId, SourceId};
use crate::register::uncertainty::Uncertainty;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// What a claim is about.
///
/// Opaque here on purpose. In this repository it will be a transition, whose
/// identity is #22; in a sibling register it would be something else. Naming it
/// here would put the domain on the generic side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubjectId(String);

/// Which quantity a claim is about.
///
/// Opaque for the same reason. The set of quantities is domain data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuantityId(String);

/// The unit a value is in, as the source writes it.
///
/// Required, and not defaulted anywhere. A value stored without its unit is a
/// number whose meaning depends on which source it came from, which is the
/// failure this board exists to object to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unit(String);

macro_rules! opaque {
    ($name:ident) => {
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

opaque!(SubjectId);
opaque!(QuantityId);
opaque!(Unit);

/// A calibration against a reference object whose own parameters came from
/// somewhere.
///
/// All three parts are required together and none may be empty. Any two of them
/// is decoration, and the case worth naming is the tempting one: an object and
/// its parameters without their origin reads as a careful calibration and hides
/// exactly the question the record exists to make answerable, which is whether
/// those parameters were themselves derived from the data being calibrated.
/// `docs/decisions/astrophysical-calibration.md` is where that is argued.
#[derive(Debug, Clone, PartialEq)]
pub struct Calibration {
    reference_object: String,
    assumed_parameters: BTreeMap<String, f64>,
    parameters_from: ReferenceId,
}

impl Calibration {
    pub fn new(
        reference_object: impl Into<String>,
        assumed_parameters: BTreeMap<String, f64>,
        parameters_from: ReferenceId,
    ) -> Result<Self, Refused> {
        let reference_object = reference_object.into();
        if reference_object.trim().is_empty() {
            return Err(Refused::CalibrationMissing("the reference object"));
        }
        if assumed_parameters.is_empty() {
            return Err(Refused::CalibrationMissing(
                "the parameters assumed for the reference object",
            ));
        }
        if identifier_of(&parameters_from).trim().is_empty() {
            return Err(Refused::CalibrationMissing(
                "where the assumed parameters came from",
            ));
        }
        Ok(Calibration {
            reference_object,
            assumed_parameters,
            parameters_from,
        })
    }

    pub fn reference_object(&self) -> &str {
        &self.reference_object
    }

    pub fn assumed_parameters(&self) -> &BTreeMap<String, f64> {
        &self.assumed_parameters
    }

    pub fn parameters_from(&self) -> &ReferenceId {
        &self.parameters_from
    }
}

fn identifier_of(reference: &ReferenceId) -> &str {
    match reference {
        ReferenceId::Doi(text) | ReferenceId::Bibcode(text) | ReferenceId::Local(text) => text,
    }
}

/// How the value was produced.
///
/// An enumeration rather than free text, so that adding a category forces every
/// place that decides on one to be revisited. The categories are chosen so that
/// the distinction this board is about survives, and the load bearing one is the
/// last: a schema without it forces every compilation to look like a primary
/// source.
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    MeasuredInLaboratory,
    /// Computed, with the code and the approximation named where the source says
    /// them and absent where it does not. Absent is what the source did not
    /// state rather than a computation with no method.
    Computed {
        code: Option<String>,
        approximation: Option<String>,
    },
    SemiEmpirical,
    /// Tuned so that a model reproduces a reference object, which carries the
    /// three parts of that calibration.
    Calibrated(Calibration),
    /// The source is quoting somebody else and is not itself the origin.
    Compiled,
}

/// A method with what distinguishes one instance of it from another removed.
///
/// [`Method::Computed`] carries the code and the approximation, and
/// [`Method::Calibrated`] carries three fields, so neither can be compared or
/// held in a table as it stands. What a weighting weights by is the category
/// rather than the instance, and this is that category with nothing else on it.
///
/// The map below is exhaustive, so a category added to [`Method`] does not
/// compile until it has one here, and [`MethodClass::ALL`] is what every table
/// over the categories is checked for completeness against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MethodClass {
    MeasuredInLaboratory,
    Computed,
    SemiEmpirical,
    Calibrated,
    Compiled,
}

impl MethodClass {
    /// Every category, in the order [`Method`] declares them.
    pub const ALL: [MethodClass; 5] = [
        MethodClass::MeasuredInLaboratory,
        MethodClass::Computed,
        MethodClass::SemiEmpirical,
        MethodClass::Calibrated,
        MethodClass::Compiled,
    ];

    /// Where this category sits in [`MethodClass::ALL`].
    pub fn at(self) -> usize {
        match self {
            MethodClass::MeasuredInLaboratory => 0,
            MethodClass::Computed => 1,
            MethodClass::SemiEmpirical => 2,
            MethodClass::Calibrated => 3,
            MethodClass::Compiled => 4,
        }
    }
}

impl fmt::Display for MethodClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MethodClass::MeasuredInLaboratory => "measured in a laboratory",
            MethodClass::Computed => "computed",
            MethodClass::SemiEmpirical => "semi-empirical",
            MethodClass::Calibrated => "calibrated against a reference object",
            MethodClass::Compiled => "compiled from somebody else",
        })
    }
}

impl Method {
    /// Which category this method is in.
    pub fn class(&self) -> MethodClass {
        match self {
            Method::MeasuredInLaboratory => MethodClass::MeasuredInLaboratory,
            Method::Computed { .. } => MethodClass::Computed,
            Method::SemiEmpirical => MethodClass::SemiEmpirical,
            Method::Calibrated(_) => MethodClass::Calibrated,
            Method::Compiled => MethodClass::Compiled,
        }
    }
}

/// What kind of derivation an edge is.
///
/// Typed so that a straight quotation, a unit conversion, a renormalisation and
/// a recalculation are distinguishable. Collapsing them would make a value that
/// was recomputed indistinguishable from one that was copied, and only one of
/// those is independent of its ancestor in any useful sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Derivation {
    Quotation,
    UnitConversion,
    Renormalisation,
    Recalculation,
}

/// A claim points at what it derives from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: ClaimId,
    pub to: Ancestor,
    pub derivation: Derivation,
}

/// What a claim derives from: another claim in this register, or a piece of
/// literature this register does not hold a claim for.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ancestor {
    Claim(ClaimId),
    Reference(ReferenceId),
}

/// One value, and everything needed to say where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub id: ClaimId,
    /// Which quantity, and what the subject is. Both opaque here.
    pub quantity: QuantityId,
    pub about: SubjectId,
    pub value: f64,
    /// Required. There is no default unit and no unit inferred from a quantity.
    pub unit: Unit,
    pub uncertainty: Uncertainty,
    pub method: Method,
    /// The year the source attaches to the value, where it attaches one.
    pub year: Option<u16>,
    pub source: SourceId,
    /// The snapshot the value was read out of.
    ///
    /// Not optional, and that is `docs/decisions/snapshots.md`'s rule held by the
    /// type rather than by a check beside it. A claim with no snapshot cannot be
    /// constructed, so no route into this register produces one.
    pub snapshot: Digest,
}

/// Why a claim or an edge was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// A calibration missing one of its three parts. The string names which.
    CalibrationMissing(&'static str),
    /// A compiled claim with no outgoing edge, so nothing says who it quotes.
    CompiledWithNoAncestor(ClaimId),
    /// An edge from a claim this register does not hold.
    UnknownClaim(ClaimId),
    /// An edge to a claim this register does not hold.
    UnknownAncestor(ClaimId),
    /// Two claims registered under one identity with different contents.
    ClaimContradicted(ClaimId),
    /// A cycle, with the claims on it in the order they were walked.
    Cycle(Vec<ClaimId>),
}

impl Refused {
    /// A stable name for the constraint this refusal is about.
    ///
    /// The match is exhaustive, so adding a variant does not compile until it is
    /// named here, and naming it here does not pass `tests/schema_validation.rs`
    /// until a fixture trips it. Two steps, and neither is a thing anybody can
    /// forget quietly.
    pub fn constraint(&self) -> &'static str {
        match self {
            Refused::CalibrationMissing(_) => "a calibrated claim missing one of its three parts",
            Refused::CompiledWithNoAncestor(_) => "a compiled claim with no outgoing edge",
            Refused::UnknownClaim(_) => "an edge from a claim the register does not hold",
            Refused::UnknownAncestor(_) => "an edge to a claim the register does not hold",
            Refused::ClaimContradicted(_) => "one identity holding two different claims",
            Refused::Cycle(_) => "a cycle in the provenance graph",
        }
    }

    /// Every constraint this type can refuse.
    ///
    /// A list, and the reason it is not derived is that Rust has no way to
    /// enumerate an enum's variants. What stops it drifting is the exhaustive
    /// match above and the coverage test that reads both.
    pub const CONSTRAINTS: [&'static str; 6] = [
        "a calibrated claim missing one of its three parts",
        "a compiled claim with no outgoing edge",
        "an edge from a claim the register does not hold",
        "an edge to a claim the register does not hold",
        "one identity holding two different claims",
        "a cycle in the provenance graph",
    ];
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::CalibrationMissing(part) => {
                write!(f, "a calibrated claim states no {part}")
            }
            Refused::CompiledWithNoAncestor(id) => write!(
                f,
                "{id} is compiled and points at nothing, so it cannot be told from an origin"
            ),
            Refused::UnknownClaim(id) => write!(f, "no claim {id} in this register"),
            Refused::UnknownAncestor(id) => write!(f, "no ancestor {id} in this register"),
            Refused::ClaimContradicted(id) => write!(f, "{id} is registered twice, differently"),
            Refused::Cycle(on) => {
                let names: Vec<&str> = on.iter().map(ClaimId::as_str).collect();
                write!(
                    f,
                    "a claim derives from itself, around {}",
                    names.join(" -> ")
                )
            }
        }
    }
}

/// The claims and the edges between them.
#[derive(Debug, Default)]
pub struct Claims {
    claims: BTreeMap<ClaimId, Claim>,
    edges: Vec<Edge>,
}

impl Claims {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a claim. Adding the identical claim twice changes nothing; adding a
    /// different claim under one identity is refused rather than resolved.
    pub fn add(&mut self, claim: Claim) -> Result<(), Refused> {
        match self.claims.get(&claim.id) {
            Some(held) if *held != claim => Err(Refused::ClaimContradicted(claim.id)),
            Some(_) => Ok(()),
            None => {
                self.claims.insert(claim.id.clone(), claim);
                Ok(())
            }
        }
    }

    /// Add an edge. Both ends must be held, and the edge must not close a cycle.
    ///
    /// The cycle is refused at the moment the edge is offered rather than found
    /// later by a traversal, because a traversal that meets one has already lost
    /// the ability to say which edge created it.
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), Refused> {
        if !self.claims.contains_key(&edge.from) {
            return Err(Refused::UnknownClaim(edge.from));
        }
        if let Ancestor::Claim(to) = &edge.to
            && !self.claims.contains_key(to)
        {
            return Err(Refused::UnknownAncestor(to.clone()));
        }
        if let Ancestor::Claim(to) = &edge.to
            && let Some(path) = self.path_from(to, &edge.from)
        {
            let mut cycle = vec![edge.from.clone()];
            cycle.extend(path);
            return Err(Refused::Cycle(cycle));
        }
        self.edges.push(edge);
        Ok(())
    }

    /// A walk from one claim to another along the edges that already exist.
    fn path_from(&self, start: &ClaimId, target: &ClaimId) -> Option<Vec<ClaimId>> {
        let mut seen: BTreeSet<&ClaimId> = BTreeSet::new();
        let mut stack: Vec<(&ClaimId, Vec<ClaimId>)> = vec![(start, vec![start.clone()])];
        while let Some((at, path)) = stack.pop() {
            if at == target {
                return Some(path);
            }
            if !seen.insert(at) {
                continue;
            }
            for edge in self.edges.iter().filter(|edge| edge.from == *at) {
                if let Ancestor::Claim(next) = &edge.to {
                    let mut onward = path.clone();
                    onward.push(next.clone());
                    stack.push((next, onward));
                }
            }
        }
        None
    }

    /// Every claim that is compiled and points at nothing.
    ///
    /// Reported rather than refused at insertion, because a compiled claim and
    /// the edge that says who it quotes arrive as two calls and the first cannot
    /// be refused for what the second has not done yet. This is the check the
    /// register is asked before it is used.
    pub fn compiled_with_no_ancestor(&self) -> Vec<Refused> {
        self.claims
            .values()
            .filter(|claim| claim.method == Method::Compiled)
            .filter(|claim| !self.edges.iter().any(|edge| edge.from == claim.id))
            .map(|claim| Refused::CompiledWithNoAncestor(claim.id.clone()))
            .collect()
    }

    pub fn get(&self, id: &ClaimId) -> Option<&Claim> {
        self.claims.get(id)
    }

    pub fn len(&self) -> usize {
        self.claims.len()
    }

    pub fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    /// What one claim derives from, directly.
    pub fn ancestors_of(&self, id: &ClaimId) -> Vec<(&Ancestor, Derivation)> {
        self.edges
            .iter()
            .filter(|edge| edge.from == *id)
            .map(|edge| (&edge.to, edge.derivation))
            .collect()
    }
}
