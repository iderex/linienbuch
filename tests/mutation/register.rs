//! The register a mutation run is judged against, and the comparison itself.
//!
//! A module rather than a test file, so the parsing and the comparison can be
//! proved on constructed input by the default suite in
//! `tests/mutation_register.rs` and used against the real thing by the leg in
//! `tests/integration/main.rs`. The run costs minutes and needs a tool that is
//! not part of the toolchain; the rules it is judged by cost neither and are
//! proved where every run sees them.
//!
//! The format is line based and parsed here rather than taken from a crate.
//! `Cargo.toml` carries no dependency and the first one is a decision recorded
//! on #1, so a register that needed a parser from elsewhere would be that
//! decision arriving inside a test file. What the format has to carry is four
//! kinds of line, which is less than a general one would cost to read.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// A surviving mutant that is accepted, with the debt it carries.
///
/// Not a dispensation. `because` says why no test kills it and `retired_by`
/// says what would remove the entry, so an entry nobody can retire is visible
/// as one rather than as a line somebody stopped reading.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Accepted {
    pub mutant: String,
    pub because: String,
    pub retired_by: String,
}

/// The tracked register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Register {
    /// The tool, by the name that runs it.
    pub tool: String,
    /// The exact version. Two runs of one commit under different versions are
    /// two runs of different things, and nothing else in this tree pins it:
    /// `rust-toolchain.toml` pins the compiler and says so in its own comment.
    pub version: String,
    /// The files a run covers, in the order they are listed.
    pub scope: Vec<String>,
    /// The survivors that are accounted for.
    pub accepted: Vec<Accepted>,
}

/// What a register cannot be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotARegister {
    /// A key the format does not know. Refused rather than skipped, because a
    /// misspelled `Retired-by` would otherwise remove an obligation silently.
    UnknownKey(String),
    /// A line that is not a comment, not blank and carries no key.
    Unkeyed(String),
    /// A required field appears twice or not at all.
    Field(String),
    /// An accepted entry missing the debt it has to carry.
    Incomplete(String),
    /// A `Because` or `Retired-by` with no `Accepted` above it.
    Orphaned(String),
    /// Two accepted entries naming one mutant.
    Duplicate(String),
}

impl std::fmt::Display for NotARegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotARegister::UnknownKey(key) => write!(f, "{key:?} is not a key this format knows"),
            NotARegister::Unkeyed(line) => write!(f, "{line:?} carries no key"),
            NotARegister::Field(what) => write!(f, "{what}"),
            NotARegister::Incomplete(mutant) => {
                write!(f, "accepted with no reason and no retirement: {mutant:?}")
            }
            NotARegister::Orphaned(key) => write!(f, "{key:?} with no accepted entry above it"),
            NotARegister::Duplicate(mutant) => write!(f, "accepted twice: {mutant:?}"),
        }
    }
}

/// Read a register.
pub fn parse(text: &str) -> Result<Register, NotARegister> {
    let mut tool: Option<String> = None;
    let mut version: Option<String> = None;
    let mut scope: Vec<String> = Vec::new();
    let mut accepted: Vec<Accepted> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(NotARegister::Unkeyed(line.to_owned()));
        };
        let value = value.trim().to_owned();
        match key.trim() {
            "Tool" => once("Tool", &mut tool, value)?,
            "Version" => once("Version", &mut version, value)?,
            "Scope" => scope.push(value),
            "Accepted" => {
                if accepted.iter().any(|one| one.mutant == value) {
                    return Err(NotARegister::Duplicate(value));
                }
                accepted.push(Accepted {
                    mutant: value,
                    because: String::new(),
                    retired_by: String::new(),
                });
            }
            "Because" => match accepted.last_mut() {
                Some(entry) => entry.because = value,
                None => return Err(NotARegister::Orphaned("Because".to_owned())),
            },
            "Retired-by" => match accepted.last_mut() {
                Some(entry) => entry.retired_by = value,
                None => return Err(NotARegister::Orphaned("Retired-by".to_owned())),
            },
            other => return Err(NotARegister::UnknownKey(other.to_owned())),
        }
    }

    for entry in &accepted {
        if entry.because.is_empty() || entry.retired_by.is_empty() {
            return Err(NotARegister::Incomplete(entry.mutant.clone()));
        }
    }

    let tool = tool.ok_or_else(|| NotARegister::Field("no Tool".to_owned()))?;
    let version = version.ok_or_else(|| NotARegister::Field("no Version".to_owned()))?;
    if scope.is_empty() {
        return Err(NotARegister::Field("no Scope".to_owned()));
    }

    Ok(Register {
        tool,
        version,
        scope,
        accepted,
    })
}

fn once(what: &str, held: &mut Option<String>, value: String) -> Result<(), NotARegister> {
    if held.is_some() {
        return Err(NotARegister::Field(format!("{what} twice")));
    }
    *held = Some(value);
    Ok(())
}

/// A line of the tool's own output, without the position it was at.
///
/// The position moves whenever anything above it in the file moves, and an
/// entry that goes stale because a comment was added above it is an entry
/// somebody deletes rather than reads. What identifies a mutant here is the
/// file and what was done to it, which is stable under an edit elsewhere and
/// still distinguishes the mutants this tree has.
pub fn without_position(line: &str) -> String {
    let mut fields = line.splitn(4, ':');
    let (Some(path), Some(_line), Some(_column), Some(what)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return line.trim().to_owned();
    };
    if !_line.trim().chars().all(|c| c.is_ascii_digit()) {
        return line.trim().to_owned();
    }
    format!("{}: {}", path.trim().replace('\\', "/"), what.trim())
}

/// The two ways the register and a run disagree.
///
/// Both directions, because a register that only failed on a new survivor
/// would keep an entry for a mutant a later test started killing, and the entry
/// would then be a claim that the tree still cannot kill it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disagreement {
    /// Survivors the register does not account for.
    pub unaccounted: Vec<String>,
    /// Entries naming a mutant the run did not report surviving.
    pub stale: Vec<String>,
}

impl Disagreement {
    pub fn is_empty(&self) -> bool {
        self.unaccounted.is_empty() && self.stale.is_empty()
    }
}

/// Compare a run's survivors against the register.
pub fn compare(register: &Register, survived: &[String]) -> Disagreement {
    let survived: BTreeSet<String> = survived.iter().map(|one| without_position(one)).collect();
    let accepted: BTreeSet<String> = register
        .accepted
        .iter()
        .map(|one| without_position(&one.mutant))
        .collect();

    Disagreement {
        unaccounted: survived.difference(&accepted).cloned().collect(),
        stale: accepted.difference(&survived).cloned().collect(),
    }
}

/// The disagreement as text somebody can act on.
pub fn report(disagreement: &Disagreement) -> String {
    let mut out = String::new();
    for one in &disagreement.unaccounted {
        writeln!(out, "survived and is not in the register: {one}").expect("writing to a String");
    }
    for one in &disagreement.stale {
        writeln!(out, "in the register and did not survive: {one}").expect("writing to a String");
    }
    out
}

/// What a run turned out to be, once its own output is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judged {
    pub caught: usize,
    pub survived: Vec<String>,
    pub unviable: usize,
}

/// The three shapes of run that say nothing about this tree.
///
/// Each of them ends with the tool having reported no surviving mutant, which
/// is the same thing a run in which every seeded fault was noticed reports. A
/// harness that read all four as agreement would be one whose failure looks
/// exactly like its success, and that is how this kind of tooling dies: it
/// stops working, nobody is told, and the green it prints is believed for as
/// long as nobody checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAVerdict {
    /// The tool wrote no report, or wrote an empty one. A run that fell over
    /// before it started leaves this and an exit status, and the exit status
    /// alone does not separate it from a run in which a mutant survived.
    NoReport,
    /// The report is there and names no mutant. Nothing was judged, so nothing
    /// survived, so the comparison would agree with any register at all.
    JudgedNothing,
    /// Mutants the run stopped waiting for. Neither killed nor surviving:
    /// nothing decided about them, and folding them into either column would
    /// be a claim the run did not make.
    Undecided(Vec<String>),
}

impl std::fmt::Display for NotAVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotAVerdict::NoReport => write!(f, "the run wrote no report"),
            NotAVerdict::JudgedNothing => write!(f, "the run judged no mutants at all"),
            NotAVerdict::Undecided(mutants) => write!(
                f,
                "{} mutant(s) reached the time limit and were not judged: {mutants:?}",
                mutants.len()
            ),
        }
    }
}

/// Read a run's own output as a verdict, or refuse it as none.
///
/// A pure function over what the four files held, so that every refusal above
/// can be shown to bite on constructed input in the default suite. The leg that
/// performs the run reads the files and does nothing else, because a rule that
/// is only exercised by a twelve minute run is a rule nobody checks.
pub fn verdict(
    report: &str,
    caught: &[String],
    missed: &[String],
    unviable: &[String],
    timed_out: &[String],
) -> Result<Judged, NotAVerdict> {
    if report.trim().is_empty() {
        return Err(NotAVerdict::NoReport);
    }
    if caught.len() + missed.len() + unviable.len() + timed_out.len() == 0 {
        return Err(NotAVerdict::JudgedNothing);
    }
    if !timed_out.is_empty() {
        return Err(NotAVerdict::Undecided(timed_out.to_vec()));
    }
    Ok(Judged {
        caught: caught.len(),
        survived: missed.to_vec(),
        unviable: unviable.len(),
    })
}
