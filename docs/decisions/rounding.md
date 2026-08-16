# How a value and its uncertainty are rounded

Decided for issue #40, before the first formatting path and not after five of
them. The rule, the reason for each half of it, and what it does not cover.

## The decision

An uncertainty is rounded away from zero. Never toward it, and never to nearest.

An uncertainty is shown to two significant figures.

A value is shown to the decimal place its uncertainty reaches, and no further.
The value itself is rounded to nearest.

All of it happens in one function, `render` in `src/register/rounding.rs`, and
nothing else in the crate turns an uncertainty into text.

## Why the direction is fixed in code

The uncertainty this board produces is larger than the one the field is used to
quoting, and it makes published precision look worse. That is the intended
result.

The pressure to soften it does not arrive as an argument about honesty. It
arrives as a small reasonable request about display precision, from somebody who
is right that the output looks untidy, and it is granted in one line somewhere
that formats a number to two places because two places looked tidy. Nobody
decides anything. A rule that lives in a paragraph is argued with once per
request; a rule that lives in one function that is searched for is argued with by
editing it, and an edit is a thing a reader can see.

So the direction is arithmetic and not a review habit. Rounding to nearest
reports a number smaller than the true one about half the time, and the half it
understates is the half a reader would want to have been told about.

## Why two significant figures

Three answers were available and they produce three different headline numbers,
which is why the choice is written down instead of arrived at by whichever
rounding call went in first.

**The number of figures the source quoted** fails the sentence it has to satisfy,
which is that the policy is stated once and applied everywhere. A marginalised
uncertainty over four competing sources has no source figure count, and neither
does one propagated from level energies. A policy that cannot be applied to the
board's own derived numbers is not a policy, and the gap would be filled by
whatever the first derived path happened to do.

**One significant figure** is the common convention and it is too coarse in the
place it matters most. Rounding away from zero at one figure turns 1.01 into 2,
which is an inflation of ninety-nine per cent. The usual patch, two figures where
the leading digit is one, is an admission that one figure was wrong there; a rule
with an exception in it is two rules, and the exception is the part people
remember incorrectly.

**Two significant figures** is what is adopted. It is what the guide to the
expression of uncertainty in measurement recommends as the upper end of what to
quote, it applies unchanged to a quoted uncertainty and to a derived one, and it
bounds the inflation that the fixed direction introduces: rounding away from zero
at two figures overstates by at most one part in the second digit, which is under
ten per cent everywhere and under one per cent for a leading digit of nine.

That bound is the reason the two halves of this rule belong together. The
direction is what makes the number honest and it costs something; the figure
count is chosen so that what it costs stays small.

## Why the value is not rounded the same way

Rounding a value away from zero is a bias in the value, which is a different
defect rather than a safer version of this one. A board that systematically moved
every value away from zero would be reporting a quantity nobody measured. The
value is rounded to nearest, in both directions, and the honesty property belongs
to the uncertainty alone.

The value stops where the uncertainty stops because a rendering that shows more
digits than the uncertainty reaches implies a precision that was not claimed.
That is the same failure as understating the uncertainty, arriving through the
other side of the pair.

## Two consequences that are easy to get wrong

**A carry moves the place.** Rounding 99.01 away from zero at two figures gives
100, whose second significant figure sits at tens rather than at units. The place
the value stops at is read off the uncertainty after it is rounded and not
before, or the value beside it shows a digit its uncertainty no longer reaches.

**A width of exactly zero constrains nothing.** It says the value is exact in
that direction. It does not say the value should be shown to no decimal places,
so it does not get a vote on where the rendering stops, and a pair of zero halves
leaves the value written out as it stands.

## Which states this covers

`docs/decisions/uncertainty-representation.md` names four states. Two of them are
in the type today, quoted and absent, and this rule is written against those two.

An absent uncertainty is rendered as absent, in words, rather than as an empty
place. An empty place beside a number reads as a small number.

A published bound and a derived uncertainty have no representation yet. When they
arrive this rule needs revisiting, because a bound is already a maximum and
rounding a maximum away from zero is a second conservatism on top of the first,
and a derived uncertainty carries what it was derived from and may want to say
so. Writing this file as though it covered four states would be the quiet kind of
wrong.

## What refuses what

The direction is a property of the shared rule and it is proven rather than
asserted. The proof is a sweep over the decades this board's quantities live in,
asserting that the number a reader parses back out of the rendering is not
smaller than the number that was rendered, together with the cases where rounding
to nearest and rounding away from zero give different answers:

    cargo test --locked --test uncertainty_rounding
    test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

That the rule is the only route is a search over `src/`, with a fixture that
violates it and a neighbour one call away that does not:

    cargo test --locked --test uncertainty_formatting
    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

Both of those have stated bounds and neither is a guarantee. The sweep proves the
property over the decades it sweeps and not over the whole range of `f64`, and it
asserts it on the parsed value rather than on the decimal, which is the number a
reader actually gets. The search reads words rather than parsing Rust, so a
formatting call split across two lines is not seen, and a name absent from its
vocabulary passes. Each file states its own bounds at the top.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for the figure count being the right one.
Whether two figures rather than one or three is correct is a judgement about what
a reader needs, no reading of this tree makes it, and the review is where a wrong
answer is caught. What is checkable is that the number is stated in one place and
that the code uses that constant, which the sweep asserts.

## What this does not decide

What an answer looks like on the wire. That is #44, and it takes the parts of a
rendering one at a time rather than rounding again.

How an uncertainty propagates into a derived quantity, which is #37, and how
competing sources are weighted when they are marginalised, which is #38 and whose
default is entry 7 of #1.

What happens to a claim carrying only a lower bound. The rule this file inherits
from `docs/decisions/uncertainty-representation.md` is that such a value may not
be used as a weight, and rendering it is not the same question.

## The means for this file

Markdown. The artefact is a decision somebody reads before writing an output
format, the tree already carries Markdown, and it adds no language, runtime or
dependency.

The means for the rule itself is Rust, in the crate that already holds the
uncertainty type. It carries the three rules: the direction is a refusable
property with a test that reds when it is broken, the proof is executed rather
than described, and every number in this file has the command that produced it.
It adds no dependency; the rounding is digit arithmetic over the shortest decimal
that reads back as the same number, which the standard library already produces.
