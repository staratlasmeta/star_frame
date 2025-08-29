#!/bin/bash

# Detailed script to check Star Frame tutorials compilation
# Supports various options for testing

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
VERBOSE=false
CHECK_TESTS=false
CHECK_IDL=false
CHECK_CLIPPY=false
PARALLEL=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -t|--tests)
            CHECK_TESTS=true
            shift
            ;;
        -i|--idl)
            CHECK_IDL=true
            shift
            ;;
        -c|--clippy)
            CHECK_CLIPPY=true
            shift
            ;;
        -p|--parallel)
            PARALLEL=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose    Show detailed output"
            echo "  -t, --tests      Also run tests"
            echo "  -i, --idl        Check with IDL feature"
            echo "  -c, --clippy     Run clippy lints"
            echo "  -p, --parallel   Run checks in parallel"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

# Track results
TOTAL=0
PASSED=0
FAILED=0
FAILED_TUTORIALS=()

echo "========================================="
echo "Star Frame Tutorial Compilation Checker"
echo "========================================="
echo ""

if [ "$VERBOSE" = true ]; then
    echo -e "${CYAN}Configuration:${NC}"
    echo "  Verbose: $VERBOSE"
    echo "  Check tests: $CHECK_TESTS"
    echo "  Check IDL: $CHECK_IDL"
    echo "  Check clippy: $CHECK_CLIPPY"
    echo "  Parallel: $PARALLEL"
    echo ""
fi

# Function to run cargo command with proper verbosity
run_cargo() {
    local cmd=$1
    if [ "$VERBOSE" = true ]; then
        $cmd
    else
        $cmd --quiet 2>/dev/null
    fi
}

# Function to check a single tutorial
check_tutorial() {
    local dir=$1
    local name=$(basename "$dir")
    
    echo -e "${BLUE}Checking $name...${NC}"
    TOTAL=$((TOTAL + 1))
    
    if [ ! -f "$dir/Cargo.toml" ]; then
        echo -e "  ${YELLOW}⚠ SKIPPED (no Cargo.toml)${NC}"
        return
    fi
    
    local all_passed=true
    
    # Change to tutorial directory
    pushd "$dir" > /dev/null
    
    # Basic compilation check
    echo -n "  Compile check: "
    if run_cargo "cargo check"; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        all_passed=false
    fi
    
    # IDL feature check
    if [ "$CHECK_IDL" = true ]; then
        echo -n "  IDL feature: "
        if run_cargo "cargo check --features idl"; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
            all_passed=false
        fi
    fi
    
    # Test check
    if [ "$CHECK_TESTS" = true ]; then
        echo -n "  Tests: "
        if run_cargo "cargo test --features idl"; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
            all_passed=false
        fi
    fi
    
    # Clippy check
    if [ "$CHECK_CLIPPY" = true ]; then
        echo -n "  Clippy: "
        if run_cargo "cargo clippy -- -D warnings"; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
            all_passed=false
        fi
    fi
    
    popd > /dev/null
    
    # Update counters
    if [ "$all_passed" = true ]; then
        echo -e "  ${GREEN}Overall: PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}Overall: FAILED${NC}"
        FAILED=$((FAILED + 1))
        FAILED_TUTORIALS+=("$name")
    fi
    echo ""
}

# Function to check tutorial in parallel
check_tutorial_parallel() {
    local dir=$1
    check_tutorial "$dir" &
}

# Main checking loop
if [ "$PARALLEL" = true ]; then
    echo -e "${MAGENTA}Running checks in parallel...${NC}"
    echo ""
fi

for i in {0..5}; do
    if [ -d "basic-$i" ]; then
        # Special handling for basic-3 which has two sub-projects
        if [ $i -eq 3 ]; then
            if [ -d "basic-3/puppet" ]; then
                if [ "$PARALLEL" = true ]; then
                    check_tutorial_parallel "basic-3/puppet"
                else
                    check_tutorial "basic-3/puppet"
                fi
            fi
            if [ -d "basic-3/puppet-master" ]; then
                if [ "$PARALLEL" = true ]; then
                    check_tutorial_parallel "basic-3/puppet-master"
                else
                    check_tutorial "basic-3/puppet-master"
                fi
            fi
        else
            if [ "$PARALLEL" = true ]; then
                check_tutorial_parallel "basic-$i"
            else
                check_tutorial "basic-$i"
            fi
        fi
    fi
done

# Wait for parallel jobs to complete
if [ "$PARALLEL" = true ]; then
    wait
fi

# Summary
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
    
    # Provide helpful next steps
    echo ""
    echo -e "${YELLOW}To debug failures, run with --verbose flag:${NC}"
    echo "  $0 --verbose"
    exit 1
else
    echo ""
    echo -e "${GREEN}✨ All tutorials compiled successfully!${NC}"
    exit 0
fi