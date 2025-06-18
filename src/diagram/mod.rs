// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Visual representation of Event Models.
//!
//! This module handles the visualization aspects of Event Models, including
//! layout computation, styling, theming, and rendering to various formats
//! like SVG. This is a core domain concept - how Event Models are presented
//! visually to users.

pub mod layout;
pub mod node;
pub mod node_layout;
pub mod style;
pub mod svg;
pub mod theme;

pub use layout::{Layout, LayoutConfig, LayoutEngine, LayoutError};
pub use node::{DiagramNode, EntityReference, NodeConnection, NodeGenerator, NodeId, Position};
pub use node_layout::{NodeLayout, NodeLayoutEngine, PositionedNode};
pub use style::Theme;
pub use svg::{SvgDocument, SvgRenderConfig, SvgRenderError, SvgRenderer};
pub use theme::{GithubDark, GithubLight, ThemedRenderer};
