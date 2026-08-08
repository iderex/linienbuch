//! The hygiene checks, which reason about the change rather than about the code.
//!
//! Every other check in this tree reads the tree. This one reads what arrived:
//! whether the change is linked to an issue, whether its messages are written in
//! characters that survive being pasted somewhere else, and whether a version
//! bump brought its changelog entry. No other check covers that class, and the
//! failures it catches are the ones that are invisible in a diff.
//!
//! Two tiers, and the split is the reason a hygiene check is trusted rather than
//! argued with. The failing tier holds rules whose answer is unambiguous, so a
//! red one is a fact rather than an opinion. The warning tier holds the rules
//! that are useful and sometimes wrong, and it never contributes to the exit
//! code, because a check that reds on a legitimate bulk rename is a check people
//! route around instead of obeying.
//!
//! Almost all of it reads git and nothing else, so it answers the same way on a
//! machine and on a runner. Exactly one rule needs something git does not hold,
//! which is the pull request body, and that rule says at itself that it was not
//! reached rather than passing quietly. Every rule that cannot be reached prints
//! why, at the rule, so a run that judged four of six can never be read as a run
//! that judged six.
//!
//! It is a leg of the gate rather than a workflow with commands of its own.
//! `.github/workflows/hygiene.yml` invokes the gate and asks for this leg, the
//! same as every other workflow here, so there is one implementation and not
//! two.

use std::env;
use std::process::{Command, ExitCode};

/// Which tier a rule reports in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tier {
    /// Refuses the change. Only rules whose answer is unambiguous belong here.
    Failing,
    /// Annotates and never refuses, whatever it finds.
    Warning,
}

/// What a rule decided about the change in front of it.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Verdict {
    /// The rule was reached and nothing broke it.
    Held,
    /// The rule was reached and these things broke it, one line each.
    Broken(Vec<String>),
    /// The rule was not reached, and this is why. Never a pass.
    Skipped(String),
}

/// One rule's answer, with the tier that decides what the answer costs.
struct Reported {
    tier: Tier,
    rule: &'static str,
    verdict: Verdict,
}

const BODY_NAMES_AN_ISSUE: &str = "the pull request body names an issue";
const EVERY_COMMIT_NAMES_AN_ISSUE: &str = "every commit message names an issue";
const MESSAGES_STAY_INSIDE_THE_CHARACTER_SET: &str =
    "every commit message stays inside the declared character set";
const A_VERSION_BUMP_BRINGS_ITS_CHANGELOG: &str = "a version bump arrives with a changelog entry";
const THE_CHANGE_FITS_ONE_SITTING: &str = "the change is small enough to read in one sitting";
const SOURCE_ARRIVES_WITH_A_TEST: &str = "source changed with a test changed beside it";

/// Every rule and the tier it reports in.
///
/// The list is here so that a rule cannot be added without deciding which tier
/// it is in, and so that a test can read the split rather than being told it.
const RULES: [(Tier, &str); 6] = [
    (Tier::Failing, BODY_NAMES_AN_ISSUE),
    (Tier::Failing, EVERY_COMMIT_NAMES_AN_ISSUE),
    (Tier::Failing, MESSAGES_STAY_INSIDE_THE_CHARACTER_SET),
    (Tier::Failing, A_VERSION_BUMP_BRINGS_ITS_CHANGELOG),
    (Tier::Warning, THE_CHANGE_FITS_ONE_SITTING),
    (Tier::Warning, SOURCE_ARRIVES_WITH_A_TEST),
];

/// The number of changed lines above which the warning tier annotates.
///
/// Chosen rather than measured, and it is in the warning tier for exactly that
/// reason. A bulk rename or a data table legitimately passes it, so a gate that
/// reds here would be a gate people bypass, and a bypassed gate protects
/// nothing at all.
const A_LARGE_CHANGE: usize = 400;

/// One commit, as this check reads one.
struct Commit {
    id: String,
    message: String,
}

impl Commit {
    /// The first line, for naming the commit in a refusal.
    fn subject(&self) -> &str {
        self.message.lines().next().unwrap_or("").trim_end()
    }
}

/// Whether a text carries a reference to an issue on this board.
///
/// A hash followed by at least one digit. That is a floor and it is written
/// down as one: a message quoting a colour, a column number or anything else of
/// that shape passes, so what this refuses is a change that names no issue
/// anywhere rather than a change naming a number that is not an issue. Reading
/// the tracker to tell the two apart would put the forge in the path of a rule
/// whose whole value is that it answers identically on a runner and on a
/// machine.
fn names_an_issue(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.iter().enumerate().any(|(at, byte)| {
        *byte == b'#' && bytes.get(at + 1).is_some_and(|next| next.is_ascii_digit())
    })
}

fn verdict(broken: Vec<String>) -> Verdict {
    if broken.is_empty() {
        Verdict::Held
    } else {
        Verdict::Broken(broken)
    }
}

/// Every commit in the range names an issue somewhere in its message.
///
/// The message rather than the subject, and that is a departure from the words
/// this rule was asked for in. It is measured rather than preferred. No commit
/// on this board carries a reference in its subject and almost every one
/// carries it in a trailer, so the subject reading would refuse the convention
/// the repository already writes in, including the commits that landed the
/// document stating the convention. The evidence is in the pull request that
/// brought this file. What the rule is for, that a change is linked to an
/// issue, is unchanged by which line of the message carries the link.
fn commits_naming_no_issue(commits: &[Commit]) -> Verdict {
    if commits.is_empty() {
        return Verdict::Skipped("the range holds no commit for this rule to read".to_owned());
    }
    verdict(
        commits
            .iter()
            .filter(|commit| !names_an_issue(&commit.message))
            .map(|commit| format!("{} names no issue: {}", commit.id, commit.subject()))
            .collect(),
    )
}

/// The characters a commit message on this board is written in.
///
/// Printable ASCII, plus the tab and the line endings a message is laid out
/// with. The carriage return is inside the set deliberately: it is a line ending
/// some platforms produce, and refusing it would aim this rule at an artefact of
/// where a commit was made instead of at what it exists for.
///
/// What it exists for is the subject matter. Greek letters, superscripts and
/// angstrom signs arrive by copy and paste from a source and then break tooling
/// somewhere downstream that nobody is looking at. Spelling the character out is
/// one edit; finding out months later which tool dropped it is not.
fn outside_the_character_set(text: &str) -> Vec<char> {
    text.chars()
        .filter(|c| !matches!(c, ' '..='~' | '\n' | '\t' | '\r'))
        .collect()
}

fn commits_outside_the_character_set(commits: &[Commit]) -> Verdict {
    if commits.is_empty() {
        return Verdict::Skipped("the range holds no commit for this rule to read".to_owned());
    }
    verdict(
        commits
            .iter()
            .filter_map(|commit| {
                let found = outside_the_character_set(&commit.message);
                if found.is_empty() {
                    return None;
                }
                let named: Vec<String> = found
                    .iter()
                    .map(|c| format!("U+{:04X}", u32::from(*c)))
                    .collect();
                Some(format!(
                    "{} carries {}: {}",
                    commit.id,
                    named.join(", "),
                    commit.subject()
                ))
            })
            .collect(),
    )
}

/// The changelog this board keeps, named once so the rule and its message
/// cannot disagree about which file they mean.
const CHANGELOG: &str = "CHANGELOG.md";

/// A version bump arrives with a changelog entry.
///
/// A bump is a removed `version = ` line and an added one that differ, read out
/// of the manifest's own diff. A manifest arriving for the first time carries
/// only the added line and is not a bump, which is why both are required.
fn version_bump_without_a_changelog_entry(manifest_diff: &str, changed: &[String]) -> Verdict {
    let value = |line: &str, marker: char| -> Option<String> {
        line.strip_prefix(marker)
            .and_then(|rest| rest.strip_prefix("version = "))
            .map(str::to_owned)
    };
    let removed: Vec<String> = manifest_diff
        .lines()
        .filter_map(|line| value(line, '-'))
        .collect();
    let added: Vec<String> = manifest_diff
        .lines()
        .filter_map(|line| value(line, '+'))
        .collect();

    let bumped = !removed.is_empty() && !added.is_empty() && removed != added;
    if !bumped {
        return Verdict::Held;
    }
    if changed.iter().any(|path| path == CHANGELOG) {
        return Verdict::Held;
    }
    Verdict::Broken(vec![format!(
        "the package version moved from {} to {} and {CHANGELOG} is not in the change",
        removed.join(", "),
        added.join(", ")
    )])
}

/// The pull request body names an issue.
///
/// The one rule here that git cannot answer. The workflow supplies the body in
/// an environment variable rather than interpolating it into a command, so a
/// body is data to this program and never something the shell reads. A local run
/// has no body and says so at the rule.
fn pull_request_body_naming_no_issue(body: Option<&str>) -> Verdict {
    match body {
        None => Verdict::Skipped(
            "a pull request body is not something this machine holds; the workflow supplies \
             it in HYGIENE_PR_BODY and a local run has none"
                .to_owned(),
        ),
        Some(body) if names_an_issue(body) => Verdict::Held,
        Some(_) => Verdict::Broken(vec![
            "the pull request body names no issue, so nothing says what this change is for"
                .to_owned(),
        ]),
    }
}

/// The change is small enough to read in one sitting.
fn a_large_change(changed_lines: usize) -> Verdict {
    if changed_lines > A_LARGE_CHANGE {
        Verdict::Broken(vec![format!(
            "{changed_lines} changed lines, above the {A_LARGE_CHANGE} this tier annotates at"
        )])
    } else {
        Verdict::Held
    }
}

/// Source changed with a test changed beside it.
///
/// Paths rather than contents, so a test added inside a source file's own test
/// module is not seen. That bound is why this is in the warning tier: the rule
/// is right often enough to be worth printing and wrong often enough that
/// refusing on it would teach people to ignore it.
fn source_without_a_test(changed: &[String]) -> Verdict {
    let touched = |prefix: &str| changed.iter().any(|path| path.starts_with(prefix));
    if touched("src/") && !touched("tests/") {
        Verdict::Broken(vec![
            "src/ changed and nothing under tests/ did; a test inside a source file's own \
             test module is not visible to this rule"
                .to_owned(),
        ])
    } else {
        Verdict::Held
    }
}

/// What refuses the change.
///
/// The warning tier is not consulted, and that is the whole of what makes it a
/// warning tier.
fn refused(reported: &[Reported]) -> Vec<&'static str> {
    reported
        .iter()
        .filter(|entry| entry.tier == Tier::Failing)
        .filter(|entry| matches!(entry.verdict, Verdict::Broken(_)))
        .map(|entry| entry.rule)
        .collect()
}

/// What was not reached, and so was not judged.
fn not_reached(reported: &[Reported]) -> Vec<&'static str> {
    reported
        .iter()
        .filter(|entry| matches!(entry.verdict, Verdict::Skipped(_)))
        .map(|entry| entry.rule)
        .collect()
}

fn manifest_dir() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

/// One git command, reading only.
fn git(args: &[&str]) -> Result<String, String> {
    let printed = format!("git {}", args.join(" "));
    let output = Command::new("git")
        .args(args)
        .current_dir(manifest_dir())
        .output()
        .map_err(|e| format!("{printed} could not start: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{printed} refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("{printed} printed no UTF-8: {e}"))
}

/// Where the change begins.
///
/// The workflow names the base of the pull request. A local run has no pull
/// request, so it asks git for the point this branch left the default branch. A
/// clone that holds neither reference has nothing to compare against, and that
/// is an absence rather than a pass.
fn base() -> Result<String, String> {
    if let Ok(named) = env::var("HYGIENE_BASE_SHA") {
        let named = named.trim().to_owned();
        if !named.is_empty() {
            return git(&["merge-base", named.as_str(), "HEAD"]).map(|s| s.trim().to_owned());
        }
    }
    for candidate in ["origin/main", "main"] {
        if let Ok(found) = git(&["merge-base", candidate, "HEAD"]) {
            return Ok(found.trim().to_owned());
        }
    }
    Err(
        "no base to compare against: HYGIENE_BASE_SHA is unset and this clone holds neither \
         origin/main nor main"
            .to_owned(),
    )
}

/// Every commit the change adds, merges excluded.
///
/// A merge commit's message is written by the forge rather than by whoever made
/// the change, so holding it to a rule about what an author wrote would refuse
/// something no author can fix.
fn commits(base: &str) -> Result<Vec<Commit>, String> {
    let range = format!("{base}..HEAD");
    let raw = git(&["log", "-z", "--no-merges", "--format=%H%x1f%B", &range])?;
    Ok(raw
        .split('\0')
        .filter(|entry| !entry.trim().is_empty())
        .filter_map(|entry| entry.split_once('\u{1f}'))
        .map(|(id, message)| Commit {
            id: id.trim().to_owned(),
            message: message.to_owned(),
        })
        .collect())
}

/// Every path the change touches.
fn changed_files(base: &str) -> Result<Vec<String>, String> {
    Ok(git(&["diff", "--name-only", "-z", base, "HEAD"])?
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

/// How many lines the change adds and removes.
///
/// A binary file reports a dash in both columns and contributes nothing here,
/// which understates the size of a change that is mostly binary. It is a
/// warning tier input, so the understatement costs an annotation rather than a
/// verdict.
fn changed_lines(base: &str) -> Result<usize, String> {
    let raw = git(&["diff", "--numstat", base, "HEAD"])?;
    let mut total = 0usize;
    for line in raw.lines() {
        for field in line.split('\t').take(2) {
            total += field.parse::<usize>().unwrap_or(0);
        }
    }
    Ok(total)
}

fn report(tier: Tier, rule: &'static str, verdict: Verdict) -> Reported {
    Reported {
        tier,
        rule,
        verdict,
    }
}

/// Every rule that needs the range, all skipped for one reason.
fn every_range_rule_skipped(why: &str) -> Vec<Reported> {
    RULES
        .iter()
        .filter(|(_, rule)| *rule != BODY_NAMES_AN_ISSUE)
        .map(|(tier, rule)| report(*tier, rule, Verdict::Skipped(why.to_owned())))
        .collect()
}

fn print(reported: &[Reported]) {
    for entry in reported {
        let tier = match entry.tier {
            Tier::Failing => "failing",
            Tier::Warning => "warning",
        };
        match &entry.verdict {
            Verdict::Held => println!("hygiene: [{tier}] {}: held", entry.rule),
            Verdict::Skipped(why) => {
                println!(
                    "hygiene: [{tier}] {}: not reached, so not judged",
                    entry.rule
                );
                println!("hygiene:   because {why}");
            }
            Verdict::Broken(lines) => {
                println!("hygiene: [{tier}] {}: refused", entry.rule);
                for line in lines {
                    println!("hygiene:   {line}");
                }
            }
        }
    }
}

fn main() -> ExitCode {
    let mut reported = vec![report(
        Tier::Failing,
        BODY_NAMES_AN_ISSUE,
        pull_request_body_naming_no_issue(env::var("HYGIENE_PR_BODY").ok().as_deref()),
    )];

    match base() {
        Err(why) => {
            println!("hygiene: no range to judge");
            reported.extend(every_range_rule_skipped(&why));
        }
        Ok(base) => match (commits(&base), changed_files(&base), changed_lines(&base)) {
            (Ok(commits), Ok(files), Ok(lines)) => {
                println!(
                    "hygiene: judging {base}..HEAD, {} commit(s) and {} file(s)",
                    commits.len(),
                    files.len()
                );
                let manifest =
                    git(&["diff", &base, "HEAD", "--", "Cargo.toml"]).unwrap_or_default();
                reported.push(report(
                    Tier::Failing,
                    EVERY_COMMIT_NAMES_AN_ISSUE,
                    commits_naming_no_issue(&commits),
                ));
                reported.push(report(
                    Tier::Failing,
                    MESSAGES_STAY_INSIDE_THE_CHARACTER_SET,
                    commits_outside_the_character_set(&commits),
                ));
                reported.push(report(
                    Tier::Failing,
                    A_VERSION_BUMP_BRINGS_ITS_CHANGELOG,
                    version_bump_without_a_changelog_entry(&manifest, &files),
                ));
                reported.push(report(
                    Tier::Warning,
                    THE_CHANGE_FITS_ONE_SITTING,
                    a_large_change(lines),
                ));
                reported.push(report(
                    Tier::Warning,
                    SOURCE_ARRIVES_WITH_A_TEST,
                    source_without_a_test(&files),
                ));
            }
            (commits, files, lines) => {
                let why = [commits.err(), files.err(), lines.err()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<String>>()
                    .join("; ");
                println!("hygiene: the range is there and could not be read");
                reported.extend(every_range_rule_skipped(&why));
            }
        },
    }

    print(&reported);

    let missed = not_reached(&reported);
    println!(
        "hygiene: judged {} of {} rules",
        reported.len() - missed.len(),
        RULES.len()
    );
    if !missed.is_empty() {
        println!("hygiene: not judged: {}", missed.join(", "));
    }

    let refusals = refused(&reported);
    if refusals.is_empty() {
        println!("hygiene: nothing in the failing tier refused this change");
        return ExitCode::SUCCESS;
    }
    println!("hygiene: refused by: {}", refusals.join(", "));
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::{
        A_LARGE_CHANGE, BODY_NAMES_AN_ISSUE, Commit, EVERY_COMMIT_NAMES_AN_ISSUE, RULES, Reported,
        Tier, Verdict, a_large_change, commits_naming_no_issue, commits_outside_the_character_set,
        every_range_rule_skipped, names_an_issue, not_reached, pull_request_body_naming_no_issue,
        refused, report, source_without_a_test, version_bump_without_a_changelog_entry,
    };

    fn commit(id: &str, message: &str) -> Commit {
        Commit {
            id: id.to_owned(),
            message: message.to_owned(),
        }
    }

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|p| (*p).to_owned()).collect()
    }

    /// What counts as naming an issue, and the near misses either side of it.
    #[test]
    fn an_issue_reference_is_a_hash_against_a_digit() {
        assert!(names_an_issue("Refs #49."));
        assert!(names_an_issue("Closes #7"));

        assert!(
            !names_an_issue("Refs # 49."),
            "a hash separated from its number names nothing"
        );
        assert!(
            !names_an_issue("the #[test] attribute"),
            "an attribute is not an issue reference"
        );
        assert!(!names_an_issue("Refs #"), "a hash at the end names nothing");
    }

    /// A commit naming no issue is refused, and the neighbour one trailer away
    /// is not.
    #[test]
    fn a_commit_naming_no_issue_is_refused() {
        let bare = "Parse the accuracy grade column\n\nSigned-off-by: A B <a@example.invalid>\n";
        let refused = commits_naming_no_issue(&[commit("1111111", bare)]);
        assert_eq!(
            refused,
            Verdict::Broken(vec![
                "1111111 names no issue: Parse the accuracy grade column".to_owned()
            ])
        );

        let linked = format!("{bare}\nRefs #27.\n");
        assert_eq!(
            commits_naming_no_issue(&[commit("1111111", &linked)]),
            Verdict::Held,
            "the same commit with a reference must not be refused"
        );
    }

    /// An empty range is not a pass.
    ///
    /// A branch that is level with its base has nothing for this rule to read,
    /// and reporting that as held would make a run over no commits look like a
    /// run over good ones.
    #[test]
    fn a_range_with_no_commits_is_not_reached_rather_than_held() {
        assert!(matches!(commits_naming_no_issue(&[]), Verdict::Skipped(_)));
        assert!(matches!(
            commits_outside_the_character_set(&[]),
            Verdict::Skipped(_)
        ));
    }

    /// The character the subject matter actually delivers, and the same subject
    /// with it spelled out.
    ///
    /// Written as an escape rather than as the character itself so that the
    /// bytes the rule is given are exact and cannot be normalised on the way
    /// into git by anything this repository does to tracked text.
    #[test]
    fn a_commit_message_outside_the_character_set_is_refused() {
        let pasted = "Record the \u{3bb} of the resonance line\n\nRefs #27.\n";
        assert_eq!(
            commits_outside_the_character_set(&[commit("2222222", pasted)]),
            Verdict::Broken(vec![
                "2222222 carries U+03BB: Record the \u{3bb} of the resonance line".to_owned()
            ])
        );

        let spelled = "Record the lambda of the resonance line\n\nRefs #27.\n";
        assert_eq!(
            commits_outside_the_character_set(&[commit("2222222", spelled)]),
            Verdict::Held,
            "the same subject with the character spelled out must not be refused"
        );

        assert_eq!(
            commits_naming_no_issue(&[commit("2222222", pasted)]),
            Verdict::Held,
            "the fixture must break the character set rule and nothing else"
        );
    }

    /// A carriage return is inside the set, and an angstrom sign is not.
    #[test]
    fn a_line_ending_is_not_what_this_rule_refuses() {
        let windows = "Pin the toolchain\r\n\r\nRefs #4.\r\n";
        assert_eq!(
            commits_outside_the_character_set(&[commit("3333333", windows)]),
            Verdict::Held
        );

        let angstrom = "Pin the toolchain at 5890 \u{212b}\r\n\r\nRefs #4.\r\n";
        assert!(matches!(
            commits_outside_the_character_set(&[commit("3333333", angstrom)]),
            Verdict::Broken(_)
        ));
    }

    /// A version bump without its changelog entry is refused, and the same bump
    /// with the entry is not.
    #[test]
    fn a_version_bump_without_a_changelog_entry_is_refused() {
        let bump =
            "--- a/Cargo.toml\n+++ b/Cargo.toml\n-version = \"0.1.0\"\n+version = \"0.2.0\"\n";

        assert!(matches!(
            version_bump_without_a_changelog_entry(bump, &paths(&["Cargo.toml"])),
            Verdict::Broken(_)
        ));
        assert_eq!(
            version_bump_without_a_changelog_entry(bump, &paths(&["Cargo.toml", "CHANGELOG.md"])),
            Verdict::Held,
            "the same bump with its entry must not be refused"
        );
    }

    /// The near misses this rule has to survive.
    ///
    /// A manifest arriving for the first time carries an added version line and
    /// no removed one, and the compiler pin sits one word away from the line
    /// this rule reads.
    #[test]
    fn a_manifest_that_did_not_bump_is_not_refused() {
        let arriving = "--- /dev/null\n+++ b/Cargo.toml\n+version = \"0.1.0\"\n";
        assert_eq!(
            version_bump_without_a_changelog_entry(arriving, &paths(&["Cargo.toml"])),
            Verdict::Held
        );

        let compiler = "--- a/Cargo.toml\n+++ b/Cargo.toml\n-rust-version = \"1.96.0\"\n+rust-version = \"1.97.0\"\n";
        assert_eq!(
            version_bump_without_a_changelog_entry(compiler, &paths(&["Cargo.toml"])),
            Verdict::Held,
            "the compiler pin is not the package version"
        );

        assert_eq!(
            version_bump_without_a_changelog_entry("", &paths(&["src/lib.rs"])),
            Verdict::Held,
            "a change that does not touch the manifest is not a bump"
        );
    }

    /// A body naming no issue is refused, one naming one is not, and an absent
    /// body is not reached.
    #[test]
    fn a_pull_request_body_naming_no_issue_is_refused() {
        assert!(matches!(
            pull_request_body_naming_no_issue(Some("Adds the parser and its fixtures.")),
            Verdict::Broken(_)
        ));
        assert_eq!(
            pull_request_body_naming_no_issue(Some("Closes #27. Adds the parser.")),
            Verdict::Held
        );
        assert!(matches!(
            pull_request_body_naming_no_issue(None),
            Verdict::Skipped(_)
        ));
    }

    /// The warning tier's own fixtures, one line either side of the number.
    #[test]
    fn a_large_change_is_annotated_and_a_small_one_is_not() {
        assert!(matches!(
            a_large_change(A_LARGE_CHANGE + 1),
            Verdict::Broken(_)
        ));
        assert_eq!(a_large_change(A_LARGE_CHANGE), Verdict::Held);
    }

    #[test]
    fn source_without_a_test_is_annotated_and_source_with_one_is_not() {
        assert!(matches!(
            source_without_a_test(&paths(&["src/spectroscopy/levels.rs"])),
            Verdict::Broken(_)
        ));
        assert_eq!(
            source_without_a_test(&paths(&["src/spectroscopy/levels.rs", "tests/levels.rs"])),
            Verdict::Held
        );
        assert_eq!(
            source_without_a_test(&paths(&["docs/decisions/layout.md"])),
            Verdict::Held,
            "a change that touches no source owes no test to this rule"
        );
    }

    /// The whole of what makes the warning tier a warning tier.
    ///
    /// Both warning rules broken and nothing refuses. One failing rule broken
    /// beside them and it does. Without this the split is a label on a
    /// printout.
    #[test]
    fn the_warning_tier_never_refuses_and_the_failing_tier_does() {
        let only_warnings = vec![
            report(
                Tier::Warning,
                super::THE_CHANGE_FITS_ONE_SITTING,
                Verdict::Broken(vec!["large".to_owned()]),
            ),
            report(
                Tier::Warning,
                super::SOURCE_ARRIVES_WITH_A_TEST,
                Verdict::Broken(vec!["untested".to_owned()]),
            ),
        ];
        assert!(refused(&only_warnings).is_empty());

        let with_a_failing: Vec<Reported> = only_warnings
            .into_iter()
            .chain([report(
                Tier::Failing,
                EVERY_COMMIT_NAMES_AN_ISSUE,
                Verdict::Broken(vec!["unlinked".to_owned()]),
            )])
            .collect();
        assert_eq!(refused(&with_a_failing), vec![EVERY_COMMIT_NAMES_AN_ISSUE]);
    }

    /// A rule that was not reached is named and refuses nothing.
    #[test]
    fn a_rule_that_was_not_reached_is_named_and_refuses_nothing() {
        let skipped = every_range_rule_skipped("no base in this clone");
        assert_eq!(
            skipped.len(),
            RULES.len() - 1,
            "every rule but the body rule reads the range"
        );
        assert!(refused(&skipped).is_empty());
        assert_eq!(not_reached(&skipped).len(), RULES.len() - 1);
        assert!(
            !not_reached(&skipped).contains(&BODY_NAMES_AN_ISSUE),
            "the body rule does not read the range and is not skipped with it"
        );
    }

    /// Every rule is named once and sits in one tier.
    #[test]
    fn every_rule_is_named_once() {
        for (_, rule) in RULES {
            assert_eq!(
                RULES.iter().filter(|(_, other)| *other == rule).count(),
                1,
                "{rule} is listed twice"
            );
        }
        assert_eq!(
            RULES
                .iter()
                .filter(|(tier, _)| *tier == Tier::Failing)
                .count(),
            4,
            "the failing tier holds the four rules whose answer is unambiguous"
        );
    }
}
