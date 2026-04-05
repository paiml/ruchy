//! Provability: types and runtime helpers for the §14.10 hard requirements.
//!
//! This module ships the **runtime skeletons** for the five hard requirements
//! added in `docs/specifications/ruchy-5.0-sovereign-platform.md` §14.10.
//! Static enforcement (lint passes, compile errors) lands incrementally in
//! later sprints; these types are the contract that static checks will honour.
//!
//! Ticket prefixes covered here:
//! - **SECRET-001**: `Secret<T>` / `Public<T>` newtype skeletons (§14.10.1)
//! - **CAP-001**: `RootCapability` and derived scoped capabilities (§14.10.2)
//! - **TOTAL-001**: `Totality` marker enum backing the `@total`/`@partial`
//!   decorator attribute (§14.10.3)
//!
//! Static analysis (§14.10.4 differential check, §14.10.5 refinement) is
//! scheduled for subsequent sprints and not represented here.

pub mod capabilities;
pub mod secret;
pub mod tier;
pub mod totality;

pub use capabilities::{ClockCap, EnvCap, FsCap, NetCap, RandomCap, RootCapability};
pub use secret::{declassify, Public, Secret};
pub use tier::{classify, Tier, TierInputs};
pub use totality::Totality;
