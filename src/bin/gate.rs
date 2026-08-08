//! The whole gate, in one command, run before a push.
//!
//! The checks exist so that a machine can refuse a change. They are worth much
//! less if the only way to find out is to push and wait, because then the loop
//! is minutes long and people stop running it.
//!
//! This is one implementation rather than two. The workflows do not restate the
//! commands below; each of them invokes this binary and asks for one leg. Two
//! procedures drift, and the drift is found when they disagree about a red run,
//! which is the moment nobody has time for it. A test in this file reads the
//! workflow files and reds the suite if one of them ever states a cargo command
//! of its own.
//!
//! It is not the enforcement. Nothing forces anybody to run it, the hook that
//! calls it is absent from a fresh clone and skippable in every clone that has
//! it, and what stands behind a merge is the ruleset on the default branch and
//! the checks that run on the pull request.
//!
//! It says what it examined. A leg it did not run is printed with what it needs
//! and how to ask for it, so a run that covered less than the whole set cannot
//! be read as one that covered it and found nothing.

use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// One leg of the gate.
struct Leg {
    /// What a caller asks for, and the stem of the workflow file that runs the
    /// same leg on a pull request. One string for both, so a leg cannot be
    /// renamed on one side only.
    name: &'static str,
    /// The command lines this leg runs, in order, stopping at the first failure.
    commands: &'static [&'static [&'static str]],
    /// Environment set for this leg's commands and for nothing else.
    env: &'static [(&'static str, &'static str)],
}

/// The legs, the first four in the order #5 names them.
///
/// The workflows are independent files with no ordering between them, so there
/// is no order there to copy. The first four are the order the issue that named
/// those checks lists them in, and that is stated rather than presented as an
/// order the workflows impose. The fifth is #49's, and it is last because it is
/// the only one that reads git rather than the tree, so a run that stops before
/// it has still judged everything the tree can be judged on.
const LEGS: [Leg; 5] = [
    Leg {
        name: "build",
        commands: &[
            &["build", "--locked", "--all-targets"],
            // --all-targets does not reach the integration harness, which
            // `Cargo.toml` declares with `test = false`. Compiling it is not
            // running it, and no leg here runs it.
            &["build", "--locked", "--test", "integration"],
        ],
        // A warning that is allowed to accumulate is a warning nobody reads.
        //
        // An environment RUSTFLAGS replaces whatever `.cargo/config.toml` would
        // have supplied rather than adding to it, and the flags this tree
        // carries there are what make a release artefact reproducible on the
        // msvc host. Nothing is lost here: every command above builds the dev
        // profile, which produces no release artefact for anybody to compare.
        // A release build must not be run with this set, and none is run here.
        env: &[("RUSTFLAGS", "-D warnings")],
    },
    Leg {
        name: "test",
        // The default suite. The integration harness is outside this by
        // construction rather than by a filter written here.
        commands: &[&["test", "--locked"]],
        env: &[],
    },
    Leg {
        name: "format",
        // --check reports and refuses rather than rewriting, so the leg never
        // changes the tree it is judging.
        commands: &[&["fmt", "--all", "--", "--check"]],
        env: &[],
    },
    Leg {
        name: "lint",
        // The default lint level with warnings denied, which is the level the
        // tree passes at. There is no allow list here and none anywhere else: an
        // exception is written at the code it excuses, where the person changing
        // that code will read it.
        commands: &[
            &[
                "clippy",
                "--locked",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            &[
                "clippy",
                "--locked",
                "--test",
                "integration",
                "--",
                "-D",
                "warnings",
            ],
        ],
        env: &[],
    },
    Leg {
        name: "hygiene",
        // The only leg whose subject is the change rather than the tree. Its
        // rules, its two tiers and what it does when a rule cannot be reached
        // are in `src/bin/hygiene.rs`; nothing about them is restated here,
        // because this file decides which legs run and not what any of them
        // means.
        commands: &[&["run", "--locked", "--quiet", "--bin", "hygiene"]],
        env: &[],
    },
];

/// Something this gate does not examine, printed by every run.
struct NotAsked {
    /// What it is called.
    name: &'static str,
    /// The workflow file stem where it runs on a pull request, where it is a
    /// workflow. `None` where no workflow runs it either.
    workflow: Option<&'static str>,
    /// Why this gate does not run it.
    needs: &'static str,
    /// How to ask for it.
    ask_with: &'static str,
}

/// What judges a change somewhere in this repository and is not run here.
///
/// Compared against the workflow directory by a test below, in both directions,
/// so a workflow added later is either a leg or an entry here and cannot be
/// silently absent from both. That comparison is what stops this disclosure from
/// going quiet as the tree grows.
const NOT_ASKED: [NotAsked; 6] = [
    NotAsked {
        name: "the integration harness",
        workflow: None,
        needs: "the network, an upstream source that is answering, and an authenticated gh \
                for the leg that reads the target gate's required set",
        ask_with: "cargo test --test integration",
    },
    NotAsked {
        name: "the sign-off check",
        workflow: Some("dco"),
        needs: "the base and the head of a pull request, which do not exist here",
        ask_with: "open the pull request",
    },
    NotAsked {
        name: "the dependency review",
        workflow: Some("dependency-review"),
        needs: "the advisory database the forge queries, which is not on this machine",
        ask_with: "open the pull request",
    },
    NotAsked {
        name: "the supply-chain self-audit",
        workflow: Some("scorecard"),
        needs: "the default branch's ruleset and the forge's own view of this repository",
        ask_with: "open the code-scanning tab",
    },
    NotAsked {
        name: "the Unicode control-character scan",
        workflow: Some("unicode-guard"),
        needs: "nothing this machine lacks; it is not one of the four checks this gate \
                runs, and reproducing its pattern here would put that pattern in a \
                second place",
        ask_with: "the git grep in .github/workflows/unicode-guard.yml, or open the pull request",
    },
    NotAsked {
        name: "the workflow audit",
        workflow: Some("zizmor"),
        needs: "a tool this tree does not carry, and a network fetch to install it",
        ask_with: "open the pull request",
    },
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Where the legs put what they build.
///
/// Not the ordinary build directory, and the reason is that this program is one
/// of the things the build leg builds. A build writing into the ordinary
/// directory has to replace the executable that is running it, and on at least
/// one platform that is refused rather than allowed:
///
///     error: failed to remove file `...\target\debug\gate.exe`
///     Caused by: Zugriff verweigert (os error 5)
///
/// Measured, on x86_64-pc-windows-msvc, before this directory existed. The same
/// run on a platform that lets a running executable be unlinked would have
/// passed, which is worse than a plain failure: the gate would be green on one
/// machine and red on another for a reason that is not in the tree. One
/// directory, everywhere, removes the difference.
///
/// The cost is real and is not hidden. These artefacts are not shared with an
/// ordinary `cargo build`, so a gate run after a hand build compiles again.
fn target_dir() -> PathBuf {
    manifest_dir().join("target").join("gate")
}

/// One child command, built but not run, so that what the gate is about to do
/// can be read by a test as well as by a person.
fn command_for(leg: &Leg, args: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args(args)
        .current_dir(manifest_dir())
        .env("CARGO_TARGET_DIR", target_dir());
    for (key, value) in leg.env {
        command.env(key, value);
    }
    command
}

/// Run one leg, stopping at its first failing command. Returns whether it
/// passed.
fn run_leg(leg: &Leg) -> bool {
    for args in leg.commands {
        let printed = format!("cargo {}", args.join(" "));
        println!("gate: {} runs {printed}", leg.name);

        let mut command = command_for(leg, args);

        let status = match command.status() {
            Ok(status) => status,
            Err(e) => {
                println!("gate: {} could not start {printed}: {e}", leg.name);
                return false;
            }
        };
        if !status.success() {
            println!("gate: {} refused at {printed}", leg.name);
            return false;
        }
    }
    true
}

fn names(list: &[&str]) -> String {
    if list.is_empty() {
        "nothing".to_owned()
    } else {
        list.join(", ")
    }
}

/// What every run prints about what it did not examine.
fn print_not_asked() {
    println!("gate: not asked for, and so not examined:");
    for entry in &NOT_ASKED {
        let where_it_runs = match entry.workflow {
            Some(stem) => format!(" (.github/workflows/{stem}.yml)"),
            None => String::new(),
        };
        println!("gate:   {}{where_it_runs}", entry.name);
        println!("gate:     needs {}", entry.needs);
        println!("gate:     ask for it with: {}", entry.ask_with);
    }
}

fn usage() -> String {
    let all: Vec<&str> = LEGS.iter().map(|leg| leg.name).collect();
    format!(
        "usage: cargo run --bin gate [LEG]\n  \
         with no LEG, every leg in order: {}\n  \
         stopping at the first failure",
        all.join(", ")
    )
}

fn main() -> ExitCode {
    let asked: Vec<String> = env::args().skip(1).collect();

    let legs: Vec<&Leg> = match asked.as_slice() {
        [] => LEGS.iter().collect(),
        [one] => match LEGS.iter().find(|leg| leg.name == one) {
            Some(leg) => vec![leg],
            None => {
                println!("gate: there is no leg called {one}");
                println!("{}", usage());
                return ExitCode::FAILURE;
            }
        },
        _ => {
            println!("gate: one leg at a time, or none for all of them");
            println!("{}", usage());
            return ExitCode::FAILURE;
        }
    };

    println!(
        "gate: the legs build into target/gate rather than target, because this \
         program is one of the things they build"
    );

    let mut examined: Vec<&str> = Vec::new();
    let mut refused: Option<&str> = None;
    for leg in &legs {
        if run_leg(leg) {
            examined.push(leg.name);
        } else {
            refused = Some(leg.name);
            break;
        }
    }

    println!("gate: examined {}", names(&examined));

    if let Some(name) = refused {
        println!("gate: refused by {name}");
        let not_reached: Vec<&str> = legs
            .iter()
            .map(|leg| leg.name)
            .skip(examined.len() + 1)
            .collect();
        println!(
            "gate: not reached, because the gate stops at the first failure: {}",
            names(&not_reached)
        );
    }

    let not_named: Vec<&str> = LEGS
        .iter()
        .map(|leg| leg.name)
        .filter(|name| !legs.iter().any(|leg| leg.name == *name))
        .collect();
    if !not_named.is_empty() {
        println!(
            "gate: one leg was asked for, so these were not run: {}",
            names(&not_named)
        );
    }

    print_not_asked();

    match refused {
        Some(_) => ExitCode::FAILURE,
        None => ExitCode::SUCCESS,
    }
}

#[cfg(test)]
mod tests {
    use super::{LEGS, NOT_ASKED, command_for, manifest_dir, target_dir};
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn workflow_dir() -> PathBuf {
        manifest_dir().join(".github").join("workflows")
    }

    /// The one command a workflow is allowed to state for a leg.
    ///
    /// Written once, here, and read by the checks below. A workflow that says
    /// anything else about cargo is a second procedure.
    fn gate_call(leg: &str) -> String {
        format!("cargo run --locked --quiet --bin gate -- {leg}")
    }

    /// Every cargo command line a file states.
    ///
    /// A comment is not a command. Prose in these files talks about cargo often,
    /// so a line whose first non-blank character is a hash is skipped.
    /// Everything else carrying the word is read as an invocation, including a
    /// line inside a block scalar, so a workflow cannot hide a second procedure
    /// by changing how the step is written.
    fn cargo_invocations(text: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        for line in text.lines() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            if let Some(at) = line.find("cargo ") {
                found.insert(line[at..].trim_end().to_owned());
            }
        }
        found
    }

    /// What a file states about cargo that is not the one gate call it is
    /// allowed to state.
    ///
    /// A pure function over the text and the leg it belongs to, so it can be
    /// proved on constructed input rather than only against the tree, which is
    /// the only way to show it refuses anything at all.
    fn restatements(text: &str, leg: &str) -> Vec<String> {
        let allowed = gate_call(leg);
        cargo_invocations(text)
            .into_iter()
            .filter(|line| *line != allowed)
            .collect()
    }

    /// What one set holds and the other does not, in both directions.
    fn drift<'a>(
        here: &'a BTreeSet<String>,
        there: &'a BTreeSet<String>,
    ) -> (Vec<&'a String>, Vec<&'a String>) {
        (
            here.difference(there).collect(),
            there.difference(here).collect(),
        )
    }

    fn workflow_stems(dir: &Path) -> BTreeSet<String> {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        let mut stems = BTreeSet::new();
        for entry in entries {
            let path = entry
                .expect("cannot read a workflow directory entry")
                .path();
            if path.extension().and_then(|e| e.to_str()) != Some("yml") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_else(|| panic!("{} has no readable stem", path.display()));
            stems.insert(stem.to_owned());
        }
        stems
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    }

    /// The refusal, on constructed input, with its neighbour beside it.
    #[test]
    fn a_workflow_that_states_its_own_cargo_command_is_refused() {
        let restated = "      - name: Compile\n        run: cargo build --locked --all-targets\n";
        assert_eq!(
            restatements(restated, "build"),
            vec!["cargo build --locked --all-targets".to_owned()],
            "a workflow stating the command itself must be reported"
        );

        let calls_the_gate =
            "      - name: Compile\n        run: cargo run --locked --quiet --bin gate -- build\n";
        assert!(
            restatements(calls_the_gate, "build").is_empty(),
            "the neighbour, one command away, must not be reported"
        );
    }

    /// The near miss. The gate is called, and for the wrong leg, which is one
    /// word different from the line that is correct and is the mistake a copied
    /// workflow file actually carries.
    #[test]
    fn a_gate_call_for_the_wrong_leg_is_refused() {
        let wrong_leg =
            "      - name: Compile\n        run: cargo run --locked --quiet --bin gate -- lint\n";
        assert_eq!(
            restatements(wrong_leg, "build"),
            vec!["cargo run --locked --quiet --bin gate -- lint".to_owned()],
            "a gate call for another leg must be reported"
        );
        assert!(
            restatements(wrong_leg, "lint").is_empty(),
            "the same line in the workflow it belongs to must not be reported"
        );
    }

    /// Prose about cargo is not a command, and a command written inside a block
    /// scalar still is one.
    #[test]
    fn a_comment_is_not_an_invocation_and_a_block_scalar_line_is() {
        let commented = "        # cargo build --locked would be a second procedure\n";
        assert!(cargo_invocations(commented).is_empty());

        let block = "        run: |\n          cargo build --locked --all-targets\n";
        assert_eq!(
            cargo_invocations(block).into_iter().collect::<Vec<_>>(),
            vec!["cargo build --locked --all-targets".to_owned()]
        );
    }

    /// Every workflow that runs a leg invokes this binary for that leg and
    /// states nothing of its own.
    #[test]
    fn every_leg_workflow_calls_the_gate_and_restates_nothing() {
        for leg in &LEGS {
            let path = workflow_dir().join(format!("{}.yml", leg.name));
            let text = read(&path);

            assert!(
                cargo_invocations(&text).contains(&gate_call(leg.name)),
                "{} does not invoke the gate for its own leg",
                path.display()
            );
            let restated = restatements(&text, leg.name);
            assert!(
                restated.is_empty(),
                "{} states cargo commands of its own: {restated:?}",
                path.display()
            );
        }
    }

    /// No workflow anywhere states a cargo command that is not a gate call.
    ///
    /// The check above reads the four files by name. This one reads the whole
    /// directory, so a fifth workflow restating a command is refused as well.
    #[test]
    fn no_workflow_states_a_cargo_command_that_is_not_a_gate_call() {
        let allowed: BTreeSet<String> = LEGS.iter().map(|leg| gate_call(leg.name)).collect();
        for stem in workflow_stems(&workflow_dir()) {
            let path = workflow_dir().join(format!("{stem}.yml"));
            for invocation in cargo_invocations(&read(&path)) {
                assert!(
                    allowed.contains(&invocation),
                    "{} states {invocation:?}, which is not a call into this gate",
                    path.display()
                );
            }
        }
    }

    /// The comparison refuses in both directions, on constructed sets.
    #[test]
    fn drift_is_reported_in_both_directions() {
        let a: BTreeSet<String> = ["one".to_owned(), "two".to_owned()].into();
        let b: BTreeSet<String> = ["two".to_owned(), "three".to_owned()].into();

        let (only_here, only_there) = drift(&a, &b);
        assert_eq!(only_here, vec![&"one".to_owned()]);
        assert_eq!(only_there, vec![&"three".to_owned()]);

        let (none_here, none_there) = drift(&a, &a);
        assert!(none_here.is_empty() && none_there.is_empty());
    }

    /// Every workflow in the tree is either a leg this gate runs or an entry in
    /// what it declares it did not examine.
    ///
    /// A workflow added later and named in neither place would make a green run
    /// of this command look like a run that covered everything.
    #[test]
    fn every_workflow_is_either_a_leg_or_declared_unexamined() {
        let accounted: BTreeSet<String> = LEGS
            .iter()
            .map(|leg| leg.name.to_owned())
            .chain(
                NOT_ASKED
                    .iter()
                    .filter_map(|entry| entry.workflow)
                    .map(str::to_owned),
            )
            .collect();
        let in_tree = workflow_stems(&workflow_dir());

        let (accounted_but_absent, in_tree_but_unaccounted) = drift(&accounted, &in_tree);
        assert!(
            accounted_but_absent.is_empty(),
            "named in this file and absent from the workflow directory: {accounted_but_absent:?}"
        );
        assert!(
            in_tree_but_unaccounted.is_empty(),
            "in the workflow directory and neither run nor declared unexamined: \
             {in_tree_but_unaccounted:?}"
        );
    }

    /// The hook runs the whole gate and states nothing of its own.
    #[test]
    fn the_hook_execs_the_gate() {
        let path = manifest_dir().join(".githooks").join("pre-push");
        let invocations: Vec<String> = cargo_invocations(&read(&path)).into_iter().collect();
        assert_eq!(
            invocations,
            vec!["cargo run --locked --quiet --bin gate".to_owned()],
            "the hook must run the whole gate and nothing else"
        );
    }

    /// The contributing document names the command that exists.
    ///
    /// One direction only. Whether every cargo command written in that file is
    /// a gate call is not asserted, because prose about a command is not a
    /// command and refusing a sentence that quotes one would be a guard that
    /// fires on the wrong thing.
    #[test]
    fn the_contributing_document_names_the_whole_gate() {
        let path = manifest_dir().join("CONTRIBUTING.md");
        assert!(
            read(&path).contains("cargo run --locked --quiet --bin gate"),
            "CONTRIBUTING.md does not give the command that runs the whole gate"
        );
    }

    /// Every child command builds somewhere other than the directory holding
    /// the running gate.
    ///
    /// This is the guard against a failure that showed up on one platform and
    /// would have stayed invisible on another: a build that has to replace the
    /// executable running it. The test reads the command the gate would run
    /// rather than the outcome of running it, so it holds for every leg
    /// including the ones no fixture can afford to execute.
    #[test]
    fn every_leg_builds_away_from_the_running_gate() {
        let expected = target_dir();
        assert_ne!(
            expected,
            manifest_dir().join("target"),
            "the gate's build directory must not be the ordinary one"
        );

        for leg in &LEGS {
            for args in leg.commands {
                let command = command_for(leg, args);
                let set: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();
                let target = set
                    .iter()
                    .find(|(key, _)| *key == OsStr::new("CARGO_TARGET_DIR"))
                    .unwrap_or_else(|| {
                        panic!("{} runs a command with no build directory set", leg.name)
                    });
                assert_eq!(
                    target.1,
                    Some(expected.as_os_str()),
                    "{} runs a command that would build into the wrong directory",
                    leg.name
                );
            }
        }
    }

    /// The build leg, and only the build leg, denies warnings through the
    /// environment.
    ///
    /// The other legs pass their level as arguments. A leg that quietly gained
    /// this variable would lose the linker flags `.cargo/config.toml` supplies,
    /// which is the thing the comment beside it warns about.
    #[test]
    fn only_the_build_leg_sets_rustflags() {
        for leg in &LEGS {
            let carries = leg.env.iter().any(|(key, _)| *key == "RUSTFLAGS");
            assert_eq!(
                carries,
                leg.name == "build",
                "{} disagrees with where RUSTFLAGS belongs",
                leg.name
            );
        }
    }

    /// Every leg names a workflow file that exists, and names it once.
    #[test]
    fn each_leg_names_one_workflow_that_exists() {
        for leg in &LEGS {
            assert!(
                workflow_dir().join(format!("{}.yml", leg.name)).is_file(),
                "{} names no workflow file",
                leg.name
            );
            assert_eq!(
                LEGS.iter().filter(|other| other.name == leg.name).count(),
                1,
                "{} is named twice",
                leg.name
            );
        }
    }
}
