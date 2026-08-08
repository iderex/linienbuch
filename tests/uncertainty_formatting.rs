//! No second path turns an uncertainty into text.
//!
//! `src/register/rounding.rs` rounds an uncertainty away from zero. That is
//! worth what it is worth only while it is the only route, because the request
//! that softens it does not arrive as an argument about honesty. It arrives as
//! one line somewhere else that formats a number to two places because two
//! places looked tidy, and by then the rule is a paragraph rather than a
//! property.
//!
//! So the rule is searched for rather than remembered. A line under `src/` that
//! formats something and names an uncertainty, in a file that is not the shared
//! rule, is refused.
//!
//! This is the second half of #40's done condition and one of the invariants
//! #50 names. It is held here, beside its fixtures, rather than inside a check
//! that would read the same tree a second time. #50's own thread already
//! reaches that conclusion for the invariant `tests/fixture_policy.rs` holds:
//! the row it should carry is a pointer at the test, not a reimplementation of
//! it.
//!
//! Three bounds, and none of them is softened.
//!
//! The vocabulary is a floor. An uncertainty reached through a name this file
//! does not hold passes, and the entry is added when the name arrives. What the
//! list buys is that the names somebody would actually reach for are refused.
//!
//! It reads words rather than parsing Rust, so a formatting call split across
//! two lines is not seen, and neither is a number turned into text by
//! arithmetic on its digits.
//!
//! A line that names the shared rule is not a path outside it, so it is not
//! refused. That exemption is by name, and a line that names the rule while
//! doing something else as well walks through this search.

use std::fs;
use std::path::{Path, PathBuf};

/// The file that is allowed to format an uncertainty, relative to the root.
const THE_SHARED_RULE: &str = "src/register/rounding.rs";

/// The entry point of the shared rule. A line that names it has gone through it.
const THE_RULE_IS_NAMED: &str = "render";

/// The ways text is made in this language.
///
/// Spelled with their punctuation, so `format!` is found and a function called
/// `format` is not.
const FORMATTING: [&str; 9] = [
    "format!",
    "write!",
    "writeln!",
    "print!",
    "println!",
    "eprint!",
    "eprintln!",
    "format_args!",
    "to_string()",
];

/// Words that name an uncertainty or one of its halves.
///
/// `sigma` is here although nothing in the tree spells it that way yet, because
/// it is what somebody writing a formatting path in a hurry would reach for.
/// Ordinary words that happen to appear near numbers are deliberately absent:
/// `tolerance` is a matching width in the level comparison and has nothing to do
/// with this rule, and a list holding it would refuse correct code.
const AN_UNCERTAINTY: [&str; 6] = [
    "uncertainty",
    "uncertainties",
    "sigma",
    "widest",
    "minus",
    "plus",
];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    file: String,
    line: usize,
    /// The formatting call that was found.
    formatting: String,
    /// The word that made it an uncertainty.
    names: String,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A line with its comment removed.
///
/// String literals are kept, because an inline format argument is spelled
/// inside one and `format!("{uncertainty}")` is exactly the path this refuses.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// The words of a line, lowercased, split at everything that cannot be part of
/// an identifier.
fn words(code: &str) -> Vec<String> {
    code.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// What is wrong in one file that is not the shared rule.
///
/// A pure function over the text, so it can be shown to refuse and to accept on
/// constructed input rather than only on whatever the tree happens to hold.
fn examine(name: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code = code_of(line);
        if words(code).iter().any(|word| word == THE_RULE_IS_NAMED) {
            continue;
        }
        let Some(formatting) = FORMATTING.iter().find(|call| code.contains(**call)) else {
            continue;
        };
        let spoken = words(code);
        let Some(names) = spoken
            .iter()
            .find(|word| AN_UNCERTAINTY.contains(&word.as_str()))
        else {
            continue;
        };
        findings.push(Finding {
            file: name.to_owned(),
            line: index + 1,
            formatting: (*formatting).to_owned(),
            names: names.clone(),
        });
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
fn a_formatting_path_of_its_own_is_refused() {
    let rolls_its_own = "    let text = format!(\"{:.2}\", claim.uncertainty.widest().unwrap());\n";
    assert_eq!(
        examine("constructed.rs", rolls_its_own),
        vec![Finding {
            file: "constructed.rs".to_owned(),
            line: 1,
            formatting: "format!".to_owned(),
            names: "uncertainty".to_owned(),
        }]
    );

    // The neighbour, one call away and nothing else: the same line, going
    // through the shared rule.
    let goes_through_it = "    let text = render(claim.value, claim.uncertainty)?.to_string();\n";
    assert!(
        examine("constructed.rs", goes_through_it).is_empty(),
        "a line that names the shared rule must not be refused"
    );
}

/// An inline format argument is inside a string literal, which is why the
/// string is not stripped.
#[test]
fn an_uncertainty_named_inside_the_format_string_is_refused() {
    let inline = "    println!(\"the uncertainty is {uncertainty}\");\n";
    assert_eq!(examine("constructed.rs", inline).len(), 1);

    // The neighbour: the same call, formatting something this rule is not about.
    let a_plain_number = "    println!(\"the value is {value}\");\n";
    assert!(examine("constructed.rs", a_plain_number).is_empty());
}

/// A half of an uncertainty is an uncertainty.
#[test]
fn formatting_one_half_is_refused_like_formatting_both() {
    for half in [
        "    write!(f, \"{:.3}\", plus)?;\n",
        "    let text = minus.to_string();\n",
    ] {
        assert_eq!(
            examine("constructed.rs", half).len(),
            1,
            "a half must be refused: {half:?}"
        );
    }

    // The neighbour, one identifier away: arithmetic on the same halves, which
    // is not a formatting path and is not this rule's subject.
    let arithmetic = "    let widest = minus.max(plus);\n";
    assert!(examine("constructed.rs", arithmetic).is_empty());
}

/// Prose about the rule is not a violation of it.
///
/// A file explaining why an uncertainty must not be formatted here has to name
/// both halves of that sentence, and a search that refused the explanation would
/// make the rule undocumentable where a reader looks for it.
#[test]
fn a_comment_about_formatting_an_uncertainty_is_not_one() {
    let documented = "//! Never format an uncertainty with println! here.\n\
                      /// The uncertainty is rendered elsewhere, by format!.\n\
                      pub fn describe() {} // format! the uncertainty\n";
    assert!(
        examine("constructed.rs", documented).is_empty(),
        "a comment must not be read as code"
    );

    // The same words outside a comment still are refused, so the exemption is
    // for comments rather than for the words.
    let in_code = "    println!(\"{}\", uncertainty);\n";
    assert_eq!(examine("constructed.rs", in_code).len(), 1);
}

/// The exemption points at a file that exists.
///
/// An exemption naming a path the tree does not carry excuses nothing and reads
/// as though it excused something.
#[test]
fn the_shared_rule_is_where_the_exemption_says_it_is() {
    let path = manifest_dir().join(THE_SHARED_RULE);
    assert!(
        path.is_file(),
        "the exempted file {THE_SHARED_RULE} is not in the tree"
    );

    // And it is exempted for a reason: it is a formatting path itself, so this
    // search over it would refuse it.
    let text = fs::read_to_string(&path).expect("the shared rule is readable");
    assert!(
        !examine(THE_SHARED_RULE, &text).is_empty(),
        "the shared rule no longer formats an uncertainty, so the exemption is stale"
    );
}

/// The tree itself, which is what all of the above exists for.
#[test]
fn nothing_outside_the_shared_rule_formats_an_uncertainty() {
    let root = manifest_dir().join("src");
    let files = rust_files_below(&root);
    assert!(
        !files.is_empty(),
        "{} holds no source, so this test would pass over nothing",
        root.display()
    );

    let mut findings = Vec::new();
    let mut examined = 0usize;
    for path in &files {
        let name = path
            .strip_prefix(manifest_dir())
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if name == THE_SHARED_RULE {
            continue;
        }
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        findings.extend(examine(&name, &text));
        examined += 1;
    }
    findings.sort();
    assert!(
        findings.is_empty(),
        "an uncertainty is formatted outside {THE_SHARED_RULE}: {findings:?}"
    );

    println!(
        "uncertainty-formatting: examined {examined} file(s) under src/ against \
         {} formatting call(s) and {} name(s)",
        FORMATTING.len(),
        AN_UNCERTAINTY.len()
    );
}
