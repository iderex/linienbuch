//! Transitions, and the edges to their levels.
//!
//! A transition is the edge between two levels of one species. The levels are
//! fields of it rather than annotation hung on a wavelength, and that ordering
//! is what makes the states this board is about unrepresentable rather than
//! invalid. A transition with one level cannot be constructed, and a transition
//! cannot be moved onto other levels afterwards, because the levels are set at
//! construction and are not reachable to write.
//!
//! `docs/decisions/transition-identity.md` is the decision this file
//! implements and it is not restated here. Three of its sentences are what a
//! reader of this code needs in front of them.
//!
//! Nothing about the line position is part of the identity. A position is a
//! property sources disagree about, and a property sources disagree about
//! cannot also be the key they are joined on.
//!
//! An observed position and a Ritz position are different things and are never
//! interchanged. A row that does not say which of the two it published is
//! recorded as not saying, which is a third field rather than a guess into one
//! of the first two.
//!
//! A resolved component and an unresolved multiplet are different objects, so
//! the distinction is a kind rather than a flag, and a value for one is not a
//! value for the other.
//!
//! A position is stored as a vacuum wavenumber in cm^-1 and as nothing else,
//! which is `docs/decisions/line-position.md`. No air wavelength is produced
//! here. Which formula a source converts with is data belonging to that source,
//! the two formulae that record names are marked there as taken from citations
//! and not re-fetched, and writing one into this file would compile somebody's
//! remembered arithmetic into the place it is hardest to see. The conversion
//! arrives with the source it belongs to.
//!
//! ## What the recomputed Ritz position does not carry
//!
//! A width. [`Transition::ritz_from_levels`] returns a position and no
//! uncertainty, and [`Transition::ritz_agrees`] compares against the width the
//! source stated on its own Ritz value rather than against a propagated one.
//!
//! Propagating the two level energies into it in quadrature would state that
//! they are independent. Two levels of one term system published by one
//! compilation usually come out of a single fit and are not, so that sum would
//! make the recomputed position look better known than it is, which is the
//! direction that flatters the board. `docs/decisions/shared-ancestry.md`
//! argues the rule and `src/register/ancestry.rs` holds it. The width is left
//! absent here rather than assumed.
//!
//! ## What this file does not decide
//!
//! The subject key a claim points at. `SubjectId` in `src/register/claims.rs`
//! names this issue for it, and it is not produced here, because two sources'
//! rows for one transition become one subject only once a match has been made,
//! and that match is made on level energies within a tolerance rather than on
//! anything derivable from a single row. Deriving a key here would fix an
//! identity that the decision record deliberately leaves to a tolerance, and
//! the matcher is where it belongs.

use crate::register::provenance::{ClaimId, SourceId};
use crate::register::uncertainty::Uncertainty;
use crate::spectroscopy::levels::Level;
use crate::spectroscopy::species::{Convention, Species};
use std::cmp::Ordering;
use std::fmt;

/// A line position in the one representation this board stores: a vacuum
/// wavenumber in cm^-1.
///
/// A separate type from any wavelength, so that a number in one representation
/// cannot be handed to something expecting the other. There is no conversion on
/// this type and no arithmetic between it and anything else.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VacuumWavenumber(f64);

/// Why a number was refused as a line position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPosition {
    /// Not a finite number. Reachable from arithmetic on energies that were
    /// themselves finite, so it is refused here rather than assumed away.
    NotFinite,
    /// Zero or below. A transition between two levels at one energy is not a
    /// line, and a negative wavenumber is an upper and a lower level the wrong
    /// way round.
    NotAboveZero,
}

impl fmt::Display for NotAPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotAPosition::NotFinite => f.write_str("a line position must be a finite number"),
            NotAPosition::NotAboveZero => f.write_str("a line position must be above zero"),
        }
    }
}

impl VacuumWavenumber {
    /// A vacuum wavenumber in cm^-1.
    pub fn new(cm_inverse: f64) -> Result<Self, NotAPosition> {
        if !cm_inverse.is_finite() {
            return Err(NotAPosition::NotFinite);
        }
        if cm_inverse <= 0.0 {
            return Err(NotAPosition::NotAboveZero);
        }
        Ok(VacuumWavenumber(cm_inverse))
    }

    pub fn cm_inverse(self) -> f64 {
        self.0
    }
}

impl fmt::Display for VacuumWavenumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cm^-1", self.0)
    }
}

/// A position a source published, with the claim it is.
///
/// The provenance is the claim's rather than a second copy of it here. An
/// observed position and a Ritz position for one transition therefore point at
/// two different claims, with their own method, year, source and snapshot, and
/// nothing in this record makes them share any of that.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub wavenumber: VacuumWavenumber,
    /// What the source said about how well it knows this position. Absent is a
    /// source that stated none, and it is not consumed as a zero anywhere.
    pub uncertainty: Uncertainty,
    /// The claim this position is.
    pub claim: ClaimId,
}

/// Whether a row is a resolved component or an unresolved multiplet.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// One resolved transition between the two levels named.
    Component,
    /// An unresolved blend published as one row.
    Multiplet {
        /// The components the source says the row may contain, where it says
        /// them. `None` is a source that published an unresolved row and listed
        /// nothing, which is a different state from one that listed an empty
        /// set.
        components: Option<Vec<Transition>>,
    },
}

/// Why a transition was refused.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// The two levels belong to different species. A transition is an edge
    /// inside one species, and a pair drawn across two is not a weaker
    /// transition.
    LevelsOfDifferentSpecies { lower: Species, upper: Species },
    /// The two energies are measured from different zeros. Their difference is
    /// the offset between two scales plus a transition energy, and nothing
    /// separates the two terms, so no position is derivable from the pair.
    EnergiesOnDifferentScales { lower: SourceId, upper: SourceId },
    /// The upper level is not above the lower one. The mistake this catches is
    /// the two arguments the wrong way round, which is otherwise a transition
    /// with a plausible looking negative position.
    UpperNotAboveLower { lower: f64, upper: f64 },
    /// The difference between the two energies is not a position.
    NotAPosition(NotAPosition),
    /// A multiplet listed among the components of a multiplet. A component is
    /// resolved by definition, and a nested one would make the set of
    /// components of a row a question about depth.
    MultipletInsideAMultiplet,
    /// A component of a species other than the multiplet's own.
    ComponentOfAnotherSpecies {
        multiplet: Species,
        component: Species,
    },
}

/// Spell a species for a message.
///
/// There is no `Display` on [`Species`] and this does not add one. Which
/// spelling is canonical is a per-source convention rather than a property of
/// the identity, so a message that has to name one says which convention it
/// spelled in rather than letting a default appear to be the identity itself.
fn spelled(species: Species) -> String {
    Convention::SpectroscopicSpaced.render(species)
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::LevelsOfDifferentSpecies { lower, upper } => write!(
                f,
                "a transition joins two levels of one species, and these are {} and {} in the \
                 spaced spectroscopic spelling",
                spelled(*lower),
                spelled(*upper)
            ),
            Refused::EnergiesOnDifferentScales { lower, upper } => write!(
                f,
                "the lower energy is on {lower}'s scale and the upper is on {upper}'s, and the \
                 offset between the two is not known here"
            ),
            Refused::UpperNotAboveLower { lower, upper } => write!(
                f,
                "the upper level at {upper} cm^-1 is not above the lower at {lower} cm^-1"
            ),
            Refused::NotAPosition(why) => {
                write!(f, "the two energies do not differ by a position: {why}")
            }
            Refused::MultipletInsideAMultiplet => {
                f.write_str("a multiplet may not be listed among the components of a multiplet")
            }
            Refused::ComponentOfAnotherSpecies {
                multiplet,
                component,
            } => write!(
                f,
                "a {} multiplet may not contain a {} component, in the spaced spectroscopic \
                 spelling",
                spelled(*multiplet),
                spelled(*component)
            ),
        }
    }
}

/// One transition, as one source records it.
///
/// The two levels and the kind are set at construction and are readable and not
/// writable, so a transition that is missing a level, that spans two species or
/// whose upper level is not above its lower one does not exist rather than
/// existing and failing a check somewhere later.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    lower: Level,
    upper: Level,
    kind: Kind,
    /// The position derived from the two level energies, computed once at
    /// construction. It cannot drift from them, because neither level can be
    /// replaced afterwards.
    ritz_from_levels: VacuumWavenumber,
    observed: Option<Position>,
    ritz: Option<Position>,
    position_not_stated: Option<Position>,
}

/// What comparing a stored Ritz position against the levels found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Agreement {
    /// The two agree inside the width the source stated on its own value.
    Within { difference: f64, stated: f64 },
    /// They do not.
    Beyond { difference: f64, stated: f64 },
    /// The source stated no width on its Ritz position, so there is nothing to
    /// compare the difference against. The difference is reported and no
    /// verdict is, because the alternative is to invent a width and then pass
    /// or fail against it.
    NoStatedWidth { difference: f64 },
    /// The source published no Ritz position. Its own state rather than an
    /// agreement of zero, which reads exactly like a perfect one.
    NoStoredRitz,
}

impl Transition {
    /// A transition between two levels.
    ///
    /// Both levels are arguments rather than fields anybody sets, which is the
    /// whole of the first clause this file exists for: there is no value of
    /// this type with one level, and none with none.
    pub fn new(lower: Level, upper: Level, kind: Kind) -> Result<Self, Refused> {
        if lower.species != upper.species {
            return Err(Refused::LevelsOfDifferentSpecies {
                lower: lower.species,
                upper: upper.species,
            });
        }
        if lower.energy.zero != upper.energy.zero {
            return Err(Refused::EnergiesOnDifferentScales {
                lower: lower.energy.zero.clone(),
                upper: upper.energy.zero.clone(),
            });
        }
        // Not an absolute difference. A pair the wrong way round is refused
        // rather than silently sorted, because sorting it would accept a row
        // whose two columns were swapped and produce a transition nothing
        // downstream could tell from a correct one.
        //
        // Written as a comparison that has to come back `Less` rather than as a
        // negated `>`, so that a pair with no ordering at all falls here too.
        // Two energies one of which is not a number do not compare, and the
        // arithmetic below would otherwise carry that through.
        if lower.energy.value.partial_cmp(&upper.energy.value) != Some(Ordering::Less) {
            return Err(Refused::UpperNotAboveLower {
                lower: lower.energy.value,
                upper: upper.energy.value,
            });
        }
        let ritz_from_levels = VacuumWavenumber::new(upper.energy.value - lower.energy.value)
            .map_err(Refused::NotAPosition)?;

        if let Kind::Multiplet {
            components: Some(components),
        } = &kind
        {
            for component in components {
                if matches!(component.kind, Kind::Multiplet { .. }) {
                    return Err(Refused::MultipletInsideAMultiplet);
                }
                if component.species() != lower.species {
                    return Err(Refused::ComponentOfAnotherSpecies {
                        multiplet: lower.species,
                        component: component.species(),
                    });
                }
            }
        }

        Ok(Transition {
            lower,
            upper,
            kind,
            ritz_from_levels,
            observed: None,
            ritz: None,
            position_not_stated: None,
        })
    }

    /// The same transition carrying the position the source measured.
    #[must_use]
    pub fn with_observed(mut self, position: Position) -> Self {
        self.observed = Some(position);
        self
    }

    /// The same transition carrying the position the source computed from its
    /// level energies.
    #[must_use]
    pub fn with_ritz(mut self, position: Position) -> Self {
        self.ritz = Some(position);
        self
    }

    /// The same transition carrying a position the source published without
    /// saying whether it measured it or computed it.
    ///
    /// A third field rather than a flag on one of the other two, so that no
    /// reader of [`Transition::observed`] or [`Transition::ritz`] has to know
    /// that either might be holding a value the source did not describe.
    #[must_use]
    pub fn with_position_not_stated(mut self, position: Position) -> Self {
        self.position_not_stated = Some(position);
        self
    }

    /// The species this transition belongs to, which is both levels' and cannot
    /// be anything else.
    pub fn species(&self) -> Species {
        self.lower.species
    }

    pub fn lower(&self) -> &Level {
        &self.lower
    }

    pub fn upper(&self) -> &Level {
        &self.upper
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub fn observed(&self) -> Option<&Position> {
        self.observed.as_ref()
    }

    pub fn ritz(&self) -> Option<&Position> {
        self.ritz.as_ref()
    }

    pub fn position_not_stated(&self) -> Option<&Position> {
        self.position_not_stated.as_ref()
    }

    /// The Ritz position of this transition, from the stored level energies and
    /// from nothing else.
    ///
    /// This is a wavenumber because the levels are, which is the reason
    /// `docs/decisions/line-position.md` gives for the representation: the
    /// derived position and the observed one are the same kind of number and no
    /// conversion stands between them.
    pub fn ritz_from_levels(&self) -> VacuumWavenumber {
        self.ritz_from_levels
    }

    /// Whether the Ritz position the source published agrees with the one its
    /// own level energies give, inside the width the source stated on it.
    ///
    /// The width is the source's, not one derived here, for the reason at the
    /// top of this file.
    pub fn ritz_agrees(&self) -> Agreement {
        let Some(stored) = self.ritz.as_ref() else {
            return Agreement::NoStoredRitz;
        };
        let difference = stored.wavenumber.cm_inverse() - self.ritz_from_levels.cm_inverse();
        match stored.uncertainty.widest() {
            Err(_) => Agreement::NoStatedWidth { difference },
            Ok(stated) if difference.abs() <= stated => Agreement::Within { difference, stated },
            Ok(stated) => Agreement::Beyond { difference, stated },
        }
    }
}
