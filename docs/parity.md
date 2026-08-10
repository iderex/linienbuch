# Parity with the target gate

Decided for issue #47. Present tense: this file says what this board's merge gate
is measured against, where each check of that gate lands here, and why each
deviation is one.

## The target

The merge gate of `iderex/jellyfin-plugin-sso`.

It is the target for its shape rather than for its contents. Three properties,
and each is why copying it is worth doing at all.

Its ruleset is active and has no bypass actors, so the maintainer is inside it
too:

    gh api repos/iderex/jellyfin-plugin-sso/rulesets/18802863 \
      --jq '{enforcement, bypass: .bypass_actors}'
    {"bypass":[],"enforcement":"active"}

Its required set is built from first party checks rather than from a third party
review service, so nothing in the merge path is a runtime operated by somebody
else.

Its required set is matched by literal check run name. A check renamed on one
side and not the other stops being required, and nothing anywhere says so. That
is why the names this board's scaffolding milestone produced were fixed before
anything was written, and it is the reason a row below can be satisfied by a
workflow that exists and still fail on the name.

## Neither required set is listed here

The target's is printed by:

    gh api repos/iderex/jellyfin-plugin-sso/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context'

This board's is printed by:

    gh api repos/iderex/linienbuch/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context'

Those two commands are the authority for what either forge requires, and this
file restates neither. A list in a document drifts against the thing it
describes, and the drift is found by whoever trusted the document. The rows below
name checks and say where each one lands. They are not a statement of what is
required today.

What the second command returns matters for how the rest of this file is read. At
the commit this file landed on it returned nothing, because this board's ruleset
carries no required status checks at all:

    gh api repos/iderex/linienbuch/rulesets/20528058 \
      --jq '{enforcement, bypass: .bypass_actors, rules: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","rules":["deletion","non_fast_forward","pull_request"]}

So the map is a plan for a gate rather than a description of one, and a red check
on a pull request here refuses nothing. Issue #48 is what makes the checks
already in this tree required. Nothing in this file changes that, and reading the
map as a gate that is standing would be reading a plan as a measurement.

## How many categories there are

Seven headings follow. #47's done condition says five.

The two are not reconciled here and the seven are kept, because each heading
draws a distinction the issue draws on purpose and folding any two of them
together loses one. Which five the done condition meant is not recoverable from
the issue text, and picking five by guess and dropping the rest quietly would be
the worse error of the two available.

This is the one place this file departs from its done condition as literally
written, and it is written at the departure rather than left for a reader to
find.

## Adopted unchanged

The purpose and the implementation both carry over, because neither depends on
the language or on what the artefact is. Each of these already exists in this
tree, under the target's own name, which is what makes them adoptions rather than
plans:

    gh api repos/iderex/linienbuch/contents/.github/workflows --jq '.[].name'

`DCO sign-off`. A sign off is a property of a commit message, and a commit
message is the same object on both boards.

`Reject Trojan Source Unicode`. A bidirectional control character hides the same
thing in either language, and the scan reads bytes rather than syntax.

`Audit workflows (zizmor)`. Its subject is the workflow files, which are the same
format here as there.

`dependency-review`. Its subject is the dependency manifest diff, which the forge
reads for both ecosystems.

## Adopted under the same name with a different implementation

The purpose does not depend on the language and the command that serves it does.

`build`. Refusing a tree that does not compile cleanly is the same purpose in any
language, and this board's build leg is the one its gate command already runs.

`Enforce greppable invariants`. A search over tracked text that either matches or
does not is language independent in shape, and every invariant it searches for is
this board's own. It is #50 here.

`Deterministic PR-hygiene checks`. It reasons about the change rather than about
the code, so nothing in it is language specific, and the rules in its failing
tier are this board's. It is #49 here.

## Adopted with an adaptation

The purpose survives and the thing being checked does not exist here, so the
check is pointed at this board's nearest equivalent.

`ABI floor build`. The target's floor compiles against the oldest host
application interface it supports; this board plugs into no host, so the floor
becomes the oldest toolchain version it declares support for.

`Package (JPRM) / Build package`. The target builds a plugin archive for a
marketplace; this board ships binaries, so the check builds those. It is #51
here.

`Package (JPRM) / Generate SBOM`. The bill of materials is generated from
whichever build ships, so it follows the packaging row above rather than being
assembled on a separate path, which would describe a different artefact. It is
#51 here.

`prettier`. The target's formatting check covers web assets and this board has
none, so the same tool runs over the Markdown, JSON and YAML this tree does
carry.

## Adopted but resolved separately

The purpose carries over, the answer is language specific, and the row names the
issue that decides it rather than an implementation.

`CodeQL`. Static analysis, and which analyser fits this board's language is #52.

`Analyze (csharp)`. The same check's language specific job, whose equivalent here
is whatever #52 decides rather than a job named for a language this board does
not use.

## Dropped

Present on the target and with no analogue here, because each exists to ship a
plugin into somebody else's application through a marketplace. This board ships a
binary an operator runs.

The end to end login run. There is no host application to log in to.

Marketplace manifest freshness. There is no marketplace manifest.

Beta publication. There is no channel to publish a beta into.

Wiki lint. There is no wiki.

None of the four is in the target's required set today. They are dropped from the
map rather than from a required set, and that is why this section names them in
prose rather than by check run name.

## Added beyond the target

Present here and not there, each with the reason it is worth a required check.

A separate test check. The target runs its tests inside its build check, and a
red compile is then indistinguishable from a red test.

A dependency licence and advisory check. This repository's own licence exposure
interacts with the terms of the data it reads, which is a risk the target does
not carry.

A schema check. The register's constraints are this board's equivalent of a type
error, and nothing else in the gate would catch one.

## Carried as non gating

Run, reported, and never required, exactly as the target runs them.

Supply chain scoring. Already on a schedule in this tree, and a score is a report
rather than a verdict.

Fuzzing. Weekly and on request, because a fuzzing run has no bounded duration and
a merge must not wait on one. It is #31 here.

Mutation testing. Scoped and reported rather than enforced, because a low score
names a missing test rather than a broken change. It is #41 here.

## What refuses a row going missing

`tests/parity_map.rs` reads this file and takes the check run name out of every
row that carries one. It refuses three things: a name placed under two of the
headings above, a name placed under one of the three headings that name no check
run, and a heading renamed away from what the parser looks under, which would
otherwise take its rows out of every comparison in silence. It runs in the
default suite and reaches nothing off this machine.

The comparison against the target's live required set is in the integration
harness, in `the_target_required_set_is_covered_by_the_parity_map`, because it
needs the network and the default suite has none. A name the target requires and
this file does not place fails that leg with the name printed.

It is the live set that the leg reads rather than a copy of it, and that is
deliberate. A pinned copy of somebody else's required set is the nearest thing to
hand rather than the thing itself, and a claim about the target made from a copy
is the failure `CONTRIBUTING.md` calls the canonical one.

The cost is that the refusal sits outside the gate. Nothing in the merge path
runs it, so a check added to the target's required set today is not found until
somebody runs:

    cargo test --test integration

That is stated rather than softened. The offline half runs on every change and
holds this file internally consistent; the half that compares against the target
runs when it is asked for.

One direction only, and that is not an omission. A name the target requires and
this file does not place is refused. A row here naming a check the target does not
require is not, because the dropped rows and the non gating rows are exactly that,
and refusing them would refuse the map for being complete.

`PROSE, NOT ENFORCEMENT`, `OWED`, issue #48, for the map as a whole. Nothing makes
any row's check required on this board, so a row can be satisfied by a workflow
nobody runs and neither half above would notice. #48 is the issue that turns the
plan into a gate.

## What this does not decide

Which analyser serves the static analysis row, which is #52.

What this board's own check run names are. The rows name the target's names,
because those are what the map is placed against. The name each check reports
here is decided where that check is built, and #48 is where the two are made to
agree.

Nothing about runner cost. Entry 8 of #1 is answered and closed there: every
workflow in this tree runs on the standard runner, this repository is public, and
no run is billed. It becomes a decision again on the day a run leaves
`ubuntu-latest`, and that is the trigger rather than a review date.

What the answer rules out belongs beside it. A leg that reads a multi-gigabyte
upstream file has to fit the standard runner's time and memory, or it is not a
leg of this gate.
