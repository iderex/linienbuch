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

The fixture directory `tests/environment_guard/fixtures/` is skipped, because it
is where the files that violate the rule deliberately live. That is one named
exclusion and it is the only one.

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

None of the four is compiled or run. Cargo builds only the `.rs` files directly
under `tests/`, and these are three directories down.

## How the suite is run, and what that run showed

    cargo test --locked
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
