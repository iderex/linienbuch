//! The part that is about line data.
//!
//! Species and ionisation stages, energy levels, transitions, the air and vacuum
//! conversion, the upstream formats, oscillator strengths, and the physics of
//! propagating into an abundance. Most of that does not exist yet; what is here
//! is what has been decided and built so far.
//!
//! Nothing on the other side of the boundary refers to anything here, and
//! `tests/layout.rs` is what refuses it if something starts to.

pub mod accuracy;
pub mod intensity;
pub mod levels;
pub mod propagation;
pub mod species;
pub mod transitions;
