// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Exporting Event Model diagrams to documentation formats.
//!
//! Once you have an Event Model diagram, you want to share it with your team.
//! This module handles exporting diagrams to different formats suitable for
//! documentation, presentations, and reports.

pub mod pdf;
pub mod markdown;

pub use pdf::{PdfExporter, PdfExportConfig, PdfExportError};
pub use markdown::{MarkdownExporter, MarkdownExportConfig, MarkdownExportError};