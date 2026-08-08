//! An uncertainty, and the states it can be in.
//!
//! `docs/decisions/uncertainty-representation.md` names four states that are not
//! interchangeable: a quoted number, a published bound, absent, and derived.
//! Two of them are here. The other two are not, and their absence is deliberate
//! rather than an omission.
//!
//! A bound already has a home: the accuracy grade scale is what publishes one in
//! this field, and turning it into a number is decided and implemented in the
//! spectroscopy side. Lifting that into a general state belongs with the claim
//! record in #23, where the two meet, and doing it here would leave two partial
//! representations of one idea in the tree at once.
//!
//! Derived is the same case. It carries what it was derived from, which is a
//! provenance edge, and provenance edges are #23.
//!
//! What forced the two that are here is the level energy in #21. A level energy
//! is a measurement with an uncertainty of its own, and a schema that stores it
//! as a bare number throws that away at the point where it is cheapest to keep.
//!
//! Absent is not zero and it is not infinity. Nothing here converts it into a
//! number, and the only way to read one out is to say what happens when there is
//! none.

use std::fmt;

/// What a source said about how well it knows a value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Uncertainty {
    /// A quoted standard uncertainty, in the unit of the value it is about.
    ///
    /// Asymmetric is stored as two numbers rather than as the larger of them,
    /// because collapsing it loses the direction the value is uncertain in and
    /// that direction is often the interesting half. A symmetric one is the same
    /// number twice, so nothing downstream has to know which it was reading.
    Quoted { minus: f64, plus: f64 },
    /// The source gave none.
    ///
    /// A distinct state rather than a sentinel, because every sentinel this
    /// could be spelled as is a value some source legitimately quotes.
    Absent,
}

/// What an operation says when it needed a number and there was none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoNumber;

impl fmt::Display for NoNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the source quoted no uncertainty, and there is no number to use")
    }
}

/// A quoted uncertainty that was refused before it existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// One half of a quoted uncertainty was negative. A standard uncertainty is
    /// a width and a negative width is not a smaller one.
    Negative,
    /// One half was not a number at all, which is what arithmetic on an absent
    /// value produces if somebody routed around this type.
    NotFinite,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::Negative => f.write_str("an uncertainty may not be negative"),
            Refused::NotFinite => f.write_str("an uncertainty must be a finite number"),
        }
    }
}

impl Uncertainty {
    /// A symmetric quoted uncertainty.
    pub fn symmetric(width: f64) -> Result<Self, Refused> {
        Self::asymmetric(width, width)
    }

    /// A quoted uncertainty with two halves.
    pub fn asymmetric(minus: f64, plus: f64) -> Result<Self, Refused> {
        for half in [minus, plus] {
            if !half.is_finite() {
                return Err(Refused::NotFinite);
            }
            if half < 0.0 {
                return Err(Refused::Negative);
            }
        }
        Ok(Uncertainty::Quoted { minus, plus })
    }

    /// The widest of the two halves, for an operation that needs one number.
    ///
    /// Returns [`NoNumber`] where the source quoted none. That is the whole of
    /// the rule this type exists for: an absent uncertainty is not consumed
    /// silently, and every caller that wants a number has to say what it does
    /// when there is not one.
    pub fn widest(self) -> Result<f64, NoNumber> {
        match self {
            Uncertainty::Quoted { minus, plus } => Ok(minus.max(plus)),
            Uncertainty::Absent => Err(NoNumber),
        }
    }

    pub fn is_absent(self) -> bool {
        matches!(self, Uncertainty::Absent)
    }
}
