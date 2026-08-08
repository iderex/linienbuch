# Security policy

Written for issue #58.

## Reporting a flaw

Use the private route:

    https://github.com/iderex/linienbuch/security/advisories/new

Do not open an issue on the public tracker for something that is exploitable.
The tracker is world readable, so an issue describing a flaw publishes it to
everyone who might use it before there is anything to update to.

Private reporting is enabled on this repository. Read rather than assumed:

    gh api repos/iderex/linienbuch/private-vulnerability-reporting
    {"enabled":true}

If that route is unavailable to you, open an issue that says only that you have
something to report and gives no detail, and a private channel will be arranged
from there.

## What is in scope

Three areas, and they are named for what this repository is going to hold as
well as for what it holds today.

The parsers. Everything that turns bytes this repository did not write into a
record. That includes upstream line lists and catalogue output, and it includes
text an operator hands to the command.

The retrieval path. Everything that reaches a source, everything that decides
what came back is what was asked for, and everything that stores it.

Anything that writes to the operator's filesystem. A path assembled from an
upstream field is the case worth looking at first.

What of that exists today is a shorter list than the paragraphs above, and
saying so here is the point of writing them down before the code arrives.

There is no retrieval path. Nothing under `src/` opens a socket, which the
default suite refuses rather than the sentence asserting it:

    cargo test --locked --test environment_guard
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

`docs/testing.md` states what that guard sees and what it does not, and the
limits there are limits on this sentence too.

There is no upstream parser. Two parsers exist and both read text an operator
supplies rather than bytes from a source:

    git grep -n 'pub fn parse' -- src/
    src/spectroscopy/accuracy.rs:134:    pub fn parse(text: &str) -> Result<Self, NotAGrade> {
    src/spectroscopy/species.rs:169:    pub fn parse(self, text: &str) -> Result<Species, Unparseable> {

Both are in scope. They are the smaller case, and they are the ones a report can
be about now.

## What is not a vulnerability here

A value that disagrees with another source. That is the subject matter of this
repository rather than a defect in it. Two databases putting one line in two
places, or quoting two oscillator strengths, is the disagreement the board exists
to show, and a report saying one of them is wrong is a finding about the field.
It goes on the public tracker like any other finding, with the source, the
snapshot and the command that produced the numbers.

An uncertainty that is larger than the one you are used to seeing quoted. The
propagation here is intended to produce a larger number than a single source
does, and `docs/decisions/uncertainty-representation.md` is where that is argued.

A source that changed its format, moved, or went away. `docs/testing.md` says
why that is information about the field rather than a broken test, and it is an
issue rather than a report.

A missing licence on this repository. It is known, it is entry 1 of issue #1,
and it is the maintainer's to answer.

## What a reporter can expect

The report is read.

No response time is promised. This is a small repository with one maintainer,
and a stated number of hours or days would be a commitment it cannot keep. A
policy that promises one and then misses it teaches the next reporter that
nothing here is worth reading.

A fix, if there is one to make, lands the way everything else here lands: an
issue that says what is wrong and what done means, and a pull request against
it. The issue is public once the fix exists, because the record of what was
wrong is worth more to a reader than the few days of quiet are worth to this
repository.

Credit in the advisory, under whatever name you give, unless you ask for none.

No payment. There is no bounty programme and there is not going to be one.

## What is enforced, and what is written down

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for everything in `## Reporting a flaw`.
Nothing in this repository can refuse a report that arrives in the wrong place.
The private route exists because of a repository setting, quoted above, and that
setting cannot stop somebody opening a public issue instead. No check reads this
file, and none could read the thing this section is about, which is what a person
outside the repository chooses to do. No issue is owed for it.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #56, for the scope. The claim that
nothing on the query path reaches the network is today a claim about the default
suite's sources, which is what the guard from #6 reads. The declared egress list
in #56 is what makes it a property of the tree rather than of the suite, and
until that lands this section rests on a check whose subject is narrower than the
sentence it supports.
