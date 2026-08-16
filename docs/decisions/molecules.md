# What changes when the species is a molecule

Written for issue #66. This file is the first half of that Done-when. The second
half, the part of the record model that carries the reference temperature and the
partition function identity alongside any intensity, is
`src/spectroscopy/intensity.rs`, and `## Enforcement` below says what it refuses
and what it does not.

The atomic case shapes the schema. If the molecular case is considered only
afterwards, the schema is wrong in a way that is expensive to fix, so the four
things that differ are decided before the record model is built and not after.

## Identity

A molecular transition is between rovibrational or rovibronic states, labelled
with quantum numbers whose set depends on the molecule and whose notation differs
between sources.

The atomic level identity in `docs/decisions/transition-identity.md` does not
carry that. It rests on a level energy within a tolerance, total angular momentum
exactly, parity exactly, and a configuration or term designation where both
sources give one. Two of those four have no molecular counterpart and the other
two are not enough on their own.

So the molecular level identity is its own identity and not a widened version of
the atomic one. It is the molecule, the isotopologue, the electronic state where
the transition is rovibronic, and the set of quantum numbers the source gives,
recorded as a set of named values, not as a fixed list of fields, because
the set differs by molecule and a schema with a column per quantum number would
be wrong for the next molecule.

The degradation the atomic case allows is refused here. An atomic match on level
energy and J alone is weak and is recorded as weak. A molecular match on energy
alone is not weak, it is meaningless, because the level density is far higher and
the number of levels within any usable energy tolerance is large. Where the
quantum numbers do not match, the transition does not join, and there is no
fallback that recovers it.

This is the first place the two cases genuinely diverge instead of differing in
degree, and it is the reason the identity is a variant rather than a field.

## Isotopologues

Separate species with separate values. A value for one is not a value for
another, and every source spells them differently.

This is already the rule in `docs/decisions/claims-not-values.md` and in the
species identity from #20, where the identity is an enumeration with one variant
today for exactly this reason: a molecular species arrives as a second variant,
and adding it makes every match over a species in the tree fail to compile until
it has been revisited. That is the mechanism doing the work, not a comment.

What the molecular case adds is that the isotopologue is part of the identity and
not an attribute of it. A record where the isotopologue is a nullable column
beside the formula allows a claim about an unspecified isotopologue, and there is
no such thing: a source that does not say which isotopologue it measured has told
you less than it appears to, and that is a gap to record, not a default to
fill.

## Intensity is not a property of a transition alone

The load bearing point, and the one that reaches furthest into the schema.

Three source families quote three different things. A millimetre catalogue quotes
a line intensity at a reference temperature in one set of units. An infrared
database quotes an intensity at a different reference temperature in another set.
A line list project quotes Einstein coefficients.

Converting between them requires the partition function at the reference
temperature, and different sources use different partition functions. The
conversion between two of these families is documented upstream at

    curl -s https://hitran.org/docs/jpl-cdms-conversion/

which issue #66 names. It was not fetched while writing this, for the same reason
as the accuracy table in `docs/decisions/accuracy-grades.md`: nothing in this tree
retrieves anything yet, and #26 is what makes a quotation from it a quotation
rather than a transcription.

So a comparison of intensities across those families is not a comparison of like
with like unless three things are recorded per claim. The reference temperature.
The units. And the identity of the partition function used, which is a reference
to a specific tabulation and not the name of a function.

The decision is that all three sit alongside any intensity as required parts of
the claim and never as optional metadata. A schema storing an intensity as a
number and a unit cannot express the disagreement between two sources, which
means it reports agreement where there is none, which is the failure this whole
board is against arriving through a door the atomic case never opens.

The consequence is a refusal, and it is the same shape as the refusal in
`docs/decisions/shared-ancestry.md`. Two intensities whose reference temperatures
differ, or whose partition function identities differ, are not comparable until
both conversions are recorded. The operation refuses and says which part is
missing instead of converting with an assumed partition function, because an
assumed partition function is an invented number that then propagates as data.

An Einstein coefficient is not in this family at all. It is a property of the
transition and needs no reference temperature, so it converts into an intensity
only in the direction that adds information, and the direction that removes it is
not a conversion this board makes silently.

## Scale

The assumptions behind `docs/decisions/storage.md` were checked against the
atomic case, and that record says so. A single molecular line list can hold order
1e10 transitions, which is one to two orders above the largest atomic case in the
calculation there.

The resolution is already in that record and is repeated here because this is
where somebody will look for it. Upstream files are not the register. A line list
with 1e10 transitions is a pinned snapshot that an ingest reads, and the register
holds what the board has actually matched and can say something about. What
follows from that is that the molecular case constrains ingest and matching
throughput rather than register size, and the numbers that would test it are
measurements #26 and #29 produce rather than estimates to put here.

The one thing this does change in the storage decision is the export. A columnar
export over a molecular subset is the case where predicate pushdown stops being a
convenience, which strengthens rather than revises what that record chose.

## What this does not decide

The molecular level identity's own tolerances, which need a measurement of how
far apart two sources put the same molecular line, and which is the molecular
half of #34.

Which molecular sources this board reads, which is #65.

Whether the register carries any upstream molecular data at all, which is entry 2
of #1.

## Enforcement

The typed half is held. `src/spectroscopy/intensity.rs` makes an intensity
without its reference temperature, its unit and its partition function identity
unrepresentable rather than invalid: the three are the parts of a `Convention`,
the fields are private, and the one constructor takes all three. Two intensities
in two conventions are refused and not converted, and the refusal names which
part differs.

    cargo test --locked --test intensity_conventions
    test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Eight constraints, each with a case that trips exactly it and a neighbour one
change away that it does not trip, and a test that reds if a constraint the type
declares has no case. What that file does not cover is printed by every run of it.

The species identity refuses a molecular species by name rather than accepting it
as an atom, which is the other piece a machine holds:

    cargo test --locked --test species_round_trip
    test an_unparseable_species_is_refused_with_its_reason ... ok

The refusal it asserts for `H2O` is `MoreThanOneElement`, which says the input is
a species this parser does not cover instead of nonsense, and points here.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50, for what is left. The type refuses an
intensity built without its convention; nothing refuses a second intensity type
appearing elsewhere in the tree, or an intensity formatted by a path that never
went through this one. That is a search over the tree rather than a property of a
type, and #50 is where it belongs.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for whether a recorded conversion factor is
the right factor. The record carries which tabulation was used so a reader can go
and check it, and nothing here evaluates a partition function. That is a judgement
about physics, no reading of this tree makes it, and no check is owed.

The molecular level identity in `## Identity` is not held by anything yet. There
is no level record for it to be part of, which is #21, and no transition record
for a level to be an end of, which is #22.

## The means for this file

Markdown. The artefact is a decision read before the record model is built, the
tree already carries Markdown, and it adds no language, runtime or dependency.
