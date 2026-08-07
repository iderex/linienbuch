//! The record model and the operations over it.
//!
//! Two units, and the line between them is the subject of
//! `docs/decisions/layout.md`. [`register`] is what this crate would still need
//! if it held no spectroscopy at all: sources, snapshots, references, and later
//! claims, provenance edges and the reporting that names what an answer rested
//! on. [`spectroscopy`] is the part that is about line data.
//!
//! The line is drawn here rather than later because four sibling registers need
//! the first half and none of them needs the second, and drawing it after the
//! code has grown across it means drawing it through the code.
//!
//! `tests/layout.rs` refuses an identifier under [`register`] that names a
//! quantity specific to spectroscopy, and refuses a reference from that side to
//! this one. What lives here is decided issue by issue and recorded under
//! `docs/decisions/` before the code that depends on it exists.

pub mod register;
pub mod spectroscopy;
