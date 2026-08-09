//! Carrying an uncertainty in log gf into the derived quantity.
//!
//! `docs/decisions/propagation.md` carries the argument and this file does not
//! restate it. Three sentences from it are repeated here because they are what
//! a reader of this code needs in front of them.
//!
//! The derivative is minus one exactly, under an assumption the record names
//! and four exceptions that break it. The curve of growth does not enter,
//! because the equivalent width is not an input.
//!
//! The map has a negative slope, so an asymmetric uncertainty comes out with
//! its halves exchanged. The near miss worth having in mind while reading is
//! the symmetric case, where an implementation that forgets the exchange
//! produces the same bytes as a correct one.
//!
//! Nothing is assumed about the caller's line. Six things have to be
//! established before the arithmetic runs, each of them in one of three states,
//! and the third state refuses rather than taking the common branch.

use std::fmt;

use crate::register::uncertainty::Uncertainty;

/// The derivative the whole propagation is.
///
/// A named constant rather than a literal at the one site that uses it, because
/// it is the conclusion of the record's derivation and not an implementation
/// choice somebody may tune.
pub const DERIVATIVE: f64 = -1.0;

/// Something the caller has to establish before the propagation runs.
///
/// The first four are the exceptions that break the degeneracy. The last two
/// are the inputs #37 names and this board does not carry into the answer: a
/// caller may decide they are negligible, and the point of asking is that the
/// decision cannot be made silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Precondition {
    /// The species does not contribute to the continuous opacity or to the
    /// electron pressure.
    NotAnOpacityContributor,
    /// The line is not blended.
    NotBlended,
    /// The level populations are in local thermodynamic equilibrium.
    InLocalThermodynamicEquilibrium,
    /// No parameter was calibrated by requiring that derived abundance show no
    /// trend with line strength.
    NoCalibrationFeedback,
    /// The uncertainty in the line position may be neglected for this line.
    LinePositionNegligible,
    /// The uncertainty in the level energies may be neglected for this line.
    LevelEnergiesNegligible,
}

impl Precondition {
    /// Every one of them, in the order a refusal reports them in.
    ///
    /// The four exceptions first, in the order the record names them, then the
    /// two negligibility declarations. A caller meeting several refusals at
    /// once meets the physics before the bookkeeping.
    pub const ALL: [Precondition; 6] = [
        Precondition::NotAnOpacityContributor,
        Precondition::NotBlended,
        Precondition::InLocalThermodynamicEquilibrium,
        Precondition::NoCalibrationFeedback,
        Precondition::LinePositionNegligible,
        Precondition::LevelEnergiesNegligible,
    ];

    fn at(self) -> usize {
        match self {
            Precondition::NotAnOpacityContributor => 0,
            Precondition::NotBlended => 1,
            Precondition::InLocalThermodynamicEquilibrium => 2,
            Precondition::NoCalibrationFeedback => 3,
            Precondition::LinePositionNegligible => 4,
            Precondition::LevelEnergiesNegligible => 5,
        }
    }
}

impl fmt::Display for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Precondition::NotAnOpacityContributor => {
                "the species does not contribute to the continuous opacity or the electron pressure"
            }
            Precondition::NotBlended => "the line is not blended",
            Precondition::InLocalThermodynamicEquilibrium => {
                "the level populations are in local thermodynamic equilibrium"
            }
            Precondition::NoCalibrationFeedback => {
                "no parameter was calibrated by requiring no trend with line strength"
            }
            Precondition::LinePositionNegligible => {
                "the line position is known well enough to neglect"
            }
            Precondition::LevelEnergiesNegligible => {
                "the level energies are known well enough to neglect"
            }
        })
    }
}

/// What the caller says about one precondition.
///
/// Three states rather than two, and the third is the reason this type exists.
/// A boolean would spell "not established" as false, which reads as established
/// to be untrue, and the two lead to different answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Answer {
    /// Established, and it holds.
    Holds,
    /// Established, and it does not hold.
    DoesNotHold,
    /// Not established either way.
    #[default]
    NotEstablished,
}

/// What the caller established about the line.
///
/// There is no constructor that establishes everything at once, deliberately.
/// A caller answers each one, and the default for an unanswered precondition is
/// the state that refuses.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Conditions {
    answers: [Answer; Precondition::ALL.len()],
}

impl Conditions {
    /// Nothing established. Every propagation from here refuses.
    pub fn nothing_established() -> Self {
        Self::default()
    }

    /// The same conditions with one precondition answered.
    #[must_use]
    pub fn with(mut self, precondition: Precondition, answer: Answer) -> Self {
        self.answers[precondition.at()] = answer;
        self
    }

    /// What the caller said about one precondition.
    pub fn answer(&self, precondition: Precondition) -> Answer {
        self.answers[precondition.at()]
    }

    /// The first precondition that stops the propagation, if any.
    fn check(&self) -> Result<(), Refused> {
        for precondition in Precondition::ALL {
            match self.answer(precondition) {
                Answer::Holds => {}
                Answer::DoesNotHold => return Err(Refused::DoesNotHold(precondition)),
                Answer::NotEstablished => return Err(Refused::NotEstablished(precondition)),
            }
        }
        Ok(())
    }
}

/// Why a propagation produced no number.
///
/// A refusal is an answer here rather than an error beside a number. What the
/// output does with it is #43 and #44; what this type owes is that the reason
/// is specific enough to act on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Refused {
    /// A precondition the caller established as not holding. The exact
    /// degeneracy does not hold for this line, and what holds instead is not
    /// decided anywhere yet.
    DoesNotHold(Precondition),
    /// A precondition nobody established. The convenient branch is to treat it
    /// as holding, because that is the common case, and taking it would put a
    /// number in front of a reader that rests on an assumption nobody made.
    NotEstablished(Precondition),
    /// The claim carried no quoted uncertainty. Distinct from an uncertainty of
    /// zero, which is a claim that the value is exact.
    NoNumberToCarry,
    /// A sampled propagation was asked for with no draws. Without this the
    /// sample is empty and both halves come out zero, which is the shape of an
    /// exact answer.
    NoDraws,
    /// The two halves that came back were not an uncertainty. Reachable because
    /// the state is constructible without going through its own constructor.
    NotAnUncertainty(crate::register::uncertainty::Refused),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::DoesNotHold(precondition) => write!(
                f,
                "the caller established that {precondition} is false, and the mapping this \
                 board propagates by does not hold there"
            ),
            Refused::NotEstablished(precondition) => write!(
                f,
                "nobody established that {precondition}, so a number here would rest on an \
                 assumption nobody made"
            ),
            Refused::NoNumberToCarry => {
                f.write_str("the claim quoted no uncertainty, and there is nothing to carry")
            }
            Refused::NoDraws => {
                f.write_str("a sampled propagation with no draws has an empty sample")
            }
            Refused::NotAnUncertainty(why) => write!(f, "what came back is not one: {why}"),
        }
    }
}

/// The propagation, applied.
///
/// The halves exchange, because the slope is negative. What was the upper half
/// of the input is the lower half of the answer.
pub fn analytic(log_gf: Uncertainty, established: &Conditions) -> Result<Uncertainty, Refused> {
    established.check()?;
    let (lower, upper) = halves(log_gf)?;
    // The magnitude scales each half and the sign exchanges them.
    let magnitude = DERIVATIVE.abs();
    Uncertainty::asymmetric(upper * magnitude, lower * magnitude).map_err(Refused::NotAnUncertainty)
}

/// The same propagation, computed by sampling instead.
///
/// The independent calculation the analytic route is checked against. It shares
/// no arithmetic with it: the draws are mapped one at a time through
/// [`DERIVATIVE`] and the two halves are recovered from the sample afterwards,
/// so an exchange the analytic route got wrong is not repeated here.
///
/// Deterministic in the seed. A check whose verdict moves between two runs of
/// one commit is not a check, and the record's worked example has to be a
/// number somebody else can reproduce.
pub fn monte_carlo(
    log_gf: Uncertainty,
    established: &Conditions,
    draws: usize,
    seed: u64,
) -> Result<Uncertainty, Refused> {
    established.check()?;
    let (lower, upper) = halves(log_gf)?;
    if draws == 0 {
        return Err(Refused::NoDraws);
    }

    // The input is two half-normals joined at zero, one scale on each side,
    // taken in proportion to their scales. That is the distribution whose two
    // conditional root mean squares are the two halves it was built from, which
    // is what makes the recovery below the inverse of the construction rather
    // than an estimator that happens to be close.
    let width = lower + upper;
    let below_share = if width > 0.0 { lower / width } else { 0.0 };

    let mut stream = Stream::seeded(seed);
    let mut below = Sample::default();
    let mut above = Sample::default();
    for _ in 0..draws {
        let side = stream.unit();
        let magnitude = stream.half_normal();
        let drawn = if side < below_share {
            -magnitude * lower
        } else {
            magnitude * upper
        };
        let carried = DERIVATIVE * drawn;
        if carried < 0.0 {
            below.push(carried);
        } else if carried > 0.0 {
            above.push(carried);
        }
    }

    Uncertainty::asymmetric(below.root_mean_square(), above.root_mean_square())
        .map_err(Refused::NotAnUncertainty)
}

/// The two halves of a quoted uncertainty, or the refusal for an absent one.
fn halves(quoted: Uncertainty) -> Result<(f64, f64), Refused> {
    match quoted {
        Uncertainty::Quoted { minus, plus } => Ok((minus, plus)),
        Uncertainty::Absent => Err(Refused::NoNumberToCarry),
    }
}

/// One side of the sample, kept as a running second moment.
#[derive(Default)]
struct Sample {
    squares: f64,
    count: usize,
}

impl Sample {
    fn push(&mut self, drawn: f64) {
        self.squares += drawn * drawn;
        self.count += 1;
    }

    /// Zero for an empty side, which is the right answer rather than a fallback:
    /// a side with no draws is a side the input gave no width to.
    fn root_mean_square(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        (self.squares / self.count as f64).sqrt()
    }
}

/// The draws.
///
/// Written here rather than taken from a crate. `Cargo.toml` carries no
/// dependency, and the first one is a decision recorded on #1 rather than
/// something a sampler in a test path introduces. This is the sixty-four bit
/// mixing function that a seed is usually expanded with; it is used because it
/// is short enough to read and its state is one integer, so a seed and a draw
/// count fix every number this produces.
struct Stream {
    state: u64,
}

impl Stream {
    fn seeded(seed: u64) -> Self {
        Stream { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// Strictly between zero and one, so the logarithm below is finite.
    fn unit(&mut self) -> f64 {
        const STEPS: f64 = 9_007_199_254_740_992.0;
        ((self.next() >> 11) as f64 + 0.5) / STEPS
    }

    /// The magnitude of a standard normal draw.
    fn half_normal(&mut self) -> f64 {
        let radius = (-2.0 * self.unit().ln()).sqrt();
        let angle = std::f64::consts::TAU * self.unit();
        (radius * angle.cos()).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first draws of one seed, pasted from a run.
    ///
    /// `docs/decisions/propagation.md` says a seed and a draw count fix every
    /// digit of its worked example, and until this case nothing held that. The
    /// comparison in `tests/propagation.rs` reads two conditional second
    /// moments out of the sample, and a stream whose mixing arithmetic changed
    /// still produces the same two moments. Measured rather than supposed, by
    /// seeding faults into this file for #41.
    #[test]
    fn the_stream_is_the_one_the_record_promises() {
        let mut stream = Stream::seeded(20_260_809);
        let drawn: Vec<f64> = (0..4).map(|_| stream.half_normal()).collect();
        let shown: Vec<String> = drawn.iter().map(|one| format!("{one:.9}")).collect();
        assert_eq!(
            shown,
            ["0.749817072", "0.062123399", "0.086027397", "0.034995640"]
        );
    }

    /// The magnitudes have the shape they are meant to have.
    ///
    /// The case above pins the arithmetic and would pass over a constant stream
    /// if the constant were the pasted one. This is the half that says the
    /// magnitudes are a standard normal's, against two published numbers rather
    /// than against another implementation in this tree: the mean magnitude is
    /// the square root of two over pi, and just under a third of draws exceed
    /// one.
    ///
    /// Replacing the whole draw with a constant is what a seeded fault found
    /// surviving, and it survived because a point mass at a width carries the
    /// same second moment as a normal of that width.
    #[test]
    fn the_magnitudes_are_those_of_a_standard_normal() {
        const DRAWS: usize = 200_000;
        let mut stream = Stream::seeded(20_260_809);
        let mut total = 0.0;
        let mut beyond_one = 0usize;
        for _ in 0..DRAWS {
            let magnitude = stream.half_normal();
            assert!(magnitude >= 0.0, "a magnitude is never negative");
            total += magnitude;
            if magnitude > 1.0 {
                beyond_one += 1;
            }
        }

        let mean = total / DRAWS as f64;
        let published = (2.0 / std::f64::consts::PI).sqrt();
        assert!(
            (mean - published).abs() < 0.01,
            "the mean magnitude is {mean}, against {published}"
        );

        let share = beyond_one as f64 / DRAWS as f64;
        assert!((share - 0.317_310_5).abs() < 0.01, "the share is {share}");
    }
}
