//! FFI bindings for libavoid using C wrapper.
//!
//! This module provides safe Rust bindings to the libavoid C++ library
//! for orthogonal connector routing through a C wrapper interface.

#![allow(non_camel_case_types)]
#![allow(dead_code)] // FFI code is only used when mock-router feature is disabled

use std::os::raw::{c_double, c_int, c_uint};

/// Opaque pointer type for the libavoid router
pub enum AvoidRouterOpaque {}
pub type AvoidRouter = *mut AvoidRouterOpaque;

/// Shape ID type
pub type AvoidShapeId = c_uint;

/// Connector ID type
pub type AvoidConnectorId = c_uint;

/// Point structure matching C definition
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AvoidPoint {
    pub x: c_double,
    pub y: c_double,
}

/// Rectangle structure matching C definition
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AvoidRectangle {
    pub min_x: c_double,
    pub min_y: c_double,
    pub max_x: c_double,
    pub max_y: c_double,
}

// Routing flags from libavoid
pub const POLYLINE_ROUTING: c_uint = 1;
pub const ORTHOGONAL_ROUTING: c_uint = 2;

#[cfg_attr(not(feature = "mock-router"), link(name = "avoid_c_wrapper"))]
#[cfg_attr(not(feature = "mock-router"), link(name = "avoid"))]
#[cfg_attr(not(feature = "mock-router"), link(name = "stdc++"))]
unsafe extern "C" {
    // Router creation and destruction
    pub fn avoid_router_new(flags: c_uint) -> AvoidRouter;
    pub fn avoid_router_delete(router: AvoidRouter);

    // Shape management
    pub fn avoid_router_add_shape(router: AvoidRouter, rect: AvoidRectangle) -> AvoidShapeId;
    pub fn avoid_router_delete_shape(router: AvoidRouter, shape_id: AvoidShapeId);

    // Connector management
    pub fn avoid_router_add_connector(
        router: AvoidRouter,
        start: AvoidPoint,
        end: AvoidPoint,
    ) -> AvoidConnectorId;
    pub fn avoid_router_delete_connector(router: AvoidRouter, conn_id: AvoidConnectorId);

    // Routing
    pub fn avoid_router_process_transaction(router: AvoidRouter);

    // Get route points
    pub fn avoid_router_get_route_points(
        router: AvoidRouter,
        conn_id: AvoidConnectorId,
        points: *mut *mut AvoidPoint,
    ) -> c_int;
    pub fn avoid_free_points(points: *mut AvoidPoint);
}

