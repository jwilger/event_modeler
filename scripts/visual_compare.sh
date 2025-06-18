#!/usr/bin/env bash
# Script to visually compare generated SVG with gold master
# Usage: ./scripts/visual_compare.sh <generated.svg> <goldmaster.svg>

set -euo pipefail

if [ $# -ne 2 ]; then
    echo "Usage: $0 <generated.svg> <goldmaster.svg>"
    exit 1
fi

GENERATED="$1"
GOLDMASTER="$2"

# Check if files exist
if [ ! -f "$GENERATED" ]; then
    echo "Error: Generated file '$GENERATED' does not exist"
    exit 1
fi

if [ ! -f "$GOLDMASTER" ]; then
    echo "Error: Gold master file '$GOLDMASTER' does not exist"
    exit 1
fi

# Create temporary directory for comparison
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# Copy files to temp dir with descriptive names
cp "$GENERATED" "$TMPDIR/generated.svg"
cp "$GOLDMASTER" "$TMPDIR/goldmaster.svg"

# Try to open in default SVG viewer/browser
echo "Opening SVG files for visual comparison..."
echo "Generated: $GENERATED"
echo "Gold Master: $GOLDMASTER"

# Platform-specific file opening
if command -v xdg-open >/dev/null 2>&1; then
    # Linux
    xdg-open "$TMPDIR/generated.svg" &
    xdg-open "$TMPDIR/goldmaster.svg" &
elif command -v open >/dev/null 2>&1; then
    # macOS
    open "$TMPDIR/generated.svg"
    open "$TMPDIR/goldmaster.svg"
elif command -v start >/dev/null 2>&1; then
    # Windows (Git Bash)
    start "$TMPDIR/generated.svg"
    start "$TMPDIR/goldmaster.svg"
else
    echo "Could not find a command to open files. Please manually open:"
    echo "  $TMPDIR/generated.svg"
    echo "  $TMPDIR/goldmaster.svg"
fi

# Optional: If ImageMagick is installed, create a visual diff
if command -v compare >/dev/null 2>&1; then
    echo "Creating visual diff with ImageMagick..."
    compare "$GENERATED" "$GOLDMASTER" "$TMPDIR/diff.png" 2>/dev/null || true
    if [ -f "$TMPDIR/diff.png" ]; then
        echo "Visual diff created at: $TMPDIR/diff.png"
        if command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$TMPDIR/diff.png" &
        elif command -v open >/dev/null 2>&1; then
            open "$TMPDIR/diff.png"
        elif command -v start >/dev/null 2>&1; then
            start "$TMPDIR/diff.png"
        fi
    fi
fi

echo "Press Enter when done reviewing..."
read -r

echo "Done."