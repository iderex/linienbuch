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

use std::fmt::Write as _;
use std::net::{TcpStream, ToSocketAddrs};
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
