// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Visual representation of Event Models.
//!
//! This module handles the visualization aspects of Event Models, including
//! layout computation, styling, theming, and rendering to various formats
//! like SVG. This is a core domain concept - how Event Models are presented
//! visually to users.

pub mod layout;
pub mod style;
pub mod theme;
pub mod svg;

pub use layout::{Layout, LayoutEngine, LayoutConfig, LayoutError};
pub use svg::{SvgDocument, SvgRenderer, SvgRenderConfig, SvgRenderError};
pub use style::Theme;
pub use theme::{ThemedRenderer, GithubLight, GithubDark};