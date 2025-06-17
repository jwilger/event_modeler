//! Tests for SVG rendering.

use super::*;
use crate::diagram::layout::{Canvas, CanvasHeight, CanvasWidth, Layout, Padding, PaddingValue};
use crate::infrastructure::types::{NonNegativeFloat, PositiveInt};
use std::collections::HashMap;

mod basic_tests {
    use super::*;

    #[test]
    fn svg_renderer_can_be_created() {
        let config = SvgRenderConfig {
            precision: DecimalPrecision::new(PositiveInt::parse(2).unwrap()),
            optimize: OptimizationLevel::Basic,
            embed_fonts: EmbedFonts::new(false),
        };

        // Create a default theme for testing
        let theme = crate::diagram::theme::ThemedRenderer::<crate::diagram::theme::GithubLight>::github_light().theme().clone();
        let renderer = SvgRenderer::new(config, theme);
        assert!(matches!(
            renderer.config().optimize,
            OptimizationLevel::Basic
        ));
    }

    #[test]
    fn svg_document_has_viewbox() {
        let viewbox = ViewBox {
            x: XCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            width: Width::new(PositiveFloat::parse(1200.0).unwrap()),
            height: Height::new(PositiveFloat::parse(800.0).unwrap()),
        };

        let doc = SvgDocument {
            viewbox,
            elements: vec![],
            defs: SvgDefs {
                patterns: vec![],
                gradients: vec![],
                markers: vec![],
            },
        };

        assert_eq!(doc.viewbox.width.into_inner().value(), 1200.0);
    }
}

mod rendering_tests {
    use super::*;

    #[test]
    fn render_empty_layout_produces_svg_with_canvas() {
        let layout = Layout {
            canvas: Canvas {
                width: CanvasWidth::new(PositiveInt::parse(1200).unwrap()),
                height: CanvasHeight::new(PositiveInt::parse(800).unwrap()),
                padding: Padding {
                    top: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                    right: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                    bottom: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                    left: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                },
            },
            swimlane_layouts: HashMap::new(),
            entity_positions: HashMap::new(),
            slice_layouts: HashMap::new(),
            connections: vec![],
        };

        let config = SvgRenderConfig {
            precision: DecimalPrecision::new(PositiveInt::parse(2).unwrap()),
            optimize: OptimizationLevel::None,
            embed_fonts: EmbedFonts::new(false),
        };

        // Create a default theme for testing
        let theme = crate::diagram::theme::ThemedRenderer::<crate::diagram::theme::GithubLight>::github_light().theme().clone();
        let renderer = SvgRenderer::new(config, theme);
        let result = renderer.render(&layout);

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.viewbox.width.into_inner().value(), 1200.0);
        assert_eq!(doc.viewbox.height.into_inner().value(), 800.0);
    }
}
