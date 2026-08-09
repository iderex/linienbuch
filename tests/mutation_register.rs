//! The rules a mutation run is judged by, proved without running one.
//!
//! The run itself costs minutes and needs a tool that is not part of the pinned
//! toolchain, so it lives in the integration harness and is asked for by name.
//! What it is judged by is a parser and a set comparison, and both of those cost
//! nothing. They are proved here, in the default suite, so that a harness leg
//! nobody ran today is still standing on rules somebody checked today.
//!
//! Each refusal below has a fixture that violates exactly it and a neighbour one
//! change away that parses. The neighbour is the point: a parser that refused
//! every register would satisfy the first half of every case here and prove
//! nothing.
//!
//! The tracked register is read at the end, so an entry written in a shape the
//! parser does not accept is caught by every default run rather than by whoever
//! next asks for the twelve minute leg.

#[path = "mutation/register.rs"]
mod register;

use register::{
    Accepted, Judged, NotARegister, NotAVerdict, compare, parse, report, verdict, without_position,
};
use std::path::PathBuf;

/// A register with one of everything, used as the neighbour every refusing
/// fixture below is one change away from.
const WELL_FORMED: &str = "\
# a comment, and a blank line after it

Tool: cargo-mutants
Version: 27.1.0
Scope: src/register/rounding.rs
Accepted: src/register/rounding.rs:1:1: replace a with b
Because: a reason
Retired-by: a retirement
";

/// The neighbour parses, and every field arrives where it was written.
#[test]
fn a_well_formed_register_parses() {
    let register = parse(WELL_FORMED).expect("the neighbour parses");
    assert_eq!(register.tool, "cargo-mutants");
    assert_eq!(register.version, "27.1.0");
    assert_eq!(register.scope, vec!["src/register/rounding.rs".to_owned()]);
    assert_eq!(
        register.accepted,
        vec![Accepted {
            mutant: "src/register/rounding.rs:1:1: replace a with b".to_owned(),
            because: "a reason".to_owned(),
            retired_by: "a retirement".to_owned(),
        }]
    );
}

/// A key the format does not know is refused rather than skipped.
///
/// The near miss is the one somebody writes: `Retired-By` for `Retired-by`. A
/// parser that skipped what it did not recognise would read that register as an
/// accepted entry carrying no retirement, which is the obligation this format
/// exists to hold.
#[test]
fn a_misspelled_key_is_refused_rather_than_skipped() {
    let misspelled = WELL_FORMED.replace("Retired-by:", "Retired-By:");
    assert_eq!(
        parse(&misspelled),
        Err(NotARegister::UnknownKey("Retired-By".to_owned()))
    );
}

/// A line carrying no key at all.
#[test]
fn a_line_with_no_key_is_refused() {
    let unkeyed = format!("{WELL_FORMED}a line somebody meant as a note\n");
    assert_eq!(
        parse(&unkeyed),
        Err(NotARegister::Unkeyed(
            "a line somebody meant as a note".to_owned()
        ))
    );
}

/// The two single valued fields, each absent and each twice.
#[test]
fn a_single_valued_field_is_required_once_and_only_once() {
    for (key, absent) in [("Tool", "no Tool"), ("Version", "no Version")] {
        let without = WELL_FORMED
            .lines()
            .filter(|line| !line.starts_with(key))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            parse(&without),
            Err(NotARegister::Field(absent.to_owned())),
            "a register with no {key} line"
        );

        let twice = format!("{key}: second\n{WELL_FORMED}");
        assert_eq!(
            parse(&twice),
            Err(NotARegister::Field(format!("{key} twice"))),
            "a register naming {key} twice"
        );
    }
}

/// A register that judges no file at all.
///
/// The one that would otherwise pass quietly: a run over an empty scope reports
/// no survivors, and no survivors reads exactly like every mutant killed.
#[test]
fn a_register_with_no_scope_is_refused() {
    let without = WELL_FORMED
        .lines()
        .filter(|line| !line.starts_with("Scope"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        parse(&without),
        Err(NotARegister::Field("no Scope".to_owned()))
    );
}

/// An accepted entry has to carry both halves of its debt.
///
/// Either half alone is refused, and the two cases are separated because they
/// are different mistakes: a reason with no retirement is an entry nobody can
/// remove, and a retirement with no reason is one nobody can judge.
#[test]
fn an_accepted_entry_with_half_its_debt_is_refused() {
    let mutant = "src/register/rounding.rs:1:1: replace a with b".to_owned();

    for missing in ["Because:", "Retired-by:"] {
        let half = WELL_FORMED
            .lines()
            .filter(|line| !line.starts_with(missing))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            parse(&half),
            Err(NotARegister::Incomplete(mutant.clone())),
            "an accepted entry with no {missing} line"
        );
    }
}

/// A debt line above the entry it belongs to belongs to nothing.
#[test]
fn a_debt_line_with_no_entry_above_it_is_refused() {
    for key in ["Because", "Retired-by"] {
        let orphaned = format!("Tool: cargo-mutants\nVersion: 1\nScope: a\n{key}: adrift\n");
        assert_eq!(
            parse(&orphaned),
            Err(NotARegister::Orphaned(key.to_owned())),
            "a {key} line with no accepted entry above it"
        );
    }
}

/// One mutant, one entry.
///
/// Two entries for one mutant means the second one's reason is invisible: the
/// comparison is over a set, so whichever reason is wrong stays in the file
/// unread. The near miss is a copied entry somebody edited half of.
#[test]
fn one_mutant_may_not_be_accepted_twice() {
    let twice = format!(
        "{WELL_FORMED}Accepted: src/register/rounding.rs:1:1: replace a with b\n\
         Because: a second reason\nRetired-by: a second retirement\n"
    );
    assert_eq!(
        parse(&twice),
        Err(NotARegister::Duplicate(
            "src/register/rounding.rs:1:1: replace a with b".to_owned()
        ))
    );
}

/// Every refusal says what it is about.
///
/// The same property `tests/refusal_messages.rs` holds over the crate's own
/// refusals, for the same reason: this one is read by whoever wrote the register
/// line that was refused, and an empty message leaves them the file to search.
#[test]
fn every_refusal_names_what_it_refused() {
    let said = [
        NotARegister::UnknownKey("Retired-By".to_owned()).to_string(),
        NotARegister::Unkeyed("a note".to_owned()).to_string(),
        NotARegister::Field("Tool twice".to_owned()).to_string(),
        NotARegister::Incomplete("a mutant".to_owned()).to_string(),
        NotARegister::Orphaned("Because".to_owned()).to_string(),
        NotARegister::Duplicate("a mutant".to_owned()).to_string(),
    ];

    for one in &said {
        assert!(!one.trim().is_empty(), "a refusal with no message");
    }

    let mut sorted = said.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), said.len(), "two refusals say the same thing");

    assert!(said[0].contains("Retired-By"));
    assert!(said[1].contains("a note"));
    assert!(said[3].contains("a mutant"));
}

/// What identifies a mutant survives an edit above it and a Windows path.
///
/// The line and column move whenever anything above them moves, so an entry
/// keyed on them goes stale for a reason that has nothing to do with the
/// arithmetic. What is left still separates the mutants this tree has.
#[test]
fn the_position_is_dropped_and_the_rest_is_kept() {
    assert_eq!(
        without_position("src/register/rounding.rs:197:25: replace > with >= in Decimal::rounded"),
        "src/register/rounding.rs: replace > with >= in Decimal::rounded"
    );

    // The same mutant after twenty lines of comment landed above it.
    assert_eq!(
        without_position("src/register/rounding.rs:217:25: replace > with >= in Decimal::rounded"),
        without_position("src/register/rounding.rs:197:25: replace > with >= in Decimal::rounded")
    );

    // The tool reports the path it was given, and on this platform that is a
    // backslash path. Two spellings of one file are one mutant.
    assert_eq!(
        without_position("src\\register\\rounding.rs:1:1: replace a with b"),
        without_position("src/register/rounding.rs:1:1: replace a with b")
    );

    // A line that is not in that shape is kept whole rather than cut at the
    // first colon, because a colon appears inside the description as well.
    assert_eq!(
        without_position("something else entirely"),
        "something else entirely"
    );
    assert_eq!(
        without_position("src/register/rounding.rs:not a line number:1: replace a with b"),
        "src/register/rounding.rs:not a line number:1: replace a with b"
    );
}

/// The comparison refuses in both directions, on constructed sets.
///
/// The stale direction is the one that is easy to leave out and is half the
/// value: an entry saying a mutant cannot be killed, kept after a test started
/// killing it, is a false statement about the suite that nothing else would
/// find.
#[test]
fn the_comparison_reports_a_new_survivor_and_a_stale_entry() {
    let register = parse(WELL_FORMED).expect("the neighbour parses");

    let agreed = compare(
        &register,
        &["src/register/rounding.rs:9:9: replace a with b".to_owned()],
    );
    assert!(agreed.is_empty(), "{agreed:?}");
    assert_eq!(report(&agreed), "");

    let new_survivor = compare(
        &register,
        &[
            "src/register/rounding.rs:1:1: replace a with b".to_owned(),
            "src/register/rounding.rs:4:4: replace c with d".to_owned(),
        ],
    );
    assert_eq!(
        new_survivor.unaccounted,
        vec!["src/register/rounding.rs: replace c with d".to_owned()]
    );
    assert!(new_survivor.stale.is_empty());
    assert_eq!(
        report(&new_survivor),
        "survived and is not in the register: src/register/rounding.rs: replace c with d\n"
    );

    let killed_since = compare(&register, &[]);
    assert!(killed_since.unaccounted.is_empty());
    assert_eq!(
        killed_since.stale,
        vec!["src/register/rounding.rs: replace a with b".to_owned()]
    );
    assert_eq!(
        report(&killed_since),
        "in the register and did not survive: src/register/rounding.rs: replace a with b\n"
    );
}

/// The tracked register parses, and every file it judges is in the tree.
///
/// A scope line naming a file that was renamed would otherwise reach the run,
/// where the tool reports no mutants for it and the register reads as though
/// that file were clean.
#[test]
fn the_tracked_register_parses_and_names_files_that_exist() {
    let register = parse(TRACKED).expect("the tracked register parses");

    assert_eq!(register.tool, "cargo-mutants");
    assert!(
        !register.version.is_empty(),
        "the tracked register pins no version"
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for file in &register.scope {
        let path = root.join(file);
        assert!(
            path.is_file(),
            "the register judges {file}, which is not a file"
        );
    }
}

/// A run whose report is absent or empty is not a run that found nothing.
///
/// The clause this whole harness turns on. The tool exits non zero both when a
/// mutant survived and when it never started, so the exit status does not
/// separate them, and a harness that read a missing report as an empty survivor
/// list would report a green run over nothing. The neighbour is the same call
/// with a report present, which is a verdict.
#[test]
fn a_run_that_wrote_no_report_is_refused() {
    let one = ["a mutant".to_owned()];

    assert_eq!(verdict("", &one, &[], &[], &[]), Err(NotAVerdict::NoReport));
    assert_eq!(
        verdict("  \n \n", &one, &[], &[], &[]),
        Err(NotAVerdict::NoReport)
    );

    assert_eq!(
        verdict("{}", &one, &[], &[], &[]),
        Ok(Judged {
            caught: 1,
            survived: vec![],
            unviable: 0,
        })
    );
}

/// A run that judged nothing is refused rather than read as everything killed.
///
/// The scope is checked for emptiness when the register is parsed and its files
/// are checked to exist, so a count of zero here is the tool having judged
/// nothing rather than the tree having nothing to judge. The neighbour is one
/// unviable mutant, which is a run that reached the tree and got an answer.
#[test]
fn a_run_that_judged_no_mutants_is_refused() {
    assert_eq!(
        verdict("{}", &[], &[], &[], &[]),
        Err(NotAVerdict::JudgedNothing)
    );

    assert_eq!(
        verdict(
            "{}",
            &[],
            &[],
            &["a mutant that does not compile".to_owned()],
            &[]
        ),
        Ok(Judged {
            caught: 0,
            survived: vec![],
            unviable: 1,
        })
    );
}

/// A mutant the run stopped waiting for is neither killed nor surviving.
///
/// Counting it as caught would let a machine under load report a suite that
/// notices faults it does not notice. Counting it as a survivor would put an
/// entry in the register for a mutant nothing has established anything about.
/// The neighbour is the same run with the limit not reached.
#[test]
fn a_mutant_that_reached_the_time_limit_is_refused_rather_than_counted() {
    let caught = ["a mutant".to_owned()];
    let slow = ["a mutant nothing waited for".to_owned()];

    assert_eq!(
        verdict("{}", &caught, &[], &[], &slow),
        Err(NotAVerdict::Undecided(slow.to_vec()))
    );

    assert_eq!(
        verdict("{}", &caught, &[], &[], &[]),
        Ok(Judged {
            caught: 1,
            survived: vec![],
            unviable: 0,
        })
    );
}

/// A verdict carries the survivors through in the order they were reported.
#[test]
fn a_verdict_carries_the_survivors_it_was_given() {
    let missed = [
        "src/register/rounding.rs:1:1: replace a with b".to_owned(),
        "src/register/rounding.rs:2:2: replace c with d".to_owned(),
    ];
    assert_eq!(
        verdict("{}", &["one".to_owned()], &missed, &["two".to_owned()], &[]),
        Ok(Judged {
            caught: 1,
            survived: missed.to_vec(),
            unviable: 1,
        })
    );
}

/// Every way of not being a verdict says which way it was.
#[test]
fn every_refused_run_says_why_it_was_not_a_verdict() {
    let said = [
        NotAVerdict::NoReport.to_string(),
        NotAVerdict::JudgedNothing.to_string(),
        NotAVerdict::Undecided(vec!["a slow mutant".to_owned()]).to_string(),
    ];

    for one in &said {
        assert!(!one.trim().is_empty(), "a refused run with no message");
    }

    let mut sorted = said.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), said.len(), "two of them say the same thing");

    assert!(said[2].contains("a slow mutant"));
}

/// The register as the tree holds it.
const TRACKED: &str = include_str!("mutation/register.txt");
