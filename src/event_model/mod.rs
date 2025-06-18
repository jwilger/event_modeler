// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! The core concepts of Event Modeling.
//!
//! An Event Model describes a system through the lens of state changes (Events)
//! and the activities that cause or respond to those changes. This module
//! contains all the building blocks that make up an Event Model:
//!
//! - **Commands**: User intentions to change the system
//! - **Events**: Records of things that actually happened
//! - **Projections**: Views of data derived from events
//! - **Queries**: Ways to retrieve information from projections
//! - **Automations**: System reactions to events
//! - **Wireframes**: Visual mockups showing user interactions

pub mod converter;
pub mod diagram;
pub mod entities;
pub mod registry;
pub mod yaml_types;

pub use diagram::{DiagramMetadata, EventModelDiagram};
pub use entities::{Automation, Command, Event, Projection, Query, Wireframe};
pub use registry::{EntityRef, EntityRegistry};
