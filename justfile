# Dialog CLI Shortcuts
# Run `just` to see all available commands

# Show all available commands
@default:
    just --list

# Setup environment file with Alice and Bob keys
setup:
    #!/usr/bin/env bash
    echo "Setting up .env.local with Alice and Bob keys..."
    cat > .env.local << 'EOF'
    ALICE_SK_HEX=7c2e3f5a8b9c1d4e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e
    BOB_SK_HEX=1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b
    EOF
    BOB_PK=$(cargo run -p dialog_cli -- get-pubkey --key bob)
    echo "BOB_PK_HEX=$BOB_PK" >> .env.local
    echo "✅ Environment setup complete!"
    echo ""
    echo "💡 For easier CLI usage, add shell aliases:"
    just aliases

# Run tests
test:
    cargo test -p dialog_cli

# Show recommended shell aliases (best approach)
aliases:
    @echo "Add these to your ~/.bashrc or ~/.zshrc:"
    @echo ""
    @echo "alias alice='f() { cargo run -p dialog_cli -- \"\$@\" --key alice; }; f'"
    @echo "alias bob='f() { cargo run -p dialog_cli -- \"\$@\" --key bob; }; f'"
    @echo ""
    @echo "Then use them like:"
    @echo "  alice get-pubkey"
    @echo "  alice send-message --group-id \$GROUP_ID --message \"any message with spaces\""
    @echo "  bob list-invites"
    @echo "  bob accept-invite --group-id \$GROUP_ID"
    @echo ""
    @echo "This handles quotes and spaces perfectly!" 