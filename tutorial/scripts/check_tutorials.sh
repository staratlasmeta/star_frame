#!/bin/bash

# Script to check that all basic tutorials compile successfully
# Run from the tutorial directory

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TOTAL=0
PASSED=0
FAILED=0
FAILED_TUTORIALS=()

echo "========================================="
echo "Star Frame Tutorial Compilation Checker"
echo "========================================="
echo ""

# Function to check a single tutorial
check_tutorial() {
    local dir=$1
    local name=$(basename "$dir")
    
    echo -n "Checking $name... "
    TOTAL=$((TOTAL + 1))
    
    if [ -f "$dir/Cargo.toml" ]; then
        # Try to compile
        if cd "$dir" && cargo check --quiet 2>/dev/null; then
            echo -e "${GREEN}✓ PASSED${NC}"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}✗ FAILED${NC}"
            FAILED=$((FAILED + 1))
            FAILED_TUTORIALS+=("$name")
        fi
        cd - > /dev/null
    else
        echo -e "${YELLOW}⚠ SKIPPED (no Cargo.toml)${NC}"
    fi
}

# Check basic-0 through basic-5
for i in {0..5}; do
    if [ -d "basic-$i" ]; then
        # Special handling for basic-3 which has two sub-projects
        if [ $i -eq 3 ]; then
            if [ -d "basic-3/puppet" ]; then
                check_tutorial "basic-3/puppet"
            fi
            if [ -d "basic-3/puppet-master" ]; then
                check_tutorial "basic-3/puppet-master"
            fi
        else
            check_tutorial "basic-$i"
        fi
    fi
done

# Summary
echo ""
echo "========================================="
echo "Summary"
echo "========================================="
echo -e "Total:  $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [ ${#FAILED_TUTORIALS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed tutorials:${NC}"
    for tutorial in "${FAILED_TUTORIALS[@]}"; do
        echo "  - $tutorial"
    done
    exit 1
else
    echo ""
    echo -e "${GREEN}All tutorials compiled successfully!${NC}"
    exit 0
fi