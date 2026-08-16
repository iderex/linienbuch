# The version scheme, and what a changelog entry owes the reader

Decided for issue #63. What a version number on this board means, which part of
it a given change moves, where an upstream change is recorded instead, and what
an entry in `CHANGELOG.md` has to say.

## The question an ordinary scheme does not answer

Semantic versioning answers a question about interfaces. Does code that was
written against the last version still compile, and does it still mean the same
thing.

That is the wrong question here, or rather it is only half of it. Most of the
people this board is aimed at do not consume an interface. They consume an
answer, and they put it in a paper. Three kinds of change move an answer without
touching any interface at all.

A change to the record model, which decides what is stored and what is refused,
and therefore which claims reach a calculation.

A change to the grade conversion, which is `docs/decisions/accuracy-grades.md`.
The letter scale it converts is published by the source; the number it converts
into is this board's, and moving it moves every uncertainty derived from a
graded value.

A change to the default weighting when several sources are marginalised over.
Entry 7 of #1 answers its shape, a weight by the category of the method with
every claim inside one category weighing the same, and leaves the number each
category carries unset. The issue that raised it says why neither half is an
implementation detail: a uniform weight and an inverse-variance weight are
different answers to the same question, and so are two tables under one shape.

A scheme that versions only interfaces tells the person who upgraded and got a
different number nothing at all. This one is built so that the number tells them
before they run it.

## The scheme

Three components, `MAJOR.MINOR.PATCH`, and the first one answers a different
question from the usual one: can an answer this board has already given change.

The major component moves when a released answer can move, or when a public
interface breaks. Both, because both are things the reader has to act on, and
the first of the two is invisible in a diff of the interface.

What counts as an answer that can move. The record model. The grade conversion.
The marginalisation weighting, or which profile is selected by default. The
propagation method or the rule that chooses between methods. The matcher's
identity rule or its tolerance. The conversion attached to a source, since
`docs/decisions/line-position.md` records that a source's own formula is what
reproduces that source's own numbers.

The minor component moves when something arrives and no released answer moves. A
source, an output format, an operation that did not exist.

The patch component moves when neither. A fix to something that never reached an
answer, a documentation correction, a build repair.

A defect fix is not automatically a patch, and this is the part of the scheme
that will feel wrong the first time it is applied. A fix to the propagation is a
change that moves numbers, so it takes the major component, exactly like a
redesign would. The reader who has to be told is the one who published a number
that was wrong, and telling them costs the same whether the cause was a mistake
or a decision.

## Before the first release

The major component is zero and nothing has been released:

    gh release list --repo iderex/linienbuch
    git ls-remote --tags origin

Both print nothing at the commit this file lands on, and `Cargo.toml` carries
`version = "0.0.0"`, which is a placeholder rather than a version this scheme has
been applied to.

While the major component is zero, the minor component carries what the major
component carries above. That is the ordinary reading of a zero major and it is
written down rather than assumed, because the whole point of this file is that
the rule is not left to be inferred.

The major component reaches one at the first release this board is willing to
have an answer held against, which is the milestone `A first release an operator
can run` rather than a date. What that release owes is #61's, not this file's.

## An upstream that changed is not a version of this board

The same code against a changed upstream produces a different answer, and it is
not a different version of this program. Those are two different causes and
collapsing them makes both untraceable.

A snapshot's identity is its content digest, which is
`docs/decisions/snapshots.md`, and the release notes carry the identity of every
snapshot the release was verified against, which is #61. So a number that moved
because a database was corrected is found in the snapshot identities, and a
number that moved because this board changed is found in the version. A reader
comparing two results can tell which happened, which is the whole reason the
release notes carry the identities at all.

## What a changelog entry says

`CHANGELOG.md` is written for the reader who upgraded and got a different
number, and it is not a list of commits. A history already exists and is better
at being one.

Every entry says what changed. An entry for a change that can move an answer
also says that it can and why, and it sits under its own heading so that it can
be found without reading everything else. The headings a release carries, in
this order:

    ### Answers that can move
    ### Added
    ### Changed
    ### Fixed

The first is this board's addition and the reason the others are not enough. A
change filed under `Fixed` that moves every derived uncertainty is filed
correctly and read wrongly, and the person reading is the one who needs it most.

An entry names the issue it came from. That is how a reader gets from a sentence
in the changelog to the argument behind it, and this board's arguments are in its
issues rather than in its commit messages.

## What refuses what

A version bump arriving without a changelog entry is refused by the hygiene leg
of the gate, in `src/bin/hygiene.rs`, under the rule `a version bump arrives
with a changelog entry`. It reads the manifest's own diff for a removed and an
added `version` line that differ, and then requires `CHANGELOG.md` to be in the
same change:

    cargo run --locked --quiet --bin gate -- hygiene

Its fixture is `a_version_bump_without_a_changelog_entry_is_refused` and its
neighbours are in `a_manifest_that_did_not_bump_is_not_refused`, which hold a
manifest arriving for the first time and the compiler pin, the line one word
away from the one the rule reads.

A bump also moves `Cargo.lock`, which records the package's own version. Every
route in this tree passes `--locked`, so a commit that moves the manifest and
leaves the lock file behind does not run at all:

    error: cannot update the lock file ...\Cargo.lock because --locked was passed to prevent this

That is not the rule above catching anything. It is the build refusing first,
and it is written here because whoever bumps the version will meet it before
they meet anything else.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50, for what an entry says. The rule
above refuses an absent entry and cannot read the one that is present, so an
entry saying nothing about a number that moved passes. A search over the file is
the shape #50 describes and is where the heading above becomes refusable.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for which component a change takes.
Whether a change can move an answer is a judgement about meaning. No reading of
this tree makes it, so no check is owed and none is coming, and the review is
where a wrong answer is caught. That is the same state
`docs/decisions/layout.md` records for the judgement its own boundary rests on.

## What this does not decide

The default weighting itself. Entry 7 of #1 has answered its shape and not the
numbers under it, so there is still nothing here to version. This file says what
a change to it costs once it exists, which is the right way round: the cost can
be agreed before the answer is.

How a release is cut, tagged and published, and what its notes carry beyond the
snapshot identities. That is #61.

What the package is called. Entry 9 of #1 answers the command, which is
`linienbuch`, with the repository, the package and the command kept as one name.
The manifest still carries its own comment saying the name there answers
nothing, and that comment is not repaired here.
