# Event Modeler

A type-safe Event Modeling diagram generator written in Rust.

## Overview

Event Modeler converts text-based Event Model descriptions (`.eventmodel` files) into visual diagrams suitable for documentation and analysis. It generates SVG and PDF outputs optimized for inclusion in GitHub markdown documentation.

## Features

- Text-based DSL for Event Modeling
- SVG and PDF output formats
- Type-safe design with compile-time guarantees
- Support for all Event Modeling concepts:
  - Swimlanes (actors/systems)
  - Commands (user intentions)
  - Events (state changes)
  - Projections (read models)
  - Queries (data retrieval)
  - Automations (system reactions)
  - Wireframes (UI mockups)
  - Slices (feature boundaries)

## Installation

```bash
cargo install event_modeler
```

## Usage

```bash
# Render a single event model file
event_modeler render model.eventmodel

# Watch a directory for changes
event_modeler watch ./models

# Validate without rendering
event_modeler validate model.eventmodel
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 John Wilger