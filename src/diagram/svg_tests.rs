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

mod serialization_tests {
    use super::*;

    #[test]
    fn svg_document_serializes_to_valid_xml() {
        let viewbox = ViewBox {
            x: XCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            width: Width::new(PositiveFloat::parse(800.0).unwrap()),
            height: Height::new(PositiveFloat::parse(600.0).unwrap()),
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

        let xml = doc.to_xml();

        // Check XML declaration
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));

        // Check SVG root element
        assert!(xml.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(xml.contains("viewBox=\"0 0 800 600\""));
        assert!(xml.contains("width=\"800\""));
        assert!(xml.contains("height=\"600\""));

        // Check closing tag
        assert!(xml.ends_with("</svg>\n"));
    }

    #[test]
    fn serializes_rectangle_with_styles() {
        let viewbox = ViewBox {
            x: XCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            width: Width::new(PositiveFloat::parse(100.0).unwrap()),
            height: Height::new(PositiveFloat::parse(100.0).unwrap()),
        };

        let rect = SvgRectangle {
            id: Some(ElementId::new(
                NonEmptyString::parse("rect1".to_string()).unwrap(),
            )),
            class: Some(CssClass::new(
                NonEmptyString::parse("entity".to_string()).unwrap(),
            )),
            x: XCoordinate::new(NonNegativeFloat::parse(10.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(20.0).unwrap()),
            width: Width::new(PositiveFloat::parse(30.0).unwrap()),
            height: Height::new(PositiveFloat::parse(40.0).unwrap()),
            rx: Some(BorderRadius::new(NonNegativeFloat::parse(5.0).unwrap())),
            ry: None,
            style: crate::diagram::style::EntityStyle {
                fill: crate::diagram::style::FillStyle {
                    color: crate::diagram::style::StyleColor::new(
                        NonEmptyString::parse("#ff0000".to_string()).unwrap(),
                    ),
                    opacity: None,
                },
                stroke: crate::diagram::style::StrokeStyle {
                    color: crate::diagram::style::StyleColor::new(
                        NonEmptyString::parse("#000000".to_string()).unwrap(),
                    ),
                    width: crate::diagram::style::StrokeWidth::new(
                        PositiveFloat::parse(2.0).unwrap(),
                    ),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },
        };

        let doc = SvgDocument {
            viewbox,
            elements: vec![SvgElement::Rectangle(rect)],
            defs: SvgDefs {
                patterns: vec![],
                gradients: vec![],
                markers: vec![],
            },
        };

        let xml = doc.to_xml();

        // Check rectangle attributes
        assert!(xml.contains("<rect id=\"rect1\""));
        assert!(xml.contains("class=\"entity\""));
        assert!(xml.contains("x=\"10\""));
        assert!(xml.contains("y=\"20\""));
        assert!(xml.contains("width=\"30\""));
        assert!(xml.contains("height=\"40\""));
        assert!(xml.contains("rx=\"5\""));
        assert!(xml.contains("fill=\"#ff0000\""));
        assert!(xml.contains("stroke=\"#000000\""));
        assert!(xml.contains("stroke-width=\"2\""));
    }

    #[test]
    fn serializes_text_with_font_styles() {
        let viewbox = ViewBox {
            x: XCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            width: Width::new(PositiveFloat::parse(200.0).unwrap()),
            height: Height::new(PositiveFloat::parse(100.0).unwrap()),
        };

        let text = SvgText {
            id: None,
            class: Some(CssClass::new(
                NonEmptyString::parse("label".to_string()).unwrap(),
            )),
            x: XCoordinate::new(NonNegativeFloat::parse(50.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(30.0).unwrap()),
            content: TextContent::new(NonEmptyString::parse("Hello SVG".to_string()).unwrap()),
            style: TextStyle {
                font_family: FontFamily::new(
                    NonEmptyString::parse("Arial, sans-serif".to_string()).unwrap(),
                ),
                font_size: FontSize::new(PositiveFloat::parse(14.0).unwrap()),
                font_weight: Some(FontWeight::Bold),
                fill: Color::new(NonEmptyString::parse("#333333".to_string()).unwrap()),
                anchor: Some(TextAnchor::Middle),
            },
        };

        let doc = SvgDocument {
            viewbox,
            elements: vec![SvgElement::Text(text)],
            defs: SvgDefs {
                patterns: vec![],
                gradients: vec![],
                markers: vec![],
            },
        };

        let xml = doc.to_xml();

        // Check text element
        assert!(xml.contains("<text"));
        assert!(xml.contains("class=\"label\""));
        assert!(xml.contains("x=\"50\""));
        assert!(xml.contains("y=\"30\""));
        assert!(xml.contains("font-family=\"Arial, sans-serif\""));
        assert!(xml.contains("font-size=\"14\""));
        assert!(xml.contains("font-weight=\"bold\""));
        assert!(xml.contains("fill=\"#333333\""));
        assert!(xml.contains("text-anchor=\"middle\""));
        assert!(xml.contains(">Hello SVG</text>"));
    }
}
