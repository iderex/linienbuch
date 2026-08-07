# How a transition is identified across sources

Decided for issue #10. Everything the board does rests on being able to say that
two rows in two databases are about the same transition, so this is decided
before anything joins.

## The identity

A transition is the species, its ionisation stage, its lower level and its upper
level.

Nothing about the line position is part of the identity. Wavelength is a
property of the transition that sources disagree about, and a property sources
disagree about cannot also be the key they are joined on.

That pushes the problem down one layer, to level identity, which is where it
belongs. Sources agree about levels more often than they agree about
wavelengths, and where they disagree about a level they usually say so.

## Level identity, and what it is made of

A level is identified by four components. Each is recorded as present or absent
per source, because a match made on two of them is weaker than one made on four
and an answer that does not say which it made is overclaiming.

**Level energy**, within a tolerance. The tolerance is the quadrature sum of the
two sources' stated level uncertainties, with a floor of 0.05 cm^-1 so that a
source declaring an implausibly small uncertainty cannot narrow the window to
nothing. Where a source states no uncertainty for a level, the floor is used
alone and the match is recorded as having used a defaulted tolerance, which is
one of the things a partial match may not claim its way out of.

**Total angular momentum J**, exactly. There is no tolerance on J. A mismatch is
a non-match, not a weak match, because J is an integer or a half integer that
both sources either got right or got wrong, and a source that disagrees about it
is talking about a different level.

**Parity**, exactly, where both sources give it. Same reasoning as J. Absent from
one side, it is recorded as absent rather than assumed to agree.

**Configuration and term designation**, compared after normalisation, and only
where both sources give them. A mismatch here does not refuse the match on its
own, because notation genuinely differs between sources in ways that are not
disagreements. It downgrades what the match may claim, and the disagreement is
recorded rather than dropped.

The floor of 0.05 cm^-1 is a starting value and not a measurement. #34 measures
how far apart two sources actually put the same line, and the number that
measurement produces replaces this one. Recording it as a chosen default rather
than as a derived one is the point of writing it down here.

## What a partial match may claim

A match records which components it matched on. That record travels with every
answer derived from it.

A match on energy and J alone is a match on energy and J. It is reported as
that. It is never reported as the same transition without the qualifier, and it
never becomes a full match later because nothing contradicted it. The absence of
a contradiction is not evidence.

A match that used a defaulted tolerance, because a source stated no level
uncertainty, says so in the same place.

Downstream, this is what decides whether two claims may be combined at all. A
claim resting on a two-component match and a claim resting on a four-component
match are not equally attached to the transition, and #14's disjointness rule is
not the only thing that can refuse a marginalisation.

## Observed and Ritz wavelengths

A wavelength is observed or it is Ritz, meaning derived from the level energies,
and the two are recorded as different things. They are never interchanged and a
row that does not say which it is, is recorded as not saying.

This matters more here than it looks. A Ritz wavelength is a function of the
level energies, so using one to identify a transition asks the level energies to
confirm a match that was made from the level energies. Where the fallback below
is permitted at all, it may use observed wavelengths only.

## Multiplets

A resolved component and an unresolved multiplet are not the same object and are
not two descriptions of one object.

A multiplet row is recorded as a multiplet, with the set of components it may
contain where the source says. It never matches a component row as an equal.

Combining a multiplet's value with a component's value is refused rather than
caveated, in the same way and for a related reason to #14's refusal: the two
quantities are about different things, and an arithmetic that treats them as
competing values for one thing produces a number that means nothing.

## Where levels cannot be matched

The transition does not join. It is recorded as unmatched, with the reason, and
the reason is one of a fixed set rather than free text: no level identification
in one source, level identification present but disagreeing, J disagreeing,
species or stage disagreeing.

An unmatched row is not a failure to be tidied away. #35 is the report of what
did not join, and it is one of the more informative things this board can
produce, because a line that two major compilations cannot be made to agree on
is exactly the line somebody should look at.

## The fallback, and the conditions on it

A match on line position is permitted, and only under all of the following.

Neither source gives a level identification for the row. The fallback is for
absent level identity, never for present level identity that disagreed. A
disagreement is an answer, and overriding it with a weaker method is how a wrong
join gets made and then trusted.

Both rows carry observed wavelengths, for the reason in the section above.

Both rows carry the same convention, or both have been converted to the internal
one. Air against vacuum is #11's, and a fallback match is exactly the place a
convention confusion turns into a wrong join that looks right.

The match is labelled `line-position-fallback` and that label travels with every
answer derived from it, in the same way as the component record above. It is
never rescued into an ordinary match.

Everything about the fallback is stated as its own decision because the failure
it invites is silent in both directions, which is the next section.

## The worked case

A tolerance wide enough to match one source's line to another source's version
of it is also wide enough to merge two different lines, and a tolerance narrow
enough to keep them apart drops real matches. This is the constructed case that
shows it. The numbers are constructed to make the arithmetic checkable rather
than taken from a catalogue; the regime is real and is the dense optical Fe I
forest.

Source A holds two genuinely different Fe I transitions:

    row A1   5000.100 A   lower 0.000 cm^-1  J=4   upper 19999.60 cm^-1  J=3
    row A2   5000.116 A   lower 415.933 cm^-1 J=3  upper 20415.55 cm^-1  J=2

Source B holds one row, its own measurement of the same transition as A1:

    row B1   5000.121 A   lower 0.000 cm^-1  J=4   upper 19999.60 cm^-1  J=3

Match on line position with any tolerance that can accommodate B's disagreement
with A about A1, which is 21 mA. Nearest neighbour picks A2, five mA away, over
A1, twenty one mA away. The join is made, it is wrong, and nothing about it looks
wrong: two rows agreeing to five mA is a better agreement than most of this
catalogue achieves.

Narrow the tolerance to ten mA and A1 is dropped instead. Both failures are
silent, and they are silent in opposite directions, so no single tolerance is
safe.

Match on levels and B1 joins A1, because both carry lower 0.000 cm^-1 with J=4
and upper 19999.60 cm^-1 with J=3, while A2 carries a different lower level and
a different J. The right answer, and it does not depend on a tolerance at all,
because J and the level energies are what the two sources are actually agreeing
about.

## What this does not decide

The internal representation of a line position, and where air and vacuum are
converted, which is #11.

How far apart two sources actually put the same line, which is the measurement
in #34 that replaces the 0.05 cm^-1 floor above.

What happens to identity when the species is a molecule, where the level
identity built here does not carry and the level density is far higher. That is
#66, and it is named here because a schema shaped only around this document will
be wrong in a way that is expensive to fix.

## Enforcement

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50. Nothing in the tree refuses a join
made on line position without the label, or an answer that drops the component
record. #50 is the issue for searches over the tree and two of its listed
invariants are neighbours of this one; the match record travelling with the
answer is a third.

## The means for this file

Markdown. The artefact is a decision read before the matching code exists, the
tree already carries Markdown, and it adds no language, runtime or dependency.
