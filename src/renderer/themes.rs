//! Pre-defined themes for Event Model diagrams.
//!
//! This module provides built-in themes optimized for different environments,
//! particularly GitHub's light and dark modes. Themes are selected at compile
//! time using phantom types.

use super::styles::Theme;
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
        todo!()
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
        todo!()
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