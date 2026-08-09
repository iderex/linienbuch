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
//! Neither of the two it carries today is one of those six. Both are rules
//! `CONTRIBUTING.md` marks against #50, which has become the home for every rule
//! whose shape is a search while its body names six. So a green run here must
//! not read as a green run over what the issue lists, and every one of the six
//! is printed with what it is waiting on or with the file that holds it.
//! Repeating one of those here would put two readers of one rule in the tree,
//! and the day the rule changes both have to be right about it while only one
//! has the fixtures.
//!
//! Each invariant names the tracked paths it reads, because the two here do not
//! read the same set: one is about what a document says and the other is about
//! every tracked byte. A path an invariant does not read is a hole, so the
//! exemptions are fields too, they are printed by every run, and a test refuses
//! one that names a file the tree no longer has.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The words each search is run against.
///
/// Neither list is written into this file. One is derived from the workflow
/// files, because a list of check run names here would drift against the
/// workflows that decide them. The other is a tracked data file, because a
/// marker written as a literal in this program would make the marker search
/// refuse its own source.
struct Vocabulary {
    /// The check run names this repository produces.
    checks: BTreeSet<String>,
    /// The tool names and generated-by markers, lowercased on the way in.
    markers: Vec<String>,
}

/// One invariant, with the proof that it bites beside it.
struct Invariant {
    /// What a finding names, and what a reader greps for.
    id: &'static str,
    /// The sentence a document states and this search refuses a departure from.
    states: &'static str,
    /// The tracked paths this search reads, as pathspecs for `git ls-files`. An
    /// empty list is every tracked path.
    subject: &'static [&'static str],
    /// Paths inside that subject which this search does not read, with the
    /// reason written where the entry is. Every one of them is a hole, so the
    /// list is meant to stay short and the run prints it.
    exempt: &'static [&'static str],
    /// Applied to one line of one file. `Some` is a violation and carries what
    /// was found, so a failure names the byte rather than the rule.
    finds: fn(&str, &Vocabulary) -> Option<String>,
    /// A line that violates exactly this invariant.
    ///
    /// Built from the vocabulary rather than written as a literal. An invariant
    /// whose subject includes this file cannot carry its own violation as
    /// source text, and a fixture derived from the vocabulary also stays a real
    /// violation on the day the vocabulary changes.
    fixture: fn(&Vocabulary) -> String,
    /// A line one change away that it does not refuse.
    neighbour: fn(&Vocabulary) -> String,
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

/// The file holding the marker vocabulary, relative to the root.
const MARKER_VOCABULARY: &str = "src/bin/invariants/markers.txt";

/// The searches this check runs.
const INVARIANTS: [Invariant; 2] = [
    Invariant {
        id: "no-check-enumerated-in-a-document",
        states: "no document lists this repository's checks",
        // A document is what this one is about, so it reads the tracked
        // markdown and nothing else.
        subject: &["*.md"],
        exempt: &[],
        finds: check_named_as_a_list_item,
        fixture: a_check_run_name_as_a_list_item,
        neighbour: a_sentence_about_a_check,
    },
    Invariant {
        id: "no-generated-by-marker-in-tracked-text",
        states: "no tracked file credits its text to something other than its author",
        // Every tracked path, because the rule in `CONTRIBUTING.md` is about
        // tracked text rather than about documents. A marker in a comment in a
        // source file is the same defect as one in prose.
        subject: &[],
        // The file that says what the search looks for. Reading it would make
        // every run refuse the vocabulary itself, and this is the only path
        // excluded: a marker in the program below is found like any other.
        exempt: &[MARKER_VOCABULARY],
        finds: names_a_marker,
        fixture: a_line_carrying_a_marker,
        neighbour: a_line_crediting_a_command,
    },
];

/// The rest of what #50 names, and where each of them is.
///
/// Read off the issue rather than invented here. Nothing in this table is
/// searched; it exists so that a run covering two searches cannot be read as a
/// run covering the six the issue lists.
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

/// What the run says about the two counts, so that neither is read as the other.
///
/// The searches above are rules `CONTRIBUTING.md` marks against #50 and are not
/// among the six the issue's body lists. Every one of those six is in the two
/// tables above.
const THE_TWO_COUNTS: &str = "the searches above are rules CONTRIBUTING.md marks against #50 \
     and are not among the six its body lists, every one of which is named here";

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

/// The first invariant: a list item that is a check run name and nothing else.
///
/// The vocabulary is derived from the workflow files rather than written here.
/// A list of the names inside this file would be the very thing the search
/// refuses, one directory over, and it would drift against the workflows the
/// same way.
fn check_named_as_a_list_item(line: &str, vocabulary: &Vocabulary) -> Option<String> {
    let item = list_item(line)?;
    vocabulary.checks.contains(item).then(|| item.to_owned())
}

/// A list whose items are check run names is the shape that drifts: the check is
/// renamed, the list is not, and the document goes on describing a gate that has
/// moved.
fn a_check_run_name_as_a_list_item(_: &Vocabulary) -> String {
    "- DCO sign-off".to_owned()
}

/// One change away. The same name in a sentence is a reference rather than a
/// list, and refusing it would refuse every document that talks about a check at
/// all.
fn a_sentence_about_a_check(_: &Vocabulary) -> String {
    "- the sign-off check refuses a commit whose sign off does not match its author".to_owned()
}

/// The second invariant: a line naming a tool or a generated-by marker.
///
/// Whole words and without regard to case. The word boundary is not decoration:
/// a substring search is the one-character mistake here, and it refuses a
/// correct word that happens to contain an entry.
fn names_a_marker(line: &str, vocabulary: &Vocabulary) -> Option<String> {
    let haystack = line.to_lowercase();
    vocabulary
        .markers
        .iter()
        .find(|marker| holds_whole_word(&haystack, marker))
        .cloned()
}

/// `needle`, already lowercased, appearing in `haystack` with a non-alphanumeric
/// character or nothing on each side.
fn holds_whole_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut from = 0;
    while let Some(offset) = haystack[from..].find(needle) {
        let start = from + offset;
        let end = start + needle.len();
        let before = haystack[..start].chars().next_back();
        let after = haystack[end..].chars().next();
        if !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) {
            return true;
        }
        from = start + 1;
    }
    false
}

/// A line carrying the first marker the vocabulary holds.
///
/// The carrier sentence is neutral on purpose. What makes the line a violation
/// is the marker, so the fixture is a violation for every entry the file can
/// hold rather than only for the one that happens to be first today.
fn a_line_carrying_a_marker(vocabulary: &Vocabulary) -> String {
    match vocabulary.markers.first() {
        Some(marker) => format!("This text carries the marker {marker}."),
        None => String::new(),
    }
}

/// One change away, and it is the near miss this tree would actually hit.
///
/// A search for the phrase rather than for the vocabulary refuses this line, and
/// it refuses the tree today: `Cargo.lock` opens with a comment saying the file
/// was generated by cargo, and #30 asks for a coverage table generated by a
/// command. Neither credits anything to something other than its author.
fn a_line_crediting_a_command(_: &Vocabulary) -> String {
    "The coverage table is generated by the command above.".to_owned()
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

/// The marker vocabulary, from the one file the marker search does not read.
///
/// Blank lines and lines beginning with a hash are ignored, which is what makes
/// the reasoning in that file possible without every sentence of it becoming a
/// marker.
fn marker_vocabulary(root: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(root.join(MARKER_VOCABULARY)) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_lowercase)
        .collect()
}

/// The tracked paths matching `pathspecs`, from git rather than from a walk,
/// because the rule is about tracked text and a walk would also read whatever
/// happens to be lying in the directory.
///
/// An empty pathspec list is every tracked path, which is what `git ls-files`
/// does with no pathspec of its own.
fn tracked_paths(root: &Path, pathspecs: &[&str]) -> Result<Vec<String>, String> {
    let mut command = Command::new("git");
    command.args(["ls-files", "-z", "--"]);
    command.args(pathspecs);
    let output = command
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

/// What one invariant read, so that a run can say what it covered.
struct Coverage {
    /// Paths searched.
    searched: usize,
    /// Paths in the subject that could not be read as text, so were not
    /// searched. Never silent: a byte sequence this program cannot read is a
    /// place a marker could sit.
    unreadable: Vec<String>,
}

fn examine(
    root: &Path,
    invariant: &Invariant,
    vocabulary: &Vocabulary,
) -> Result<(Coverage, Vec<Finding>), String> {
    let paths = tracked_paths(root, invariant.subject)?;
    let mut read = Coverage {
        searched: 0,
        unreadable: Vec::new(),
    };
    let mut findings = Vec::new();
    for path in paths {
        if invariant.exempt.contains(&path.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join(&path)) else {
            read.unreadable.push(path);
            continue;
        };
        read.searched += 1;
        for (number, line) in text.lines().enumerate() {
            if let Some(found) = (invariant.finds)(line, vocabulary) {
                findings.push(Finding {
                    invariant: invariant.id,
                    path: path.clone(),
                    line: number + 1,
                    found,
                });
            }
        }
    }
    Ok((read, findings))
}

/// Every invariant put to its own two fixtures, before any of them is pointed
/// at the tree.
///
/// This is what refuses an invariant that arrived without a proof that it bites.
/// The two fixtures are fields, so one cannot be left out and still compile; a
/// fixture that does not bite, or a neighbour the search refuses anyway, is what
/// this catches, and it catches it on every run rather than only under `cargo
/// test`. It is run against the vocabulary the real search uses, so a fixture
/// built from a vocabulary that has since lost the entry it relied on stops
/// proving anything and says so instead of passing quietly.
fn proofs_that_do_not_hold(invariants: &[Invariant], vocabulary: &Vocabulary) -> Vec<String> {
    let mut broken = Vec::new();
    for invariant in invariants {
        let fixture = (invariant.fixture)(vocabulary);
        let neighbour = (invariant.neighbour)(vocabulary);
        if (invariant.finds)(&fixture, vocabulary).is_none() {
            broken.push(format!(
                "{} does not refuse its own fixture {fixture:?}",
                invariant.id
            ));
        }
        if (invariant.finds)(&neighbour, vocabulary).is_some() {
            broken.push(format!(
                "{} refuses its neighbour {neighbour:?}, so it is not the invariant it says it is",
                invariant.id
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
    println!("invariants: {THE_TWO_COUNTS}");
}

fn main() -> ExitCode {
    let root = manifest_dir();
    let vocabulary = Vocabulary {
        checks: check_run_names(&root.join(".github").join("workflows")),
        markers: marker_vocabulary(&root),
    };
    if vocabulary.checks.is_empty() {
        println!("invariants: no check run name could be read from .github/workflows");
        return ExitCode::FAILURE;
    }
    if vocabulary.markers.is_empty() {
        println!("invariants: no marker could be read from {MARKER_VOCABULARY}");
        return ExitCode::FAILURE;
    }

    let broken = proofs_that_do_not_hold(&INVARIANTS, &vocabulary);
    if !broken.is_empty() {
        for what in &broken {
            println!("invariants: {what}");
        }
        return ExitCode::FAILURE;
    }

    println!(
        "invariants: {} check run name(s) and {} marker(s) in the vocabularies",
        vocabulary.checks.len(),
        vocabulary.markers.len()
    );

    let mut findings = Vec::new();
    for invariant in &INVARIANTS {
        let (read, found) = match examine(&root, invariant, &vocabulary) {
            Ok(outcome) => outcome,
            Err(why) => {
                println!("invariants: {why}");
                return ExitCode::FAILURE;
            }
        };
        println!(
            "invariants: ran {} over {} tracked path(s) - {}",
            invariant.id, read.searched, invariant.states
        );
        for path in invariant.exempt {
            println!("invariants:   not read by it: {path}");
        }
        for path in &read.unreadable {
            println!("invariants:   in its subject and not readable as text: {path}");
        }
        findings.extend(found);
    }
    print_not_covered();

    if findings.is_empty() {
        println!("invariants: nothing refused");
        return ExitCode::SUCCESS;
    }
    for finding in &findings {
        println!(
            "invariants: {}:{} carries {:?}, which {} refuses",
            finding.path, finding.line, finding.found, finding.invariant
        );
    }
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocabulary() -> Vocabulary {
        Vocabulary {
            checks: ["DCO sign-off".to_owned(), "build".to_owned()].into(),
            markers: vec!["gizmo".to_owned(), "made by a gizmo".to_owned()],
        }
    }

    fn tree_vocabulary() -> Vocabulary {
        let root = manifest_dir();
        Vocabulary {
            checks: check_run_names(&root.join(".github").join("workflows")),
            markers: marker_vocabulary(&root),
        }
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
        let broken = proofs_that_do_not_hold(&INVARIANTS, &tree_vocabulary());
        assert!(broken.is_empty(), "{broken:?}");
    }

    /// The same reading, against invariants that are deliberately wrong, so that
    /// the paragraph above is not a test that could not have failed.
    ///
    /// Two shapes, and both are what an invariant looks like when somebody adds
    /// one in a hurry: a fixture that violates nothing, and a neighbour the
    /// search refuses anyway, which is a search wider than the rule it claims.
    #[test]
    fn a_proof_that_does_not_hold_is_reported() {
        fn a_line_that_violates_nothing(_: &Vocabulary) -> String {
            "an ordinary sentence".to_owned()
        }

        let toothless = [Invariant {
            id: "fixture-that-does-not-bite",
            states: "nothing, because its fixture is not a violation",
            subject: &["*.md"],
            exempt: &[],
            finds: check_named_as_a_list_item,
            fixture: a_line_that_violates_nothing,
            neighbour: a_sentence_about_a_check,
        }];
        let broken = proofs_that_do_not_hold(&toothless, &vocabulary());
        assert_eq!(broken.len(), 1, "{broken:?}");
        assert!(broken[0].contains("does not refuse its own fixture"));

        let too_wide = [Invariant {
            id: "neighbour-that-is-refused",
            states: "more than it says, because it refuses its own neighbour",
            subject: &["*.md"],
            exempt: &[],
            finds: check_named_as_a_list_item,
            fixture: a_check_run_name_as_a_list_item,
            neighbour: a_check_run_name_as_a_list_item,
        }];
        let broken = proofs_that_do_not_hold(&too_wide, &vocabulary());
        assert_eq!(broken.len(), 1, "{broken:?}");
        assert!(broken[0].contains("refuses its neighbour"));
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

    /// A marker is read as a whole word and without regard to case.
    ///
    /// The substring is the mistake, and it is not hypothetical: an entry that
    /// is also the opening of a longer word would refuse every line carrying
    /// that word. The vocabulary here is invented so that the case is made
    /// without a real marker appearing in this file, which the marker search
    /// reads.
    #[test]
    fn a_marker_is_a_whole_word_and_its_case_does_not_matter() {
        let words = vocabulary();
        assert_eq!(
            names_a_marker("this section was made by a gizmo, once", &words),
            Some("gizmo".to_owned())
        );
        assert_eq!(
            names_a_marker("THIS SECTION WAS MADE BY A GIZMO", &words),
            Some("gizmo".to_owned())
        );
        assert!(
            names_a_marker("the gizmotron is a different word", &words).is_none(),
            "a marker inside a longer word is not the marker"
        );
        assert!(
            names_a_marker("a line about nothing in particular", &words).is_none(),
            "a line with no marker is not refused"
        );
    }

    /// The boundary is on both sides, which a search anchored on one is not.
    #[test]
    fn a_marker_is_bounded_at_both_ends() {
        assert!(holds_whole_word("made by a gizmo", "gizmo"));
        assert!(holds_whole_word("gizmo made this", "gizmo"));
        assert!(!holds_whole_word("gizmotron", "gizmo"));
        assert!(!holds_whole_word("subgizmo", "gizmo"));
        assert!(holds_whole_word("sub-gizmo", "gizmo"));
        assert!(!holds_whole_word("anything", ""));
    }

    /// The phrase is not the invariant, and this is the line that proves it.
    ///
    /// A search for the words rather than for the vocabulary refuses a sentence
    /// about a table a command produced, which is what #30 asks this board to
    /// build, and it refuses `Cargo.lock` as it stands.
    #[test]
    fn a_line_about_something_a_command_produced_is_not_a_marker() {
        let words = tree_vocabulary();
        assert!(
            names_a_marker(&a_line_crediting_a_command(&words), &words).is_none(),
            "the neighbour must not be refused"
        );
        assert!(
            names_a_marker("# This file is automatically @generated by Cargo.", &words).is_none(),
            "the first line of Cargo.lock is not a marker"
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

    /// The marker vocabulary is a tracked file with entries in it, and the
    /// comments in it are not entries.
    #[test]
    fn the_marker_vocabulary_is_read_from_the_file_the_search_does_not_read() {
        let markers = marker_vocabulary(&manifest_dir());
        assert!(
            !markers.is_empty(),
            "{MARKER_VOCABULARY} holds no marker, so the search would find nothing"
        );
        for marker in &markers {
            assert!(!marker.starts_with('#'), "a comment was read as a marker");
            assert_eq!(
                marker.to_lowercase(),
                *marker,
                "a marker arrives lowercased, because matching ignores case"
            );
        }
    }

    /// Every path an invariant does not read is a file the tree still has.
    ///
    /// An exemption pointing at something deleted reads as a hole that is being
    /// kept open for a reason nobody can check, and the natural next step is to
    /// widen it. The only exemption here is the marker vocabulary, and that is
    /// asserted rather than left to a reader counting entries.
    #[test]
    fn every_exemption_names_a_file_in_the_tree_and_there_is_one() {
        let exempt: Vec<&str> = INVARIANTS
            .iter()
            .flat_map(|invariant| invariant.exempt.iter().copied())
            .collect();
        assert_eq!(
            exempt,
            vec![MARKER_VOCABULARY],
            "the exemptions have changed, and each one is a place a violation can sit"
        );
        for path in exempt {
            assert!(
                manifest_dir().join(path).exists(),
                "{path} is exempted from a search and is not in the tree"
            );
        }
    }

    /// The tree this check runs over passes it.
    ///
    /// Not the proof that a guard bites, which is the fixture above. This is the
    /// statement that each guard is pointed at a tree it agrees with, so a red
    /// run later is a change rather than a rule that never held.
    #[test]
    fn this_tree_passes_every_invariant() {
        let root = manifest_dir();
        let words = tree_vocabulary();
        for invariant in &INVARIANTS {
            let (_, found) = examine(&root, invariant, &words).expect("git ls-files");
            let findings: Vec<String> = found
                .into_iter()
                .map(|f| format!("{}:{} {:?}", f.path, f.line, f.found))
                .collect();
            assert!(
                findings.is_empty(),
                "{} refuses this tree: {findings:?}",
                invariant.id
            );
        }
    }

    /// Nothing in the subject of an invariant is skipped without being said.
    ///
    /// The marker search reads every tracked path, so a file it cannot read as
    /// text is a place a marker could sit unseen. Today there is none, and the
    /// run prints any there ever is.
    #[test]
    fn nothing_in_the_tree_is_unreadable_and_therefore_unsearched() {
        let root = manifest_dir();
        let words = tree_vocabulary();
        for invariant in &INVARIANTS {
            let (read, _) = examine(&root, invariant, &words).expect("git ls-files");
            assert!(
                read.unreadable.is_empty(),
                "{} could not read {:?}",
                invariant.id,
                read.unreadable
            );
            assert!(read.searched > 0, "{} searched nothing", invariant.id);
        }
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

    /// The six #50's body lists are all named by the run.
    ///
    /// The count is what a reader takes from a green run, and the two counts
    /// here are different numbers about different things. If a search ever does
    /// cover one of the six, it leaves these tables in the same change and this
    /// test is what says so.
    #[test]
    fn the_six_the_issue_lists_are_all_named() {
        assert_eq!(
            NOT_COVERED.len() + 1,
            6,
            "the six #50's body lists no longer add up"
        );
        assert!(!THE_TWO_COUNTS.is_empty());
    }
}
