# Source preference is data, and every answer names the rule it used

Decided for issue #16.

## What this decision is about

Where a line appears in two sources, something has to happen. The field's current
answer is a hand decision, taken once, written into a file, and never revisited.

The objection is not that a preference was expressed. Somebody has to choose, and
refusing to choose is itself a choice that pushes the work onto every caller.
The objection is that the preference became invisible: it stopped being a
statement anybody could disagree with and became a behaviour nobody could see.

So this decision is about visibility, not about which source wins.

## The profile model

A preference is a named profile, stored as data.

A profile is a document in the register, not a branch in the code. It has an
identifier, a version, and an ordered set of rules. Each rule states a condition
over a claim's recorded properties and what that condition implies: prefer,
exclude, or refuse. The properties a rule may read are the ones a claim already
carries under #12, which are the source, the method, the year, the uncertainty
and the snapshot, plus the match record from #10 and the ancestry from #14.

A profile can be read, diffed, cited in a paper and disagreed with by name. A
branch cannot be any of those things, and that is the whole of the argument.

A profile is versioned because it will change, and an answer produced under one
version is not the same answer as one produced under the next. An answer citing
a profile without its version cites a moving target.

Nothing about this makes a profile correct. A profile expressing a bad
preference is exactly as bad as the same preference in a branch. It is merely
visible, which is the difference between a claim somebody can refute and a
behaviour somebody has to reverse engineer.

## The naming rule for answers

Every answer names the profile that produced it, with its version, in the answer,
next to the number.

Not in the documentation, not in a log, not available on request. In the answer,
because the answer is the thing that gets copied into somebody else's script and
then into somebody else's paper, and everything not carried in the answer is
lost at the first copy.

An answer produced without a profile says so in the same place. That is a
different statement from an answer produced under a profile that happened to
apply no rule, and the two are not spelled the same way.

The board ships no profile as the default unless the maintainer decides
otherwise, which is entry 5 of #1 and is not settled here. What this decision
fixes is that whichever way that goes, a preference is a citable object rather
than a behaviour.

## The refusal case

A profile is allowed to decline.

A profile that says the sources disagree beyond a stated threshold, and returns
the competing claims rather than picking one, is a legitimate profile and not a
degenerate one. It is probably the most honest profile for the cases this board
most wants to expose, because a transition where two critically assessed sources
disagree by more than their combined uncertainties is a result about the field
rather than a value to be averaged.

The refusal is an answer with its own shape. It names the profile that refused,
the threshold that was crossed, and the claims that crossed it. It is not an
error and it is not an empty result, both of which a caller would reasonably
treat as "no data".

This also gives the refusal in #14 a place to live in the interface. A
marginalisation refused for shared ancestry and a selection refused for
disagreement are different refusals with different remedies, and an answer says
which one it is.

## What is greppable, and what is not

The rule this decision has to survive is that a preference cannot reappear as a
branch. That is a search over the tree: no selection between competing claims
anywhere except inside the evaluation of a named profile.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50, which lists this invariant by name
among the searches it will run. Nothing refuses it today, and nothing could yet:

    grep -rn --include=*.rs -E '^\s*(pub )?(struct|enum|type|trait) ' src tests
    tests/environment_guard.rs:26:struct Finding {
    tests/environment_guard.rs:34:enum Reason {

There is no claim type, no profile type and no selection anywhere in the tree.
This document is a constraint on work that has not started, which is the point
of recording it now, and the search that enforces it lands with the code it is
about rather than before it.

The half that no search can reach is whether a given profile expresses a
defensible preference. That is a judgement about meaning, no reading of the tree
makes it, and marking it changes nothing. `PROSE, NOT ENFORCEMENT`, `TERMINAL`.

## What this does not decide

Which source wins, and whether any profile is the default. Entry 5 of #1.

The weighting used when claims are marginalised rather than selected between.
Entry 7 of #1, and a weighting is not a profile: one combines claims and the
other chooses among them, and a design that lets a profile express a weighting
would put the same decision in two places.

## The means for this file

Markdown. The artefact is a decision read before the query surface exists, the
tree already carries Markdown, and it adds no language, runtime or dependency.
