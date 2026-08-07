// A fixture, not a test. Cargo builds only the .rs files directly under tests/,
// and this one is three directories down, so it is never compiled and never run.
// It exists so that the refusal in tests/environment_guard.rs is proved against a
// file that really breaks the rule rather than asserted about one.
//
// What it does wrong: it fetches from an upstream catalogue while the suite is
// running. A run containing this reports that catalogue's availability rather
// than this repository's behaviour, and when the catalogue is slow it does not
// fail, it waits.

use std::io::Write;
use std::net::TcpStream;

#[test]
fn fe_i_lines_match_the_recorded_extract() {
    let mut upstream = TcpStream::connect("physics.nist.gov:443").unwrap();
    upstream.write_all(b"GET /cgi-bin/ASD/lines1.pl\r\n\r\n").unwrap();
    // and then compare the response against the recorded extract
}
