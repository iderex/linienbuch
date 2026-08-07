//! The default suite refuses a test that reaches off this machine.
//!
//! A suite whose result depends on a remote service reports that service's
//! availability, and a red run then means nothing until somebody has
//! investigated. The failure is also slow: a socket to a host that is not there
//! hangs for a timeout rather than failing, so the cost lands on whoever is
//! waiting rather than on whoever wrote it.
//!
//! The refusal is a search over the tracked sources rather than a sandbox around
//! the running test, because nothing in the standard library lets one test
//! revoke another test's access to a socket. What that buys and what it does not
//! is written out in `docs/testing.md` and is not softened here: a call reached
//! through a name this guard does not know about passes.
//!
//! A bind to an address that is not loopback is refused by the same rule and for
//! a second reason. On at least one platform such a bind raises a firewall
//! consent dialog whose subject is the executable path, so answering it settles
//! nothing beyond one build directory and every new one asks again. That is the
//! elevation constraint, arriving through the same door as the network one.

use std::fs;
use std::path::{Path, PathBuf};

/// A place where a scanned file broke the rule, and why it broke it.
#[derive(Debug, PartialEq, Eq)]
struct Finding {
    file: PathBuf,
    line: usize,
    api: String,
    reason: Reason,
}

#[derive(Debug, PartialEq, Eq)]
enum Reason {
    /// The line names a network API and an address that is not loopback. The
    /// string is the host as the guard read it.
    OffThisMachine(String),
    /// The line names a network API and carries no address literal, so the guard
    /// cannot tell where it goes. Refused rather than allowed, because a guard
    /// that passes what it cannot read passes everything eventually.
    AddressNotVisible,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn guard_dir() -> PathBuf {
    manifest_dir().join("tests").join("environment_guard")
}

/// The API surfaces to look for, read from the file that holds them.
fn needles() -> Vec<String> {
    let path = guard_dir().join("needles.txt");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// The host part of an address literal, for the two spellings that occur.
fn host_of(literal: &str) -> &str {
    if let Some(rest) = literal.strip_prefix('[')
        && let Some(end) = rest.find(']')
    {
        return &rest[..end];
    }
    match literal.rfind(':') {
        Some(colon) => &literal[..colon],
        None => literal,
    }
}

/// Loopback stays on this machine, so it is allowed. `0.0.0.0` and `::` are not
/// loopback: they are every interface, which is the case the firewall dialog is
/// about.
fn is_loopback(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

/// The first double quoted literal on a line, if there is one.
fn first_literal(line: &str) -> Option<&str> {
    let open = line.find('"')?;
    let rest = &line[open + 1..];
    let close = rest.find('"')?;
    Some(&rest[..close])
}

fn scan_file(path: &Path, needles: &[String]) -> Vec<Finding> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => panic!("cannot read {}: {e}", path.display()),
    };
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        for needle in needles {
            if !line.contains(needle.as_str()) {
                continue;
            }
            let reason = match first_literal(line) {
                Some(literal) => {
                    let host = host_of(literal);
                    if is_loopback(host) {
                        continue;
                    }
                    Reason::OffThisMachine(host.to_owned())
                }
                None => Reason::AddressNotVisible,
            };
            findings.push(Finding {
                file: path.to_owned(),
                line: index + 1,
                api: needle.clone(),
                reason,
            });
        }
    }
    findings
}

fn scan(root: &Path, skip: &[PathBuf]) -> Vec<Finding> {
    let needles = needles();
    let mut files = Vec::new();
    collect(root, skip, &mut files);
    files.sort();
    files.iter().flat_map(|f| scan_file(f, &needles)).collect()
}

fn collect(dir: &Path, skip: &[PathBuf], out: &mut Vec<PathBuf>) {
    if !dir.exists() || skip.iter().any(|s| s == dir) {
        return;
    }
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect(&path, skip, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn fixture(kind: &str, name: &str) -> PathBuf {
    guard_dir().join("fixtures").join(kind).join(name)
}

/// The needle containing `fragment`, looked up rather than written out.
///
/// Writing the expected API in an assertion would put a needle in the guard's
/// own source, and the guard scans its own source, so it would refuse itself.
/// A fragment that is not itself a needle avoids that without excluding this
/// file from the scan.
fn needle_named(fragment: &str) -> String {
    needles()
        .into_iter()
        .find(|needle| needle.contains(fragment))
        .unwrap_or_else(|| panic!("no needle contains {fragment}"))
}

/// The fixture reaches an upstream catalogue. One finding, naming the host it
/// tried to reach.
#[test]
fn refuses_a_test_that_reaches_an_upstream_host() {
    let path = fixture("refused", "fetches_from_upstream.rs");
    let findings = scan_file(&path, &needles());

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding in {}, got {findings:#?}",
        path.display()
    );
    assert_eq!(findings[0].api, needle_named("TcpStream"));
    assert_eq!(
        findings[0].reason,
        Reason::OffThisMachine("physics.nist.gov".to_owned())
    );
}

/// The fixture builds its address somewhere else, so the guard cannot read it.
/// Refused, and refused for a different reason than the one above, which is what
/// makes it a second site rather than the same one twice.
#[test]
fn refuses_an_address_the_guard_cannot_read() {
    let path = fixture("refused", "address_from_a_variable.rs");
    let findings = scan_file(&path, &needles());

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding in {}, got {findings:#?}",
        path.display()
    );
    assert_eq!(findings[0].api, needle_named("TcpStream"));
    assert_eq!(findings[0].reason, Reason::AddressNotVisible);
}

/// The neighbour. One address literal away from the first fixture and refused by
/// nothing, because a guard that refuses its neighbour too has proved only that
/// it refuses things.
#[test]
fn does_not_refuse_the_loopback_neighbour() {
    let path = fixture("allowed", "loopback_only.rs");
    let findings = scan_file(&path, &needles());

    assert!(
        findings.is_empty(),
        "the loopback neighbour must not be refused, got {findings:#?}"
    );
}

/// A bind to every interface is refused, which is the case that raises a
/// firewall consent dialog rather than the case that reaches a remote host.
#[test]
fn refuses_a_bind_to_every_interface() {
    let path = fixture("refused", "binds_every_interface.rs");
    let findings = scan_file(&path, &needles());

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding in {}, got {findings:#?}",
        path.display()
    );
    assert_eq!(findings[0].api, needle_named("TcpListener"));
    assert_eq!(
        findings[0].reason,
        Reason::OffThisMachine("0.0.0.0".to_owned())
    );
}

/// The tree itself. The fixture directory is the one place violations are meant
/// to live, so it is the one place skipped, and it is named here rather than
/// configured somewhere a reader would not look.
#[test]
fn the_tracked_sources_reach_nothing() {
    let root = manifest_dir();
    let skip = vec![
        guard_dir().join("fixtures"),
        root.join("tests").join("integration"),
    ];
    let mut findings = scan(&root.join("src"), &skip);
    findings.extend(scan(&root.join("tests"), &skip));

    assert!(
        findings.is_empty(),
        "tracked sources must not reach off this machine, got {findings:#?}"
    );
}
