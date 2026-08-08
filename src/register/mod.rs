//! Being a register at all, with no spectroscopy in it.
//!
//! Everything here would survive being lifted out and used by a register of
//! material parameters or of measurement histories. That is the test the
//! boundary is drawn by, and it is stated in `docs/decisions/layout.md` rather
//! than only implied by what happens to be in this directory today.
//!
//! Where this part eventually lives, whether it stays a module here or becomes
//! something the sibling registers depend on, is entry 4 of #1 and is open. What
//! this side owes is that the question can still be answered later without a
//! rewrite.

pub mod ancestry;
pub mod claims;
pub mod provenance;
pub mod uncertainty;
