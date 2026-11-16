//! Audio assets for Echo Canvas Desktop.
//!
//! Milestones M6 (built-in loops) and M8 (user library and relink) fill this crate:
//! decoding, resampling to the device rate, content hashing, the library index and the
//! missing-file relink flow (`docs/spec/04-audio.md`, `docs/spec/08-target-architecture.md`
//! §7.4).
//!
//! # Boundaries
//!
//! - Projects store audio references and hashes, never audio bytes.
//! - The built-in loops ship with their license file (`assets/audio/LICENSE.md`).
#![forbid(unsafe_code)]
