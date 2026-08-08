//! Reading `docs/parity.md` as data.
//!
//! Two targets need the same reading and there is one implementation of it. The
//! default suite checks that the map is internally consistent, and the
//! integration harness compares it against the target's live required set. A
//! second copy of this parser would drift, and the drift would be discovered
//! when the two disagreed about whether a check is placed.
//!
//! Cargo does not build this file as a test target of its own. It is neither
//! `tests/*.rs` nor `tests/*/main.rs`, so it exists only where it is included.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// The headings a row may sit under.
///
/// Declared here rather than discovered, because a heading renamed in the
/// document would otherwise take its rows out of every comparison silently. A
/// test asserts each of these appears in the document exactly once, so the
/// rename is a red suite rather than a quiet loss.
pub const CATEGORIES: [&str; 7] = [
    "Adopted unchanged",
    "Adopted under the same name with a different implementation",
    "Adopted with an adaptation",
    "Adopted but resolved separately",
    "Dropped",
    "Added beyond the target",
    "Carried as non gating",
];

pub fn parity_document() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("parity.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The document split at its second-level headings, in the order it writes them.
///
/// Everything before the first such heading belongs to no section and is
/// dropped, which is the title and the sentence under it.
pub fn sections(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        match line.strip_prefix("## ") {
            Some(title) => found.push((title.trim().to_owned(), String::new())),
            None => {
                if let Some((_, body)) = found.last_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    found
}

/// The check run names a section places.
///
/// A row is a paragraph that opens with a check run name in backticks at the
/// start of a line, followed immediately by a full stop and the one line of
/// reasoning. Prose that mentions a name mid-sentence is not a row, and a
/// backticked path or command at the start of a line is not one either, because
/// neither is followed by a full stop against the closing backtick.
pub fn row_names(body: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in body.lines() {
        let Some(rest) = line.strip_prefix('`') else {
            continue;
        };
        let Some(end) = rest.find('`') else {
            continue;
        };
        if !rest[end + 1..].starts_with('.') {
            continue;
        }
        names.push(rest[..end].to_owned());
    }
    names
}

/// Every check run name the map places, and the headings each one sits under.
///
/// A name under two headings keeps both, because reporting the first and
/// dropping the second would hide exactly the case a reader needs to see.
pub fn placed(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (title, body) in sections(text) {
        if !CATEGORIES.contains(&title.as_str()) {
            continue;
        }
        for name in row_names(&body) {
            found.entry(name).or_default().push(title.clone());
        }
    }
    found
}

/// The names in `required` that the map does not place, in the order they were
/// given.
///
/// One direction only. A row naming a check the target does not require is not
/// reported, because the dropped rows and the non gating rows are exactly that
/// and the map is not wrong for carrying them.
pub fn unplaced(text: &str, required: &[String]) -> Vec<String> {
    let placed = placed(text);
    required
        .iter()
        .filter(|name| !placed.contains_key(*name))
        .cloned()
        .collect()
}
