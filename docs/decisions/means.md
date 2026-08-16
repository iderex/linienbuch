# The implementation language and toolchain

Decided for issue #2. The repository is built in Rust, and the reasons follow
the decision.

## The decision

The core is Rust. The build and test driver is Cargo. The compiler version is
pinned in a tracked file, not taken from whatever the machine has, and
the dependency set is locked, so a build is a function of the commit and not of
the day it ran. Issue #4 owns the pin, the lock and the demonstration that two
builds agree. What is decided here is the choice, not its proof.

No second language is added to the core. Whether a Python facing binding is
added beside it is entry 3 of #1, answered there for the first release: none.
The route into Python is the register file and the export beside it, which
`docs/decisions/storage.md` already names, and not a set of function calls.
What that costs is named with it: the answer logic, the profile and the refusal,
reaches a Python caller only through the command.

The version this decision was made against:

    rustc --version
    rustc 1.97.0 (2d8144b78 2026-07-07)
    cargo --version
    cargo 1.97.0 (c980f4866 2026-06-30)

## What the decision had to carry

### Conventions that look alike and are not

Five pairs in this field look alike and are not: air wavelength against vacuum
wavelength, gf against log gf, Einstein A against oscillator strength,
wavenumber in cm^-1 against energy in eV, and angstrom against nanometre. Each
pair produces a plausible number when confused, which is why a
test written by somebody who already understands the distinction does not catch
it. Rust gives two things against this class. A wrapper type over a float
carries no arithmetic it is not given, so adding a vacuum wavenumber to an air
wavelength does not compile, and no implicit widening exists anywhere in the
language to route around it. Matching on an enumeration is exhaustive, so adding
a source, a method class or a wavelength convention forces every place that
decides on one to be revisited instead of falling through a default arm.

The bound on that, stated and not left implied. A type system refuses only
what somebody modelled as distinct types. It does not discover a convention
nobody encoded, and a field parsed into a bare `f64` is exactly as dangerous
here as anywhere else. What the language buys is that the discipline is
available and mechanical once applied. Whether it was applied to a given field
is what review and #50 are for.

### An absent value is not a value

The failure this board exists to object to is a wrong number that looks right.
A language whose uninitialised numeric field reads as zero converts a missing
log gf into a real measurement of a very weak line, silently, at the point of
parsing, and every downstream propagation then treats it as data. Rust has no
zero value: an absent field is `Option` and cannot be read as a number without
the reader saying what happens when it is absent. This is the single strongest
line in favour of the choice and it applies to the register more than to the
parsers, because #12 and #13 commit the board to representing absent and graded
uncertainty as first class states, not as sentinel numbers.

### Upstream files are large

Some line lists run to tens of gigabytes uncompressed, so parsing streams rather
than loads. Rust has no garbage collector, so the resident set is a consequence
of what the program is holding and not of when a collector last ran, which
means a memory ceiling can be written as a bound on the buffers a parser owns
and then tested against. In a collected runtime the same ceiling is a tuning
parameter that holds on the machine it was tuned on.

That is an argument about what can be measured, not a measurement. No parser
exists yet and no ceiling has been observed. #31 and the ingest milestone are
where a number appears, and it will carry the command that produced it.

### The artefact an operator runs

Somebody reproducing a number in three years will not have this machine.
Cargo produces a native executable with no interpreter and no dependency tree to
resolve at run time, which is the shape that survives that gap when paired with
a pinned input snapshot. The honest limit is that "self contained" is a property
of a target and not of the language: the platform C runtime is still linked
in the usual configuration, and which target triples the release names, with
what linkage, belongs to #60 and is not claimed here.

### Reproducible from a command

Cargo has a lockfile in the tree and a `--locked` mode that refuses to resolve
anything the lock does not already name, so a new transitive version cannot
arrive between two builds of one commit. The toolchain file pins the compiler in
the same tree, so a compiler upgrade arrives as a commit. Path remapping in the
release profile is what keeps the build directory out of the bytes.

Those are mechanisms, and this file asserts only that the language offers them.
Whether two clean clones at one commit actually produce byte identical
artefacts is #4's Done-when, is not asserted here, and has not been measured at
the time of writing.

### Tests without a display, without elevation, without the network

`cargo test` is in the toolchain and not beside it, needs no display server
and no elevated account, and a fresh crate builds with the network unavailable
to it:

    cargo build --offline
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.25s

Property testing is a library and not a language feature in every candidate
here, so it is not a discriminator. Coverage guided fuzzing is, because #31
requires a fuzz target for every upstream parser. Rust's usual answer is
cargo-fuzz, and it is not installed on the machine this decision was written on:

    cargo fuzz --version
    error: no such command: `fuzz`

So the fuzzing requirement is carried as a cost and not as a solved point.
What it costs to install it, on which platforms, and whether the release
platforms in #60 all support it, is #31's to answer. Naming it here as available
would be a claim nothing backs.

## The candidates that lost

### Go

The strongest of the three, and the one the argument had to be made against
rather than around. Go refuses arithmetic between distinct named types just as
Rust does, so the headline convention argument does not separate them, and Go's
build is simpler, its binary is more genuinely self contained, and coverage
guided fuzzing is in `go test` and not in a separate tool.

It loses on two specific points. The zero value, above: a struct field that was
never set reads as `0.0` instead of refusing to be read, which is the exact
shape of the defect this board is built to expose. And the absence of exhaustive
matching, which matters for a register whose whole design is a growing set of
sources, method classes and conventions; in Go a new variant compiles everywhere
and takes the default branch, and nothing points at the places that now decide
wrongly.

### Python

Where the readers of this data already work, which is a real pull and the reason
entry 3 of #1 exists. It loses the core on three counts that are not close. It
ships a runtime the operator has to have. It cannot refuse a unit confusion
before the program runs, so the entire first argument above becomes a test suite
that has to enumerate the confusions somebody thought of. And the reproducible
build requirement is considerably harder, because the artefact is a source tree
plus an environment rather than bytes.

Rejecting it for the core says nothing about a binding. A binding over a fixed
core is a different question with different costs, and it is #1's.

### C++

Has the numeric libraries and loses nearly everywhere else. No lockfile and no
dependency resolution in the toolchain, so the reproducibility requirement is
solved by adopting a third party package manager and pinning that too. No test
harness and no fuzzing story in the standard toolchain. And the parsers here
read untrusted bytes from external catalogues, which is the worst place to spend
memory safety, since #31 is committing to fuzz exactly those paths and a crash
found there would be a vulnerability rather than a bug.

## The means check for this file

Markdown, because the artefact is prose that a contributor reads, the tree
already carries Markdown and adds no language, runtime or dependency to do so,
and nothing outside this repository forces another format.

## What would overturn this

Written as conditions and not as sentiment, so that a later reader can check
whether one has occurred.

Entry 3 of #1 is answered such that Python is the primary interface rather than
a binding over a fixed core. That makes the core language the second language,
and the whole comparison is re-run with the weights reversed.

A required upstream format turns out to have only a C or a Python reference
implementation, and reimplementing it is judged a larger risk than the runtime
it drags in. Then the forced means wins for that surface and only that surface,
held to its smallest boundary, and the exception is recorded here without
replacing the decision.

The fuzzing cost in #31 comes back as unpayable on a platform #60 names, and a
candidate exists that pays it. The first half alone does not overturn anything;
the second half is what makes it a decision rather than a complaint.

A measured parse of a real upstream file fails to hold the memory ceiling set by
the ingest milestone, and the cause is a property of the language rather than of
the parser. This is the least likely of the four and is listed because leaving
it out would make the list read as unfalsifiable.
