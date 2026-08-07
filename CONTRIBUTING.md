# Contributing

## What is enforced and what is only written down

Read this section first, because it changes how every other section is read.

A sentence in a document is not a rule. It is an explanation of one. A rule is a
thing a machine refuses. Where this file states a rule that nothing in this
repository refuses, it says so at the rule, in the same paragraph, with the
marker `PROSE, NOT ENFORCEMENT`. A reader who cannot tell which sentences bite
has to treat all of them as aspirations, and that is how a document full of good
rules ends up enforcing nothing.

Every mark says which of two kinds it is.

A mark that is `OWED` means a mechanism is possible and an issue is open for it.
The issue number is in the mark. When that issue lands the mark goes.

A mark that is `TERMINAL` means nothing in this tree could read the thing the
rule is about, so no check is owed and none is coming. Rules about how work was
done rather than about what landed are usually this kind: the tree holds
artefacts, and a check that reads the tree cannot reach the conduct that
produced them. Marking one changes nothing about its enforceability, and calling
that a placeholder would be a false promise.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for the marks themselves. `PROSE, NOT
ENFORCEMENT` is a string, and nothing in this repository reads it, resolves the
issue number beside it, or notices when that issue closes and the mark goes
stale. A reader is the only route. This is marked terminal rather than owed
because no issue on this board asks for the route, and inventing one here would
make the mark point at nothing.

## Before anything else

    cargo run --locked --quiet --bin gate

That is the whole gate. Its legs run in order and it stops at the first failure.
What the legs are is not written here: the run prints them, and a list in this
file would drift against the thing that decides them. Ask for one leg by naming
it, and the command tells you which names it knows.

The workflows that run on a pull request do not restate those legs. Each of them
invokes the command above and asks for one, so there is one procedure and not
two. A test refuses a workflow that states a command of its own.

The run says what it examined, and it says what it did not. Everything in this
repository that judges a change and that the command does not run is printed at
the end of every run, with what it needs and how to ask for it, so a green run
that covered less than the whole set cannot be read as one that covered it.

Install the hook once per clone:

    git config core.hooksPath .githooks

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for installing it. Whether a clone pointed
`core.hooksPath` at that directory is a fact of that clone's local git
configuration. No tree holds it, so nothing here can read it and no check is
owed. The sentence above is the whole disclosure and this mark does not soften
it.

The hook is a courtesy that shortens the loop and it is not the enforcement. It
is absent from a fresh clone until the line above is run, and `git push
--no-verify` skips it in a clone that has it. That is written here rather than
left to be discovered, because a hook whose escape hatch is a secret is one
people work around by deleting.

What stands behind a merge is the ruleset on the default branch, quoted under
`## No work without an issue`, and the checks that run on the pull request.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #48, and it is the larger half. The
ruleset quoted below carries no `required_status_checks`, so a red check refuses
nothing today and a change can be merged over one. #48 is the issue that makes
them required. Until it lands, the command above is the only thing that will tell
you before somebody reads the run and notices.

## This document lists no checks

No check is named here and no list of them appears. A list in a document drifts
against the thing it describes, and the drift is found by whoever followed the
document exactly and then met a red run naming something the document never
mentioned. What ran, and what it refused, is printed by the run:

    gh pr checks <number>

That command is the authority. This file is not.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50. Nothing refuses a future edit that
adds a list of checks to this file. #50 is the issue for searches over the tree
that either match or do not, and a search for an enumeration in the documents is
the shape it would take.

## No work without an issue

Every change starts as an issue and lands as a pull request. Direct pushes to
the default branch are refused:

    gh api repos/iderex/linienbuch/rulesets/20528058 \
      --jq '{enforcement, bypass: .bypass_actors, rules: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","rules":["deletion","non_fast_forward","pull_request"]}

There are no bypass actors, so this holds for everyone including the maintainer.

An issue says three things. What is wrong. What the evidence is. What done
means, specifically enough that a reader can decide whether it has been met
without asking the author.

Where the evidence is a number, the issue carries the command that produced it,
run against the reference the reader will have rather than against a working
tree. A number without its command is a number the reader has to take on trust,
and this repository exists because numbers taken on trust turned out to be
wrong.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #49. Nothing reads an issue body on this
board. An issue that states no problem, offers no evidence and defines no done
condition passes every route here. #49 is the pull request hygiene check, and
the shape it describes, a failing tier for what is unambiguous, is where the
issue reference on a change becomes refusable. Whether a body's three parts are
present is a weaker version of the same reading and would go in the same place.

## One topic per change

One topic per commit and per pull request. A commit carrying two unrelated
changes has a message describing one of them, and the other arrives in the
history with no explanation attached.

A commit message says what changed and what failure it prevents. Where it is a
correction, it also says what was wrong and how that was found.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for whether two changes are one topic.
Whether two edits inside one area are related is a judgement about meaning. No
reading of this tree makes it, so no check is owed, and the review is where a
wrong answer is caught. The neighbouring question of whether a message contains
an issue reference at all is refusable, is not this rule, and is #49's.

## Claims and their commands

Every asserted fact carries the command that produced it, run at the commit
being pushed. Where a claim cannot be backed by a command, it is written as a
claim, and the words for the different states are different words. Verified, not
measured, and not evaluated on this route mean three things and are not
interchangeable.

The canonical way this fails is a claim about one artefact made by reading the
nearest thing to hand instead of the thing itself. Reading a local checkout and
reporting it as the state of the default branch is the usual form.

A statement that something was not done survives every later edit of the text
around it. Turning an admission into an assurance is worse than deleting it,
because the reader now has a positive claim where there was an accurate
negative one.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for both paragraphs. Nothing on this board
judges the prose in a document, an issue, a pull request body or a commit
message against the tree it describes, and nothing could: deciding whether a
sentence is true of an artefact is not a property a search or a type can hold.
The review is the only route and it is a person.

## No guard without proof that it bites

A check, an invariant or a schema constraint ships with a fixture that violates
exactly it, a test asserting the guard refuses that fixture for that reason, and
a neighbouring fixture one change away that the guard does not refuse. A guard
that refuses everything proves nothing, and a near miss that could not have
failed proves less than one that nearly did.

The obligation belongs to the refusal site rather than to the guard. A second
condition added to an existing guard adds no new guard and still owes its own
proof, and that is the case that gets missed, because it reads as an edit rather
than as an addition.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #53. Today nothing enumerates the
refusal sites in this tree and compares them against the proofs that reach them,
so a guard can ship with no fixture at all and every route stays green. #53 is
that issue and it carries the register for a site a fixture genuinely cannot
reach.

## Style

English, in everything tracked and in everything written on the tracker. No tool
names, no generated-by markers and no attribution of work to anything other than
its author in tracked text.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50. Nothing on this board searches
tracked text for those markers. A search is exactly the shape #50 describes and
is where this becomes refusable.

Sign off every commit:

    git commit -s

A check on every pull request refuses a commit whose sign off does not match its
author, so this one is not marked. Run the command above from the start rather
than repairing a branch afterwards.

## When a change will not fit

A change that cannot be read in one sitting is usually an issue whose scope was
planned wrong, and the first response is to re-plan that issue into smaller
issues rather than to carve the finished diff into two pull requests that only
make sense together. Two halves that are each unreviewable alone satisfy a size
cap and defeat the reason for it.

Some large changes really are one readable thing, because a single property
holds across every changed byte and the reader checks the property instead of
the diff. That case is what is left after re-planning has been tried, not an
option beside it.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`. No size is named here and none would help.
A size is knowable only once the work exists, which is after the point where
re-planning was the answer, and no reading of a diff separates a scope that was
planned badly from one that was not.
