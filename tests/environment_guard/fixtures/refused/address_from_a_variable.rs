// A fixture, not a test, for the same reason as its neighbours here.
//
// What it does wrong is not visible on the line that does it. The address comes
// from somewhere else, so a search over the source cannot tell whether it stays
// on this machine. The guard refuses it rather than passing it, because a guard
// that passes what it cannot read is a guard anybody can walk through by moving
// one string into a variable.

use std::io::Write;
use std::net::TcpStream;

fn upstream_address() -> String {
    std::env::var("LINIENBUCH_UPSTREAM").unwrap()
}

#[test]
fn fe_i_lines_match_the_recorded_extract() {
    let address = upstream_address();
    let mut upstream = TcpStream::connect(address).unwrap();
    upstream.write_all(b"GET /cgi-bin/ASD/lines1.pl\r\n\r\n").unwrap();
    // and then compare the response against the recorded extract
}
