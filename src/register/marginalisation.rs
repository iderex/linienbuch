//! Combining competing claims about one quantity, with the weighting stated in
//! the answer.
//!
//! The second number the board's argument rests on. One source quotes one
//! width. Marginalising over the sources that compete with it gives a wider
//! one wherever they disagree, and the gap between the two is the thing a
//! reader of a single compilation never sees.
//!
//! What comes back is a mixture rather than a replacement. The components and
//! their weights are in the return value, so the claims are still there after
//! the operation and the two summary numbers can be recomputed from them. That
//! is `docs/decisions/claims-not-values.md` held by the shape of the return
//! rather than by whoever remembers it.
//!
//! ## The arithmetic
//!
//! With weights `w` normalised over the contributing claims, values `x`, and
//! the two halves `m` and `p` of each quoted width:
//!
//! ```text
//! mean     = sum(w * x)
//! between² = sum(w * (x - mean)²)
//! lower    = sqrt(sum(w * m²) + between²)
//! upper    = sqrt(sum(w * p²) + between²)
//! ```
//!
//! The law of total variance for a mixture, applied to each half on its own so
//! that an asymmetric width stays asymmetric. The between term is the spread of
//! the sources against each other and is the half a single source cannot see.
//!
//! One property of that formula is worth stating because it is the direction
//! this board guards: `sum(w * p²) >= min(p)²`, so the answer is never narrower
//! than the narrowest claim it was given, whatever the weights are. It is not
//! always wider than the widest, and it must not be advertised as if it were.
//! Two sources that agree exactly and quote the same width come out at that
//! width, which is the correct answer and not a failure of the operation.
//!
//! ## The weighting
//!
//! Entry 7 of #1 is answered: weight by the category of the method, equal
//! within a category. Which numbers those categories carry is a table that
//! lives in a record, not here. Nothing in this file ships a default, and
//! [`marginalise`] takes the weighting it was handed, so the number this board
//! prints always names the table it came from.
//!
//! ## Three preconditions, all refusals
//!
//! The claims are about one quantity and one subject. Their ancestries are
//! pairwise disjoint, which [`may_marginalise`] decides. And every contributing
//! claim carries a width, so a set holding one that does not either leaves it
//! out and says which, or refuses.
//!
//! Nothing here names a quantity, a unit or a subject beyond carrying the ones
//! the claims already hold. A register of material parameters would combine its
//! competing values in these words.

use std::fmt;

use crate::register::ancestry::{NotIndependent, may_marginalise};
use crate::register::claims::{Claim, Claims, MethodClass, QuantityId, SubjectId, Unit};
use crate::register::provenance::ClaimId;
use crate::register::uncertainty::{self, Uncertainty};

/// A weighting over the categories of method, under a name.
///
/// The name is not decoration. It travels into [`Formation`] and out with the
/// number, so that a reader who disagrees with the weighting can say which one
/// they are disagreeing with.
#[derive(Debug, Clone, PartialEq)]
pub struct Weighting {
    name: String,
    weights: [f64; MethodClass::ALL.len()],
}

/// Why a table of weights is not a weighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAWeighting {
    /// No name, so the answer could not say what produced it.
    Unnamed,
    /// A category with no weight. The table is one row per category and holds
    /// as many rows as there are categories, so a category left out is a
    /// category named twice, and both arrive here.
    NoWeightFor(MethodClass),
    /// A weight that is not a number.
    NotFinite(MethodClass),
    /// A negative weight. A weight is a share of the answer and a negative
    /// share is not a smaller one.
    Negative(MethodClass),
    /// Every weight zero, which weights nothing rather than weighting evenly.
    NothingWeighs,
}

impl fmt::Display for NotAWeighting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotAWeighting::Unnamed => {
                f.write_str("a weighting with no name cannot be stated in an answer")
            }
            NotAWeighting::NoWeightFor(class) => {
                write!(f, "nothing weighs a claim that is {class}")
            }
            NotAWeighting::NotFinite(class) => {
                write!(f, "the weight for {class} is not a number")
            }
            NotAWeighting::Negative(class) => {
                write!(f, "the weight for {class} is below zero")
            }
            NotAWeighting::NothingWeighs => {
                f.write_str("every weight is zero, which weights nothing rather than evenly")
            }
        }
    }
}

impl Weighting {
    /// A weighting from a table with one row per category.
    ///
    /// The table is a fixed length array of pairs rather than a map, so the
    /// compiler counts the rows and each row says which category it is for.
    /// What is left to refuse is a category named twice, which is the same
    /// table as one left out.
    pub fn named(
        name: impl Into<String>,
        table: [(MethodClass, f64); MethodClass::ALL.len()],
    ) -> Result<Self, NotAWeighting> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(NotAWeighting::Unnamed);
        }

        let mut weights = [f64::NAN; MethodClass::ALL.len()];
        let mut given = [false; MethodClass::ALL.len()];
        for (class, weight) in table {
            weights[class.at()] = weight;
            given[class.at()] = true;
        }

        for class in MethodClass::ALL {
            if !given[class.at()] {
                return Err(NotAWeighting::NoWeightFor(class));
            }
            let weight = weights[class.at()];
            if !weight.is_finite() {
                return Err(NotAWeighting::NotFinite(class));
            }
            if weight < 0.0 {
                return Err(NotAWeighting::Negative(class));
            }
        }

        if weights.iter().all(|weight| *weight == 0.0) {
            return Err(NotAWeighting::NothingWeighs);
        }

        Ok(Weighting { name, weights })
    }

    /// What this weighting is called.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The weight this table gives one category, before normalisation.
    pub fn of(&self, class: MethodClass) -> f64 {
        self.weights[class.at()]
    }
}

/// What to do about a claim whose source quoted no width.
///
/// Two states and no default. The issue this implements says the set either
/// leaves such a claim out explicitly and says so, or the operation refuses,
/// and a default here would pick one of those for a caller who never thought
/// about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithoutAWidth {
    /// Refuse the whole set.
    Refuse,
    /// Leave the claim out, and name it in the [`Formation`].
    LeaveOut,
}

/// One claim's part in the answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub claim: ClaimId,
    pub class: MethodClass,
    /// The weight this claim carried in the answer, normalised over the parts,
    /// so the parts sum to one.
    pub weight: f64,
    pub value: f64,
    pub quoted: Uncertainty,
}

/// How the answer was formed.
///
/// Part of the return value rather than a line printed somewhere, because the
/// number is going to be copied into somebody else's document and the weighting
/// has to arrive with it.
#[derive(Debug, Clone, PartialEq)]
pub struct Formation {
    /// The name of the weighting, as it was handed in.
    pub weighting: String,
    /// Every contributing claim, in the order they were given.
    pub parts: Vec<Part>,
    /// The claims left out, which today is exactly those that quoted no width.
    /// Named rather than counted, so a reader can go and look at them.
    pub left_out: Vec<ClaimId>,
    /// The weighted spread of the claims against each other. Zero where they
    /// agree exactly.
    pub between: f64,
    /// The weighted mean of the claims' own squared halves, as a width. This is
    /// what a reader who does not want the spread term is asking for.
    pub within: Uncertainty,
}

/// A mixture over the competing claims, and the statement of how it was made.
#[derive(Debug, Clone, PartialEq)]
pub struct Marginal {
    quantity: QuantityId,
    about: SubjectId,
    unit: Unit,
    value: f64,
    spread: Uncertainty,
    formation: Formation,
}

impl Marginal {
    pub fn quantity(&self) -> &QuantityId {
        &self.quantity
    }

    pub fn about(&self) -> &SubjectId {
        &self.about
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    /// The mean of the mixture.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The width of the mixture, both halves.
    pub fn spread(&self) -> Uncertainty {
        self.spread
    }

    /// What formed it.
    pub fn formation(&self) -> &Formation {
        &self.formation
    }
}

/// Why a set of claims produced no answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// No claims were named, so there was nothing to combine.
    NoClaims,
    /// A claim this register does not hold. Reported rather than skipped, for
    /// the reason [`NotIndependent::UnknownClaim`] gives: answering about the
    /// claims that happen to exist answers a question nobody asked.
    UnknownClaim(ClaimId),
    /// Two claims that are not about one thing. The pair is named rather than
    /// the set, because the pair is what a reader has to look at.
    NotOneSubject { left: ClaimId, right: ClaimId },
    /// Two claims about one thing whose sources wrote the value in different
    /// units. Nothing here converts, and combining them would be arithmetic on
    /// numbers that do not mean the same thing.
    NotOneUnit { left: ClaimId, right: ClaimId },
    /// A value that is not a number. The record holds the value as a bare
    /// number with no constructor to go through, so this is reachable, and one
    /// of these poisons a mean silently.
    NotANumber(ClaimId),
    /// The ancestries are not pairwise disjoint, or one of them was never
    /// followed to an end this register holds.
    NotIndependent(NotIndependent),
    /// A claim quoting no width, where the caller asked to be refused.
    NoWidth(ClaimId),
    /// Every claim was left out, so the mixture has no components.
    NothingLeft,
    /// The weighting gives every contributing claim a weight of zero. Distinct
    /// from a table that is zero everywhere, which is refused when the table is
    /// built: this is a usable table meeting a set it has nothing to say about.
    NoWeightOverThisSet,
    /// What came back was not a width. Reachable because the state is
    /// constructible without going through its own constructor.
    NotAWidth(uncertainty::Refused),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::NoClaims => {
                f.write_str("no claims were named, so there is nothing to combine")
            }
            Refused::UnknownClaim(id) => write!(f, "no claim {id} in this register"),
            Refused::NotOneSubject { left, right } => write!(
                f,
                "{left} and {right} are not about one quantity of one thing"
            ),
            Refused::NotOneUnit { left, right } => write!(
                f,
                "{left} and {right} are written in different units, and nothing here converts"
            ),
            Refused::NotANumber(id) => write!(f, "the value of {id} is not a number"),
            Refused::NotIndependent(why) => write!(f, "{why}"),
            Refused::NoWidth(id) => write!(
                f,
                "{id} quoted no width, and the caller asked to be refused rather than to drop it"
            ),
            Refused::NothingLeft => {
                f.write_str("every claim was left out, so there are no components to combine")
            }
            Refused::NoWeightOverThisSet => f.write_str(
                "this weighting gives every one of these claims a weight of zero, so the \
                 mixture has no shares to divide",
            ),
            Refused::NotAWidth(why) => write!(f, "what came back is not one: {why}"),
        }
    }
}

/// Marginalise over a set of competing claims.
///
/// The order the refusals are tried in is chosen rather than incidental. What a
/// claim is about is settled before anything is read off it, independence
/// before any arithmetic, and the width policy last, so that a set with two
/// defects reports the one a reader has to fix first.
pub fn marginalise(
    register: &Claims,
    over: &[ClaimId],
    weighting: &Weighting,
    without: WithoutAWidth,
) -> Result<Marginal, Refused> {
    if over.is_empty() {
        return Err(Refused::NoClaims);
    }

    let mut held: Vec<&Claim> = Vec::with_capacity(over.len());
    for id in over {
        match register.get(id) {
            Some(claim) => held.push(claim),
            None => return Err(Refused::UnknownClaim(id.clone())),
        }
    }

    let first = held[0];
    for claim in &held[1..] {
        if claim.quantity != first.quantity || claim.about != first.about {
            return Err(Refused::NotOneSubject {
                left: first.id.clone(),
                right: claim.id.clone(),
            });
        }
        if claim.unit != first.unit {
            return Err(Refused::NotOneUnit {
                left: first.id.clone(),
                right: claim.id.clone(),
            });
        }
    }

    for claim in &held {
        if !claim.value.is_finite() {
            return Err(Refused::NotANumber(claim.id.clone()));
        }
    }

    may_marginalise(register, over).map_err(Refused::NotIndependent)?;

    // The two halves are taken here rather than later, so that the arithmetic
    // below runs on numbers this function has already refused the bad cases of.
    // The match is exhaustive and neither arm is dead: a state built by hand
    // rather than through its own constructor reaches both of the inner two.
    let mut contributing: Vec<(&Claim, f64, f64)> = Vec::with_capacity(held.len());
    let mut left_out: Vec<ClaimId> = Vec::new();
    for claim in held {
        match claim.uncertainty {
            Uncertainty::Absent => match without {
                WithoutAWidth::Refuse => return Err(Refused::NoWidth(claim.id.clone())),
                WithoutAWidth::LeaveOut => left_out.push(claim.id.clone()),
            },
            Uncertainty::Quoted { minus, plus } => {
                if !minus.is_finite() || !plus.is_finite() {
                    return Err(Refused::NotAWidth(uncertainty::Refused::NotFinite));
                }
                if minus < 0.0 || plus < 0.0 {
                    return Err(Refused::NotAWidth(uncertainty::Refused::Negative));
                }
                contributing.push((claim, minus, plus));
            }
        }
    }
    if contributing.is_empty() {
        return Err(Refused::NothingLeft);
    }

    let total: f64 = contributing
        .iter()
        .map(|(claim, _, _)| weighting.of(claim.method.class()))
        .sum();
    if total <= 0.0 {
        return Err(Refused::NoWeightOverThisSet);
    }

    let parts: Vec<Part> = contributing
        .iter()
        .map(|(claim, _, _)| Part {
            claim: claim.id.clone(),
            class: claim.method.class(),
            weight: weighting.of(claim.method.class()) / total,
            value: claim.value,
            quoted: claim.uncertainty,
        })
        .collect();

    let mean: f64 = parts.iter().map(|part| part.weight * part.value).sum();
    let between_squared: f64 = parts
        .iter()
        .map(|part| part.weight * (part.value - mean).powi(2))
        .sum();
    // The weighted mean of each half squared. Kept squared, because both widths
    // below add the spread term before taking a root and rooting twice would be
    // arithmetic nobody asked for.
    let mut own_lower = 0.0;
    let mut own_upper = 0.0;
    for (part, (_, lower, upper)) in parts.iter().zip(&contributing) {
        own_lower += part.weight * lower * lower;
        own_upper += part.weight * upper * upper;
    }

    let within = width(own_lower, own_upper)?;
    let spread = width(own_lower + between_squared, own_upper + between_squared)?;

    // Every held claim agrees with the first about all three of these, which is
    // what the second loop above refused a set for not doing. Reading them off
    // the first rather than off a contributing one keeps this independent of
    // which claims the width policy left out.
    Ok(Marginal {
        quantity: first.quantity.clone(),
        about: first.about.clone(),
        unit: first.unit.clone(),
        value: mean,
        spread,
        formation: Formation {
            weighting: weighting.name().to_owned(),
            parts,
            left_out,
            between: between_squared.sqrt(),
            within,
        },
    })
}

/// A width from the two squared halves.
///
/// The one place a pair of numbers becomes a width in this file, so the refusal
/// that a pair is not one has a single site rather than one per caller. It is
/// reached where a quoted half is large enough that its square is not a
/// representable number, which is the last way a set of well formed inputs can
/// arrive here as something that is not a width.
fn width(lower_squared: f64, upper_squared: f64) -> Result<Uncertainty, Refused> {
    Uncertainty::asymmetric(lower_squared.sqrt(), upper_squared.sqrt()).map_err(Refused::NotAWidth)
}
