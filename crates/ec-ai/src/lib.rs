//! Optional assistant for Echo Canvas Desktop.
//!
//! Milestone M9 (`docs/spec/10-build-plan.md`, owner decision D2) fills this crate from
//! `docs/spec/07-backend-ai-infra-tests.md` §2: provider calls, the scene-compile contract
//! with exactly one repair, the explain contract and the content policy.
//!
//! # Boundaries
//!
//! - Off by default. The app makes no network request unless the user turns the
//!   assistant on.
//! - The API key lives in the OS keychain only and is never returned to the UI.
//! - Output is validated like a file, previewed, and applied only after the user
//!   confirms. It never sets audio parameters and never invents asset or registry IDs.
#![forbid(unsafe_code)]
