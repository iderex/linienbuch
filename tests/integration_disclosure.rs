//! What the default suite did not run, printed by the default suite.
//!
//! A run that covered less than the whole set reads exactly like one that
//! covered everything and found nothing, unless it says so in its own output.
//! That is the failure a separate harness creates if nobody discloses it, and
//! this file is the disclosure.
//!
//! Each leg below is an ignored test whose reason says what the leg needs and
//! what running it costs. `cargo test` prints an ignored test's reason on the
//! line for that test, so every default run names the legs it did not run
//! without anybody having to look for them.
//!
//! The last test in this file is not ignored. It compares the legs named here
//! against the legs the harness actually holds, so the disclosure cannot drift
//! away from the thing it is disclosing.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// The legs the harness holds, named here so the disclosure and the harness can
/// be compared. Adding a leg to one and not the other reds the suite.
const LEGS: [&str; 4] = [
    "a_full_line_list_parses_within_the_memory_ceiling",
    "the_date_stamp_on_a_finding_is_a_real_date",
    "the_first_source_host_is_reachable",
    "the_published_format_matches_what_the_server_serves",
];

#[test]
#[ignore = "in the integration harness: needs the network, one socket to an upstream host. Run it with: cargo test --test integration"]
fn the_first_source_host_is_reachable() {
    unreachable!("this is a disclosure, the leg is in tests/integration/main.rs")
}

#[test]
#[ignore = "in the integration harness: needs the network and a retrieval that does not exist yet, #26 and #27. Run it with: cargo test --test integration"]
fn the_published_format_matches_what_the_server_serves() {
    unreachable!("this is a disclosure, the leg is in tests/integration/main.rs")
}

#[test]
#[ignore = "in the integration harness: needs a download measured in gigabytes and the parser in #29. Run it with: cargo test --test integration"]
fn a_full_line_list_parses_within_the_memory_ceiling() {
    unreachable!("this is a disclosure, the leg is in tests/integration/main.rs")
}

#[test]
#[ignore = "in the integration harness: not network bound, but it lives beside the code it checks. Run it with: cargo test --test integration"]
fn the_date_stamp_on_a_finding_is_a_real_date() {
    unreachable!("this is a disclosure, the leg is in tests/integration/main.rs")
}

/// What is declared here but missing from the harness, and the other way round.
///
/// A pure function so that the comparison can be proved on constructed sets
/// rather than only against the tree, which is the only way to show it refuses
/// anything at all.
fn drift<'a>(
    declared: &'a BTreeSet<String>,
    in_harness: &'a BTreeSet<String>,
) -> (Vec<&'a String>, Vec<&'a String>) {
    (
        declared.difference(in_harness).collect(),
        in_harness.difference(declared).collect(),
    )
}

/// Every test function in the harness source.
///
/// A test function rather than every function, because the harness also holds
/// the helpers a leg uses and those are not legs. The marker is the `#[test]`
/// attribute, which is what makes a function a leg in the first place.
fn harness_legs() -> BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration")
        .join("main.rs");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let mut legs = BTreeSet::new();
    let mut is_a_test = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[test]") {
            is_a_test = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("fn ")
            && let Some((name, _)) = rest.split_once('(')
        {
            if is_a_test {
                legs.insert(name.to_owned());
            }
            is_a_test = false;
        }
    }
    legs
}

/// The comparison, refusing in both directions, on constructed sets.
#[test]
fn drift_is_reported_in_both_directions() {
    let a: BTreeSet<String> = ["one".to_owned(), "two".to_owned()].into();
    let b: BTreeSet<String> = ["two".to_owned(), "three".to_owned()].into();

    let (undeclared_here, undisclosed_there) = drift(&a, &b);
    assert_eq!(undeclared_here, vec![&"one".to_owned()]);
    assert_eq!(undisclosed_there, vec![&"three".to_owned()]);

    let (none_here, none_there) = drift(&a, &a);
    assert!(none_here.is_empty() && none_there.is_empty());
}

/// The disclosure and the harness name the same legs.
#[test]
fn the_disclosure_names_every_leg_the_harness_holds() {
    let declared: BTreeSet<String> = LEGS.iter().map(|s| (*s).to_owned()).collect();
    let in_harness = harness_legs();

    let (missing_from_harness, missing_from_disclosure) = drift(&declared, &in_harness);
    assert!(
        missing_from_harness.is_empty(),
        "declared here and absent from the harness: {missing_from_harness:?}"
    );
    assert!(
        missing_from_disclosure.is_empty(),
        "in the harness and not disclosed here: {missing_from_disclosure:?}"
    );
}

/// Every leg named in `LEGS` has an ignored test above it, so the reason for
/// each one is printed by every default run rather than only counted.
#[test]
fn every_declared_leg_has_a_printed_reason() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration_disclosure.rs");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    for leg in LEGS {
        let signature = format!("fn {leg}(");
        let start = text
            .find(&signature)
            .unwrap_or_else(|| panic!("{leg} has no disclosure function"));
        let before = &text[..start];
        assert!(
            before
                .lines()
                .rev()
                .take(3)
                .any(|line| line.trim_start().starts_with("#[ignore = \"")),
            "{leg} has a disclosure function with no printed reason"
        );
    }
}
