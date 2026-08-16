# How an astrophysically calibrated value is marked

Decided for issue #15. An astrophysically calibrated value carries a mark, and
the mark binds what an answer that used one has to say and what the arithmetic
may not do with it.

## The circularity, stated once

One documented way out of poor line lists is to derive oscillator strengths
astrophysically: require that a model reproduce the spectrum of a reference
object, and take the values that make it do so. The reference object's parameters
were themselves determined from atomic data.

The circularity is known in the field, accepted, and recorded in no traceable
form. This board is in a position to record it, which means it has to decide what
recording it means, and that is the only thing this file decides.

Whether such claims are admitted at all by default is entry 6 of #1 and is the
maintainer's. This decision says what marking one means, so that whichever answer
comes back has something to attach to.

## The three fields, and why fewer is decoration

A claim derived astrophysically carries all of:

**The reference object it was calibrated against.** Which star, named the way the
source names it, resolvable to something a reader can look up.

**The parameters assumed for that object.** Effective temperature, surface
gravity, metallicity, microturbulence, whatever the calibration rested on. Not a
citation to where they might be found: the values used.

**Where those parameters came from.** The determination the parameters were taken
from, as a reference this board can point at.

Any two of the three is decoration. Object and parameters without their origin
says the calibration was done carefully and gives a reader no way to see whether
the parameters were themselves derived from the atomic data being calibrated,
which is the exact question the mark exists to make answerable. Object and origin
without the parameters cannot be checked against a later redetermination. And
parameters without the object are numbers about nothing.

So a claim marked as astrophysically calibrated and missing any of the three is
refused at ingest rather than stored with a gap. A partial mark is worse than no
mark, because it looks like the information is there.

## What an answer says

A derived quantity that used such a claim says so in the answer itself.

Not in a footnote in the documentation and not in a verbose mode nobody turns on.
The answer is the artefact that ends up in somebody else's table, and a
disclosure that does not travel with it is a disclosure that reaches the person
who already knew.

What it says: that a calibrated claim was used, which reference object it was
calibrated against, and where that object's parameters came from. That is the
minimum from which a reader can decide whether the answer is circular for their
purpose, which is a judgement this board does not make for them.

## What the arithmetic may not do

A calibrated claim is not marginalised together with the laboratory claims it was
calibrated against.

That is the circularity restated as a weighted mean: the calibrated value was
chosen to reproduce a spectrum modelled with those laboratory values, so the two
are not independent measurements of one quantity and averaging them narrows an
uncertainty that has not actually narrowed. `docs/decisions/shared-ancestry.md`
is the general form of that rule and this is one instance of it, named here
because this is the instance the field walks into.

Establishing the calibration relationship is not always possible. Sources do not
reliably document which laboratory values a calibration was run against, and
where the relationship cannot be established, that is recorded as not established
rather than assumed absent.

Those two are different states and the difference is the whole content of this
paragraph. Not established means the marginalisation is unsafe and the answer
says so. Assumed absent means the marginalisation went ahead and nobody was told.
A register that collapses them has the second while reporting the first.

## Enforcement

Nothing is refused today. There is no ingest and no claim record, so there is
nothing that can carry a calibration mark and nothing to refuse a partial one.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #23, for the three fields. The claim
record is where the mark lives, and #15's refusal belongs in the schema that
defines it rather than in a check beside it. #25 is where the constraint gets a
fixture that violates exactly it and a neighbour one field away that it does not
refuse, which for this rule means three fixtures, one per missing field.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #44, for the output rule. Nothing here
produces an answer yet.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #39, for the marginalisation rule. That
issue is the refusal to double count two values sharing an ancestor, and a
calibrated claim and its laboratory inputs are the case with the least documented
ancestry, so it is the hardest instance rather than a separate mechanism.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for whether a given claim is
astrophysically calibrated at all. That is read out of what a source says about
itself, and where a source does not say, no reading of this tree discovers it.
The mark is only as good as the ingest that set it, and #30 is where what each
source does not give is recorded field by field.

## What this does not decide

Whether calibrated claims are admitted by default, which is entry 6 of #1.

The weighting used when claims are marginalised, which is entry 7 of #1.

How the calibration relationship is detected where a source does not state it.
That is a research question rather than a schema question, and the honest state
until it is answered is the one this file makes representable: not established.
