//! The integration harness. It needs the network, or a large download, or both.
//!
//! The name says what it needs rather than when it runs. It is not an extended
//! suite and not a nightly, because those names describe a schedule and hide a
//! dependency, and the dependency is the thing somebody has to decide about
//! before running it.
//!
//! It is excluded from the default suite by construction. `Cargo.toml` declares
//! this target with `test = false`, so `cargo test` does not build or run it and
//! no naming convention is doing the work. Its own command is
//!
//!     cargo test --test integration
//!
//! Nothing in the merge gate runs it. Today nothing in the merge gate runs cargo
//! at all, which is checkable:
//!
//!     grep -rn "cargo" .github/workflows/
//!
//! and the only hit is the word inside a comment in an unrelated file. The
//! checks themselves are #5's and this harness must not be added to them.
//!
//! A failure here is a finding rather than a broken test. A source that changed
//! its format, or moved, or went away, is real information about the field, so a
//! failure carries the retrieval date and what came back, in a form somebody can
//! paste into an issue.

#[path = "../parity/map.rs"]
mod map;

#[path = "../mutation/register.rs"]
mod mutation;

use std::fmt::Write as _;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How long to wait for a host before calling it unreachable.
const REACH_TIMEOUT: Duration = Duration::from_secs(10);

/// The first source this board intends to read. Reachability only: opening a
/// socket says the host is answering on that port, and says nothing about what
/// it would serve. What it serves is the leg below that #26 and #27 owe.
const FIRST_SOURCE_HOST: &str = "physics.nist.gov";
const FIRST_SOURCE_PORT: u16 = 443;

/// The text of a finding, in the shape somebody can paste into an issue.
///
/// The retrieval date and what came back are both in it, because a failure here
/// is a statement about a source on a date rather than about this repository.
fn finding(leg: &str, detail: &str) -> String {
    let mut out = String::new();
    writeln!(out, "finding: integration leg {leg} failed").expect("writing to a String");
    writeln!(out, "retrieved: {}", today_utc()).expect("writing to a String");
    writeln!(out, "response: {detail}").expect("writing to a String");
    out
}

/// Today's date in UTC, as `YYYY-MM-DD`.
fn today_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after 1970")
        .as_secs();
    let (year, month, day) = civil_from_days(i64::try_from(seconds / 86_400).expect("in range"));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Days since the epoch to a calendar date, by the standard civil-from-days
/// algorithm. Written out rather than taken from a crate, because the tree
/// declares no dependency and this is the whole of what a date stamp needs.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (
        year,
        u32::try_from(m).expect("a month"),
        u32::try_from(d).expect("a day"),
    )
}

/// The date stamp on a finding is a real date.
///
/// Not network bound, and it is here rather than in the default suite because
/// the thing it checks is here. Three fixed days with known answers, including
/// the leap day that the naive version of this arithmetic gets wrong.
#[test]
fn the_date_stamp_on_a_finding_is_a_real_date() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    assert_eq!(civil_from_days(-1), (1969, 12, 31));
}

/// Whether the first source is answering at all.
///
/// Reachability, and nothing more. A host that answers on the port is not a
/// host that still serves the format this board parses, and reading it as such
/// would be the larger claim made from the smaller measurement.
#[test]
fn the_first_source_host_is_reachable() {
    let target = format!("{FIRST_SOURCE_HOST}:{FIRST_SOURCE_PORT}");
    let mut addresses = match target.to_socket_addrs() {
        Ok(addresses) => addresses,
        Err(e) => panic!(
            "{}",
            finding(
                "the_first_source_host_is_reachable",
                &format!("{target} did not resolve: {e}")
            )
        ),
    };
    let address = match addresses.next() {
        Some(address) => address,
        None => panic!(
            "{}",
            finding(
                "the_first_source_host_is_reachable",
                &format!("{target} resolved to no address")
            )
        ),
    };
    if let Err(e) = TcpStream::connect_timeout(&address, REACH_TIMEOUT) {
        panic!(
            "{}",
            finding(
                "the_first_source_host_is_reachable",
                &format!("connecting to {address} failed after {REACH_TIMEOUT:?}: {e}")
            )
        );
    }
}

/// Whether the format the parser expects is the format the server serves today.
///
/// Declared and not implemented. It needs the retrieval code from #26 and the
/// parser from #27, and writing a second retrieval here to get the leg running
/// sooner would put two retrievals in the tree, which is the thing #26 exists to
/// stop.
#[test]
#[ignore = "not implemented: needs the retrieval in #26 and the parser in #27"]
fn the_published_format_matches_what_the_server_serves() {
    unimplemented!("#26 and #27")
}

/// Whether a full line list parses with the memory ceiling holding.
///
/// Declared and not implemented. It needs the Kurucz parser from #29 and a
/// download measured in gigabytes, and the ceiling it asserts against does not
/// exist until there is a parser to set one on.
#[test]
#[ignore = "not implemented: needs the parser in #29 and a download measured in gigabytes"]
fn a_full_line_list_parses_within_the_memory_ceiling() {
    unimplemented!("#29")
}

/// The gate `docs/parity.md` is measured against.
const TARGET_REPOSITORY: &str = "iderex/jellyfin-plugin-sso";

/// The check run names a repository's default branch requires today.
///
/// Read from the forge rather than from anything tracked here. A pinned copy of
/// somebody else's required set is the nearest thing to hand rather than the
/// thing itself, and a map checked against a copy would agree with the copy for
/// as long as the copy was stale.
///
/// The subprocess is the means because the command is the one `docs/parity.md`
/// prints, and reimplementing it here would need an HTTP client with TLS and a
/// credential store, which is three dependencies this tree does not carry to ask
/// a question one already-installed program answers.
fn required_checks(repository: &str) -> Result<Vec<String>, String> {
    let jq = ".[] | select(.type==\"required_status_checks\") \
              | .parameters.required_status_checks[].context";
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{repository}/rules/branches/main"),
            "--jq",
            jq,
        ])
        .output()
        .map_err(|e| format!("gh could not be run: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "gh exited with {} and said: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Every check the target gate requires today is placed by the parity map.
///
/// It needs the network, and an authenticated `gh`, which is why it is here
/// rather than in the default suite. The other direction is deliberately not
/// checked: the map also names checks the target does not require, which is what
/// its dropped and non-gating rows are.
#[test]
fn the_target_required_set_is_covered_by_the_parity_map() {
    let leg = "the_target_required_set_is_covered_by_the_parity_map";

    let required = match required_checks(TARGET_REPOSITORY) {
        Ok(names) => names,
        Err(detail) => panic!("{}", finding(leg, &detail)),
    };

    // An empty answer would make the comparison below pass over nothing, which
    // reads exactly like a map that covers everything. A target with no required
    // checks is a real change in the thing this board copies, so it is reported
    // rather than treated as agreement.
    if required.is_empty() {
        panic!(
            "{}",
            finding(
                leg,
                &format!("{TARGET_REPOSITORY} requires no status checks on its default branch")
            )
        );
    }

    let missing = map::unplaced(&map::parity_document(), &required);
    assert!(
        missing.is_empty(),
        "{}",
        finding(
            leg,
            &format!(
                "{TARGET_REPOSITORY} requires {} check(s) that docs/parity.md does not place: {missing:?}",
                missing.len()
            )
        )
    );
}

/// The register the mutation run is judged against, as the tree holds it.
///
/// Compiled in rather than read at run time, so a run cannot be made to pass by
/// pointing it at a different file. What the register says, and the rules it has
/// to satisfy, are proved by `tests/mutation_register.rs` in the default suite.
const MUTATION_REGISTER: &str = include_str!("../mutation/register.txt");

/// How long one mutant's suite may take before the run stops waiting for it.
///
/// Seconds rather than a multiple of the baseline, because the default suite
/// runs in well under one second and any multiple of that is a limit the
/// machine's own load crosses on its own.
///
/// Generous on purpose, and measured rather than guessed. At sixty seconds a
/// run of these four files reported fifteen mutants as reaching the limit, and
/// the same mutants under a wider one were decided in both directions, so what
/// the tight limit produced was a statement about how busy the machine was.
/// This leg refuses a mutant it did not wait for, so the limit has to be far
/// enough above the honest case that only a fault which does not terminate
/// reaches it.
const MUTANT_TIMEOUT_SECONDS: &str = "300";

/// The most mutants built and tested at once, whatever the machine has.
///
/// The tool defaults to one, which turns this leg from long into overnight.
/// Capped rather than taken whole, because every job is a copy of the tree
/// being built and tested, and the suites slow each other down past the point
/// where another one helps.
const MOST_AT_ONCE: usize = 8;

/// Whether a seeded fault anywhere in the core is noticed by the suite.
///
/// The direct answer to whether the tests would notice if the arithmetic were
/// wrong, which line coverage over short arithmetic functions does not give: the
/// functions here are executed by any test that calls them, so coverage is
/// complete and says nothing.
///
/// It is here rather than in the default suite for two reasons and neither is a
/// schedule. It needs `cargo-mutants`, which is not part of the pinned
/// toolchain and is not on a machine that has only followed the readme. And it
/// rebuilds and re-runs the suite once per mutant, which is minutes rather than
/// the fraction of a second the default suite costs.
///
/// It never gates a merge. `docs/parity.md` places mutation testing among what
/// this board carries as non gating, and the harness this leg sits in is not run
/// by anything under `.github/workflows/`.
///
/// A run that produced no report fails here rather than reporting nothing found.
/// That clause is the one this kind of tooling usually gets wrong: a harness
/// whose failure looks like success is worse than no harness, because it is
/// believed.
#[test]
fn the_mutation_run_kills_everything_the_register_does_not_accept() {
    let leg = "the_mutation_run_kills_everything_the_register_does_not_accept";

    let register = match mutation::parse(MUTATION_REGISTER) {
        Ok(register) => register,
        Err(why) => panic!(
            "{}",
            finding(leg, &format!("the register is not one: {why}"))
        ),
    };

    // The version is checked before the run rather than recorded after it. Two
    // runs of one commit under two versions of the tool are two runs of
    // different things, and the register's survivors are a statement about the
    // version beside them.
    let expected = format!("{} {}", register.tool, register.version);
    let installed = match Command::new("cargo")
        .args(["mutants", "--version"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        Ok(output) => panic!(
            "{}",
            finding(
                leg,
                &format!(
                    "cargo mutants --version exited with {} and said: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                )
            )
        ),
        Err(e) => panic!("{}", finding(leg, &format!("cargo could not be run: {e}"))),
    };
    assert_eq!(
        installed,
        expected,
        "{}",
        finding(
            leg,
            &format!(
                "the register pins {expected:?} and this machine has {installed:?}; \
                 install the pinned version or change the register and re-measure"
            )
        )
    );

    // Outside the tree, because the tool's output directory is not tracked and
    // not ignored, and a run that left one behind would put an untracked
    // directory of build products in front of every later `git status`.
    let output = std::env::temp_dir().join(format!("linienbuch-mutants-{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&output) {
        panic!(
            "{}",
            finding(leg, &format!("cannot create {}: {e}", output.display()))
        );
    }

    // How many mutants are built at once changes how long the run takes and
    // nothing it decides, since each one is built and tested in a copy of the
    // tree of its own. It reaches the verdict only through the limit above, by
    // slowing every suite down together, which is why that limit is generous
    // rather than tight.
    let jobs = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(MOST_AT_ONCE)
        .to_string();

    let mut command = Command::new("cargo");
    command.args([
        "mutants",
        "--jobs",
        &jobs,
        "--timeout",
        MUTANT_TIMEOUT_SECONDS,
        "--output",
        &output.to_string_lossy(),
    ]);
    for file in &register.scope {
        command.args(["-f", file]);
    }

    // The exit status is deliberately not the verdict. The tool exits non zero
    // when a mutant survived, which is the case this leg is here to describe
    // rather than to restate, and it exits non zero for a run that never
    // started. The two are told apart by the report below, which the second
    // does not write.
    let ran = match command.output() {
        Ok(output) => output,
        Err(e) => panic!(
            "{}",
            finding(leg, &format!("cargo mutants could not be run: {e}"))
        ),
    };

    // What the run amounts to is decided by `mutation::verdict`, whose every
    // refusal is proved on constructed input by the default suite. This leg
    // reads the four files it wrote and does nothing else with them, so the
    // rules a twelve minute run is judged by are checked by every run of
    // `cargo test` rather than only by whoever asked for this one.
    let out = output.join("mutants.out");
    let judged = match mutation::verdict(
        &std::fs::read_to_string(out.join("outcomes.json")).unwrap_or_default(),
        &lines_of(&out.join("caught.txt")),
        &lines_of(&out.join("missed.txt")),
        &lines_of(&out.join("unviable.txt")),
        &lines_of(&out.join("timeout.txt")),
    ) {
        Ok(judged) => judged,
        Err(why) => panic!(
            "{}",
            finding(
                leg,
                &format!(
                    "{why}\nthe limit was {MUTANT_TIMEOUT_SECONDS} seconds per mutant, the \
                     output is under {}, and the tool exited with {} saying: {}",
                    output.display(),
                    ran.status,
                    String::from_utf8_lossy(&ran.stderr).trim()
                )
            )
        ),
    };

    let disagreement = mutation::compare(&register, &judged.survived);
    assert!(
        disagreement.is_empty(),
        "{}",
        finding(
            leg,
            &format!(
                "{} caught, {} survived, {} unviable\n{}",
                judged.caught,
                judged.survived.len(),
                judged.unviable,
                mutation::report(&disagreement)
            )
        )
    );

    // Only once the comparison agreed, so a failure leaves the report where
    // somebody can read it.
    let _ = std::fs::remove_dir_all(&output);
}

/// The non blank lines of one of the tool's output files.
///
/// An absent file reads as no lines. Whether the run wrote anything at all is
/// decided by the report above rather than by each of these, so that a missing
/// `missed.txt` cannot be read as a run in which nothing survived.
fn lines_of(path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}
