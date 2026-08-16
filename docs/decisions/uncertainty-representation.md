# How an uncertainty is represented

Decided for issue #13. An uncertainty is in one of four states. What may be done
with each, and what is kept from the source whatever this board derives, is set
out below.

## The four states

They are not interchangeable and none of them is a special case of another.

**Quoted.** The source gave a numerical uncertainty. It may be symmetric or not,
and an asymmetric one is stored as two numbers rather than as the larger of them,
because collapsing it loses the direction the value is uncertain in and that
direction is often the interesting half.

**A bound.** The source gave a maximum rather than an estimate. The accuracy
letters in the transition probability compilations are this: the published scale
runs from AAA at 0.3 per cent or better through to E for anything worse than 50
per cent, and each letter is a statement that the error is no larger than a
figure. A representation that stores only a number derived from a letter has
thrown away the fact that the published quantity was a maximum, which is exactly
what makes it not propagatable as a standard uncertainty without saying so.

**Absent.** No uncertainty was given. This is the common case for line
intensities in the largest compilation the field uses. It is not zero and it is
not infinity, and it is not a number at all. Any operation that needs a number
refuses it.

**Derived.** The uncertainty was computed from something else, for instance from
the number of decimal places in a quoted energy. That is a real practice and a
weak one, and a derived value carries what it was derived from, so a reader can
see that the number came from the typesetting rather than from a measurement.

## Turning a bound into something arithmetic can use

The published bound is read as one standard uncertainty. Conservative,
deliberately, and wrong in a stated direction rather than in an unstated one: a
maximum read as a one sigma overstates the uncertainty where the source's own
error distribution is anything narrower than the bound.

The alternative was a uniform distribution over the interval, which is a
different assumption with a different bias, and either is defensible. What is not
allowed is for the choice to be silent. `docs/decisions/accuracy-grades.md`
carries the argument in full and the code that does it; this file records that
the choice belongs to the representation and points there rather than restating a
table that would then drift.

Two consequences worth stating here because this is where somebody looks.

A grade that states no upper bound converts to no number at all. Not a large
number: anything worse than the last figure includes values wrong by a factor,
and a finite stand-in would be invented information propagating as data.

A primed grade marks that a multiplet was separated into components under a pure
LS coupling assumption, and the source says the real accuracy may be worse than
the letter and does not say by how much. The converted number is therefore a
lower bound on the uncertainty rather than an estimate of it, and it is marked as
one. A representation that stored only the derived percentage would have lost
that, which is the concrete reason the source's own statement is kept.

## The source's statement is always kept

Whatever this board derives, the source's own statement is stored verbatim beside
it.

That is what makes a change of convention a recomputation rather than a
re-ingest. If the reading of a bound is revisited, or the field settles on
something better than reading a maximum as a one sigma, every stored claim can be
recomputed from what the source actually said. Without the verbatim statement the
only route is to fetch every source again, which for a source that has since
changed is not a route at all.

## What refuses what

The accuracy module already carries the bound state and the refusal that comes
with it. A grade with no upper bound converts to `Unusable` rather
than to a number, a primed grade sets the flag that says the figure is a lower
bound, and the verbatim spelling is kept on the grade. Two properties are refused
in the default suite: no converted value is smaller than its own bound, and an
unrecognised suffix is refused rather than dropped.

    cargo test --locked --test accuracy_grade
    test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #23, for the other three states and for
the rule that absent is refused by any operation needing a number. There is no
claim record in this tree yet, so there is nothing that carries a quoted, an
absent or a derived uncertainty and nothing that could refuse an operation over
one. #23 is the claim record and #25 is the check that every constraint stated
across that milestone is refused by something that runs. Until they land, this
file is a decision that the code has not yet been written against, which is the
order this milestone is deliberately in.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for whether reading a maximum as a one
sigma is the right choice. That is a judgement about what a source meant, no
reading of this tree makes it, and the review is where a wrong answer is caught.
What is checkable is that the choice is written down, which is this file and the
one it points at.

## What this does not decide

How an uncertainty propagates into a derived quantity, which is #37, and what
happens when the regime cannot be established, which is #17.

How competing claims are weighted when they are marginalised, which is entry 7 of
#1 and is the maintainer's.

Whether a claim carrying only a lower bound may be marginalised at all. The rule
this file inherits is narrower and is the one already recorded: such a value may
not be used as a weight.
