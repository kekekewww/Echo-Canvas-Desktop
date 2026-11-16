//! Pure geometry and acoustics for Echo Canvas Desktop.
//!
//! This crate will hold the exact numeric ports of the Classic 2.5D model
//! (`classic`, milestone M2, `docs/spec/02-acoustics-classic.md`) and the Hybrid 3D
//! model (`hybrid3d`, milestone M3, `docs/spec/03-acoustics-hybrid.md`).
//!
//! # Boundaries
//!
//! - Pure functions over `f64` only. No `f32`, no explicit FMA, no reordering of
//!   floating-point operations: results must match `fixtures/corpus/v1` under its
//!   frozen comparison rules.
//! - No I/O, threads, clocks, randomness, logging, Tauri, audio or UI dependencies.
//! - Nothing in the workspace is a dependency of this crate
//!   (`docs/spec/08-target-architecture.md` §4).
//!
//! # Units and conventions
//!
//! Meters, seconds, milliseconds for delays, degrees, dB and Hz. Speed of sound
//! 343 m/s unless a spec section says otherwise. Plan coordinates `{x, y}` map to 3D
//! `{x, y: height, z: plan y}` (`docs/spec/01-domain-model.md` §1).
#![forbid(unsafe_code)]
