#!/bin/bash
# Script to generate and preview the event model diagram locally

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building event_modeler...${NC}"
cargo build --release

echo -e "${GREEN}Generating diagram...${NC}"
if ./target/release/event_modeler tests/fixtures/acceptance/example.eventmodel -o example.svg; then
    echo -e "${GREEN}✓ SVG generated successfully${NC}"
    
    # Convert to PNG if ImageMagick is available
    if command -v convert &> /dev/null; then
        echo -e "${GREEN}Converting to PNG...${NC}"
        convert -background white -alpha remove -alpha off example.svg example.png
        echo -e "${GREEN}✓ PNG generated successfully${NC}"
        
        # Open the PNG if xdg-open is available
        if command -v xdg-open &> /dev/null; then
            echo -e "${GREEN}Opening PNG...${NC}"
            xdg-open example.png
        else
            echo -e "${YELLOW}xdg-open not found. Please open example.png manually.${NC}"
        fi
    else
        echo -e "${YELLOW}ImageMagick not found. Install it to convert SVG to PNG.${NC}"
        echo -e "${YELLOW}You can view the SVG file: example.svg${NC}"
    fi
else
    echo -e "${RED}✗ Failed to generate diagram${NC}"
    exit 1
fi

echo -e "${GREEN}Done!${NC}"