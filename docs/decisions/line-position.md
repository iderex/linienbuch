# One internal representation for line position

Decided for issue #11. A line position has one stored form inside this board.
Conversions happen at named places, and a value that arrived without saying what
it meant is handled below.

## The decision

A line position is stored as a **vacuum wavenumber in cm^-1**. Nothing else is
stored. An air wavelength is a view produced at the boundary of the program,
computed on the way out and never on the way in.

Three reasons, in the order they carry weight.

Level energies are already in cm^-1, and a transition's position is the
difference between two of them. Storing the position in the same unit means the
Ritz position and the observed position are the same kind of number and can be
compared without a conversion standing between them.

It is linear in the quantity the physics is about. An energy difference is a
wavenumber; a wavelength is its reciprocal, so an uncertainty that is symmetric
in one is not symmetric in the other. Storing the reciprocal makes every
uncertainty a small asymmetry nobody notices until it matters.

No source has to convert into it in order to be honest. A source that publishes
air wavelengths has to be converted whatever the internal representation is, and
a source that publishes vacuum wavenumbers is stored unchanged. The choice
therefore adds no conversion that some other choice would have avoided; it
minimises them.

## The conversion is data, not a constant

Which formula a source used is a property of that source, and reproducing that
source's own numbers means reproducing its formula rather than the one this board
prefers.

So a conversion is a named thing with an identity, attached to a source, and a
claim that was converted carries which conversion produced it. A single formula
compiled into the code would silently re-derive every source's numbers with one
group's arithmetic and would make disagreements between sources look smaller than
they are, which is the direction that flatters the board and is therefore the
direction to be most careful about.

The formulae this board has to carry, and who they belong to. The modified Edlen
expression of Birch and Downs, which is what VALD states it converts with. The
Morton 2000 expression, which the IAU adopted and which a large part of the
literature uses. The two differ by an amount that is small compared to a line and
not small compared to the tolerance a matcher needs, which is why the difference
has to be representable rather than averaged away.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #26, for both attributions. They are
taken from the citations in #11 and have not been re-fetched, because nothing in
this tree retrieves anything yet. #26 is what turns a transcription into a
quotation from something with a digest, and until it lands these are sentences
about pages rather than quotations from them. This is the same state
`docs/decisions/accuracy-grades.md` records for its own table.

## The three region convention, and the misreading of it

The NIST Atomic Spectra Database does not publish one convention. It publishes
vacuum wavelengths below 200 nm, standard air wavelengths between 200 nm and
2000 nm, and vacuum wavelengths again above 2000 nm.

The widely repeated version of that rule is that the boundary is at 2000
angstrom, which is 200 nm, and that everything above it is air. A converter
written against the repeated version gets the region above 2000 nm wrong: it
converts a vacuum wavelength as though it were an air wavelength. The result is a
plausible number, off by roughly the refractive index of air, in a part of the
spectrum where nobody has an intuition for the right answer. That is the whole
reason this decision exists rather than being left to whoever writes the parser.

So the region rule is a property of the source, written down where the source is
described, and the parser reads the position and the region together. A parser
that decides the convention from the number alone is applying somebody's
remembered rule, and this is the field where the remembered rule is wrong at one
end.

Same caveat as above: the three regions are as #11 cites them and are not
re-fetched here.

## A value that arrived without a stated convention

It is not converted and it is not assumed.

A position whose convention the source did not state is stored with its
convention recorded as unstated, and any operation that needs a vacuum wavenumber
refuses it rather than picking the likely one. That is the same shape as an
absent uncertainty in `docs/decisions/uncertainty-representation.md`: the missing
thing is a state, not a default, and arithmetic does not consume it silently.

The refusal is the whole point. Guessing is right most of the time here, which is
what makes it dangerous: a rule that is right most of the time produces a
register where a small number of positions are wrong and nothing marks which.

## What the type system carries

A vacuum wavenumber and an air wavelength are separate types. Adding one to the
other does not compile, and neither is convertible into the other without naming
which conversion is being used. `docs/decisions/means.md` records that this is
the strongest reason the implementation language was chosen, and this is the
decision that spends it.

## Enforcement

Nothing in the codebase stores a line position today, in either representation:

    git grep -n -i -w -E "wavelength|wavenumber" -- src/ ; echo "exit=$?"
    exit=1

So the rule that nothing stores a wavelength without its convention is true over
an empty set, and that is written here rather than presented as a tree that was
searched and found clean.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50, for the rule itself. A search that
refuses a stored position not accompanied by its convention is the shape #50
describes and is where this becomes refusable. Until it lands the guard is the
type, which refuses an assignment and cannot refuse a type nobody wrote.

## What this does not decide

The tolerance a matcher uses, which needs the measurement in #34.

Which conversion each source actually used, beyond the two named above. That is
established per source as sources are added, which is #65's procedure.

Whether an answer reports a position in wavenumber, in vacuum wavelength or in
air wavelength. That is an output format question and is #44. The rule this file
sets is only that whatever is reported is derived at that boundary and that the
answer names what it derived it with.
