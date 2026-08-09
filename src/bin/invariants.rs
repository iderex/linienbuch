//! The invariants that are a search over the tree rather than a type, #50.
//!
//! Some properties are cheaper to hold with a search than with a type, and this
//! is where those live. An invariant here is a search that either matches or
//! does not, and it arrives with the two fixtures #53 asks of every guard: one
//! line that violates exactly it, and one line a single change away that it does
//! not refuse.
//!
//! Both fixtures are fields of the invariant rather than a convention, so an
//! invariant with no fixture is a program that does not compile, and a fixture
//! that does not bite is a red test rather than a green run. That is the part of
//! this check that has no invariant to show off and is the part most worth
//! having: the six candidates #50 names arrive one at a time, over months, and
//! the third one added in a hurry is the one that would otherwise ship without
//! its proof.
//!
//! What it covers today is one of those six, and the run says so. A green run
//! here must not read as a green run over everything #50 names, so every
//! candidate this check does not carry is printed with what it is waiting on,
//! and every one held somewhere else is printed with the file that holds it.
//! Repeating one of those here would put two readers of one rule in the tree,
//! and the day the rule changes both have to be right about it while only one
//! has the fixtures.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// One invariant, with the proof that it bites beside it.
struct Invariant {
    /// What a finding names, and what a reader greps for.
    id: &'static str,
    /// The sentence a document states and this search refuses a departure from.
    states: &'static str,
    /// Applied to one line of one document, with the check run names of this
    /// repository as the vocabulary. `Some` is a violation and carries what was
    /// found, so a failure names the byte rather than the rule.
    finds: fn(&str, &BTreeSet<String>) -> Option<String>,
    /// A line that violates exactly this invariant.
    fixture: &'static str,
    /// A line one change away that it does not refuse.
    neighbour: &'static str,
}

/// Something #50 names that this check does not carry.
struct NotCovered {
    /// The invariant, in the issue's own words.
    named: &'static str,
    /// Where it is instead, or what it is waiting on. One or the other, never
    /// neither.
    state: State,
}

enum State {
    /// Held by a check that already exists, with its own fixtures beside it.
    HeldBy(&'static str),
    /// Not writable yet, and what it is waiting on.
    Waiting(&'static str),
}

/// The searches this check runs.
const INVARIANTS: [Invariant; 1] = [Invariant {
    id: "no-check-enumerated-in-a-document",
    states: "no document lists this repository's checks",
    finds: check_named_as_a_list_item,
    // A list whose items are check run names is the shape that drifts: the
    // check is renamed, the list is not, and the document goes on describing a
    // gate that has moved.
    fixture: "- DCO sign-off",
    // One change away. The same name in a sentence is a reference rather than a
    // list, and refusing it would refuse every document that talks about a
    // check at all.
    neighbour: "- the sign-off check refuses a commit whose sign off does not match its author",
}];

/// The rest of what #50 names, and where each of them is.
///
/// Read off the issue rather than invented here. Nothing in this table is
/// searched; it exists so that a run covering one invariant cannot be read as a
/// run covering six.
const NOT_COVERED: [NotCovered; 5] = [
    NotCovered {
        named: "no line position stored without its convention",
        state: State::Waiting(
            "a line position to search for. #11 decides the representation and #27 \
             brings the parser that stores one",
        ),
    },
    NotCovered {
        named: "no selection between competing claims outside the evaluation of a named profile",
        state: State::Waiting(
            "entry 5 of #1. Whether this board ships named preference profiles at all \
             is open, so there is no named thing for a search to permit",
        ),
    },
    NotCovered {
        named: "no formatting of an uncertainty outside the shared rounding rule",
        state: State::HeldBy("tests/uncertainty_formatting.rs"),
    },
    NotCovered {
        named: "no network call site outside the declared egress list",
        state: State::Waiting(
            "#56, which owns the list, and which is itself waiting on where a scanner \
             over the tree is allowed to live",
        ),
    },
    NotCovered {
        named: "no source registered without a terms record, an attribution record and a \
                coverage row",
        state: State::Waiting(
            "three registers that do not exist, #54, #44 and #30, and a source \
             registration path to refuse, which is #26",
        ),
    },
];

/// The sixth of #50's candidates, kept apart because it is neither waiting nor
/// carried here.
const ALREADY_REFUSED_ELSEWHERE: NotCovered = NotCovered {
    named: "no fixture without its category record",
    state: State::HeldBy("tests/fixture_policy.rs"),
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A line that is a markdown list item, with the bullet and any backticks
/// taken off.
///
/// Ordered and unordered both, because a numbered list of checks drifts exactly
/// as an unordered one does.
fn list_item(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = match trimmed.split_once(' ') {
        Some((bullet, rest)) if bullet == "-" || bullet == "*" || bullet == "+" => rest,
        Some((bullet, rest))
            if bullet.ends_with('.')
                && bullet[..bullet.len() - 1]
                    .chars()
                    .all(|c| c.is_ascii_digit()) =>
        {
            rest
        }
        _ => return None,
    };
    let item = rest.trim();
    Some(
        item.strip_prefix('`')
            .and_then(|s| s.strip_suffix('`'))
            .unwrap_or(item),
    )
}

/// The invariant itself: a list item that is a check run name and nothing else.
///
/// The vocabulary is derived from the workflow files rather than written here.
/// A list of the names inside this file would be the very thing the search
/// refuses, one directory over, and it would drift against the workflows the
/// same way.
fn check_named_as_a_list_item(line: &str, checks: &BTreeSet<String>) -> Option<String> {
    let item = list_item(line)?;
    checks.contains(item).then(|| item.to_owned())
}

/// The check run names this repository produces.
///
/// A check run is named by its job's `name:` where it has one and by the job id
/// where it does not, which is the distinction `docs/parity.md` and two of the
/// workflow files turn on. Parsed by indentation, because that is what
/// distinguishes a job from a step in these files and a full YAML reader would
/// be a dependency bought for one table.
fn check_run_names(workflows: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(entries) = fs::read_dir(workflows) else {
        return names;
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "yml"))
        .collect();
    files.sort();

    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let mut in_jobs = false;
        let mut job: Option<String> = None;
        for line in text.lines() {
            if line.starts_with("jobs:") {
                in_jobs = true;
                continue;
            }
            if !in_jobs {
                continue;
            }
            if !line.starts_with(' ') && !line.trim().is_empty() {
                in_jobs = false;
                continue;
            }
            if let Some(id) = job_id(line) {
                if let Some(previous) = job.replace(id) {
                    names.insert(previous);
                }
                continue;
            }
            if let Some(name) = job_name(line) {
                job = None;
                names.insert(name);
            }
        }
        if let Some(id) = job {
            names.insert(id);
        }
    }
    names
}

/// `  some-job:` at exactly two spaces of indent, which is a job and not a step.
fn job_id(line: &str) -> Option<String> {
    let rest = line.strip_prefix("  ")?;
    if rest.starts_with(' ') {
        return None;
    }
    let id = rest.strip_suffix(':')?;
    (!id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
    .then(|| id.to_owned())
}

/// `    name: ...` at exactly four spaces of indent, which is a job's name and
/// not a step's.
fn job_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("    name: ")?;
    if rest.starts_with(' ') {
        return None;
    }
    let name = rest.trim().trim_matches('"');
    (!name.is_empty()).then(|| name.to_owned())
}

/// The tracked markdown files, from git rather than from a walk, because the
/// rule is about tracked text and a walk would also read whatever happens to be
/// lying in the directory.
fn tracked_documents(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["ls-files", "-z", "--", "*.md"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("git ls-files could not start: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

/// What one run found.
struct Finding {
    invariant: &'static str,
    path: String,
    line: usize,
    found: String,
}

fn examine(root: &Path, documents: &[String], checks: &BTreeSet<String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for path in documents {
        let Ok(text) = fs::read_to_string(root.join(path)) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for invariant in &INVARIANTS {
                if let Some(found) = (invariant.finds)(line, checks) {
                    findings.push(Finding {
                        invariant: invariant.id,
                        path: path.clone(),
                        line: number + 1,
                        found,
                    });
                }
            }
        }
    }
    findings
}

/// Every invariant put to its own two fixtures, before any of them is pointed
/// at the tree.
///
/// This is what refuses an invariant that arrived without a proof that it bites.
/// The two fixtures are fields, so one cannot be left out and still compile; a
/// fixture that does not bite, or a neighbour the search refuses anyway, is what
/// this catches, and it catches it on every run rather than only under `cargo
/// test`. It is run against the vocabulary the real search uses, so a fixture
/// naming a check that has since been renamed stops proving anything and says so
/// instead of passing quietly.
fn proofs_that_do_not_hold(checks: &BTreeSet<String>) -> Vec<String> {
    let mut broken = Vec::new();
    for invariant in &INVARIANTS {
        if (invariant.finds)(invariant.fixture, checks).is_none() {
            broken.push(format!(
                "{} does not refuse its own fixture {:?}",
                invariant.id, invariant.fixture
            ));
        }
        if (invariant.finds)(invariant.neighbour, checks).is_some() {
            broken.push(format!(
                "{} refuses its neighbour {:?}, so it is not the invariant it says it is",
                invariant.id, invariant.neighbour
            ));
        }
    }
    broken
}

/// What every run prints about what it did not search for.
fn print_not_covered() {
    println!("invariants: named in #50 and not searched for here:");
    for entry in NOT_COVERED.iter().chain([&ALREADY_REFUSED_ELSEWHERE]) {
        println!("invariants:   {}", entry.named);
        match entry.state {
            State::HeldBy(file) => {
                println!("invariants:     held by {file}, with its own fixtures beside it")
            }
            State::Waiting(what) => println!("invariants:     waits on {what}"),
        }
    }
}

fn main() -> ExitCode {
    let root = manifest_dir();
    let checks = check_run_names(&root.join(".github").join("workflows"));
    if checks.is_empty() {
        println!("invariants: no check run name could be read from .github/workflows");
        return ExitCode::FAILURE;
    }

    let broken = proofs_that_do_not_hold(&checks);
    if !broken.is_empty() {
        for what in &broken {
            println!("invariants: {what}");
        }
        return ExitCode::FAILURE;
    }

    let documents = match tracked_documents(&root) {
        Ok(documents) => documents,
        Err(why) => {
            println!("invariants: {why}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "invariants: searched {} tracked document(s) against {} check run name(s)",
        documents.len(),
        checks.len()
    );
    for invariant in &INVARIANTS {
        println!("invariants: ran {} - {}", invariant.id, invariant.states);
    }
    print_not_covered();

    let findings = examine(&root, &documents, &checks);
    if findings.is_empty() {
        println!("invariants: nothing refused");
        return ExitCode::SUCCESS;
    }
    for finding in &findings {
        println!(
            "invariants: {}:{} names the check {:?}, which {} refuses",
            finding.path, finding.line, finding.found, finding.invariant
        );
    }
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocabulary() -> BTreeSet<String> {
        ["DCO sign-off".to_owned(), "build".to_owned()].into()
    }

    /// Every invariant refuses its own fixture and does not refuse its
    /// neighbour.
    ///
    /// The two fixtures are fields rather than a convention, so one cannot be
    /// left out. This is what refuses the other half: a fixture that could not
    /// have failed, or a neighbour so far away that it proves nothing about the
    /// edge.
    #[test]
    fn every_invariant_bites_its_fixture_and_spares_its_neighbour() {
        let checks = check_run_names(&manifest_dir().join(".github").join("workflows"));
        let broken = proofs_that_do_not_hold(&checks);
        assert!(broken.is_empty(), "{broken:?}");
    }

    /// The same reading, against a fixture that is deliberately wrong, so that
    /// the paragraph above is not a test that could not have failed.
    ///
    /// A vocabulary the fixture is absent from is exactly the state an invariant
    /// arrives in when somebody adds one and writes a fixture that does not
    /// violate it.
    #[test]
    fn a_fixture_that_does_not_bite_is_reported() {
        let empty = BTreeSet::new();
        let broken = proofs_that_do_not_hold(&empty);
        assert_eq!(
            broken.len(),
            INVARIANTS.len(),
            "with nothing to find, every fixture should be reported: {broken:?}"
        );
    }

    #[test]
    fn a_neighbour_is_not_refused() {
        let checks = vocabulary();
        for invariant in &INVARIANTS {
            assert!(
                (invariant.finds)(invariant.neighbour, &checks).is_none(),
                "{} refuses its neighbour {:?}, so it is not the invariant it says it is",
                invariant.id,
                invariant.neighbour
            );
        }
    }

    /// No two invariants answer for the same id, which would make a finding
    /// name a rule the reader cannot find.
    #[test]
    fn every_invariant_has_its_own_id() {
        let ids: BTreeSet<&str> = INVARIANTS.iter().map(|i| i.id).collect();
        assert_eq!(ids.len(), INVARIANTS.len(), "two invariants share one id");
    }

    /// A numbered list drifts exactly as a bulleted one does, and both are one
    /// character away from prose that is not a list at all.
    #[test]
    fn a_list_item_is_recognised_by_its_bullet_and_prose_is_not() {
        assert_eq!(list_item("- build"), Some("build"));
        assert_eq!(list_item("  * build"), Some("build"));
        assert_eq!(list_item("+ build"), Some("build"));
        assert_eq!(list_item("3. build"), Some("build"));
        assert_eq!(list_item("- `build`"), Some("build"));
        assert_eq!(list_item("the build leg"), None);
        assert_eq!(list_item("-build"), None);
        assert_eq!(list_item("3.build"), None);
    }

    /// The near miss, which is the mistake somebody writing a document actually
    /// makes: the name is in the line, and the line is a list item, and the
    /// item is a sentence about the check rather than the check.
    #[test]
    fn a_list_item_that_mentions_a_check_inside_a_sentence_is_not_an_enumeration() {
        let checks = vocabulary();
        assert!(check_named_as_a_list_item("- DCO sign-off", &checks).is_some());
        assert!(
            check_named_as_a_list_item("- DCO sign-off is one of four", &checks).is_none(),
            "a sentence naming a check is a reference and not a list of checks"
        );
    }

    /// A check run is named by its job's `name:` where it has one and by the
    /// job id where it does not, and a step's `name:` is neither.
    #[test]
    fn a_job_is_read_apart_from_a_step() {
        assert_eq!(
            job_id("  dependency-review:"),
            Some("dependency-review".to_owned())
        );
        assert_eq!(job_id("    steps:"), None);
        assert_eq!(job_id("jobs:"), None);
        assert_eq!(
            job_name("    name: DCO sign-off"),
            Some("DCO sign-off".to_owned())
        );
        assert_eq!(job_name("      - name: Checkout"), None);
    }

    /// The names come out of the workflow directory this repository has, and
    /// the two spellings both arrive.
    #[test]
    fn the_vocabulary_is_read_from_the_workflows_in_this_tree() {
        let names = check_run_names(&manifest_dir().join(".github").join("workflows"));
        for expected in [
            "DCO sign-off",
            "dependency-review",
            "Reject Trojan Source Unicode",
        ] {
            assert!(
                names.contains(expected),
                "{expected:?} is not among the check run names read from the workflows: {names:?}"
            );
        }
    }

    /// The tree this check runs over passes it.
    ///
    /// Not the proof that the guard bites, which is the fixture above. This is
    /// the statement that the guard is pointed at a tree it agrees with, so a
    /// red run later is a change rather than a rule that never held.
    #[test]
    fn this_repository_enumerates_no_check_in_a_document() {
        let root = manifest_dir();
        let checks = check_run_names(&root.join(".github").join("workflows"));
        let documents = tracked_documents(&root).expect("git ls-files");
        let findings: Vec<String> = examine(&root, &documents, &checks)
            .into_iter()
            .map(|f| format!("{}:{} {:?}", f.path, f.line, f.found))
            .collect();
        assert!(
            findings.is_empty(),
            "documents enumerating checks: {findings:?}"
        );
    }

    /// Every entry in what the run discloses says where it is or what it waits
    /// on, and no entry claims both.
    #[test]
    fn every_uncovered_invariant_says_where_it_is() {
        for entry in NOT_COVERED.iter().chain([&ALREADY_REFUSED_ELSEWHERE]) {
            assert!(!entry.named.is_empty());
            match entry.state {
                State::HeldBy(file) => assert!(
                    manifest_dir().join(file).exists(),
                    "{} is said to be held by {file}, which is not in the tree",
                    entry.named
                ),
                State::Waiting(what) => assert!(!what.is_empty()),
            }
        }
    }
}
