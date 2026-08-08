//! The half of the parity map that can be checked without leaving this machine.
//!
//! It holds `docs/parity.md` internally consistent: every category the reader is
//! promised exists, every row sits under exactly one of them, and the rows sit
//! where rows belong. It says nothing about what the target requires today,
//! because finding that out needs the network and the default suite has none.
//! That comparison is the integration harness's leg, and `docs/parity.md` says
//! so where it says what refuses a row going missing.

#[path = "parity/map.rs"]
mod map;

use map::{CATEGORIES, parity_document, placed, row_names, sections, unplaced};

/// The categories whose rows name a check run of the target gate.
///
/// The other three describe checks that have no target check run name to place:
/// what was dropped, what this board adds, and what neither board gates on. A
/// backticked name at the start of a line in one of those is a row in the wrong
/// section, which is what the test below refuses.
const NAME_BEARING: [&str; 4] = [
    "Adopted unchanged",
    "Adopted under the same name with a different implementation",
    "Adopted with an adaptation",
    "Adopted but resolved separately",
];

/// Every category the parser looks under is a heading the document writes, and
/// writes once.
///
/// A heading renamed on one side only would take its rows out of every
/// comparison here and out of the integration leg, and nothing would say so.
#[test]
fn every_category_is_a_heading_the_document_writes_once() {
    let text = parity_document();
    let headings: Vec<String> = sections(&text)
        .into_iter()
        .map(|(title, _)| title)
        .collect();

    for category in CATEGORIES {
        let count = headings.iter().filter(|title| *title == category).count();
        assert_eq!(
            count, 1,
            "docs/parity.md writes the heading {category:?} {count} times, not once"
        );
    }

    for name in NAME_BEARING {
        assert!(
            CATEGORIES.contains(&name),
            "{name:?} is not one of the categories the parser reads"
        );
    }
}

/// No check run name is placed twice.
///
/// Two placements are two different answers to the question the map exists to
/// answer, and the reader has no way to tell which one is meant.
#[test]
fn no_check_is_placed_under_two_headings() {
    let text = parity_document();
    let twice: Vec<(String, Vec<String>)> = placed(&text)
        .into_iter()
        .filter(|(_, headings)| headings.len() > 1)
        .collect();
    assert!(
        twice.is_empty(),
        "placed under more than one heading: {twice:?}"
    );
}

/// Rows sit under a category that names check runs, and the other three carry
/// none.
///
/// A row that slid into the dropped section would still be placed, so the
/// integration leg would pass, and the map would be saying that a check the
/// target requires has no analogue here.
#[test]
fn only_the_name_bearing_categories_carry_rows() {
    let text = parity_document();
    for (title, body) in sections(&text) {
        if !CATEGORIES.contains(&title.as_str()) {
            continue;
        }
        let names = row_names(&body);
        if NAME_BEARING.contains(&title.as_str()) {
            assert!(
                !names.is_empty(),
                "{title:?} names no check run, so nothing is placed under it"
            );
        } else {
            assert!(
                names.is_empty(),
                "{title:?} names check runs {names:?}, and a check run name does not belong there"
            );
        }
    }
}

/// The document gives the command for each required set rather than the set.
#[test]
fn the_document_gives_the_command_for_each_required_set() {
    let text = parity_document();
    for repository in ["iderex/jellyfin-plugin-sso", "iderex/linienbuch"] {
        let command = format!("gh api repos/{repository}/rules/branches/main");
        assert!(
            text.contains(&command),
            "docs/parity.md does not give the command that prints {repository}'s required set"
        );
    }
}

/// The refusal, on constructed input, with its neighbour beside it.
///
/// The near miss is the one that matters here, because the forge matches a
/// required check by literal name: a map that places `Build` has placed nothing
/// that `build` will ever match, and every other word of the row is right.
#[test]
fn a_required_check_the_map_does_not_place_is_reported() {
    let required = ["build".to_owned(), "prettier".to_owned()];

    let complete = "\
## Adopted under the same name with a different implementation

`build`. The purpose does not depend on the language.

## Adopted with an adaptation

`prettier`. The target's covers web assets and this board has none.
";
    assert!(
        unplaced(complete, &required).is_empty(),
        "a map placing both must report nothing"
    );

    let missing = "\
## Adopted under the same name with a different implementation

`build`. The purpose does not depend on the language.
";
    assert_eq!(
        unplaced(missing, &required),
        vec!["prettier".to_owned()],
        "a required check the map does not place must be reported"
    );

    let wrong_case = complete.replace("`build`", "`Build`");
    assert_eq!(
        unplaced(&wrong_case, &required),
        vec!["build".to_owned()],
        "a row one character away from the required name must be reported"
    );
}

/// A row is read where a row is written, and prose is not a row.
///
/// The parser reads lines, so the shapes it must not mistake for a row are the
/// ones this document actually contains: a sentence naming a check in passing,
/// and a backticked path opening a paragraph.
#[test]
fn prose_naming_a_check_is_not_a_row() {
    assert_eq!(row_names("`build`. Reason.\n"), vec!["build".to_owned()]);

    assert!(
        row_names("The target requires `build`. This board does not.\n").is_empty(),
        "a name inside a sentence is not a row"
    );
    assert!(
        row_names("`tests/parity_map.rs` reads this file and refuses a name.\n").is_empty(),
        "a backticked path opening a paragraph is not a row"
    );
    assert!(
        row_names("`build` is refusing a tree that does not compile.\n").is_empty(),
        "a name opening a sentence without a full stop after it is not a row"
    );
}

/// A section carries the lines under its own heading and nothing else.
#[test]
fn a_section_ends_where_the_next_heading_begins() {
    let text = "# Title\n\nignored\n\n## One\n\nfirst\n\n## Two\n\nsecond\n";
    let found = sections(text);
    assert_eq!(
        found,
        vec![
            ("One".to_owned(), "\nfirst\n\n".to_owned()),
            ("Two".to_owned(), "\nsecond\n".to_owned()),
        ]
    );
}
