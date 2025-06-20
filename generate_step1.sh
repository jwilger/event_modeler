#!/bin/bash

# Step 1: Generate SVG with just swimlanes and convert to PNG

echo "Step 1: Generating swimlanes-only diagram..."

# Run the main CLI to generate SVG
cargo run --bin event_modeler -- tests/fixtures/acceptance/example.eventmodel -o output/step1_swimlanes_only.svg

# Convert to PNG for visual inspection
if command -v magick &> /dev/null; then
    magick output/step1_swimlanes_only.svg output/step1_swimlanes_only.png
    echo "Generated output/step1_swimlanes_only.png for visual inspection"
else
    echo "ImageMagick not found, please convert SVG to PNG manually"
fi