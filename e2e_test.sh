#!/bin/bash

set -e

echo "🧪 Starting Dialog E2E Test"
echo "=========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
RELAY_URL="ws://127.0.0.1:7979"
TEST_MESSAGE="Hello from E2E test! $(date)"

# Function to print colored output
print_step() {
    echo -e "${BLUE}📋 Step: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Function to cleanup on exit
cleanup() {
    print_info "Cleaning up..."
    if [ ! -z "$RELAY_PID" ]; then
        print_info "Stopping relay (PID: $RELAY_PID)"
        kill -TERM $RELAY_PID 2>/dev/null || true
        wait $RELAY_PID 2>/dev/null || true
    fi
    
    # Kill any remaining dialog_relay processes
    pkill -f dialog_relay 2>/dev/null || true
    
    print_info "Cleanup complete"
}

# Set up cleanup trap
trap cleanup EXIT

# Step 1: Build everything
print_step "Building workspace"
cargo build --workspace
print_success "Build complete"

# Step 2: Start the relay in background  
print_step "Starting relay with debug logging"
export RUST_LOG=dialog_relay=debug,nostr_relay_builder=debug,dialog_client=info,dialog_cli=info,info
cargo run -p dialog_relay &
RELAY_PID=$!

print_info "Relay started with PID: $RELAY_PID"
print_info "Waiting for relay to initialize..."

# Wait for relay to start up
sleep 5

# Check if relay is still running
if ! kill -0 $RELAY_PID 2>/dev/null; then
    print_error "Relay failed to start!"
    exit 1
fi

print_success "Relay is running"

# Step 3: Test publishing a note
print_step "Publishing test note"
print_info "Message: '$TEST_MESSAGE'"

# Run publish command and capture output
if cargo run -p dialog_cli publish "$TEST_MESSAGE" --relay $RELAY_URL; then
    print_success "Note published successfully"
else
    print_error "Failed to publish note"
    exit 1
fi

# Step 4: Wait a moment for the note to be stored
print_info "Waiting for note to be processed..."
sleep 2

# Step 5: Retrieve recent notes
print_step "Retrieving recent notes"

if cargo run -p dialog_cli fetch --limit 5 --relay $RELAY_URL; then
    print_success "Notes retrieved successfully"
else
    print_error "Failed to retrieve notes"
    exit 1
fi

# Step 6: Test the CLI test command
print_step "Running CLI test command"

if cargo run -p dialog_cli test --message "E2E test message" --relay $RELAY_URL; then
    print_success "CLI test command completed"
else
    print_error "CLI test command failed"
    exit 1
fi

# Step 7: Final health check
print_step "Final relay health check"

if kill -0 $RELAY_PID 2>/dev/null; then
    print_success "Relay is still running healthy"
else
    print_error "Relay stopped unexpectedly"
    exit 1
fi

# Success!
echo ""
echo "🎉 E2E Test Results:"
echo "===================="
print_success "✅ Relay startup"
print_success "✅ Note publishing"  
print_success "✅ Note retrieval"
print_success "✅ CLI test command"
print_success "✅ Relay health check"
echo ""
print_success "All tests passed! 🚀"

print_info "Relay will be stopped during cleanup..." 