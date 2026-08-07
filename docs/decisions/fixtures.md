# Fixtures this repository is allowed to carry

Decided for issue #7. Present tense: this file states which bytes may land under
`tests/fixtures/`, what has to be written down beside them, and how they are
stored.

It is written before the first fixture rather than after the first hundred.
Committing a convenient extract of somebody's catalogue into a public repository
is cheap to do in an afternoon and expensive to undo, because the history keeps
it after the working tree has forgotten it.

## The three categories

Every fixture is in exactly one.

**Synthetic.** Bytes written by hand to the published format specification,
carrying no upstream values. The format is the thing being tested and the numbers
are invented. This is the category to reach for first, and the one that needs no
argument about terms.

**A redistributable extract.** Bytes taken from an upstream whose terms permit
redistribution. It carries the licence it is redistributed under and the command
that retrieved it, so that a reader can check the first and repeat the second.
The terms are established before the fixture lands, not after somebody asks.

**A structural stub.** Real layout with the numeric content replaced. Used where
the shape of the file matters and the values do not, which is most column and
alignment work. It is not a redistributable extract with the numbers changed as
an afterthought: what makes it a stub is that no upstream value is in it.

Anything that fits none of the three does not land. Where a parser genuinely
needs a real extract whose terms are unclear, the test moves to the integration
harness and reads from the operator's own local store rather than from this tree.
That harness already exists and already declares what each of its legs needs.

## The record beside each fixture

A fixture at `tests/fixtures/<name>` has a record at
`tests/fixtures/<name>.record.md`. A file whose name ends in `.record.md` is a
record; every other file under that directory is a fixture and owes one.

Fields at column zero, in the form `Field: value`.

`Category:` is one of `synthetic`, `redistributable-extract`, `structural-stub`.

`Encoding:` is `raw` or `base64`, and the rule for choosing is below.

`Licence:` and `Retrieved-with:` are required for `redistributable-extract` and
are refused as absent where it is that category. They are what turns "this came
from somewhere" into something a reader can check.

The body says what the fixture is for and what a parser is supposed to do with
it. `PROSE, NOT ENFORCEMENT`, `OWED`, issue #53. Nothing refuses a record whose
body is empty or says nothing useful. Whether a sentence describes the fixture is
a judgement about meaning and the review is where a wrong one is caught, but the
weaker property of there being a body at all is refusable and is not refused
today.

## The encoding rule

Fixture bytes must be exact. That is the whole reason a fixture exists: a parser
is being shown the bytes an upstream actually produces, and a fixture that has
been tidied on the way into the repository is showing it something else.

git rewrites line endings on the way into a working tree and on the way back, so
a carriage return written into a raw file is not a carriage return the parser
will see. That is measured in this tree rather than assumed. `.gitattributes`
carries the pin that stops it happening to the pre-push hook, and the measurement
that forced it is in the comment there.

So a fixture whose bytes matter beyond their printable content is stored
`base64`, and nothing between the author and the parser can rewrite it. A fixture
that is ordinary text with ordinary line endings is stored `raw`, which stays
readable in a diff, and that readability is worth having where it costs nothing.

What refuses a wrong choice, rather than what asks for the right one. A `raw`
fixture holding a carriage return, or a line ending in a space or a tab, is
refused: those are exactly the bytes that do not survive the trip, so a fixture
needing them is in the wrong encoding. A `base64` fixture whose content is not
base64 is refused too, because a fixture nobody can decode is not a fixture.

Neither refusal can tell whether a fixture that survives storage was the right
one to record raw. A file of ordinary text stored raw is legal and is meant to
be. What the guard removes is the case where the bytes silently changed.

## What is refused, and what is not

The check is `tests/fixture_policy.rs` and it runs in the default suite. It reads
the tree, needs no network and opens nothing.

It refuses a fixture with no record, a record naming no category or an unknown
one, a record naming no encoding or an unknown one, a redistributable extract
with no licence or no retrieval command, a raw fixture holding bytes that do not
survive storage, a base64 fixture that is not base64, and a record whose fixture
is not there. That last one is the direction that is easy to forget: a record
left behind after its fixture was deleted is a claim about a file that does not
exist.

It does not read licences. `Licence: whatever-i-like` passes, because deciding
whether a licence permits redistribution is a judgement about a legal text and no
reading of this tree makes it. `PROSE, NOT ENFORCEMENT`, `TERMINAL`, for that
half. The field being present is what is refusable, and the review is where a
wrong value is caught. The same holds for the retrieval command: it is required
to be there and is never run.

`tests/fixtures/` does not exist yet, so the check examines no fixture today and
says so in its own output rather than reporting a clean run over nothing. What
proves it refuses anything at all is a set of constructed trees under
`tests/fixture_policy/cases/`, one per refusal, each with a neighbour one change
away that is not refused.
