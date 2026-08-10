# Repository layout, and the boundary inside it

Decided for issue #3. Present tense: this file states what the top level units
are, which side of one boundary each is on, and how to tell whether something has
been put on the wrong side.

## Why there is a boundary at all

Four sibling registers have the same shape as this one. All of them need
provenance edges, a value model that keeps competing claims apart, snapshot
pinning, uncertainty that survives arithmetic, and a way of saying what an answer
rests on. Building that five times produces five subtly different versions of one
idea, and the differences are found later by whoever first tries to compare two
of the registers.

Drawing the line later means drawing it through code that has already grown
across it, which is why it is drawn now, while the crate holds three modules.

## The test

Could a register of material parameters, or of measurement histories, use the
generic side unchanged?

If a type on the generic side mentions a wavelength, the line is in the wrong
place. That is the whole test and it is a judgement, which is why the mechanism
below refuses two symptoms of a wrong answer rather than claiming to make the
judgement itself.

## The units, and which side each is on

`src/register/` is the generic side. It holds `provenance`, which is sources,
snapshots and bibliographic references kept apart from one another. What is
coming here is claims, provenance edges, the machinery that refuses to collapse
two claims into one, and the reporting that names what an answer rested on. Those
are #23 and #25 and they are not here yet.

`src/spectroscopy/` is the domain side. It holds `species`, which is a species
and its ionisation stage with one canonical spelling, and `accuracy`, which turns
a published accuracy grade into a number without pretending it was measured. What
is coming here is energy levels, transitions, the air and vacuum conversion, the
upstream formats, oscillator strengths, and the propagation into an abundance.

`accuracy` is on the domain side and the argument for it is worth stating,
because the generic idea sitting next to it is what makes it look otherwise. A
bound that is not an estimate, an absent uncertainty and a derived one are shapes
any register needs, and when #13 records them they belong on the generic side.
The letter scale itself is not one of those shapes. It runs from AAA to E, it is
published by one family of sources, and a register of material parameters could
not use it unchanged. The scale is domain data; the representation it converts
into is not, and the two live apart.

`src/main.rs` is on neither side. It is the binary target, which exists so that
there is a release artefact to build twice and compare. What the command does is
#9. What it is called is entry 9 of #1, answered there: the command is
`linienbuch`, and the repository, the package and the command stay one name.

`src/bin/gate.rs` is on neither side. It is a development tool that runs the
checks, and it is not part of the library at all.

`tests/` is on neither side. It holds the default suite, which takes no network,
opens no display and needs no elevation, and `tests/integration/`, which is the
harness for the things that cannot meet those constraints. `docs/testing.md` is
where that is argued.

`docs/decisions/` holds the records, `.github/workflows/` and `.githooks/` hold
the gate, and none of them is on either side of this boundary.

## Where the generic part eventually lives

Not decided here, and answered on #1 as entry 4. It stays a module in this
repository, and it is lifted out when a second board needs the same core rather
than on a date. The trigger is the answer rather than a deferral of one.

Publishing it from here as a package four other projects depend on is what that
refuses. From the first import by anything outside this tree the boundary is
public and no longer movable, and the licence on a library is a decision every
dependent has to make for itself.

This decision requires only that the question can still be answered later without
a rewrite. That is what the boundary buys and it is the whole of what is being
claimed: nothing on the generic side depends on anything on the domain side, so
lifting it out is a move rather than a redesign.

## What refuses a crossing

`tests/layout.rs`, in the default suite. Two properties over `src/register/`.

No identifier there names a quantity, an object or a convention specific to
spectroscopy, against a list of the words somebody would reach for without
noticing which side they were on.

Nothing there refers to the domain side, which is the same failure arriving
through the type system instead of through a name.

Three bounds, and the check states them in its own source as well as here. The
word list is a floor: an idea spelled in a word it does not hold passes, and the
entry is added when the word arrives. It reads words rather than parsing Rust, so
an identifier assembled from fragments is not seen. And it reads code rather than
comments, deliberately, because explaining what the boundary excludes requires
naming what is on the other side of it, and a check that refused that would make
the boundary undocumentable in the place a reader looks for it.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for the test in the section above. Whether
a register of material parameters could use this side unchanged is a judgement
about meaning. No reading of this tree makes it, no check is owed, and the review
is where a wrong answer is caught. What the mechanism holds is the two symptoms,
which is less than the rule and is the part that can be held.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50, for the other direction. Nothing
refuses a generic idea implemented on the domain side, which is the mistake that
costs the sibling registers rather than this one: it does not break anything here
and it is why the four copies diverge. A search over the tree is the shape that
would catch a name, and no search catches a shape.
