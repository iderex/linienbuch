# Turning an accuracy grade into a number

Decided for issue #28.

## The one sentence the issue asks for

The published bound is read as one relative standard uncertainty, unchanged.

## The scale

A table of maxima, best first, in per cent:

    AAA 0.3    AA 1    A+ 2    A 3    B+ 7    B 10
    C+ 18      C 25    D+ 40   D 50   E worse than 50

These values are the ones issue #28 records from the source's help page, at

    curl -s https://physics.nist.gov/PhysRefData/ASD/Html/lineshelp.html

They were not re-fetched while writing this. Nothing in this tree retrieves
anything yet, and #26 is the issue that pins a snapshot and makes this table a
quotation from something with a digest rather than a transcription. Until it
lands, the table is a transcription and is written as one, in the source as well
as here.

## Why the choice is a decision and not a lookup

A maximum is not a standard deviation. The source says the value is within so
many per cent; it does not say what the distribution of the error inside that
range is. Anything the board does with the number after that point is an
assumption, and the assumption has to be visible.

Two readings are available and they differ by more than a factor of one and a
half.

Reading the bound as one standard uncertainty gives a standard uncertainty equal
to the bound.

Reading it as a uniform distribution over plus or minus the bound gives the
bound divided by the square root of three, which is about 58 per cent of it.

## The choice, and which direction it is conservative in

The first. The published bound is the standard uncertainty.

It is conservative, and the direction is worth being precise about. Treating a
maximum as one standard deviation puts roughly a third of the probability mass
outside a range the source said the value does not leave, so it reports more
uncertainty than the source's own statement implies. That is the safe direction
for this board: an answer that overstates its uncertainty is worse only in the
sense of being less useful, while one that understates it is wrong in a way that
propagates.

The uniform reading is rejected for a specific reason rather than a stylistic
one. It reports a smaller uncertainty than the number the source published, so an
answer built on it would claim to know more about the value than the source ever
said. A test asserts that no converted value is smaller than its own bound, and
the uniform reading fails it.

Neither reading is correct in the sense of recovering what the original authors
meant. Nobody knows what they meant, because the grade was assigned as a bound
and not as a distribution. What is available is a choice, stated.

## The unbounded grade

`E` is anything worse than fifty per cent, which includes values wrong by a
factor.

It converts to no number. Not to fifty, not to a hundred, not to a large
placeholder. A finite stand-in would be invented information that then
propagates exactly as if it were data, and the fact that it was invented would be
gone by the second arithmetic operation.

So the conversion has a second outcome and not a fallback. A grade of `E` yields
a statement that no number is available and what it is worse than, and anything
downstream that wanted a number has to handle that case rather than receive a
default. An answer carrying such a claim says so.

## The prime

A primed grade means the source split a multiplet into components under a pure LS
coupling assumption, and the source itself says the true accuracy may then be
worse than the letter.

It converts to the same number as the unprimed grade, because the source gives no
figure for how much worse and inventing one would be the same defect as inventing
a number for `E`.

What changes is the status of that number. For a primed grade the converted value
is a lower bound on the true uncertainty rather than an estimate of it, and it is
marked as such. A value marked that way may not be used to weight a
marginalisation. Weighting by an uncertainty that is only a lower bound rewards
whoever said least about their own error, which is the direction this board
exists to push against. The weighting itself is entry 7 of #1, answered there in
its shape: a weight by the category of the method, with every claim inside one
category weighing the same. The number each category carries is a table no file
in this tree holds yet. What is decided here is that this value is not eligible
for the weighting, whatever those numbers turn out to be.

The prime is never silently dropped. The parser refuses a suffix it does not
recognise rather than ignoring it, so a spelling it has not been taught reds the
ingest instead of quietly producing an unprimed grade. Which byte a served format
uses for the prime is not established here, for the same reason as the table
above, and #26 and #27 are what pin it.

## The grade survives verbatim

Whatever the conversion does, the spelling the grade arrived in is kept, exactly
as it arrived including surrounding whitespace.

That is what makes revisiting this decision a recomputation rather than a
re-ingest. Every argument above is a choice that could be made differently, and a
register that kept only the converted number would have to go back to the sources
to change its mind.

## What this does not decide

How the converted uncertainty propagates into a derived quantity, which is #37.

How competing claims are weighted when they are marginalised, which is entry 7 of
#1.

Whether a claim carrying only a lower bound may be marginalised at all once
weighting is settled. The rule here is narrower: it may not be used as a weight.

## Enforcement

The two properties that can be refused are refused, in the default suite. No
converted value is smaller than its own bound, and an unrecognised suffix is
refused rather than dropped. Both carry fixtures and both carry a neighbour one
change away that must be accepted.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for whether the choice at the top of this
file is the right one. That is a judgement about what a source meant by a letter,
no reading of this tree makes it, and the review is where a wrong answer is
caught. What is checkable is that the choice is written down, which is this file.

## The means for this file

Markdown, and Rust for the conversion beside it, as recorded in
`docs/decisions/means.md` for #2. The conversion is a type in the crate that
already exists rather than a table in a document, because a table in a document
cannot refuse anything and this one has two properties that a test does refuse.
