//! Energy levels, and the fields their identity rests on.
//!
//! Transition identity rests on level identity, so this is where the accuracy of
//! the whole join is set. Every optional field here is optional because sources
//! genuinely leave it out, and each one is separately present or absent: a level
//! matched on energy and J alone is a weaker match than one that also matched
//! configuration, and that difference has to survive into what an answer is
//! allowed to claim.
//!
//! Two things a schema is tempted to flatten and this one does not.
//!
//! A level energy is a measurement with an uncertainty of its own, which
//! propagates into a Ritz position. Storing it as a bare number throws that away
//! at the point where it is cheapest to keep.
//!
//! The reference zero of the energy scale is a property of the source rather
//! than a universal. Two sources can differ by a constant offset, which looks
//! like disagreement about every level at once and is disagreement about one
//! thing. [`compare`] is what tells those apart.

use crate::register::provenance::SourceId;
use crate::register::uncertainty::Uncertainty;
use crate::spectroscopy::species::Species;
use std::collections::BTreeMap;
use std::fmt;

/// The reference zero a source measures its level energies from.
///
/// Named rather than assumed, because "the ground state" is not one number
/// across sources: a source may measure from the ground state of the neutral
/// atom, from the ground state of the ion, or from something it defines itself,
/// and two of those differ by a constant that is invisible in any single level.
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyZero {
    pub source: SourceId,
    /// What the source calls its zero, in the source's own words.
    pub description: String,
    /// The ionisation limit on this scale in cm^-1, where the source states one.
    ///
    /// Absent is a source that did not state one, which is a different thing
    /// from a source whose limit this board did not record. The second is a gap
    /// to repair and the first is a fact about the source.
    pub ionisation_limit: Option<f64>,
}

/// A level energy, on a named scale, with what the source said about it.
#[derive(Debug, Clone, PartialEq)]
pub struct Energy {
    /// In cm^-1, measured from the zero named below and from nothing else.
    pub value: f64,
    pub uncertainty: Uncertainty,
    /// Whose scale this number is on. Two energies on different scales are not
    /// comparable until the offset between the scales is known.
    pub zero: SourceId,
}

/// The parity of a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parity {
    Even,
    Odd,
}

/// The total angular momentum J, in units of hbar.
///
/// Half integral for an odd number of electrons, so it is stored doubled and
/// exact rather than as a float. `J = 5/2` is `TotalAngularMomentum(5)`. A float
/// here would make two sources spelling one level's J differently compare
/// unequal for a reason that is about binary representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TotalAngularMomentum(u16);

impl TotalAngularMomentum {
    /// From twice J, which is always an integer.
    pub fn from_doubled(doubled: u16) -> Self {
        TotalAngularMomentum(doubled)
    }

    pub fn doubled(self) -> u16 {
        self.0
    }
}

impl fmt::Display for TotalAngularMomentum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_multiple_of(2) {
            write!(f, "{}", self.0 / 2)
        } else {
            write!(f, "{}/2", self.0)
        }
    }
}

/// One energy level as one source records it.
///
/// Species and energy are required: a level with no species is not a level, and
/// a level with no energy is nothing this board can do anything with. Everything
/// else is separately present or absent.
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub species: Species,
    pub energy: Energy,
    pub j: Option<TotalAngularMomentum>,
    pub parity: Option<Parity>,
    /// The electron configuration as the source writes it, unnormalised.
    pub configuration: Option<String>,
    /// The term designation as the source writes it, unnormalised.
    pub term: Option<String>,
}

/// The fields of a level that are present, other than the energy.
///
/// What a match rested on, so that an answer can say how strong the match was
/// rather than only that there was one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Presence {
    pub j: bool,
    pub parity: bool,
    pub configuration: bool,
    pub term: bool,
}

impl Level {
    pub fn presence(&self) -> Presence {
        Presence {
            j: self.j.is_some(),
            parity: self.parity.is_some(),
            configuration: self.configuration.is_some(),
            term: self.term.is_some(),
        }
    }

    /// The fields two levels can be paired on, where both carry them.
    ///
    /// The energy is deliberately not in it. Pairing on the energy is what makes
    /// a constant offset look like a set of mismatches, which is the failure
    /// [`compare`] exists to tell apart.
    fn key(&self) -> Key {
        Key {
            species: self.species,
            j: self.j,
            parity: self.parity,
            configuration: self.configuration.clone(),
            term: self.term.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    species: Species,
    j: Option<TotalAngularMomentum>,
    parity: Option<Parity>,
    configuration: Option<String>,
    term: Option<String>,
}

/// One level that appears in both sets and whose energies do not agree.
#[derive(Debug, Clone, PartialEq)]
pub struct Disagreement {
    pub species: Species,
    pub j: Option<TotalAngularMomentum>,
    pub configuration: Option<String>,
    /// The second energy minus the first, in cm^-1.
    pub difference: f64,
}

/// What comparing two sources' level sets found.
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    /// Every paired level differs by one constant. The two sources agree about
    /// the spectrum and disagree about where zero is, which is one finding
    /// rather than one per level.
    ConstantOffset {
        /// The second scale minus the first, in cm^-1.
        offset: f64,
        /// How many levels the constant was established over. One pair
        /// establishes nothing, and the caller can see that.
        over_pairs: usize,
    },
    /// The paired levels do not differ by one constant.
    ///
    /// The offset that best fits is carried anyway, because the interesting
    /// report is usually "a constant plus these three", and hiding it would make
    /// a reader recompute it.
    Disagreements {
        best_offset: f64,
        beyond_tolerance: Vec<Disagreement>,
        over_pairs: usize,
    },
    /// Nothing could be paired, so nothing was compared.
    ///
    /// Its own state rather than an empty disagreement list, because an empty
    /// list of mismatches over nothing reads exactly like agreement.
    NothingPaired,
}

/// Levels that appear in one set and not the other, reported beside whatever the
/// comparison found rather than dropped.
#[derive(Debug, Clone, PartialEq)]
pub struct Unpaired {
    pub only_in_first: usize,
    pub only_in_second: usize,
}

/// Compare two sources' level sets and say whether one constant explains the
/// difference.
///
/// The tolerance is the caller's, in cm^-1, and choosing it is not this
/// function's business. How far apart two sources actually put one level is a
/// measurement rather than a constant, and #34 is where it is taken. What this
/// function owes is that the answer is one finding where one constant explains
/// everything, and a list only where it does not.
pub fn compare(first: &[Level], second: &[Level], tolerance: f64) -> (Comparison, Unpaired) {
    let mut by_key: BTreeMap<Key, &Level> = BTreeMap::new();
    for level in first {
        by_key.insert(level.key(), level);
    }

    let mut paired: Vec<(&Level, &Level)> = Vec::new();
    let mut only_in_second = 0;
    for level in second {
        match by_key.remove(&level.key()) {
            Some(other) => paired.push((other, level)),
            None => only_in_second += 1,
        }
    }
    let unpaired = Unpaired {
        only_in_first: by_key.len(),
        only_in_second,
    };

    if paired.is_empty() {
        return (Comparison::NothingPaired, unpaired);
    }

    let differences: Vec<f64> = paired
        .iter()
        .map(|(a, b)| b.energy.value - a.energy.value)
        .collect();

    // The median, and neither of the two things it is easy to reach for first.
    //
    // Not the first difference: that makes the answer depend on the order the
    // levels arrived in, so two runs over one pair of sources report different
    // offsets for the same data.
    //
    // Not the mean, and this one is measured rather than supposed. With four
    // levels, three agreeing and one out by 4 cm^-1, the mean sits 1 cm^-1 away
    // from the constant the three share, and at any tolerance below that all
    // four fall outside it. One disagreement becomes a list as long as the level
    // set, which is precisely the failure this function exists to prevent, and
    // the mean produces it from the inside.
    //
    // The median moves by at most one position for one outlier, so the three
    // that agree still agree with it.
    let mut sorted = differences.clone();
    sorted.sort_by(f64::total_cmp);
    let best_offset = sorted[sorted.len() / 2];

    let beyond: Vec<Disagreement> = paired
        .iter()
        .zip(&differences)
        .filter(|(_, difference)| (**difference - best_offset).abs() > tolerance)
        .map(|((a, _), difference)| Disagreement {
            species: a.species,
            j: a.j,
            configuration: a.configuration.clone(),
            difference: *difference,
        })
        .collect();

    let over_pairs = paired.len();
    if beyond.is_empty() {
        (
            Comparison::ConstantOffset {
                offset: best_offset,
                over_pairs,
            },
            unpaired,
        )
    } else {
        (
            Comparison::Disagreements {
                best_offset,
                beyond_tolerance: beyond,
                over_pairs,
            },
            unpaired,
        )
    }
}
