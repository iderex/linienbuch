// The neighbour of refused/fetches_from_upstream.rs, one address literal away
// from it and nothing else. A fixture, not a test, for the same reason as that
// one.
//
// It stays on this machine, so the guard must not refuse it. A guard that also
// refused this would have proved only that it refuses files containing a socket
// call, which is not the rule.

use std::io::Write;
use std::net::TcpStream;

#[test]
fn fe_i_lines_match_the_recorded_extract() {
    let mut upstream = TcpStream::connect("127.0.0.1:9").unwrap();
    upstream.write_all(b"GET /cgi-bin/ASD/lines1.pl\r\n\r\n").unwrap();
    // and then compare the response against the recorded extract
}
