# Event Modeler Roadmap

## Current Implementation Status

**⚠️ Early Development Stage**: The type system and module structure are complete, but most functionality contains `todo!()` placeholders.

### Completed ✅
- Module organization with clear domain boundaries
- Type-safe design throughout (zero runtime validation)
- Comprehensive documentation
- Domain model types (Event, Command, Projection, etc.)
- Infrastructure utilities (NonEmptyString, TypedPath, etc.)

### Next Development Priorities 🚧

1. **CLI Foundation** - Implement CLI argument parsing in `src/cli.rs`
2. **Text Parsing** - Implement text parsing in `src/infrastructure/parsing/`
3. **Layout Engine** - Implement layout computation in `src/diagram/layout.rs`
4. **SVG Rendering** - Implement SVG rendering in `src/diagram/svg.rs`

### Implementation Notes
- All `todo!()` functions should maintain the existing type signatures
- Add implementation without changing the established type-safe interfaces
- Maintain zero runtime validation principle

## Future Milestones

### MVP (Minimum Viable Product)
- [ ] Parse basic .eventmodel files
- [ ] Generate simple SVG diagrams
- [ ] Command-line interface for file conversion

### Enhanced Features
- [ ] PDF export support
- [ ] Advanced layout algorithms
- [ ] Theme customization
- [ ] Markdown export

### Long-term Vision
- [ ] Interactive diagram editing
- [ ] Plugin system for custom renderers
- [ ] Integration with development tools

## Architecture Decisions

See [README.md#architecture](README.md#architecture) for core principles and [CLAUDE.md](CLAUDE.md) for implementation guidance.