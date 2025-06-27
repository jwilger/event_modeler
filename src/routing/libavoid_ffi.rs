//! FFI bindings for libavoid using autocxx.
//!
//! This module provides safe Rust bindings to the libavoid C++ library
//! for orthogonal connector routing.

#![allow(dead_code)] // Placeholder types until autocxx is enabled

// TODO: Enable autocxx bindings once libclang is available
// Current issue: autocxx requires libclang but it's not available in the build environment

/*
use autocxx::prelude::*;

include_cpp! {
    #include "libavoid/libavoid.h"

    safety!(unsafe_ffi)

    generate!("Avoid::Router")
    generate!("Avoid::ShapeRef")
    generate!("Avoid::ConnRef")
    generate!("Avoid::ConnEnd")
    generate!("Avoid::Point")
    generate!("Avoid::Rectangle")
    generate!("Avoid::Polygon")
    generate!("Avoid::RouterFlag")
    generate!("Avoid::ConnType")
    generate!("Avoid::RoutingParameter")
}

use ffi::*;

/// Re-export the FFI types for easier access
pub use ffi::Avoid_Router as Router;
pub use ffi::Avoid_ShapeRef as ShapeRef;
pub use ffi::Avoid_ConnRef as ConnRef;
pub use ffi::Avoid_ConnEnd as ConnEnd;
pub use ffi::Avoid_Point as Point;
pub use ffi::Avoid_Rectangle as Rectangle;
pub use ffi::Avoid_Polygon as Polygon;
*/

// Placeholder types until autocxx bindings are enabled
pub struct Router;
pub struct ShapeRef;
pub struct ConnRef;
pub struct ConnEnd;
pub struct Point;
pub struct Rectangle;
pub struct Polygon;
