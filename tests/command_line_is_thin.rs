//! The command line declares nothing of its own.
//!
//! #45 argues the order rather than the feature: a library that arrives after
//! the command line is shaped by whatever the command line happened to need,
//! and the command line grows flags that exist because there was no other way
//! in. The repair is cheap while `src/main.rs` is empty and expensive once an
//! operation is in it, so the guard is written while its subject is still small
//! enough that nobody has to move anything to satisfy it.
//!
//! What it refuses is one thing. `src/main.rs` may declare `fn main` and
//! nothing else. A helper function, a type, a trait, a module or a constant
//! beside it is refused, whether it sits at the top level or inside `main`.
//! That is the shape logic arrives in: not as a single large `main`, but as a
//! small helper somebody extracts because it was getting long, and then a second
//! one beside it.
//!
//! The bound is stated rather than left to be met. It reads words with comments
//! removed, in the way `tests/layout.rs` does, so it does not parse Rust. Two
//! consequences follow and neither is softened. Logic written inline in `main`
//! is not seen at all, so a green run here is not a statement that the command
//! line is thin, only that it declares nothing. And an item produced by a macro
//! expansion is not seen either, because the expansion is not in the text.
//!
//! So this covers the first clause of #45 and not the second. Whether every
//! command line operation has a library entry point cannot be checked while
//! there are no operations, and #45 stays open for it.
//!
//! The subject is `src/main.rs` and not the binaries under `src/bin/`.
//! `docs/decisions/layout.md` puts those on neither side: they are development
//! tools that run the checks and they are not the program an operator runs.
//! Holding them to this rule would refuse a check for being a check.

use std::fs;
use std::path::PathBuf;

/// The words that declare something in Rust.
///
/// A floor rather than a complete grammar. Each one is a way to put a named
/// thing in a file, and the list holds the ones somebody writing a command line
/// would reach for. A declaration spelled in a keyword this list does not carry
/// passes, and the entry is added when that spelling arrives.
const DECLARES: [&str; 10] = [
    "fn", "struct", "enum", "trait", "impl", "mod", "const", "static", "type", "union",
];

/// The one declaration the file is for.
const MAIN: &str = "main";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    line: usize,
    reason: Reason,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reason {
    /// Something declared in the command line that is not `fn main`, named by
    /// the word that declared it.
    DeclaresSomethingBesideMain(String),
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A line with its comment removed, so that prose explaining what may not be
/// declared here is not read as a declaration.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// The words of a line of code, split at everything that cannot be part of an
/// identifier.
///
/// `function` does not yield `fn` and `constant` does not yield `const`,
/// because the split is at the boundaries of a word rather than inside one.
fn words(code: &str) -> Vec<&str> {
    code.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|word| !word.is_empty())
        .collect()
}

/// What one command line file declares that it may not.
///
/// A pure function over the text, so the refusal can be shown on constructed
/// input rather than only on whatever `src/main.rs` happens to hold today.
///
/// `fn main` is allowed once. A second `fn main` cannot compile, so nothing is
/// gained by counting it, but the first is consumed rather than matched by name
/// on every line: a file that declared `fn main` twice would have its second
/// one refused here rather than passing on the strength of the first.
fn examine(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut main_seen = false;
    for (index, line) in text.lines().enumerate() {
        let found = words(code_of(line));
        for (at, word) in found.iter().enumerate() {
            if !DECLARES.contains(word) {
                continue;
            }
            let is_main = *word == "fn" && found.get(at + 1) == Some(&MAIN) && !main_seen;
            if is_main {
                main_seen = true;
                continue;
            }
            findings.push(Finding {
                line: index + 1,
                reason: Reason::DeclaresSomethingBesideMain((*word).to_owned()),
            });
        }
    }
    findings
}

/// Whether the text declares `fn main` at all.
///
/// Read separately from the findings so that a file with nothing in it cannot
/// pass this guard by declaring nothing.
fn declares_main(text: &str) -> bool {
    text.lines().any(|line| {
        let found = words(code_of(line));
        found
            .iter()
            .enumerate()
            .any(|(at, word)| *word == "fn" && found.get(at + 1) == Some(&MAIN))
    })
}

/// The refusal, on constructed input, with its neighbour beside it.
///
/// The neighbour is the same code with the helper moved into the library, which
/// is the repair #45 asks for rather than an obviously different file. The two
/// differ by where one function lives and by nothing else, so what is shown is
/// that the guard reads the position and not the arithmetic.
#[test]
fn a_helper_beside_main_is_refused() {
    let extracted = "fn main() {\n\
                     \x20   let value = widest(claim);\n\
                     \x20   println!(\"{value}\");\n\
                     }\n\
                     \n\
                     fn widest(claim: Claim) -> f64 {\n\
                     \x20   claim.plus.max(claim.minus)\n\
                     }\n";
    assert_eq!(
        examine(extracted),
        vec![Finding {
            line: 6,
            reason: Reason::DeclaresSomethingBesideMain("fn".to_owned()),
        }]
    );

    let in_the_library = "fn main() {\n\
                          \x20   let value = linienbuch::widest(claim);\n\
                          \x20   println!(\"{value}\");\n\
                          }\n";
    assert!(
        examine(in_the_library).is_empty(),
        "the neighbour must not be refused"
    );
}

/// A helper nested inside `main` is the same failure one indent further in.
///
/// Anchoring at the start of a line would have missed it, and it is the
/// spelling somebody reaches for when they know a file is being watched.
#[test]
fn a_helper_nested_inside_main_is_refused() {
    let nested = "fn main() {\n\
                  \x20   fn widest(a: f64, b: f64) -> f64 {\n\
                  \x20       a.max(b)\n\
                  \x20   }\n\
                  \x20   println!(\"{}\", widest(1.0, 2.0));\n\
                  }\n";
    assert_eq!(
        examine(nested),
        vec![Finding {
            line: 2,
            reason: Reason::DeclaresSomethingBesideMain("fn".to_owned()),
        }]
    );
}

/// Every word in the list refuses, so a green run is not the list being empty
/// in effect.
#[test]
fn each_declaring_word_is_refused() {
    for word in DECLARES {
        let text = format!("fn main() {{}}\n{word} Thing;\n");
        assert_eq!(
            examine(&text),
            vec![Finding {
                line: 2,
                reason: Reason::DeclaresSomethingBesideMain(word.to_owned()),
            }],
            "{word} was not refused"
        );
    }
}

/// What a thin command line is made of is not refused.
///
/// Imports, attributes, argument handling and formatting all have to pass, or
/// the guard refuses the file it is asking for.
#[test]
fn the_shape_this_guard_asks_for_is_not_refused() {
    let thin = "use std::env;\n\
                use std::process::ExitCode;\n\
                \n\
                fn main() -> ExitCode {\n\
                \x20   let asked: Vec<String> = env::args().skip(1).collect();\n\
                \x20   match linienbuch::answer(&asked) {\n\
                \x20       Ok(said) => {\n\
                \x20           println!(\"{said}\");\n\
                \x20           ExitCode::SUCCESS\n\
                \x20       }\n\
                \x20       Err(why) => {\n\
                \x20           eprintln!(\"{why}\");\n\
                \x20           ExitCode::FAILURE\n\
                \x20       }\n\
                \x20   }\n\
                }\n";
    assert!(
        examine(thin).is_empty(),
        "a command line that only reads arguments and formats must pass"
    );
}

/// A word that merely contains a declaring word is not one.
#[test]
fn a_word_that_only_contains_a_declaring_word_is_not_refused() {
    for ordinary in [
        "    let constant = 1;\n",
        "    let function = one;\n",
        "    let typed = text.parse();\n",
        "    let modulus = a % b;\n",
    ] {
        assert!(
            examine(ordinary).is_empty(),
            "refused an ordinary word: {ordinary:?}"
        );
    }

    // And the words themselves still are, so the test above is not passing by
    // the comparison never matching.
    assert_eq!(examine("    const A: u8 = 1;\n").len(), 1);
    assert_eq!(examine("    type A = u8;\n").len(), 1);
}

/// Prose saying what may not be declared here is not a declaration.
///
/// The module comment on `src/main.rs` has to be able to say what the file is
/// for, and a guard that refused that would make the rule undocumentable in the
/// one place a reader looks for it.
#[test]
fn prose_about_the_rule_is_not_a_declaration() {
    let documented = "//! No fn, struct or trait belongs beside main here.\n\
                      /// What this const would have held lives in the library.\n\
                      fn main() {} // no impl, no mod\n";
    assert!(
        examine(documented).is_empty(),
        "a comment must not be read as code"
    );

    // The same word outside a comment still is, so the exemption is for
    // comments rather than for the word.
    assert_eq!(examine("fn main() {}\nstruct Thing;\n").len(), 1);
}

/// A file with no `fn main` cannot satisfy this guard by declaring nothing.
#[test]
fn a_file_without_main_is_not_a_command_line() {
    assert!(examine("").is_empty());
    assert!(!declares_main(""), "an empty file declares no main");
    assert!(declares_main("fn main() {}\n"));
    assert!(
        !declares_main("// fn main() {}\n"),
        "a commented out main is not a declaration"
    );
}

/// The tree itself, which is what all of the above exists for.
#[test]
fn the_command_line_declares_nothing_beside_main() {
    let path = manifest_dir().join("src").join("main.rs");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    assert!(
        declares_main(&text),
        "{} declares no main, so this test would pass over nothing",
        path.display()
    );

    let findings = examine(&text);
    assert!(
        findings.is_empty(),
        "src/main.rs declares something beside main: {findings:?}"
    );

    println!(
        "command line: examined src/main.rs, {} line(s), against {} declaring word(s); \
         logic written inline in main is not read by this check",
        text.lines().count(),
        DECLARES.len()
    );
}
