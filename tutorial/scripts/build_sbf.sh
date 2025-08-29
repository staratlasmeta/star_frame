#!/usr/bin/env bash

# Star Frame Tutorial SBF Builder
# Build all tutorial programs as Solana BPF/SBF (.so files)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TUTORIALS=("basic-0" "basic-1" "basic-2" "basic-4" "basic-5")
BASIC_3_PROGRAMS=("puppet" "puppet-master")
FAILED_BUILDS=()

# Function to print colored output
print_color() {
    echo -e "${1}${2}${NC}"
}

# Function to print section header
print_header() {
    echo ""
    print_color "$BLUE" "========================================="
    print_color "$BLUE" "$1"
    print_color "$BLUE" "========================================="
}

# Function to build a single program
build_program() {
    local dir=$1
    local name=$2
    
    if [ -d "$dir" ]; then
        print_color "$YELLOW" "Building $name as SBF..."
        
        if (cd "$dir" && cargo build-sbf 2>&1 | tee build.log); then
            # Check if .so file was created
            if [ -f "$dir/target/deploy/${name//-/_}.so" ]; then
                print_color "$GREEN" "✓ $name built successfully"
                local size=$(du -h "$dir/target/deploy/${name//-/_}.so" | cut -f1)
                print_color "$GREEN" "  → Program size: $size"
            else
                print_color "$RED" "✗ $name build completed but .so file not found"
                FAILED_BUILDS+=("$name")
            fi
        else
            print_color "$RED" "✗ Failed to build $name"
            FAILED_BUILDS+=("$name")
        fi
    else
        print_color "$YELLOW" "⚠ Directory $dir not found, skipping..."
    fi
}

# Function to test a program with SBF
test_program() {
    local dir=$1
    local name=$2
    
    if [ -d "$dir" ] && [ -f "$dir/target/deploy/${name//-/_}.so" ]; then
        print_color "$YELLOW" "Testing $name with SBF binary..."
        
        if (cd "$dir" && USE_BIN=true cargo test 2>&1 | grep -E "(test result:|passed|failed)"); then
            print_color "$GREEN" "✓ $name tests completed"
        else
            print_color "$RED" "✗ $name tests failed or had issues"
        fi
    fi
}

# Main execution
main() {
    print_header "Star Frame Tutorial SBF Builder"
    
    # Parse command line arguments
    RUN_TESTS=false
    CLEAN_FIRST=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --test)
                RUN_TESTS=true
                shift
                ;;
            --clean)
                CLEAN_FIRST=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --test    Run tests after building"
                echo "  --clean   Clean before building"
                echo "  --help    Show this help message"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Clean if requested
    if [ "$CLEAN_FIRST" = true ]; then
        print_header "Cleaning Previous Builds"
        for tutorial in "${TUTORIALS[@]}"; do
            if [ -d "$tutorial/target/deploy" ]; then
                print_color "$YELLOW" "Cleaning $tutorial..."
                rm -rf "$tutorial/target/deploy"
            fi
        done
        
        for program in "${BASIC_3_PROGRAMS[@]}"; do
            if [ -d "basic-3/$program/target/deploy" ]; then
                print_color "$YELLOW" "Cleaning basic-3/$program..."
                rm -rf "basic-3/$program/target/deploy"
            fi
        done
    fi
    
    # Build basic tutorials
    print_header "Building Basic Tutorials"
    
    for tutorial in "${TUTORIALS[@]}"; do
        build_program "$tutorial" "$tutorial"
    done
    
    # Build basic-3 sub-programs
    print_header "Building Basic-3 Programs"
    
    for program in "${BASIC_3_PROGRAMS[@]}"; do
        build_program "basic-3/$program" "$program"
    done
    
    # Run tests if requested
    if [ "$RUN_TESTS" = true ]; then
        print_header "Running Tests with SBF Binaries"
        
        for tutorial in "${TUTORIALS[@]}"; do
            test_program "$tutorial" "$tutorial"
        done
        
        for program in "${BASIC_3_PROGRAMS[@]}"; do
            test_program "basic-3/$program" "$program"
        done
    fi
    
    # Summary
    print_header "Build Summary"
    
    local total=$((${#TUTORIALS[@]} + ${#BASIC_3_PROGRAMS[@]}))
    local succeeded=$((total - ${#FAILED_BUILDS[@]}))
    
    print_color "$BLUE" "Total programs: $total"
    print_color "$GREEN" "Succeeded: $succeeded"
    
    if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
        print_color "$RED" "Failed: ${#FAILED_BUILDS[@]}"
        print_color "$RED" "Failed builds: ${FAILED_BUILDS[*]}"
        exit 1
    else
        print_color "$GREEN" ""
        print_color "$GREEN" "✨ All tutorials built as SBF programs successfully!"
    fi
}

# Run main function
main "$@"