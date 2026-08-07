//! Turning a published accuracy grade into a number, without pretending it was
//! measured.
//!
//! Nobody propagates a letter. That is the whole problem in one sentence, and
//! converting the letter is the point at which this board either adds something
//! or quietly invents something.
//!
//! The conversion and its reasoning are recorded in
//! `docs/decisions/accuracy-grades.md`. The two sentences worth repeating at the
//! code are these. The published bound is read as one standard uncertainty,
//! which is deliberately conservative because the published quantity is a
//! maximum. And a grade with no upper bound converts to no number at all rather
//! than to a large one.

use std::fmt;

/// A published accuracy grade, as read from a source.
///
/// The verbatim spelling is kept whatever else happens, so that revisiting the
/// convention is a recomputation rather than a re-ingest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grade {
    letters: Letters,
    /// The source split a multiplet into components under a pure LS coupling
    /// assumption, and says the true accuracy may be worse than the letter.
    primed: bool,
    verbatim: String,
}

/// The letters of the published scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Letters {
    Aaa,
    Aa,
    APlus,
    A,
    BPlus,
    B,
    CPlus,
    C,
    DPlus,
    D,
    /// Worse than the last bound, with no upper bound of its own.
    E,
}

/// The whole published scale, worst-bound-per-letter, best first.
///
/// The values are the ones issue #28 records from the source's own help page.
/// They were not re-fetched here, because nothing in this tree retrieves
/// anything yet; #26 is what pins them to a snapshot, and until it lands this
/// table is a transcription and is written as one.
const SCALE: [(Letters, &str, f64); 11] = [
    (Letters::Aaa, "AAA", 0.3),
    (Letters::Aa, "AA", 1.0),
    (Letters::APlus, "A+", 2.0),
    (Letters::A, "A", 3.0),
    (Letters::BPlus, "B+", 7.0),
    (Letters::B, "B", 10.0),
    (Letters::CPlus, "C+", 18.0),
    (Letters::C, "C", 25.0),
    (Letters::DPlus, "D+", 40.0),
    (Letters::D, "D", 50.0),
    (Letters::E, "E", 50.0),
];

impl Letters {
    /// The published bound in per cent.
    ///
    /// For every letter but the last this is a maximum: the source states that
    /// the value is within this much. For `E` it is the number the source says
    /// the error is worse than, which is not a bound on anything.
    pub fn published_percent(self) -> f64 {
        SCALE
            .iter()
            .find(|(letters, _, _)| *letters == self)
            .map(|(_, _, percent)| *percent)
            .expect("every letter is in the scale")
    }

    pub fn spelling(self) -> &'static str {
        SCALE
            .iter()
            .find(|(letters, _, _)| *letters == self)
            .map(|(_, spelling, _)| *spelling)
            .expect("every letter is in the scale")
    }

    /// Every letter, best first. The order is the scale's order and is what a
    /// test over the whole table iterates.
    pub fn all() -> impl Iterator<Item = Letters> {
        SCALE.iter().map(|(letters, _, _)| *letters)
    }
}

/// Why a grade was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAGrade {
    Empty,
    /// The letters are not on the published scale.
    UnknownLetters(String),
    /// Something follows the letters that is not the prime this parser knows.
    ///
    /// Refused rather than ignored. A suffix that is dropped silently is a
    /// primed grade converted as an unprimed one, which is the failure the whole
    /// prime handling exists to prevent.
    UnknownSuffix(String),
}

impl fmt::Display for NotAGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotAGrade::Empty => write!(f, "empty grade"),
            NotAGrade::UnknownLetters(s) => write!(f, "{s:?} is not on the published scale"),
            NotAGrade::UnknownSuffix(s) => {
                write!(f, "{s:?} follows the letters and is not a prime")
            }
        }
    }
}

/// The ASCII spelling of the prime this parser accepts.
///
/// The source marks a split multiplet with a prime. Which byte a given served
/// format uses for it is not established here, because nothing in this tree has
/// retrieved anything; #26 and #27 pin it. What is established is the safe
/// direction: an unrecognised suffix is refused rather than dropped, so a
/// spelling this parser does not know reds the ingest instead of silently
/// producing an unprimed grade.
const PRIME: char = '\'';

impl Grade {
    /// Read a published grade, keeping the spelling it arrived in.
    pub fn parse(text: &str) -> Result<Self, NotAGrade> {
        let verbatim = text.to_owned();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(NotAGrade::Empty);
        }

        let (body, primed) = match trimmed.strip_suffix(PRIME) {
            Some(body) => (body, true),
            None => (trimmed, false),
        };

        let letters = SCALE
            .iter()
            .find(|(_, spelling, _)| *spelling == body)
            .map(|(letters, _, _)| *letters);

        match letters {
            Some(letters) => Ok(Grade {
                letters,
                primed,
                verbatim,
            }),
            None => {
                // Distinguish "these are not the letters" from "the letters are
                // fine and something unexpected follows them", because only the
                // second is the silent-drop hazard the prime handling is about.
                let longest = SCALE
                    .iter()
                    .filter(|(_, spelling, _)| body.starts_with(spelling))
                    .map(|(_, spelling, _)| *spelling)
                    .max_by_key(|spelling| spelling.len());
                match longest {
                    Some(spelling) => {
                        Err(NotAGrade::UnknownSuffix(body[spelling.len()..].to_owned()))
                    }
                    None => Err(NotAGrade::UnknownLetters(body.to_owned())),
                }
            }
        }
    }

    pub fn letters(&self) -> Letters {
        self.letters
    }

    pub fn is_primed(&self) -> bool {
        self.primed
    }

    /// The spelling this grade arrived in, kept unchanged.
    pub fn verbatim(&self) -> &str {
        &self.verbatim
    }

    /// The grade as a number, or the statement that it is not one.
    pub fn convert(&self) -> Converted {
        if self.letters == Letters::E {
            return Converted::Unusable {
                worse_than_percent: Letters::E.published_percent(),
            };
        }
        let bound = self.letters.published_percent();
        Converted::Standard {
            // The published bound read as one standard uncertainty. Not the
            // bound divided by the square root of three, which is what a
            // uniform distribution over the bound would give and which would
            // report less uncertainty than the source published.
            percent: bound,
            bound_percent: bound,
            is_lower_bound: self.primed,
        }
    }
}

/// What a grade converts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Converted {
    Standard {
        /// A relative standard uncertainty in per cent.
        percent: f64,
        /// The bound the source published, kept beside the number so that an
        /// answer can show what the number was derived from.
        bound_percent: f64,
        /// Whether `percent` is an estimate or only a lower bound on the true
        /// uncertainty. A primed grade sets this, because the source says the
        /// true accuracy may be worse and does not say by how much.
        is_lower_bound: bool,
    },
    /// The grade states no upper bound, so there is no number to give.
    ///
    /// Not a large number. Anything worse than the stated figure includes values
    /// wrong by a factor, and a finite stand-in would be invented information
    /// that then propagates as if it were data.
    Unusable { worse_than_percent: f64 },
}

impl Converted {
    /// Whether this may be used to weight a marginalisation.
    ///
    /// A lower bound may not. Weighting by an uncertainty that is only a lower
    /// bound rewards whoever reported the least about their own error, which is
    /// the direction this board exists to push against. The weighting itself is
    /// entry 7 of #1 and is not decided here; what is decided here is that this
    /// value is not eligible for it.
    pub fn usable_for_weighting(self) -> bool {
        matches!(
            self,
            Converted::Standard {
                is_lower_bound: false,
                ..
            }
        )
    }
}
