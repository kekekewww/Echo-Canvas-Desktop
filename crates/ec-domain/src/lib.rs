//! Authoritative domain model for Echo Canvas Desktop.
//!
//! Milestone M1 (`docs/spec/10-build-plan.md`) fills this crate from
//! `docs/spec/01-domain-model.md`: units and limits (`limits`), materials, SceneSpec v1
//! and SceneDocumentV2 with their validators, `WorkspaceProject`, the reducer, history and
//! gestures, projections and hashes, file formats with their migrations, and presets.
//!
//! # Boundaries
//!
//! - Owns all authoring state rules. The UI holds no authoritative state.
//! - The same validation applies to files, IPC payloads and assistant output.
//! - May depend on `ec-acoustics` for shared value types, never the other way round
//!   (`docs/spec/08-target-architecture.md` §4).
#![forbid(unsafe_code)]
