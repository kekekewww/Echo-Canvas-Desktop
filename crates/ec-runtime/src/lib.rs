//! Calculation runtime for Echo Canvas Desktop.
//!
//! Milestone M4 (`docs/spec/10-build-plan.md`) fills this crate from
//! `docs/spec/05-runtime-pipeline.md`: the per-mode coalescer (10–15 Hz), one job in
//! flight and one pending per mode, deterministic sharding by source ID, the static
//! cache keyed by fingerprint, shard validation, frame assembly, the acceptance rule
//! (including pose-lagging frames), the watchdog and the serial fallback.
//!
//! # Boundaries
//!
//! - Publishes only complete frames that pass the acceptance rule.
//! - Identifies scene content with a monotonic generation counter, not the revision.
//! - Depends on `ec-domain` and `ec-acoustics`; never on Tauri or the UI.
#![forbid(unsafe_code)]
