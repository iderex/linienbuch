# Shared underlying measurements, how they are detected, and what they forbid

Decided for issue #14. This is the decision that separates the board's headline
number from a decorative one.

## The problem, stated once

Two compilations quoting one underlying laboratory measurement are not two
independent values. Marginalising over them as if they were counts that
measurement twice and narrows the answer, for no reason other than that somebody
copied a number.

Compilations routinely quote each other, and the chain from a modern database
entry back to the bench often runs through two or three intermediate
compilations. This repository's own README names one such chain, for Fe I, from
a modern entry through a 1988 compilation to a 1982 measurement. So this is the
normal case and not an edge case, and a design that treats independence as the
default is wrong most of the time rather than occasionally.

## The edge model

Provenance edges reach the measurement. They do not stop at the compilation the
value was read from.

A claim carries two distinct things and they are never collapsed into one field.
The source it was read from, which is a snapshot of a database this board
retrieved. And the reference that source attributes the value to, as the source
spells it.

The second is an edge to another node, not a string in a bibliography field. The
node it points at is one of four kinds.

A primary measurement. A bench experiment that produced the number. This is a
terminal node and it is what the resolution is looking for.

A calculation. A computation that produced the number, with the method it used.
Also terminal, and distinct from a measurement, because two calculations sharing
a method and a set of input parameters are correlated in a way two independent
measurements are not.

A compilation. A node that itself attributes the value to something else. Not
terminal, and the resolution continues through it.

A dead end. The source attributes the value to something that cannot be
resolved: a private communication, a reference that does not resolve, or no
attribution at all. This is terminal and it is recorded as a dead end rather
than as an absence, because "we followed this and it stopped" and "we did not
follow this" are different states and only one of them is an answer.

## The resolution procedure

For each claim, follow the attribution edge. If it reaches a compilation, follow
that compilation's attribution for the same transition. Continue until a
terminal node is reached or until the chain revisits a node it has already
passed through, which is a cycle and is recorded as one.

The result is the claim's ancestor set: every terminal node reachable from it.
It is a set rather than a single node because a compilation may attribute one
value to several measurements it averaged, and an average of two measurements
shares an ancestor with each of them.

The chain is recorded as it was resolved, with the snapshot of each source it
was resolved from, so that a later reader can see the path rather than the
conclusion. A resolution made against a different snapshot is a different
resolution.

## The disjointness rule

Before a set of claims is marginalised, their ancestor sets must be pairwise
disjoint.

Where they are not, the operation refuses. It does not proceed with a caveat in
a footnote, and it does not silently down-weight. The refusal names which claims
share which ancestor, so that the caller can see what the overlap was and decide
what to do about it, which is a judgement and not something this board makes on
their behalf.

Refusing rather than adjusting is the choice worth arguing about, so here is the
argument. Any adjustment is a model of how correlated the two claims are, and
the information needed to build that model is exactly the information that is
missing when a chain is only partly resolved. An adjustment would therefore be a
guess presented as arithmetic, which is the failure mode this board exists to
object to, one level up from where it usually happens.

## The default when ancestry is unknown

Refuse.

An unresolved chain is not a disjoint one. Where a claim's ancestor set is
incomplete, unknown or contains a dead end that another claim's chain might also
reach, the marginalisation is refused for the same reason as an established
overlap.

This is the whole point of the decision. Assuming independence when ancestry is
unknown is the error the rule exists to prevent, and a default that assumes
independence would let the error back in through the case that is most common,
because resolving a chain is real work per species and will often be incomplete
for a long time.

The cost is honest and should be stated. Early on, the board will refuse to
marginalise a great deal. That is a correct report of what is known, and the way
out of it is resolution work rather than a softer default.

## The worked example

Two claims that look independent, resolved, and refused.

    claim 1   read from source A, snapshot 2026-01
              A attributes it to compilation C, 1988
    claim 2   read from source B, snapshot 2026-01
              B attributes it to compilation D, 1995

Nothing visible at this point connects them. Two databases, two references, two
different years. A caller marginalising over the two would get a narrower answer
than either, and would be entitled to think it meant something.

Resolve one step further.

    compilation C, 1988   attributes the value for this transition to measurement M, 1982
    compilation D, 1995   attributes the value for this transition to compilation C, 1988

Ancestor set of claim 1 is {M}. Ancestor set of claim 2 is also {M}, because D
resolves through C. The sets are not disjoint and the marginalisation is refused,
naming M as the shared ancestor and naming C as the node both chains pass
through.

What the board reports instead is both claims, both chains, and the fact that
they rest on one measurement. That is a more useful answer than a narrower
number, and it is the answer the caller would have wanted if they had known
enough to ask for it.

The same example with D attributing to a second measurement N, 1993, gives
ancestor sets {M} and {N}, which are disjoint, and the marginalisation proceeds.
The two cases differ by one edge and by nothing else, which is the point: the
difference is invisible at the level a caller works at, so the board has to be
the thing that sees it.

## What this does not decide

The weighting used once a marginalisation is permitted. That changes the headline
number and is entry 7 of #1.

How a claim's transition is matched to a compilation's row while following the
chain, which is #10's identity and its partial match rule. A chain followed
through a wrongly matched row is a wrong chain, so the two decisions constrain
each other.

Whether astrophysically calibrated values may be marginalised with the
laboratory values they were calibrated against. That is a shared ancestry of a
different kind, it is #15's to record and entry 6 of #1's to decide, and it is
named here because the disjointness rule is the mechanism that would carry it.

## Enforcement

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50 for the greppable half and #53 for
the proof. Nothing in the tree refuses a marginalisation today, because there is
no arithmetic in the tree yet. When there is, the refusal is a check with a
fixture in which two claims share an ancestor and a neighbouring fixture in which
they do not, differing by the one edge the worked example above turns on.

## The means for this file

Markdown. The artefact is a decision read before the propagation code exists, the
tree already carries Markdown, and it adds no language, runtime or dependency.
