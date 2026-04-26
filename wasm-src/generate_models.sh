#!/bin/bash
# Generate EmpiricalModel lookup tables for all zones
#
# This script generates V1-style optimized models (~79KB each) for all 13 zones
# with Hybrid battery mode only (most realistic utilization).
#
# Usage:
#   ./generate_models.sh [--all] [--zones "zone1,zone2,..."]
#
# Options:
#   --all         Generate models for all 13 zones
#   --zones       Generate models for specific zones (comma-separated)
#
# Output:
#   models/<zone>_hybrid.bin files (V1 optimized, ~79KB each)
#   web/public/models/<zone>_hybrid.bin (copied for serving)
#
# Requirements:
#   - Rust toolchain with wasm-pack
#   - rayon feature enabled (cargo build --features native)

set -e

# Default zones (all 13 from zones.json)
ALL_ZONES="california,texas,florida,new_york,mid-atlantic,midwest,mountain,new_england,northwest,plains,southeast,southwest,delta"

# Parse arguments
ZONES="california"  # Default to just California for testing
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            ZONES="$ALL_ZONES"
            shift
            ;;
        --zones)
            ZONES="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "=== EmpiricalModel Generation ==="
echo "Zones: $ZONES"
echo ""

# Build the generator
echo "Building training data generator..."
cargo build --release --features native --bin generate_training_data

# Generate models
echo ""
echo "Generating models..."
cargo run --release --features native --bin generate_training_data -- \
    --zones "$ZONES" \
    --modes hybrid \
    --grid v1 \
    --data ../data/zones.json

# Copy to web/public/models for serving
echo ""
echo "Copying models to web/public/models..."
mkdir -p ../web/public/models
cp models/*_hybrid.bin ../web/public/models/ 2>/dev/null || true

# Summary
echo ""
echo "=== Summary ==="
echo "Models generated:"
ls -la models/*.bin 2>/dev/null || echo "No models found in models/"
echo ""
echo "Models deployed to web:"
ls -la ../web/public/models/*.bin 2>/dev/null || echo "No models found in web/public/models/"
echo ""
echo "Done!"
