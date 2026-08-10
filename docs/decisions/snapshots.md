# How an upstream snapshot is pinned, and how an answer names it

Decided for issue #19. Present tense: this file states what identifies a
retrieval, what may never be done to one, and what an answer has to say about the
ones it drew on.

## Why a version string is not enough

Upstream databases change. Values are corrected, versions are superseded, and a
line list is occasionally withdrawn. A documented comparison found groups drawing
atomic lines from different versions of one database without that being visible
in the results.

A version string is assigned by the upstream and is not always incremented when
the bytes change. So a claim that names a version names what the upstream said
about itself, which is a different statement from what the board actually read.

## The identity

A snapshot is identified by the **content digest of what was retrieved**.

Beside it, and part of the record rather than part of the identity: the retrieval
date, the exact request that produced it, and whatever version string the
upstream supplied.

The digest is the identity; the version string is metadata about it. Two
retrievals with the same digest are the same snapshot however far apart they
happened and whatever the upstream called itself on those two days. Two
retrievals with different digests are different snapshots even where the upstream
version string did not move, and that case is the one this decision exists for.

The version string is optional and its absence is a state. A source that supplied
no version is not the same as a source whose version this board failed to record,
and the two are stored differently so that the second can be repaired and the
first is not mistaken for a gap.

The request is stored exactly rather than approximately, because a retrieval that
can only be described cannot be repeated, and a snapshot nobody can repeat is a
digest of something that is gone.

## A snapshot is never mutated

A newer retrieval is a new snapshot. The register holds both.

That is what lets a result computed a year ago still be reproduced after the
upstream corrects a value: the claim points at the snapshot it was computed from,
and that snapshot still says what it said. Updating a snapshot in place would
make every earlier answer unreproducible in a way that leaves no trace, which is
the failure this whole milestone is about.

One source therefore holds many snapshots and each keeps its own identity.

## The pointer, and the output rule

Every claim points at the snapshot it came from. A claim with no snapshot is not
a weaker claim, it is not a claim: nothing can be said about where its number
came from or whether it is still what the upstream says.

Every answer names every snapshot it drew on. Not in a footnote in the
documentation, in the answer, because the answer is the artefact that ends up in
somebody else's table and the documentation is not.

Two answers computed from different snapshots are different answers even when the
numbers match. The output makes that visible rather than leaving a reader to
assume that two identical numbers are the same result.

## What already exists, and what does not

The provenance module carries the snapshot with the identity above: the digest,
the retrieval date, the exact request and the optional upstream version.
Sources, snapshots and bibliographic references are kept apart there rather than
collapsed, which is what makes the pointer from a claim meaningful when there is
a claim to hang it on.

The properties refused in the default suite today:

    cargo test --locked --test provenance_records
    test an_identical_retrieval_is_the_same_snapshot ... ok
    test one_source_holds_many_snapshots_and_each_keeps_its_own_identity ... ok
    test a_malformed_digest_is_refused ... ok
    test a_malformed_date_is_refused ... ok
    test the_register_refuses_a_dangling_or_contradicted_record ... ok
    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #23, for the pointer. There is no claim
record in this tree, so nothing carries a snapshot reference and nothing could
refuse one that is missing. #23 is the claim record and the requirement belongs
in its schema rather than in a check bolted on beside it, which is what the issue
means by enforced by the schema rather than by convention. #25 is where every
constraint stated across that milestone gets a fixture that proves it bites.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #44, for the output rule. Nothing here
produces an answer yet, so there is no output to inspect for the snapshots it
names.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #26, for the digest itself. Nothing in
this tree retrieves anything, so every snapshot that could exist today is one
somebody constructed by hand. The identity is refusable as a shape and is
refused; that the digest is a digest of what an upstream actually served is not
established until there is a retrieval that produced it.

## What this does not decide

Which sources are pinned and in what order, which is #26 and #65.

Where the retrieved bytes live. Entry 2 of #1 answers what this repository may
carry: an extract only where the upstream gives explicit permission to
redistribute it, and never bytes derived from a share-alike licence. Which
posture each source takes under that is #55. This decision is about the identity
of a retrieval and holds whichever way a source lands: a digest identifies a
retrieval whether the bytes sit in a local store, in this tree, or nowhere at
all.

How two snapshots of one source are compared, which is #34's measurement.
