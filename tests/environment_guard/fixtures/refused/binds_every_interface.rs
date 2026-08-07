// A fixture, not a test, for the same reason as its neighbours here.
//
// What it does wrong is the elevation case rather than the network one. Binding
// to every interface rather than to loopback raises a firewall consent dialog on
// at least one platform, and the dialog's subject is the executable path, so
// answering it settles nothing beyond one build directory. Every rebuild into a
// new directory asks again, and whoever is running the suite learns to click
// through it.

use std::net::TcpListener;

#[test]
fn the_query_surface_answers_over_a_socket() {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let _ = listener.accept();
}
