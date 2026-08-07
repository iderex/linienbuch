//! Every fixture says which of the three categories it is in, and how it is
//! stored.
//!
//! Real bytes from the sources this board reads come with terms, and committing
//! a convenient extract of somebody's catalogue into a public repository is
//! cheap to do and expensive to undo, because the history keeps it after the
//! working tree has forgotten it. `docs/decisions/fixtures.md` is where the
//! policy is argued. This file is what refuses a departure from it.
//!
//! What it does not do is read a licence. `Licence: whatever-i-like` passes.
//! Deciding whether a licence permits redistribution is a judgement about a
//! legal text and no reading of this tree makes it, so what is refusable is the
//! field being there, and the review is where a wrong value is caught. The
//! retrieval command is the same: required to be present, never run.
//!
//! `tests/fixtures/` does not exist yet. The check reports that as its own state
//! rather than as a clean run over nothing, and what shows that it refuses
//! anything at all is the set of constructed trees under
//! `tests/fixture_policy/cases/`, one per refusal, each with a neighbour one
//! change away that it does not refuse.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The suffix that makes a file a record rather than a fixture.
const RECORD_SUFFIX: &str = ".record.md";

/// The three categories, and nothing else lands.
const CATEGORIES: [&str; 3] = ["synthetic", "redistributable-extract", "structural-stub"];

/// How a fixture is stored.
const ENCODINGS: [&str; 2] = ["raw", "base64"];

/// The category whose terms have to be written down.
const EXTRACT: &str = "redistributable-extract";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    /// The fixture the finding is about, relative to the examined root, with
    /// forward slashes so a failure reads the same on every platform.
    about: String,
    reason: Reason,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reason {
    /// A fixture with nothing beside it saying what it is or where it came
    /// from.
    NoRecord,
    /// A record that leaves out a field its own category requires.
    MissingField(&'static str),
    /// A category invented at the moment it was needed.
    UnknownCategory(String),
    UnknownEncoding(String),
    /// A raw fixture holding bytes that do not survive the trip into git and
    /// back, so what the parser is shown is not what was written. The string
    /// names what was found.
    BytesDoNotSurviveRaw(String),
    /// A fixture declared as base64 that nothing can decode.
    NotBase64(String),
    /// A record left behind after its fixture was deleted, which is a claim
    /// about a file that is not there.
    NoFixture,
}

/// What one examination of a root found, including whether there was anything
/// to examine.
///
/// The count and the presence are carried rather than only the findings,
/// because an empty finding list over a directory that is not there and an empty
/// finding list over a directory full of well recorded fixtures are different
/// statements, and a run that cannot tell them apart is one that reports the
/// first as the second.
#[derive(Debug)]
struct Examination {
    root_exists: bool,
    fixtures: usize,
    findings: Vec<Finding>,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cases_dir() -> PathBuf {
    manifest_dir()
        .join("tests")
        .join("fixture_policy")
        .join("cases")
}

/// Every file below a directory, in a stable order.
fn files_below(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => panic!("cannot read {}: {e}", dir.display()),
    };
    for entry in entries {
        let path = entry.expect("cannot read a directory entry").path();
        if path.is_dir() {
            found.extend(files_below(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The fields of a record. A line of the form `Field: value` at column zero is a
/// field; everything else is the body, which nothing here reads.
fn fields(text: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for line in text.lines() {
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        if let Some((key, value)) = line.split_once(": ")
            && !key.is_empty()
            && !key.contains(char::is_whitespace)
        {
            found.insert(key.to_owned(), value.trim().to_owned());
        }
    }
    found
}

/// Whether a run of bytes could be decoded as base64.
///
/// The alphabet, the padding position and the length, which is what a fixture
/// nobody can decode fails on. It does not decode, because what is being
/// refused is a fixture that is not base64 at all rather than one whose contents
/// are surprising.
fn looks_like_base64(text: &str) -> Result<(), String> {
    let packed: String = text.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if packed.is_empty() {
        return Err("it is empty".to_owned());
    }
    if !packed.len().is_multiple_of(4) {
        return Err(format!(
            "its length is {}, which is not a multiple of four",
            packed.len()
        ));
    }
    let body = packed.trim_end_matches('=');
    if packed.len() - body.len() > 2 {
        return Err("it carries more than two padding characters".to_owned());
    }
    for c in body.chars() {
        if !(c.is_ascii_alphanumeric() || c == '+' || c == '/') {
            return Err(format!("it holds {c:?}, which is outside the alphabet"));
        }
    }
    Ok(())
}

/// What in a raw fixture's bytes would not survive storage.
///
/// A carriage return is rewritten on the way into a working tree and back out of
/// it, so a fixture that needs one is not storing it. Trailing blanks are the
/// same class: they are what a fixture proving an alignment rule depends on and
/// are the first thing an editor removes.
fn does_not_survive_raw(bytes: &[u8]) -> Option<String> {
    if bytes.contains(&b'\r') {
        return Some("a carriage return".to_owned());
    }
    for line in bytes.split(|b| *b == b'\n') {
        match line.last() {
            Some(b' ') => return Some("a line ending in a space".to_owned()),
            Some(b'\t') => return Some("a line ending in a tab".to_owned()),
            _ => {}
        }
    }
    None
}

/// Read one root and say what is wrong under it.
fn examine(root: &Path) -> Examination {
    if !root.is_dir() {
        return Examination {
            root_exists: false,
            fixtures: 0,
            findings: Vec::new(),
        };
    }

    let all = files_below(root);
    let (records, fixtures): (Vec<&PathBuf>, Vec<&PathBuf>) = all
        .iter()
        .partition(|path| path.to_string_lossy().ends_with(RECORD_SUFFIX));

    let mut findings = Vec::new();

    for record in &records {
        let without = record.to_string_lossy();
        let fixture = PathBuf::from(&without[..without.len() - RECORD_SUFFIX.len()]);
        if !fixture.is_file() {
            findings.push(Finding {
                about: relative(root, &fixture),
                reason: Reason::NoFixture,
            });
        }
    }

    for fixture in &fixtures {
        let about = relative(root, fixture);
        let record = PathBuf::from(format!("{}{RECORD_SUFFIX}", fixture.display()));
        let Ok(text) = fs::read_to_string(&record) else {
            findings.push(Finding {
                about,
                reason: Reason::NoRecord,
            });
            continue;
        };
        let fields = fields(&text);

        let category = match fields.get("Category") {
            None => {
                findings.push(Finding {
                    about: about.clone(),
                    reason: Reason::MissingField("Category"),
                });
                None
            }
            Some(name) if !CATEGORIES.contains(&name.as_str()) => {
                findings.push(Finding {
                    about: about.clone(),
                    reason: Reason::UnknownCategory(name.clone()),
                });
                None
            }
            Some(name) => Some(name.clone()),
        };

        if category.as_deref() == Some(EXTRACT) {
            for required in ["Licence", "Retrieved-with"] {
                if !fields.contains_key(required) {
                    findings.push(Finding {
                        about: about.clone(),
                        reason: Reason::MissingField(match required {
                            "Licence" => "Licence",
                            _ => "Retrieved-with",
                        }),
                    });
                }
            }
        }

        match fields.get("Encoding") {
            None => findings.push(Finding {
                about: about.clone(),
                reason: Reason::MissingField("Encoding"),
            }),
            Some(name) if !ENCODINGS.contains(&name.as_str()) => findings.push(Finding {
                about: about.clone(),
                reason: Reason::UnknownEncoding(name.clone()),
            }),
            Some(name) if name == "raw" => {
                let bytes = fs::read(fixture)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", fixture.display()));
                if let Some(what) = does_not_survive_raw(&bytes) {
                    findings.push(Finding {
                        about: about.clone(),
                        reason: Reason::BytesDoNotSurviveRaw(what),
                    });
                }
            }
            Some(_) => {
                let text = fs::read_to_string(fixture)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", fixture.display()));
                if let Err(why) = looks_like_base64(&text) {
                    findings.push(Finding {
                        about: about.clone(),
                        reason: Reason::NotBase64(why),
                    });
                }
            }
        }
    }

    findings.sort();
    Examination {
        root_exists: true,
        fixtures: fixtures.len(),
        findings,
    }
}

/// A directory the test owns, removed when the test ends however it ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("linienbuch-fixture-policy-{name}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(path.join("fixtures")).expect("cannot make a scratch directory");
        Scratch(path)
    }

    /// Write one fixture and its record. The fixture is written as bytes so that
    /// the cases about bytes are not routed through anything that could tidy
    /// them, which is the whole subject of those cases.
    fn fixture(&self, name: &str, bytes: &[u8], record: &str) -> &Self {
        let at = self.0.join("fixtures").join(name);
        fs::write(&at, bytes).expect("cannot write a scratch fixture");
        fs::write(
            self.0
                .join("fixtures")
                .join(format!("{name}{RECORD_SUFFIX}")),
            record,
        )
        .expect("cannot write a scratch record");
        self
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn case(name: &str) -> Examination {
    examine(&cases_dir().join(name))
}

fn reasons(found: &Examination) -> Vec<&Reason> {
    found.findings.iter().map(|f| &f.reason).collect()
}

/// A fixture with nothing beside it is refused, and its neighbour is not.
#[test]
fn a_fixture_with_no_record_is_refused() {
    let refused = case("a_fixture_with_no_record");
    assert_eq!(
        refused.findings,
        vec![Finding {
            about: "fixtures/invented_lines.txt".to_owned(),
            reason: Reason::NoRecord,
        }],
        "a fixture with no record must be refused, and refused for that"
    );

    let allowed = case("the_neighbour_that_has_its_record");
    assert!(
        allowed.findings.is_empty(),
        "the same fixture with a record must not be refused, got {:?}",
        allowed.findings
    );
    assert_eq!(allowed.fixtures, 1, "the neighbour must have been examined");
}

/// An extract has to carry both the licence and the way back to the bytes, and
/// each is refused on its own.
#[test]
fn an_extract_missing_its_licence_or_its_retrieval_command_is_refused() {
    assert_eq!(
        reasons(&case("an_extract_with_no_licence")),
        vec![&Reason::MissingField("Licence")]
    );
    assert_eq!(
        reasons(&case("an_extract_with_no_retrieval_command")),
        vec![&Reason::MissingField("Retrieved-with")]
    );

    let both = case("an_extract_with_both");
    assert!(
        both.findings.is_empty(),
        "an extract carrying both must not be refused, got {:?}",
        both.findings
    );
}

/// The three are the whole list.
#[test]
fn a_category_outside_the_three_is_refused() {
    assert_eq!(
        reasons(&case("a_category_that_is_not_one_of_the_three")),
        vec![&Reason::UnknownCategory("convenient-extract".to_owned())]
    );
}

/// A record that leaves a required field out is refused for that field.
#[test]
fn a_record_naming_no_category_or_no_encoding_is_refused() {
    assert_eq!(
        reasons(&case("a_record_with_no_category")),
        vec![&Reason::MissingField("Category")]
    );
    assert_eq!(
        reasons(&case("a_record_with_no_encoding")),
        vec![&Reason::MissingField("Encoding")]
    );
}

/// The other direction. A record whose fixture is gone is a claim about a file
/// that is not there.
#[test]
fn a_record_whose_fixture_is_gone_is_refused() {
    let refused = case("a_record_whose_fixture_is_gone");
    assert_eq!(
        refused.findings,
        vec![Finding {
            about: "fixtures/invented_lines.txt".to_owned(),
            reason: Reason::NoFixture,
        }]
    );
    assert_eq!(
        refused.fixtures, 0,
        "there is no fixture there, which is the point"
    );
}

/// A raw fixture holding bytes that storage would rewrite is refused, and the
/// same fixture without them is not.
///
/// Constructed here rather than tracked, and the reason is the rule itself: a
/// tracked file holding a carriage return would have that byte rewritten on the
/// way in, so the case would arrive already repaired and the guard would be
/// proved against a file that no longer violates anything.
#[test]
fn a_raw_fixture_holding_bytes_that_do_not_survive_is_refused() {
    let record = "Category: synthetic\nEncoding: raw\n\nA constructed case.\n";

    let carriage = Scratch::new("carriage-return");
    carriage.fixture("lines.txt", b"one\r\ntwo\r\n", record);
    assert_eq!(
        reasons(&examine(carriage.path())),
        vec![&Reason::BytesDoNotSurviveRaw(
            "a carriage return".to_owned()
        )]
    );

    let trailing = Scratch::new("trailing-space");
    trailing.fixture("lines.txt", b"one \ntwo\n", record);
    assert_eq!(
        reasons(&examine(trailing.path())),
        vec![&Reason::BytesDoNotSurviveRaw(
            "a line ending in a space".to_owned()
        )]
    );

    let neighbour = Scratch::new("survives");
    neighbour.fixture("lines.txt", b"one\ntwo\n", record);
    let found = examine(neighbour.path());
    assert!(
        found.findings.is_empty(),
        "the neighbour, one byte away, must not be refused, got {:?}",
        found.findings
    );
    assert_eq!(found.fixtures, 1);
}

/// A fixture declared as base64 that nothing could decode is refused, and one
/// that could be is not.
#[test]
fn a_base64_fixture_that_is_not_base64_is_refused() {
    let record = "Category: synthetic\nEncoding: base64\n\nA constructed case.\n";

    let outside = Scratch::new("outside-the-alphabet");
    outside.fixture("lines.b64", b"b25l!Xdv\n", record);
    assert_eq!(
        reasons(&examine(outside.path())),
        vec![&Reason::NotBase64(
            "it holds '!', which is outside the alphabet".to_owned()
        )]
    );

    let short = Scratch::new("wrong-length");
    short.fixture("lines.b64", b"b25ldw\n", record);
    assert_eq!(
        reasons(&examine(short.path())),
        vec![&Reason::NotBase64(
            "its length is 6, which is not a multiple of four".to_owned()
        )]
    );

    // The neighbour, one character away from the first: `b25lXdvw` is the same
    // eight characters with the one outside the alphabet replaced.
    let neighbour = Scratch::new("decodable");
    neighbour.fixture("lines.b64", b"b25lXdvw\n", record);
    let found = examine(neighbour.path());
    assert!(
        found.findings.is_empty(),
        "the neighbour must not be refused, got {:?}",
        found.findings
    );
}

/// The absence of the fixture directory is a state of its own.
///
/// Without this, a run over a directory that is not there returns no findings
/// and reads exactly like a run over a directory full of well recorded
/// fixtures.
#[test]
fn a_root_that_is_not_there_is_not_a_clean_run() {
    let nothing = examine(&manifest_dir().join("tests").join("no-such-directory"));
    assert!(!nothing.root_exists);
    assert_eq!(nothing.fixtures, 0);
    assert!(nothing.findings.is_empty());

    let present = examine(&cases_dir().join("the_neighbour_that_has_its_record"));
    assert!(present.root_exists);
    assert_eq!(present.fixtures, 1);
}

/// The tree itself, which is what all of the above exists for.
#[test]
fn the_tree_carries_no_unrecorded_fixture() {
    let root = manifest_dir().join("tests").join("fixtures");
    let found = examine(&root);
    assert!(
        found.findings.is_empty(),
        "fixtures in this tree that are not recorded: {:?}",
        found.findings
    );

    // Said out loud rather than left to be inferred from a green run. When the
    // first fixture lands this number moves and this line stops being the
    // disclosure of an empty subject.
    println!(
        "fixture policy: examined {} fixture(s) under tests/fixtures/, which {}",
        found.fixtures,
        if found.root_exists {
            "exists"
        } else {
            "does not exist yet"
        }
    );
}
