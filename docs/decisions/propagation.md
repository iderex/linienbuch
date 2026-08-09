# How an uncertainty in log gf reaches the derived quantity

Decided for issue #17. Present tense: this file states what is propagated, the
two methods, the rule for choosing between them, what is checked before the
arithmetic runs, what happens when that check is inconclusive, and how
correlation enters.

## What is propagated

One input. The uncertainty a source quoted on log gf, carried on a claim.

An equivalent width measured by the caller is not an input. That is the
maintainer's decision recorded on #1 and on #17, and the reason it is a decision
rather than a scope note is below, because the shape of everything else here
follows from it.

Two further inputs are named by #17 and are not propagated for the first
release: the uncertainty in the line position and the uncertainty in the level
energies. Both enter a real abundance through quantities this board does not
hold. A level energy enters through the population of the lower level, which
needs a temperature, and a temperature is a property of the caller's atmosphere
rather than of anything a source published. So the treatment is the one #17 asks
for in its own words. Deciding they are negligible is allowed and deciding it
silently is not: the caller states it, the statement travels into the answer, and
without the statement the propagation refuses.

## The assumption everything rests on

The line absorption coefficient is proportional to the product of the lower
level population and the oscillator strength. The abundance enters through that
population and nowhere else, and f enters there and nowhere else.

Everything below is exactly as good as that assumption, and the section on the
four exceptions is where it stops holding.

## The derivation

Write the equivalent width as a function of the product, suppressing every
argument that involves neither the abundance nor the oscillator strength:

    W = G(N f)

An abundance is recovered by measuring W and inverting G at fixed W, so

    log N = G-inverse(W) - log f

and

    d log N / d log f = -1

exactly, with no reference to the shape of G. The statistical weight g of the
lower level is an integer that does not move, so d log gf and d log f are the
same quantity and the derivative in log gf is the same number.

The inversion of G is where a regime would have to enter, and log f sits outside
it. A shift in log gf moves the derived log abundance by the same amount in the
opposite direction whether the line is weak, saturated or on the damping part,
because gf and abundance are the same parameter as far as a curve of growth can
tell.

There is a check on this that does not depend on the algebra. Differential line
by line analysis, one star against another using the same line, is used because
gf errors cancel between the two sides. That cancellation needs the derivative
to be minus one and to be the same number on both sides, and the two stars do
not put the line at the same place on its curve. A regime dependent mapping
smaller than one on the saturated part would break the technique.

The standard result quoted for the damping part in the next section is taken as
published rather than derived here, and the derivation above is checkable by
reading and by nothing else. There is no curve of growth in this tree.

## What is regime dependent, and why it is not an input here

The amplification a reader expects here is real and it belongs to a different
input. It is

    d log N / d log W

which is one on the linear part, about two on the damping part where W goes as
the square root of the product, and grows without bound on the flat part, where a
large change in abundance moves W almost not at all and a small error in W is
read back as a large error in abundance.

That is the sense in which strong lines are where a propagation multiplying by
one goes wrong, and it attaches to the measured equivalent width rather than to
gf.

The equivalent width is not propagated here, and refusing it is the decision
this record rests on. A board that accepted it would be accepting an input for
which it holds no source, no snapshot and no provenance edge, against
`docs/decisions/claims-not-values.md`, and the claim that every number in an
answer traces back to a snapshot would fall, because part of the input would
come out of the caller's spectrum. Whoever wants a measured equivalent width
propagated does it outside this board.

## The four exceptions

The degeneracy is exact only under the assumption above, and four things break
it. They are named individually rather than as a caveat, because the check
before the arithmetic asks about each one separately.

**Continuous opacity or electron pressure.** An element that contributes to
either. Changing its abundance changes the atmosphere the line forms in, and
changing gf does not, so the two stop being one parameter.

**A blend.** The measured W has contributions from lines whose gf did not move
together.

**Departure from local thermodynamic equilibrium.** Level populations depend on
the radiation field and on f through separate terms rather than through one
product.

**Calibration feedback.** Where microturbulence, or any other parameter, is set
by requiring that derived abundance show no trend with line strength, a gf error
on a strong line moves the calibrated parameter and the moved parameter then
moves every line including the weak ones. This one is regime dependent, it is not
the curve of growth, and it is a correlation between the lines of one species,
which is the harder half of the correlation section below.

## The two methods, and the rule for choosing

**Analytic.** Apply the derivative. The map is a reflection about the shift, so
an asymmetric uncertainty comes out with its halves exchanged: what was the
upper half of the uncertainty in log gf is the lower half of the uncertainty in
log N. A propagation that carries the halves across unswapped is wrong in the
one case where anybody would notice, and it is wrong quietly, because the two
numbers are usually close.

**Monte Carlo.** Draw from the claim's distribution, map each draw, and recover
the two halves from the sample.

The rule for choosing between them. The analytic route is available exactly
where the exact degeneracy holds, which is every propagation this board performs
for the first release, so it is the route every answer is computed by. The Monte
Carlo route is the independent calculation the analytic route is checked
against, and it runs in the test suite rather than in an answer.

That is a narrower rule than #17 anticipated, and the narrowing is the point of
the decision above rather than a simplification. With the equivalent width out
of the inputs, the map is linear with slope minus one, and a linear map is the
case where an analytic propagation and a Monte Carlo agree by construction. The
worked example below therefore shows agreement, and a disagreement between the
two would be a defect in one of them rather than a property of the physics.

The Monte Carlo keeps its place for two reasons that survive the narrowing. It
is the only check that does not share an implementation with the thing it
checks, in particular for the exchange of the halves. And the moment an input
that is not linear in the answer arrives, it is the route that does not have to
be redesigned.

## The check before the arithmetic, and what it does when it cannot decide

The check is not about where the line sits on its curve of growth. Under the
derivation above the curve does not enter, and a check that asked about it would
be asking a question whose answer changes nothing.

What is checked is the four exceptions, one at a time, and whether the two
uncertainties this board does not propagate were declared negligible.

Each answer is in one of three states, and they are three rather than two on
purpose.

Established as not applying. The propagation proceeds and the answer records
that the caller established it.

Established as applying. The propagation refuses. The exact degeneracy does not
hold, and what would hold instead is not decided here.

Not established. The propagation refuses, and this is the state the record
exists for. The convenient branch is to treat an unestablished exception as one
that does not apply, because that is the common case, and taking it would put a
number in front of a reader that rests on an assumption nobody made. The
refusal names which of the four was not established.

A refusal is an answer. It is reported in place of the number rather than as an
error beside a number, which is the same rule `docs/decisions/shared-ancestry.md`
sets for a refused marginalisation and the one #43 says is most likely to be lost
when the output is written.

## How correlation enters

Two kinds, and they are not the same mechanism.

Correlation between claims about one transition is settled and is not reopened
here. `docs/decisions/shared-ancestry.md` requires pairwise disjoint ancestor
sets before a marginalisation, and `may_marginalise` in
`src/register/ancestry.rs` refuses everything else, including the case where the
ancestry is merely unresolved. A propagation over a set of claims inherits that
refusal rather than restating it.

Correlation between the lines of one species is the harder half and it has no
home yet. Shared ancestry is a property of a pair, resolved per combination. A
shared calibration across many lines is a property of a set of answers, and
treating the second with the machinery of the first would either refuse
everything or notice nothing. The fourth exception above is the concrete case,
and until there is a mechanism the honest treatment of it is the refusal that
exception already produces.

## What this does not decide

The weighting used once a marginalisation is permitted, which is #38 and whose
answer is recorded against entry 7 of #1.

What an answer looks like on the wire, which is #44. This record says what has to
be in it and not how it is spelled.

The derived quantity itself and the two numbers it is reported with, which is
#43.

## The worked example

One claim, propagated both ways. A source quoting log gf with an asymmetric
uncertainty, five hundredths of a dex low and twelve hundredths high, on a line
whose six preconditions the caller has answered.

    cargo test --locked --test propagation -- --nocapture the_two_methods_agree_on_the_worked_case
    quoted on log gf    lower 0.050000 upper 0.120000
    analytic            lower 0.120000 upper 0.050000
    sampled             lower 0.120107 upper 0.050158
    sampling            200000 draws, seed 20260809
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out

The two agree, which is what this record predicted once the equivalent width
left the inputs, and it is worth being clear about what that agreement is and is
not. It is not evidence that the derivative is minus one. A linear map is the
case where an analytic propagation and a Monte Carlo cannot disagree, so what
this example shows is that both implementations carry out the same linear map
and that neither dropped the exchange of the halves. The derivative itself
rests on the derivation above and on nothing that was run.

The exchange is the part the example is for. The lower half of the answer is the
upper half of the input, not the lower one, and the sampled route arrives there
without borrowing the analytic route's arithmetic: it draws, maps each draw
through the derivative and recovers the halves from the sample afterwards.

The sampled numbers sit nine parts in ten thousand from the exact lower half
and three parts in a thousand from the exact upper one, which is the sampling
error of the draw count rather than a disagreement. The tolerance the
suite holds is three per cent, and the smaller side of this case carries about
three parts in a thousand, so the band is about ten times the noise. A seed and a
draw count fix every digit above, so the numbers are reproducible rather than
merely repeatable.

What this example does not contain is a real line. There is no oscillator
strength in this tree to take one from, and a constructed pair of halves is what
the arithmetic can honestly be shown on today. When the first source is ingested
the same command runs against a value somebody published, and the number that
changes is the input rather than the rule.

## Enforcement

The refusals are standing. `src/spectroscopy/propagation.rs` refuses a
propagation whose preconditions are not all answered, one whose preconditions
were answered as not holding, an absent uncertainty on either route, and a
sampled run asked for no draws. `tests/propagation.rs` carries a case per
refusal, each fixture answering five preconditions and differing from a
propagating one by a single answer, and the pull request that landed them has
each site deleted with the tests that reddened.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #53 for the part that outlasts a
transcript. A deletion proof is evidence about the day somebody ran it. Nothing
in this tree connects those refusal sites to those fixtures afterwards, so an
arm that stops refusing stays green until a reader notices, and #53 is the issue
that ends that state.

The near miss is in the suite rather than only in this paragraph. A symmetric
uncertainty is the fixture an implementation that forgot the exchange passes,
for every value, which is why the case both routes are compared on is the
asymmetric one.

## The means for this file

Markdown. The artefact is a decision read before the code it governs exists, the
tree already carries Markdown for the other records, and it adds no language,
runtime or dependency.
