//! Native audio engine for Echo Canvas Desktop.
//!
//! Milestone M6 (`docs/spec/10-build-plan.md`) fills this crate from
//! `docs/spec/04-audio.md` and `docs/spec/08-target-architecture.md` §7: the per-source
//! chain, the six-tap early-reflection bank, the shared reverb, the Room/Dry crossfade,
//! equal-power panning, parameter mapping and smoothing, device I/O and an offline
//! renderer for tests.
//!
//! # Boundaries
//!
//! - The audio callback never allocates, locks, logs, does I/O or panics.
//! - Parameters arrive through a triple buffer; structural changes through bounded
//!   SPSC queues.
//! - The P5/P6/P7 helpers stay data-only and are never wired into audio without an
//!   approved ADR.
#![forbid(unsafe_code)]
