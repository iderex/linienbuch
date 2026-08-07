# The default suite

Three constraints. Every test in the default suite runs with no display server,
with no elevated privileges, and with no network. A test that cannot meet all
three does not get an exception here. It moves to the separate integration
harness, where its cost is visible.

This is a birth requirement rather than a later cleanup, because a suite that
has already grown a display dependency does not lose it again cheaply.

## No display

The machine that runs this most often is a headless box or a scheduled run. A
suite that needs a display fails there for a reason that has nothing to do with
the code, and the failure arrives as a red run that somebody then has to
investigate before learning that nothing is wrong.

## No elevation

A test that asks for administrator rights trains whoever runs it to grant them,
and that habit is worth more to an attacker than anything the test proves.

There is a second route to the same prompt that does not look like elevation at
all. On at least one platform, binding a socket to an interface that is not
loopback raises a firewall consent dialog. The subject of that dialog is the
executable path, so answering it settles nothing beyond one build directory and
every rebuild into a new one asks again. Whoever is running the suite learns to
click through a consent dialog, which is the same lesson by a different door.

## No network

A suite whose result depends on a remote service reports that service's
availability. A red run then means nothing until somebody has investigated, and
the investigation ends in a shrug often enough that the run stops being read.

The failure is also slow rather than sharp. A socket to a host that is not
answering does not fail, it waits, and the cost lands on whoever is waiting
rather than on whoever wrote the test.

## What refuses a violation, and what does not

The network constraint and the socket half of the elevation constraint are
refused by `tests/environment_guard.rs`, which is part of the default suite. It
searches the tracked sources under `src/` and `tests/` for the API surfaces
listed in `tests/environment_guard/needles.txt` and refuses a line that names
one unless the same line carries a loopback address literal.

A line that names one of those APIs and carries no address literal is refused
too. The guard cannot see where such a call goes, and a guard that passes what
it cannot read is one anybody walks through by moving a string into a variable.

The refusal is a search over the sources rather than a sandbox around the
running test, because nothing in the standard library lets one test revoke
another test's access to a socket. Three things follow and none of them is
softened.

A call reached through a name the list does not hold passes. The list is a
floor: it holds the surfaces reachable through the standard library today, and
an entry is added when a crate that wraps a socket is added.

The guard reads lines. A call split across two lines, or assembled from
fragments, is not seen.

Two directories are skipped and both are named here rather than only in the
code. `tests/environment_guard/fixtures/` is where the files that violate the
rule deliberately live. `tests/integration/` is the harness below, which is not
part of the default suite and whose whole purpose is the network, so the guard's
subject is the default suite's sources rather than every file in the tree.

Those two are the whole of the exclusion list. A third would be worth arguing
about, and there is not one.

The display constraint has no mechanism. `PROSE, NOT ENFORCEMENT`, `OWED`, issue
#50. Nothing here refuses a test that opens a window, and the reason it is not
in the guard today is that no display API is reachable from this tree yet, so
any needle for one would be a guess and any fixture proving it bites would be
contrived. #50 is the issue for searches over the tree, and this belongs there
when the first such API arrives.

## The proof that the guard bites

Four fixtures, in `tests/environment_guard/fixtures/`. Three that the guard must
refuse, one that it must not.

`refused/fetches_from_upstream.rs` reaches an upstream catalogue by name.

`refused/address_from_a_variable.rs` is the same test with the address moved out
of the line, so the guard cannot read it.

`refused/binds_every_interface.rs` binds to every interface rather than to
loopback, which is the case that raises the consent dialog.

`allowed/loopback_only.rs` is the neighbour of the first, one address literal
away from it and nothing else, and the guard must not refuse it. A guard that
refused the neighbour too would have proved only that it refuses files
containing a socket call.

None of the four is compiled or run. Cargo builds the `.rs` files directly under
`tests/` and a `main.rs` one directory below it, and these four are neither: they
sit under `tests/environment_guard/fixtures/` and none of them is named
`main.rs`.

## How the suite is run, and what that run showed

    cargo test --locked

The guard's own target, which is the one the rest of this section is about:

    Running tests\environment_guard.rs
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

The refusal, demonstrated rather than asserted. A copy of the upstream-fetching
fixture was placed at `tests/network_dependent.rs`, where cargo does compile it,
and the guard was run:

    cargo test --locked --test environment_guard
    thread 'the_tracked_sources_reach_nothing' panicked at tests\environment_guard.rs:250:5:
    tracked sources must not reach off this machine, got [
        Finding {
            file: ".../tests/network_dependent.rs",
            line: 16,
            api: "TcpStream::connect",
            reason: OffThisMachine(
                "physics.nist.gov",
            ),
        },
    ]
    test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

The absolute path prefix in the `file` field is elided, and the elision is
deliberate: it names a directory on one machine and carries nothing a reader
needs. Everything else is the output as printed.

Red, with the file, the line, the call and the host named, and the harness
reporting `finished in 0.00s` because the refusal reads a file rather than
waiting on a socket. That is the loud failure the constraint asks for rather
than the slow success it exists to prevent. Wall clock for the whole invocation
is not quoted, because it is dominated by whether the target needed rebuilding
and says nothing about the guard.

Only the guard target was run with that file
present, deliberately: running the whole suite would have executed the fetch,
which is the thing being refused. The file was removed afterwards and is not in
the tree.

The account is unelevated. The process integrity level during the run:

    whoami /groups
    S-1-16-8192

which is the medium level, not the high level an elevated process carries.

The suite has not been run on a machine without a display, and that is not
claimed. Registering a service is the only way to reach a session with no
window station on the platform this was run on, and registering one on somebody's
workstation is not a thing a test setup does.

What is established instead is a different statement, and it is put as its own
statement rather than as a stand-in that quietly becomes the first one. The
compiled test binary imports no display library:

    objdump -p target/debug/deps/environment_guard-*.exe | grep -i "DLL Name" | sort -u
    DLL Name: api-ms-win-core-synch-l1-2-0.dll
    DLL Name: api-ms-win-crt-heap-l1-1-0.dll
    DLL Name: api-ms-win-crt-locale-l1-1-0.dll
    DLL Name: api-ms-win-crt-math-l1-1-0.dll
    DLL Name: api-ms-win-crt-runtime-l1-1-0.dll
    DLL Name: api-ms-win-crt-stdio-l1-1-0.dll
    DLL Name: bcryptprimitives.dll
    DLL Name: KERNEL32.dll
    DLL Name: ntdll.dll
    DLL Name: USERENV.dll
    DLL Name: VCRUNTIME140.dll

No `user32.dll` and no `gdi32.dll`, which are the two a program touching a
window would need. So the suite cannot reach a display, which is the property
the constraint is about. Whether it passes on a headless machine remains
untested until somebody runs it on one, and that is a different sentence from
the one above.

## The integration harness

Some things cannot be tested against a fixture and still mean anything. Whether
the retrieval code reaches a source at all. Whether the format the parser expects
is the format the server serves today. Whether a line list measured in gigabytes
parses with the memory ceiling holding. Those are worth having and they break
every rule above.

They live in `tests/integration/`, under a name that says what they need rather
than when they run. Not an extended suite and not a nightly, because those names
describe a schedule and hide a dependency, and the dependency is the thing
somebody has to decide about before running it.

### It is excluded by construction

`Cargo.toml` declares the target with `test = false`, so `cargo test` does not
build it and does not run it. No filter, no attribute and no directory name is
doing the work.

That this is by construction rather than by convention is measured rather than
supposed. Before the declaration existed, cargo auto-discovered
`tests/integration/main.rs` as an ordinary test target and ran it with the rest,
and the leg that opens a socket to an upstream host ran inside a `cargo test`.
The declaration is what stopped it.

### Its own command

    cargo test --test integration

Nothing in the merge gate runs it. The gate does run cargo now, which it did not
when this section was first written, and two of the four checks #5 added name
this target by name:

    git grep -n "test integration" -- .github/workflows/
    .github/workflows/build.yml:64:        run: cargo build --locked --test integration
    .github/workflows/lint.yml:51:        run: cargo clippy --locked --test integration -- -D warnings

Both compile it and neither runs it, and that is the whole of the distinction.
Compiling the harness reads its source; only `cargo test` would execute the leg
that opens a socket. So the rule this paragraph has always carried is unchanged:
the harness must not be added to the checks, because a source going down for an
afternoon must not block unrelated work. Compiling it costs nothing an outage can
take away, and leaving it uncompiled would let tracked source rot outside every
check that reads the rest of the tree.

### What each leg needs

`the_first_source_host_is_reachable` needs the network. It opens one socket to an
upstream host and closes it. It says the host is answering on that port and says
nothing about what it would serve, which is the smaller claim and is the one it
makes.

`the_published_format_matches_what_the_server_serves` needs the network and a
retrieval that does not exist yet. Declared and not implemented; #26 owes the
retrieval and #27 owes the parser. Writing a second retrieval inside the harness
to get the leg running sooner would put two retrievals in the tree, which is what
#26 exists to stop.

`a_full_line_list_parses_within_the_memory_ceiling` needs a download measured in
gigabytes and the parser in #29. Declared and not implemented, and the ceiling it
would assert against does not exist until there is a parser to set one on.

`the_date_stamp_on_a_finding_is_a_real_date` needs nothing. It lives here because
what it checks lives here, and it is disclosed alongside the others rather than
quietly left out of the count.

### A failure here is a finding

A source that changed its format, or moved, or went away, is real information
about the field rather than a broken test. A failing leg prints the leg name, the
retrieval date and what came back, in a shape somebody can paste into an issue.

### The absence is printed, not assumed

Every default run names the legs it did not run and what each one needs:

    cargo test --locked
    test a_full_line_list_parses_within_the_memory_ceiling ... ignored, in the integration harness: needs a download measured in gigabytes and the parser in #29. Run it with: cargo test --test integration
    test the_date_stamp_on_a_finding_is_a_real_date ... ignored, in the integration harness: not network bound, but it lives beside the code it checks. Run it with: cargo test --test integration
    test the_first_source_host_is_reachable ... ignored, in the integration harness: needs the network, one socket to an upstream host. Run it with: cargo test --test integration
    test the_published_format_matches_what_the_server_serves ... ignored, in the integration harness: needs the network and a retrieval that does not exist yet, #26 and #27. Run it with: cargo test --test integration
    test result: ok. 3 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.00s

Those four lines come from `tests/integration_disclosure.rs`, which holds one
ignored test per leg. An ignored test's reason is printed on its own line by
every run, so a run covering less than the whole set cannot be read as one that
covered it.

The same file holds a test that compares the legs it names against the test
functions the harness actually holds, in both directions, so the disclosure
cannot drift away from the thing it discloses. That comparison is a function over
sets and is proved on constructed sets as well as against the tree, because a
comparison that has only ever seen an agreeing pair has not been shown to refuse
anything.
