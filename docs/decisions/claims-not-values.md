# The register holds claims, never values

Decided for issue #12, before the schema rather than after it.

## The decision

The register never holds the transition probability. It holds every claim about
it.

A claim is a row that says who said what, by what method, in what year, with
what uncertainty, and from which snapshot of which source. The quantity is a
field of the claim. It is not a field of the transition, and there is no place
in the schema where it could be one.

A transition has an identity, which is #10's, and a set of claims. It has no
value.

## Why this has to be decided first

This board exists because a preference for one compilation over another was made
by hand once, written into a file, and never unpicked. A schema with one value
column per transition rebuilds that failure underneath everything else.

It rebuilds it in the worst available place. A preference expressed in a query
can be argued with, because both values are still there to argue about. A
preference expressed in the schema has already thrown one of them away by the
time anything downstream runs, and no amount of care later recovers it. The
write is where the information is lost, so the write is where the rule has to
sit.

There is a second reason, and it is the one that survives someone deciding the
first is overcautious. A value column has to be filled by something, and
whatever fills it is a selection rule. A schema that does not hold claims does
not thereby avoid having a preference; it has one that nobody wrote down.

## The consequences, stated rather than discovered later

### Every part of the interface returns a set

There is no single value to return, so the shape of every accessor is a set of
claims and not a number. This is not a detail of one function. It reaches the
query surface, the output formats, the library interface and anything that
serialises an answer, which are #42 through #45.

A convenience accessor that returns one number is a preference rule in disguise.
If one is ever added, it names the rule it applied, in the answer, beside the
number. The rule for how a preference is expressed at all is #16's, and this
consequence is the reason that issue exists.

### The count of claims is itself information

A transition with one claim and a transition with four claims that disagree are
in genuinely different states. An answer reporting a number and an uncertainty
has thrown away the more useful half, and it has thrown it away silently, which
is worse than reporting nothing.

So the count travels with the answer. A caller that does not want it can ignore
it; a caller that is given only a number cannot ask for it.

This also fixes what the empty case means. Zero claims is a transition nobody
has published a value for, and it is a different state from a transition with
claims that were all excluded by a profile. Both return no number and they are
not the same answer.

## What this decision does not settle

Which claim wins, or whether any does. That is #16, and the question of whether
a default preference ships at all is entry 5 of #1 and belongs to the
maintainer.

How two claims are recognised as being about one transition. That is #10.

Whether two claims are independent, which decides whether they may be
marginalised together at all. That is #14, and it is the reason the claim carries
its source and the reference that source attributes it to rather than only its
value.

## The state of the tree when this was written

The rule that no type associates a transition with a single value is trivially
satisfied today, and saying so is more useful than implying it was enforced.
There is no transition type, because there is no record model yet; that is #20
through #24. The types that exist belong to the test guard and to nothing else:

    grep -rn --include=*.rs -E '^\s*(pub )?(struct|enum|type|trait) ' src tests
    tests/environment_guard.rs:26:struct Finding {
    tests/environment_guard.rs:34:enum Reason {

So this document is a constraint on work that has not started rather than a
description of work that is done. `PROSE, NOT ENFORCEMENT`, `OWED`, issue #50.
Nothing refuses a type that puts a value on a transition, and a search over the
tree is the shape that would; #50 already lists a neighbouring invariant of
exactly this kind.

## The means for this file

Markdown. The artefact is a decision a contributor reads before writing the
schema, the tree already carries Markdown, and it adds no language, runtime or
dependency.
