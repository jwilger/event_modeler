//! Connector routing using libavoid.
//!
//! This module provides orthogonal connector routing functionality
//! using the libavoid library for collision-free path finding.

mod libavoid_ffi;
mod libavoid_wrapper;

pub use libavoid_wrapper::{LibavoidRouter, ObstacleId, Result, RoutingConfig, RoutingError};

// Re-export routing types from diagram module for convenience
pub use crate::diagram::routing_types::{Point, Rectangle, RoutePath};
