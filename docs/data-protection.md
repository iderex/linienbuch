# Data protection

Where this program runs, what it keeps, and what it sends. The statement is in
the documentation and not only in the code, because a property nobody has
written down is one a later feature is not argued against.

Every sentence below is either backed by something in this tree or marked as
not backed, under `## What backs which sentence`. Nothing here is stated as an
assurance while the thing that would establish it is missing.

## What stays on the host

Personal data never leaves the host. The register, the query history, the
operator's identity and the subject of their work all stay on the machine the
program runs on.

There is no telemetry, no usage reporting, no update check and no error
submission. None of those arrives later as a setting that is off by default
either. Nothing is collected and nothing is sent, in any form, and that is the
standing answer and not a state of the tree that a feature could move: the
decision is recorded on #1 and carries no exception and no revisit condition.

## The outbound traffic

One kind of outbound traffic is intended and no other is. The retrieval of
upstream data happens when the operator asks for it, does not happen on its
own, and what it sends is limited to what the declared egress list holds.

That is the rule any retrieval arrives under. It is not a description of this
tree today, and the difference matters enough to state rather than to blur.
There is no retrieval in this program at all:

    git grep -n -i -E "TcpStream|TcpListener|UdpSocket|to_socket_addrs" origin/main -- src/ ; echo "exit=$?"
    exit=1

So the sentence that is true today is the narrower one, that this program opens
no socket, and the sentence about retrieval being the only outbound traffic is
the one it will need when #26 lands. The declared egress list is #56 and does
not exist yet, which is what `## What backs which sentence` records.

The suites that judge this program are not the program. The integration harness
in `tests/integration/main.rs` opens sockets on purpose, is excluded from the
default suite by construction rather than by a naming convention, and runs only
when somebody asks for it by name. `docs/testing.md` carries that boundary and
the argument for it. An operator running this program runs neither the harness
nor anything that reaches it, and a reader who finds a socket in the tree is
looking at the second of those two things.

## Federation, if it is ever built

Nothing here shares data between hosts, and nothing is planned that does. If a
feature is ever built that does, it is federation, and the following holds for
it.

It is off unless the operator turns it on. Turning it on is a deliberate act
rather than a default, an inherited setting, or a consequence of some other
option. Before it shares anything it says exactly what would be shared, in terms
the operator can check against what they hold, rather than in a category name.

This is written now, while nothing depends on it, so that a later feature has to
be argued against it rather than shipped past it. A rule written after the
feature is a rule the feature has already shaped.

## Why a query is sensitive

The queries are the sensitive part and it is not obvious, so it is stated rather
than left for a reader to notice.

A single query names a species and a transition, which on its own says little. A
sequence of them describes what an unpublished piece of research is about: which
element, which ionisation stage, which region of the spectrum, and in which
order the work moved through them. That is a description of somebody's
unpublished work, assembled from parts each of which looks harmless.

It is also why the rule that no network call happens at query time is a privacy
rule and not only a reproducibility one. Reproducibility is why the answer must
not depend on a remote service being up. Privacy is why the question must not
reach one at all, and the two arrive at the same rule from different directions,
so losing one of the reasons would not visibly weaken it until the other was
lost too.

## What backs which sentence

Backed. That this program opens no socket is backed by
`tests/environment_guard.rs`, which is part of the default suite. It reads every
`.rs` file under `src/` and `tests/`, looks for the API surfaces in
`tests/environment_guard/needles.txt` and refuses a line that names one without
a loopback address literal beside it. Its bounds are written in `docs/testing.md`
and are not softened here: the needle list is a floor rather than a boundary, a
call reached through a name it does not hold passes, a call split across two
lines is not seen, and two directories are outside its subject.

Backed, and narrower than it reads. The guard's subject is the source files of
the default suite, so what it establishes is a property of those files rather
than of every route by which this repository could reach a network.
`SECURITY.md` already records that gap against #56 and this document does not
restate the argument.

Not backed. The declared egress list does not exist. It is #56, and until it
lands there is nothing that says which destinations a retrieval is allowed to
reach, so the sentence about what a retrieval sends rests on no mechanism at
all. The run says the same thing rather than leaving it here:

    cargo run --locked --quiet --bin invariants
    invariants:   no network call site outside the declared egress list
    invariants:     waits on #56, which owns the list, and which is itself waiting on where a scanner over the tree is allowed to live

Not backed, and it cannot be. The federation paragraph and the paragraph about
why a query is sensitive are statements about what will not be built and about
why a rule exists. Neither is a property of an artefact, so nothing in this tree
could read either one.

## What is not enforced

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #56, for the outbound traffic. The
sentence naming the retrieval as the only outbound traffic has no mechanism
behind it, and neither does the sentence about what such a retrieval may send.
What is enforced today is narrower and is named above. When #56 lands, this
section says which of its checks reaches which sentence, and the sentences stop
being the wider claim.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for the federation rule. It is a rule
about a feature that does not exist, so there is nothing in the tree for a check
to read, and there will be nothing until such a feature is proposed. At that
point the rule is what the proposal is measured against by a person, which is
the review. Marking it changes nothing about its enforceability and no issue is
owed for it.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for the absence of telemetry as a
decision. The tree can be searched for a socket, and it is. It cannot be
searched for the absence of an intention to add one, and the decision on #1 is
the thing that holds here rather than any property of the current tree.

`PROSE, NOT ENFORCEMENT`, `TERMINAL`, for what an operator does with their own
store. Everything above is about what this program does. What leaves a machine
because somebody copied a file off it is outside anything this repository can
read or refuse, and saying so is the whole of what is owed.

## What this does not settle

Which sources are retrieved and how, which is #26, and what this repository may
carry from each of them, which is #54 and #55.

What an answer discloses about the claims behind it, which is #44.

The refusal of a network call at query time as a property of the program rather
than of its sources, which is #46.
