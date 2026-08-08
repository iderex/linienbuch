//! Whether a set of claims may be combined, and the refusal when it may not.
//!
//! `docs/decisions/shared-ancestry.md` is the decision this implements. Two
//! compilations quoting one underlying measurement are one measurement, and
//! combining them narrows the answer for no reason other than that somebody
//! copied a number. That is the false precision this board exists to object to,
//! and producing it here would be producing it with the board's name on it.
//!
//! Two rules, and the second is the one that is easy to get backwards.
//!
//! Ancestor sets must be pairwise disjoint before anything combines them, and an
//! overlap is refused with the shared end named rather than down-weighted. Any
//! adjustment is a model of how correlated two claims are, and the information
//! that model needs is exactly what is missing when a chain is only partly
//! resolved.
//!
//! An unresolved chain is not a disjoint one. Where a chain has not been
//! followed to an end this register holds, the sets may look disjoint and be
//! nothing of the kind, so that case is refused as well and with a different
//! message. Getting this default the wrong way round would make the numbers
//! wrong in the direction the board exists to correct, which is why a test
//! asserts the unresolved case does not take the path the disjoint case takes.
//!
//! Nothing here names a quantity or a subject. Whether two values may be
//! combined is a question about provenance, and a register of material
//! parameters would ask it in the same words.
//!
//! Three things this does not decide.
//!
//! The weighting used once a combination is permitted. That moves the headline
//! number and is entry 7 of #1.
//!
//! Whether a value tuned against a reference object may be combined with the
//! laboratory values it was tuned against. That is a shared ancestry of a
//! different kind, it is entry 6 of #1, and until it is answered a tuned claim
//! with nothing to follow is an end like any other here.
//!
//! What a chain that leaves this register means. The decision record separates a
//! dead end, which is followed and found to stop, from a chain nobody followed,
//! and this register has no field for the first: an edge to literature it holds
//! no claim about is the only shape available. So every such chain is unresolved
//! here and every one of them refuses. That is the record's own default rather
//! than a stricter one, and the case it costs is a recorded dead end that no
//! other chain could reach, which is narrow.

use crate::register::claims::{Ancestor, Claims, Method};
use crate::register::provenance::{ClaimId, ReferenceId};
use std::collections::BTreeSet;
use std::fmt;

/// Where a chain stopped.
///
/// Three ends and only the first is an answer. The other two are distinct states
/// rather than one absence, because "we followed this and it left the register"
/// and "it says it quotes somebody and names nobody" are different facts about
/// different defects, and collapsing them would report the wrong repair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Terminal {
    /// A claim held here with nothing further to follow and a method that is not
    /// a compilation. A measurement, a computation, or a value tuned against a
    /// reference object. This is what the walk is looking for.
    Origin(ClaimId),
    /// The chain left this register at a piece of literature no claim here is
    /// about. What that literature attributes to is unknown, so this end is
    /// reached rather than resolved.
    Unfollowed(ReferenceId),
    /// A compilation with no outgoing edge. It says it is quoting somebody and
    /// names nobody, so the chain stops without having reached anything.
    QuotesNobody(ClaimId),
}

impl Terminal {
    /// Whether the chain that stopped here was followed to an end this register
    /// holds.
    pub fn is_resolved(&self) -> bool {
        matches!(self, Terminal::Origin(_))
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminal::Origin(id) => write!(f, "{id}"),
            Terminal::Unfollowed(reference) => match reference {
                ReferenceId::Doi(text) | ReferenceId::Bibcode(text) | ReferenceId::Local(text) => {
                    write!(f, "the literature at {text}, which no claim here is about")
                }
            },
            Terminal::QuotesNobody(id) => {
                write!(f, "{id}, which is compiled and names nobody")
            }
        }
    }
}

/// What one claim rests on, and what the walk passed through to find out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ancestry {
    of: ClaimId,
    ends: BTreeSet<Terminal>,
    /// Every claim the walk visited, including the claim itself and any end that
    /// is a claim. The intersection of two of these is what the refusal reports
    /// as the nodes both chains run through, which is the part a reader needs in
    /// order to see the overlap rather than be told about it.
    visited: BTreeSet<ClaimId>,
}

impl Ancestry {
    pub fn of(&self) -> &ClaimId {
        &self.of
    }

    pub fn ends(&self) -> &BTreeSet<Terminal> {
        &self.ends
    }

    pub fn visited(&self) -> &BTreeSet<ClaimId> {
        &self.visited
    }

    /// The ends that were not followed to something this register holds.
    ///
    /// Empty is the only state in which a set of ends may be compared with
    /// another set and the comparison mean anything.
    pub fn unresolved(&self) -> Vec<&Terminal> {
        self.ends.iter().filter(|end| !end.is_resolved()).collect()
    }
}

/// Why a set of claims may not be combined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotIndependent {
    /// A claim that is not in this register. Reported rather than skipped: a
    /// caller naming a claim that is not held has asked a question about
    /// something else, and answering it about the claims that happen to exist
    /// would be answering a different question without saying so.
    UnknownClaim(ClaimId),
    /// Two claims resting on one end, with the end named and with the claims
    /// both chains run through beside it.
    SharedAncestor {
        left: ClaimId,
        right: ClaimId,
        ancestor: Terminal,
        through: Vec<ClaimId>,
    },
    /// A chain that was not followed to an end this register holds. Distinct
    /// from an overlap, because the repair is resolution work rather than a
    /// decision about which claim to drop.
    Unresolved {
        claim: ClaimId,
        stopped_at: Terminal,
    },
}

impl fmt::Display for NotIndependent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotIndependent::UnknownClaim(id) => {
                write!(f, "no claim {id} in this register")
            }
            NotIndependent::SharedAncestor {
                left,
                right,
                ancestor,
                through,
            } => {
                write!(f, "{left} and {right} both rest on {ancestor}")?;
                if !through.is_empty() {
                    let names: Vec<&str> = through.iter().map(ClaimId::as_str).collect();
                    write!(f, ", through {}", names.join(" and "))?;
                }
                Ok(())
            }
            NotIndependent::Unresolved { claim, stopped_at } => write!(
                f,
                "the chain from {claim} is unresolved: it stops at {stopped_at}, \
                 so nothing here has shown it is independent of anything"
            ),
        }
    }
}

/// What one claim rests on.
///
/// The walk follows every outgoing edge rather than stopping at the first
/// non-compilation, and that is deliberate. A claim carrying an edge is saying
/// its number came from what the edge points at, whatever its own method says,
/// and a walk that stopped at the method would lose the ancestor of a value that
/// was converted or renormalised out of another one. Where a claim has no
/// outgoing edge, its method is what decides whether the end is an origin or a
/// compilation that named nobody.
///
/// The graph is acyclic, because a cycle is refused when the edge closing it is
/// offered. The visited set here is not relying on that: a walk that met one
/// would end rather than run forever, which is the behaviour to have if the
/// guard above it ever stops holding.
///
/// There is one place here that reports a claim the register does not hold, and
/// it is inside the walk rather than in a check before it. A second check on the
/// way in would read better and would be a refusal nothing can reach: every
/// other identity the walk meets came off an edge, and an edge to a claim this
/// register does not hold is refused when the edge is offered. One site, reached
/// by the only input that can reach it.
pub fn ancestry_of(register: &Claims, id: &ClaimId) -> Result<Ancestry, NotIndependent> {
    let mut ends: BTreeSet<Terminal> = BTreeSet::new();
    let mut visited: BTreeSet<ClaimId> = BTreeSet::new();
    let mut pending: Vec<ClaimId> = vec![id.clone()];

    while let Some(at) = pending.pop() {
        if !visited.insert(at.clone()) {
            continue;
        }
        let Some(held) = register.get(&at) else {
            return Err(NotIndependent::UnknownClaim(at));
        };
        let outgoing = register.ancestors_of(&at);
        if outgoing.is_empty() {
            let end = if held.method == Method::Compiled {
                Terminal::QuotesNobody(at.clone())
            } else {
                Terminal::Origin(at.clone())
            };
            ends.insert(end);
            continue;
        }
        for (ancestor, _) in outgoing {
            match ancestor {
                Ancestor::Claim(next) => pending.push(next.clone()),
                Ancestor::Reference(reference) => {
                    ends.insert(Terminal::Unfollowed(reference.clone()));
                }
            }
        }
    }

    Ok(Ancestry {
        of: id.clone(),
        ends,
        visited,
    })
}

/// Whether these claims may be combined.
///
/// The order of the two questions is chosen rather than incidental. An overlap
/// is reported before an unresolved chain, because where both hold, the shared
/// end is the finding and the unresolved chain is a second thing to fix. Either
/// way the answer is a refusal, so nothing is let through by the order.
///
/// One claim, or none, has no pair to compare and is not refused for that. It is
/// still refused where its own chain is unresolved, because a chain nobody
/// followed says nothing about the claim whether or not there is a second one
/// beside it.
pub fn may_marginalise(register: &Claims, over: &[ClaimId]) -> Result<(), NotIndependent> {
    let mut resolved: Vec<Ancestry> = Vec::with_capacity(over.len());
    for id in over {
        resolved.push(ancestry_of(register, id)?);
    }

    for (i, left) in resolved.iter().enumerate() {
        for right in resolved.iter().skip(i + 1) {
            if let Some(shared) = left.ends.intersection(&right.ends).next() {
                let through: Vec<ClaimId> = left
                    .visited
                    .intersection(&right.visited)
                    .filter(|id| !is_named_by(shared, id))
                    .cloned()
                    .collect();
                return Err(NotIndependent::SharedAncestor {
                    left: left.of.clone(),
                    right: right.of.clone(),
                    ancestor: shared.clone(),
                    through,
                });
            }
        }
    }

    for ancestry in &resolved {
        if let Some(end) = ancestry.unresolved().first() {
            return Err(NotIndependent::Unresolved {
                claim: ancestry.of.clone(),
                stopped_at: (*end).clone(),
            });
        }
    }

    Ok(())
}

/// Whether a claim identity is the one an end already names, so that the end is
/// not repeated in the list of nodes both chains run through.
fn is_named_by(end: &Terminal, id: &ClaimId) -> bool {
    match end {
        Terminal::Origin(named) | Terminal::QuotesNobody(named) => named == id,
        Terminal::Unfollowed(_) => false,
    }
}
