# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-06-17

### Added
- Complete MVP implementation - full pipeline from .eventmodel files to SVG diagrams
- Text parsing of .eventmodel files with comprehensive error handling
- Dynamic layout computation with automatic canvas sizing based on content
- SVG rendering with GitHub light and dark themes
- Support for 6 entity types: Command, Event, Projection, Policy, External System, Aggregate
- Connector support for showing relationships between entities
- Dark theme support via `--dark` CLI flag
- Comprehensive integration tests covering various scenarios
- Example .eventmodel files demonstrating features
- Error test cases for validation

### Fixed
- Dynamic canvas sizing to handle large diagrams
- Entity positioning within swimlanes
- Theme color application for different entity types

### Internal
- Implemented conversion from ParsedEventModel to EventModelDiagram
- Added NonEmpty helper methods (first, last, get)
- Complete documentation for all public APIs

## [0.1.4] - 2025-06-16

### Added
- Initial project structure and type-safe domain model
- Module organization following Event Modeling concepts
- Comprehensive type safety using nutype
- Architecture decision records

### Internal
- Setup development environment with Nix flake
- Configure pre-commit hooks
- Establish PR-driven development workflow

[0.2.0]: https://github.com/jwilger/event_modeler/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/jwilger/event_modeler/releases/tag/v0.1.4