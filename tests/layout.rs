//! The generic side stays generic.
//!
//! Four sibling registers have the same shape as this one and all of them need
//! provenance edges, claims that do not collapse into values, snapshot pinning
//! and uncertainty that survives arithmetic. None of them needs a wavelength.
//! `docs/decisions/layout.md` draws the line and states the test: could a
//! register of material parameters, or of measurement histories, use the generic
//! side unchanged?
//!
//! That question is a judgement and this file does not make it. What it refuses
//! is the two ways the answer becomes no without anybody deciding it should. An
//! identifier under `src/register/` that names a quantity specific to
//! spectroscopy, and a reference from that side to the other one.
//!
//! Three bounds, and none of them is softened.
//!
//! The list of quantities is a floor. A spectroscopic idea spelled in a word
//! this file does not hold passes, and the entry for it is added when the word
//! arrives. What the list buys is that the words somebody would actually reach
//! for are refused.
//!
//! It reads words rather than parsing Rust, so an identifier assembled from
//! fragments is not seen.
//!
//! It reads code and not comments. A doc comment on the generic side may say the
//! word wavelength, because explaining what the boundary excludes requires
//! naming what is on the other side of it, and refusing that would make the
//! boundary undocumentable on the side that needs it explained.

use std::fs;
use std::path::{Path, PathBuf};

/// Words that name a quantity, an object or a convention specific to
/// spectroscopy.
///
/// Each is one a person writing on the generic side would reach for without
/// noticing which side they were on. Ordinary English words that happen to occur
/// in this field are deliberately absent: `line`, `term`, `configuration` and
/// `source` all have meanings here that have nothing to do with spectroscopy,
/// and a list holding them would refuse correct code often enough to be turned
/// off.
const QUANTITIES: [&str; 22] = [
    "wavelength",
    "wavenumber",
    "angstrom",
    "nanometre",
    "oscillator",
    "gf",
    "einstein",
    "ritz",
    "multiplet",
    "isotopologue",
    "ionisation",
    "ionization",
    "spectrum",
    "spectra",
    "species",
    "transition",
    "parity",
    "abundance",
    "molecule",
    "atom",
    "element",
    "vacuum",
];

/// The module path the generic side must not reach for.
const THE_OTHER_SIDE: &str = "spectroscopy";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    file: String,
    line: usize,
    reason: Reason,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reason {
    /// An identifier on the generic side naming something only this field has.
    NamesASpectroscopicQuantity(String),
    /// The generic side reaching into the domain side, which is the same failure
    /// arriving through the type system instead of through a name.
    ReachesTheOtherSide,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn generic_side() -> PathBuf {
    manifest_dir().join("src").join("register")
}

/// A line with its comment removed, so that prose about the boundary is not
/// read as code that crosses it.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// The words of a line of code, lowercased, split at everything that cannot be
/// part of an identifier.
///
/// `Wavelength`, `WAVELENGTH` and `air_wavelength` all yield the word this list
/// holds. `pair` does not yield `air`, because the split is at the boundaries of
/// a word rather than inside one.
fn words(code: &str) -> Vec<String> {
    code.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// What is wrong in one file of the generic side.
///
/// A pure function over the text, so it can be shown to refuse and to accept on
/// constructed input rather than only on whatever the tree happens to hold
/// today.
fn examine(name: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code = code_of(line);
        if code.contains(THE_OTHER_SIDE) {
            findings.push(Finding {
                file: name.to_owned(),
                line: index + 1,
                reason: Reason::ReachesTheOtherSide,
            });
        }
        for word in words(code) {
            if QUANTITIES.contains(&word.as_str()) {
                findings.push(Finding {
                    file: name.to_owned(),
                    line: index + 1,
                    reason: Reason::NamesASpectroscopicQuantity(word),
                });
            }
        }
    }
    findings
}

fn rust_files_below(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("cannot read a directory entry").path();
        if path.is_dir() {
            found.extend(rust_files_below(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// The refusal, on constructed input, with its neighbour beside it.
#[test]
fn an_identifier_naming_a_spectroscopic_quantity_is_refused() {
    let crosses = "pub struct Claim {\n    pub air_wavelength: f64,\n}\n";
    assert_eq!(
        examine("constructed.rs", crosses),
        vec![Finding {
            file: "constructed.rs".to_owned(),
            line: 2,
            reason: Reason::NamesASpectroscopicQuantity("wavelength".to_owned()),
        }]
    );

    // The neighbour, one field name away and nothing else. A claim carries a
    // value; what quantity the value is of belongs to the other side.
    let stays = "pub struct Claim {\n    pub value: f64,\n}\n";
    assert!(
        examine("constructed.rs", stays).is_empty(),
        "the neighbour must not be refused"
    );
}

/// A word that merely contains one of the listed words is not one of them.
#[test]
fn a_word_that_only_contains_a_listed_word_is_not_refused() {
    for ordinary in [
        "    let pair = (a, b);\n",
        "    let atomised = split(text);\n",
        "    let elements = set.len();\n",
    ] {
        assert!(
            examine("constructed.rs", ordinary).is_empty(),
            "refused an ordinary word: {ordinary:?}"
        );
    }

    // And the words themselves still are, so the test above is not passing by
    // the list being empty.
    assert_eq!(examine("constructed.rs", "    let atom = one;\n").len(), 1);
    assert_eq!(
        examine("constructed.rs", "    let element = one;\n").len(),
        1
    );
}

/// A doc comment naming the other side is not a crossing.
///
/// Explaining what the boundary excludes requires naming what is on the other
/// side of it. A guard that refused that would make the boundary undocumentable
/// in the one place a reader looks for it.
#[test]
fn prose_about_the_boundary_is_not_a_crossing() {
    let documented = "//! None of the sibling registers needs a wavelength.\n\
                      /// The spectroscopy side does not reach in here.\n\
                      pub struct Claim {\n    pub value: f64, // not a wavelength\n}\n";
    assert!(
        examine("constructed.rs", documented).is_empty(),
        "a comment must not be read as code"
    );

    // The same word outside a comment still is refused, so the exemption is for
    // comments rather than for the word.
    let in_code = "pub struct Claim {\n    pub wavelength: f64,\n}\n";
    assert_eq!(examine("constructed.rs", in_code).len(), 1);
}

/// The generic side reaching into the domain side, which is the same failure
/// arriving through the type system instead of through a name.
#[test]
fn a_reference_to_the_other_side_is_refused() {
    // A path into the other side whose every segment is an ordinary word, so
    // that what is refused here is the crossing and nothing else.
    let reaches = "use crate::spectroscopy::accuracy::Grade;\n";
    assert_eq!(
        examine("constructed.rs", reaches)
            .iter()
            .map(|f| &f.reason)
            .collect::<Vec<_>>(),
        vec![&Reason::ReachesTheOtherSide],
        "a use of the other side must be refused for being a crossing"
    );

    // The neighbour, one path segment away: the same shape of import, into the
    // side it is already on.
    let stays_here = "use crate::register::provenance::SourceId;\n";
    assert!(examine("constructed.rs", stays_here).is_empty());

    // A path that is both, which most of them are, is both findings rather than
    // one standing in for the other. Two for `species`, because the module and
    // the type are two words on the line.
    let both = "use crate::spectroscopy::species::Species;\n";
    assert_eq!(
        examine("constructed.rs", both)
            .iter()
            .map(|f| &f.reason)
            .collect::<Vec<_>>(),
        vec![
            &Reason::ReachesTheOtherSide,
            &Reason::NamesASpectroscopicQuantity("species".to_owned()),
            &Reason::NamesASpectroscopicQuantity("species".to_owned()),
        ]
    );
}

/// The tree itself, which is what all of the above exists for.
#[test]
fn the_generic_side_names_nothing_specific_to_spectroscopy() {
    let root = generic_side();
    let files = rust_files_below(&root);
    assert!(
        !files.is_empty(),
        "{} holds no source, so this test would pass over nothing",
        root.display()
    );

    let mut findings = Vec::new();
    for path in &files {
        let name = path
            .strip_prefix(manifest_dir())
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        findings.extend(examine(&name, &text));
    }
    findings.sort();
    assert!(
        findings.is_empty(),
        "the generic side has crossed the boundary: {findings:?}"
    );

    println!(
        "layout: examined {} file(s) under src/register/ against {} listed quantities",
        files.len(),
        QUANTITIES.len()
    );
}
