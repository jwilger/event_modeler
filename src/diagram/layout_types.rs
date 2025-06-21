// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Type-safe layout constants for diagram rendering.
//!
//! This module provides strongly-typed wrappers for layout dimensions
//! and constants used in SVG rendering, following the type-driven
//! development approach.

use nutype::nutype;

/// Width of a component in pixels.
#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct Width(u32);

/// Height of a component in pixels.
#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct Height(u32);

/// X coordinate in pixels.
#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct XCoordinate(u32);

/// Y coordinate in pixels.
#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct YCoordinate(u32);

/// Font size in pixels.
#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct FontSize(u32);

/// Color value in hex format.
#[nutype(
    validate(regex = r"^#[0-9a-fA-F]{6}$"),
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct HexColor(String);

/// Layout constants for the diagram.
///
/// These constants define the visual layout parameters for diagram rendering.
/// The typed fields ensure compile-time validation of layout values.
///
/// TODO: Consider creating more specific domain types (e.g., DiagramWidth,
/// SwimlaneHeight) instead of generic Width/Height types for better
/// semantic clarity.
#[allow(dead_code)]
pub struct LayoutConstants {
    /// Minimum width of the entire canvas.
    pub min_canvas_width: Width,
    /// Minimum height of the entire canvas.
    pub min_canvas_height: Height,
    /// Width of the swimlane label area.
    pub swimlane_label_width: Width,
    /// Minimum height of a swimlane.
    pub min_swimlane_height: Height,
    /// Height of the header area containing the title.
    pub header_height: Height,
    /// Height of slice header area.
    pub slice_header_height: Height,
    /// Minimum width per slice.
    pub min_slice_width: Width,
    /// Font size for slice headers.
    pub slice_header_font_size: FontSize,
    /// Font size for regular text.
    pub regular_font_size: FontSize,
    /// Font size for swimlane labels.
    pub swimlane_font_size: FontSize,
}

impl Default for LayoutConstants {
    fn default() -> Self {
        Self {
            min_canvas_width: Width::try_new(800).unwrap(),
            min_canvas_height: Height::try_new(600).unwrap(),
            swimlane_label_width: Width::try_new(80).unwrap(),
            min_swimlane_height: Height::try_new(200).unwrap(),
            header_height: Height::try_new(50).unwrap(),
            slice_header_height: Height::try_new(30).unwrap(),
            min_slice_width: Width::try_new(300).unwrap(),
            slice_header_font_size: FontSize::try_new(11).unwrap(),
            regular_font_size: FontSize::try_new(12).unwrap(),
            swimlane_font_size: FontSize::try_new(10).unwrap(),
        }
    }
}

/// Color scheme for the diagram.
#[allow(dead_code)]
pub struct ColorScheme {
    /// Background color of the canvas.
    pub canvas_background: HexColor,
    /// Default text color.
    pub text_color: HexColor,
    /// Color for swimlane borders.
    pub swimlane_border: HexColor,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            canvas_background: HexColor::try_new("#f8f8f8".to_string()).unwrap(),
            text_color: HexColor::try_new("#333333".to_string()).unwrap(),
            swimlane_border: HexColor::try_new("#cccccc".to_string()).unwrap(),
        }
    }
}
