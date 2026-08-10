# The storage format for the register and for bulk export

Decided for issue #18.

## The two requirements, which pull apart

An operator has to be able to open the register with tools this project did not
write. A number in a format only this program reads is a number nobody can
check, which defeats the point of a provenance register. That argues for a
single file store in a well known format with a query language other tools
already speak.

The register also has to hold a large number of rows. The claim model in #12
multiplies row count by however many sources cover a transition, and some
upstream line lists are enormous before that multiplication. That argues for a
columnar format with compression and predicate pushdown.

Both requirements are real, so the answer is both, with a stated primary.

## The decision

The primary store is a single file SQLite database. It is what the program reads
and writes, it is the authority, and it is the thing an operator copies when they
want to copy the register.

The bulk export is Apache Parquet. It is derived from the register and is never
written back.

The direction of derivation is one way and it has no exceptions. Register to
export. A second writable copy of one dataset is a consistency problem that
arrives later and is expensive when it does, and the whole value of a provenance
register is that there is one place a claim lives.

An export is stamped with the commit of the program that produced it and the
snapshot identifiers of every source that contributed, so that a Parquet file
found on its own can be traced back rather than being a table of numbers with no
history. That is the same discipline as an answer naming its profile in #16,
applied to a file rather than to a row.

## Why SQLite rather than the alternatives

It is one file. Copying the register is copying a file, which matters because
the register is the thing two operators exchange when they want to compare
results.

It is transactional, so an ingest that fails halfway leaves the register as it
was rather than as it was becoming. Ingest is the operation most likely to fail
halfway, because it reads large files from sources this board does not control.

Its query language is one other tools already speak, and the tools that speak it
are not this project's.

A directory of Parquet files as the primary store was the main alternative and it
loses on the middle point. There is no transaction across files, so a partial
ingest is a directory in a state nobody defined, and the repair is a convention
this project would have to invent and maintain.

A bespoke binary format loses on every point and is named here only so that the
record shows it was considered and why it was not chosen: it would make every
number in the register unreadable without this program, which is the failure the
first requirement is about.

## Why Parquet for the export

The analysis case is a scan over many rows and few columns, which is what
columnar storage and predicate pushdown are for, and which is the case SQLite is
worst at.

It is read by the tools the readers of this data already use, which is the same
argument as the first requirement and reaches a different population.

## The expected row count, and the calculation behind it

    claims = transitions * sources_per_transition * quantities_per_claim_set

Every input below is an estimate. None of them was measured, no command in this
tree produced any of them, and they are written as estimates rather than as
figures. #26 pins the first upstream snapshot and #34 measures how the sources
actually overlap; each of those replaces one input here with a number that
carries a command.

`transitions`. For a first release covering the species in #32's worked chain
and the optical set a critically assessed compilation covers, order 1e5 to 1e6.
For an ingest of a full computed atomic line list, order 1e8. Estimate.

`sources_per_transition`. Two to four, since the transitions this board is most
interested in are the ones more than one source covers, and the ones covered by
a single source contribute one claim each and pull the average down. Estimate.

`quantities_per_claim_set`. Two. A claim about the transition probability or
oscillator strength, and a claim about the line position, which disagree
independently and are therefore separate claims rather than fields of one.
Estimate.

So the first release register is order 1e5 times 2 times 2, which is order 1e6.
A full atomic ingest is order 1e8 times 4 times 2, which is order 1e9.

One order of magnitude of that range is comfortable for a single SQLite file and
the other needs care about indexing and about what is held in memory during an
ingest. Both are within what the format is used for elsewhere. SQLite's
documented maximum database size is far above either, and the figure is
deliberately not quoted here, because no command in this tree produced it and a
number in a document that nobody can re-derive is the defect this repository is
built against. Quote it from the version that gets pinned, when one is pinned.

The molecular case is outside this calculation and breaks its assumptions rather
than stretching them. A single molecular line list can hold order 1e10
transitions on its own, which is one to two orders above the largest case above,
and #66 names scale as one of the four things that differ. The resolution is in
the next section rather than in a larger number here.

## What is not stored

Upstream files are not the register. A retrieved snapshot of a source is an
input to an ingest, and what lands in the register is claims derived from it
plus the snapshot identity that lets an answer name where it came from.

This is what keeps the calculation above honest, and it is what the molecular
case rests on. A line list with 1e10 transitions is not 1e10 register rows
unless the board has ingested and matched all of them; it is a pinned snapshot
that an ingest reads. What the register holds is what the board has actually
matched and can say something about.

Whether any upstream file is carried in this repository at all is entry 2 of #1,
answered there: a curated extract may be tracked only where the upstream gives
explicit permission to redistribute it, and bytes derived from a share-alike
licence stay out whatever else is true of them. Being publicly retrievable is
not a permission. This decision is compatible with that and with the postures it
leaves standing, because none of them changes what the register holds; they
change where the snapshot lives.

## The tools an operator can open the register with

Named, because "a well known format" is an assurance and a list of tools is a
statement somebody can check.

For the SQLite register: the `sqlite3` command line shell; Python's `sqlite3`
module, which is in its standard library and needs nothing installed; R through
RSQLite; DB Browser for SQLite for a graphical view; DuckDB, which reads SQLite
files directly.

For the Parquet export: DuckDB; pandas and Polars through Apache Arrow; Apache
Arrow itself in C++, Rust, Java and Go; and the JVM analytics tools that read
Parquet natively.

None of these is a dependency of this project. They are what an operator already
has or can get, which is the point of choosing formats rather than inventing
one.

## What this does not decide

The schema. What the tables are and what the columns mean is #20 through #24,
constrained by #12, #10 and #14.

Whether the register is one file or one file per species, which is a question
about ingest and query performance that should be answered against a measurement
rather than in advance.

The pinning mechanism for upstream snapshots, which is #19.

## Enforcement

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #50. Nothing refuses a write to the
export or a second writable store, and the one way direction of derivation is
the kind of rule a search over the tree can hold. Nothing in the tree writes
anything yet, so there is nothing to refuse today, and the search lands with the
storage code rather than before it.

## The means for this file

Markdown. The artefact is a decision read before the store exists, the tree
already carries Markdown, and it adds no language, runtime or dependency. The
means for the store itself is decided above and is not Markdown's question.
