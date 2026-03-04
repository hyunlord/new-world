//! Name generator: re-exported from sim-data.
//!
//! The implementation lives in `sim_data::NameGenerator` since sim-engine
//! needs to hold a NameGenerator in SimResources, and sim-engine cannot
//! depend on sim-systems (that would create a circular dependency).

pub use sim_data::NameGenerator;
