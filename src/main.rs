//! The smallest binary target that gives this repository a release artefact to
//! build twice and compare.
//!
//! It exists for the reproducible build measurement and for nothing else. What
//! the command does, and what it is called, are #3, #9 and entry 9 of #1.

fn main() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
