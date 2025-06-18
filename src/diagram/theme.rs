// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Pre-defined themes for Event Model diagrams.
//!
//! This module provides built-in themes optimized for different environments,
//! particularly GitHub's light and dark modes. Themes are selected at compile
//! time using phantom types.

use super::style::Theme;
use std::marker::PhantomData;

// Phantom types for theme variants

/// Marker type for GitHub light theme.
#[derive(Debug, Clone, Copy)]
pub struct GithubLight;

/// Marker type for GitHub dark theme.
#[derive(Debug, Clone, Copy)]
pub struct GithubDark;

// Compile-time theme selection

/// A renderer with a compile-time selected theme.
///
/// The theme is determined by the type parameter `T`, which must be a valid
/// theme variant (e.g., `GithubLight` or `GithubDark`).
#[derive(Debug, Clone)]
pub struct ThemedRenderer<T> {
    /// The concrete theme instance.
    theme: Theme,
    /// Phantom data to track the theme type.
    _phantom: PhantomData<T>,
}

impl ThemedRenderer<GithubLight> {
    /// Create a renderer with the GitHub light theme.
    pub fn github_light() -> Self {
        Self {
            theme: Self::create_github_light_theme(),
            _phantom: PhantomData,
        }
    }

    /// Build the GitHub light theme configuration.
    fn create_github_light_theme() -> Theme {
        use super::style::*;
        use crate::infrastructure::types::{NonEmptyString, NonNegativeFloat, PositiveFloat};

        Theme {
            name: ThemeName::new(NonEmptyString::parse("github-light".to_string()).unwrap()),

            // Wireframe style - neutral gray
            wireframe_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#f6f8fa".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#d1d5db".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.5).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Command style - blue
            command_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#dbeafe".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#0969da".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Event style - purple
            event_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#f3e8ff".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#9333ea".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Projection style - yellow
            projection_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#fefce8".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#eab308".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Query style - blue (same as command)
            query_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#dbeafe".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#0969da".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Automation style - green
            automation_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#f0fdf4".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#22c55e".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Connection style
            connection_style: ConnectionStyle {
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#6b7280".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                marker_end: Some(MarkerStyle {
                    marker_type: MarkerType::Arrow,
                    size: MarkerStyleSize::new(PositiveFloat::parse(10.0).unwrap()),
                    color: StyleColor::new(NonEmptyString::parse("#6b7280".to_string()).unwrap()),
                }),
                marker_start: None,
            },

            // Swimlane style
            swimlane_style: SwimlaneStyle {
                background: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#ffffff".to_string()).unwrap()),
                    opacity: None,
                },
                border: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#e5e7eb".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                label_style: LabelStyle {
                    font: FontConfig {
                        family: StyleFontFamily::new(
                            NonEmptyString::parse(
                                "ui-sans-serif, system-ui, sans-serif".to_string(),
                            )
                            .unwrap(),
                        ),
                        size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                        weight: StyleFontWeight::Bold,
                    },
                    color: StyleColor::new(NonEmptyString::parse("#1f2937".to_string()).unwrap()),
                    alignment: TextAlignment::Left,
                },
            },

            // Slice style
            slice_style: SliceStyle {
                background: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#fafafa".to_string()).unwrap()),
                    opacity: Some(StyleOpacity::new(NonNegativeFloat::parse(0.5).unwrap())),
                },
                border: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#d1d5db".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.0).unwrap()),
                    dasharray: Some(DashArray {
                        pattern: vec![
                            DashValue::new(PositiveFloat::parse(5.0).unwrap()),
                            DashValue::new(PositiveFloat::parse(5.0).unwrap()),
                        ],
                    }),
                    opacity: None,
                },
                gutter_style: GutterStyle {
                    color: StyleColor::new(NonEmptyString::parse("#e5e7eb".to_string()).unwrap()),
                    pattern: GutterPattern::Dashed,
                },
            },

            // Text styles
            text_style: TextStyleConfig {
                entity_name: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(12.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
                field_name: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-monospace, monospace".to_string()).unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(10.0).unwrap()),
                    weight: StyleFontWeight::Normal,
                },
                slice_label: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
                swimlane_label: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
            },
        }
    }
}

impl ThemedRenderer<GithubDark> {
    /// Create a renderer with the GitHub dark theme.
    pub fn github_dark() -> Self {
        Self {
            theme: Self::create_github_dark_theme(),
            _phantom: PhantomData,
        }
    }

    /// Build the GitHub dark theme configuration.
    fn create_github_dark_theme() -> Theme {
        use super::style::*;
        use crate::infrastructure::types::{NonEmptyString, NonNegativeFloat, PositiveFloat};

        Theme {
            name: ThemeName::new(NonEmptyString::parse("github-dark".to_string()).unwrap()),

            // Wireframe style - dark gray
            wireframe_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#21262d".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#30363d".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.5).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Command style - blue
            command_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#0c2d6b".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#388bfd".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Event style - purple
            event_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#271052".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#a371f7".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Projection style - yellow
            projection_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#422006".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#eab308".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Query style - blue (same as command)
            query_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#0c2d6b".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#388bfd".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Automation style - green
            automation_style: EntityStyle {
                fill: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#04260f".to_string()).unwrap()),
                    opacity: None,
                },
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#22c55e".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },

            // Connection style
            connection_style: ConnectionStyle {
                stroke: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#848d97".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(2.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                marker_end: Some(MarkerStyle {
                    marker_type: MarkerType::Arrow,
                    size: MarkerStyleSize::new(PositiveFloat::parse(10.0).unwrap()),
                    color: StyleColor::new(NonEmptyString::parse("#848d97".to_string()).unwrap()),
                }),
                marker_start: None,
            },

            // Swimlane style
            swimlane_style: SwimlaneStyle {
                background: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#0d1117".to_string()).unwrap()),
                    opacity: None,
                },
                border: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#30363d".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.0).unwrap()),
                    dasharray: None,
                    opacity: None,
                },
                label_style: LabelStyle {
                    font: FontConfig {
                        family: StyleFontFamily::new(
                            NonEmptyString::parse(
                                "ui-sans-serif, system-ui, sans-serif".to_string(),
                            )
                            .unwrap(),
                        ),
                        size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                        weight: StyleFontWeight::Bold,
                    },
                    color: StyleColor::new(NonEmptyString::parse("#e6edf3".to_string()).unwrap()),
                    alignment: TextAlignment::Left,
                },
            },

            // Slice style
            slice_style: SliceStyle {
                background: FillStyle {
                    color: StyleColor::new(NonEmptyString::parse("#161b22".to_string()).unwrap()),
                    opacity: Some(StyleOpacity::new(NonNegativeFloat::parse(0.5).unwrap())),
                },
                border: StrokeStyle {
                    color: StyleColor::new(NonEmptyString::parse("#30363d".to_string()).unwrap()),
                    width: StrokeWidth::new(PositiveFloat::parse(1.0).unwrap()),
                    dasharray: Some(DashArray {
                        pattern: vec![
                            DashValue::new(PositiveFloat::parse(5.0).unwrap()),
                            DashValue::new(PositiveFloat::parse(5.0).unwrap()),
                        ],
                    }),
                    opacity: None,
                },
                gutter_style: GutterStyle {
                    color: StyleColor::new(NonEmptyString::parse("#21262d".to_string()).unwrap()),
                    pattern: GutterPattern::Dashed,
                },
            },

            // Text styles
            text_style: TextStyleConfig {
                entity_name: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(12.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
                field_name: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-monospace, monospace".to_string()).unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(10.0).unwrap()),
                    weight: StyleFontWeight::Normal,
                },
                slice_label: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
                swimlane_label: FontConfig {
                    family: StyleFontFamily::new(
                        NonEmptyString::parse("ui-sans-serif, system-ui, sans-serif".to_string())
                            .unwrap(),
                    ),
                    size: StyleFontSize::new(PositiveFloat::parse(14.0).unwrap()),
                    weight: StyleFontWeight::Bold,
                },
            },
        }
    }
}

impl<T> ThemedRenderer<T> {
    /// Get a reference to the theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

// Sealed trait pattern for theme constraints

/// Sealed module to prevent external implementations.
mod sealed {
    /// Sealed trait for theme variants.
    pub trait ThemeVariant {}
    impl ThemeVariant for super::GithubLight {}
    impl ThemeVariant for super::GithubDark {}
}

/// Trait for theme variants.
///
/// This trait is sealed and can only be implemented by types in this module.
pub trait ThemeVariant: sealed::ThemeVariant {
    /// Get the theme variant name.
    fn name() -> &'static str;
}

impl ThemeVariant for GithubLight {
    fn name() -> &'static str {
        "github-light"
    }
}

impl ThemeVariant for GithubDark {
    fn name() -> &'static str {
        "github-dark"
    }
}
