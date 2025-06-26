# Event Modeler Roadmap

## Current Implementation Status

**Active Development**: Event Modeler is undergoing a major rewrite to support YAML-based event models with rich visual diagramming.

### Tracking Progress

Development is tracked through GitHub issues:
- **Major phases**: See issues labeled "epic"
- **Current work**: Check open sub-issues of active epics
- **Phase 6**: Incremental diagram module rewrite (in progress)
- **MCP Development**: Workflow automation server (Phase 1 complete)

### Completed ✅
- Phases 1-5: Type System, YAML Parser, Domain Model, Layout Engine, Basic Rendering
- Module organization with clear domain boundaries
- Type-safe design throughout (zero runtime validation)
- Comprehensive documentation
- Domain model types (Event, Command, Projection, etc.)
- Infrastructure utilities (NonEmptyString, TypedPath, etc.)
- YAML parsing pipeline with rich error handling

### In Progress 🚧
- Phase 6: Incremental diagram module rewrite
- MCP Workflow Server for development automation

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

## Development Process

See [DEVELOPMENT_PROCESS.md](DEVELOPMENT_PROCESS.md) for critical development rules, PR workflow, and coding standards.