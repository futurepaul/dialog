# Dialog CLI

This CLI provides tools for interacting with the Nostr Messaging Layer Security (MLS) protocol.

## Setup

Before you begin, you'll need a local Nostr relay running and a `.env` file configured with secret keys for two users, "Alice" and "Bob".

### 1. Start a Local Relay

In a separate terminal window, start a local Nostr relay:

```bash
nostr-relay
```

### 2. Configure Environment

Create a `.env` file in the root of the project with the following content:

```env
# Replace with a 64-character hex secret key for Alice
ALICE_SK_HEX=...

# Replace with a 64-character hex secret key for Bob
BOB_SK_HEX=...

# This will be derived from Bob's secret key in the next step
BOB_PK_HEX=...
```

You can generate new secret keys using a Nostr tool of your choice, or by running the `dialog_cli` with a temporary key and copying the output.

### 3. Get Bob's Public Key

Use the `get-pubkey` command to derive Bob's public key from his secret key and update your `.env` file.

```bash
cargo run -- get-pubkey --key bob
```

Copy the output and paste it as the value for `BOB_PK_HEX` in your `.env` file.

## Testing Workflow

Follow these steps to test the end-to-end messaging flow between Alice and Bob.

### Step 1: Publish Key Packages

Both Alice and Bob need to publish their MLS key packages to the relay so they can be discovered by other users.

```bash
# Alice publishes her key package
cargo run -- publish-key --key alice

# Bob publishes his key package
cargo run -- publish-key --key bob
```

You can verify that the key packages were published by listing all key packages on the relay:

```bash
cargo run -- list
```

### Step 2: Alice Creates a Group

Alice creates a new secure group and invites Bob.

```bash
cargo run -- create-group --key alice --name "alice-and-bob" --counterparty $(grep BOB_PK_HEX .env | cut -d '=' -f2)
```

Take note of the `Group ID` from the output of this command. You will need it in the following steps.

### Step 3: Alice Sends a Message

Alice sends the first message to the group.

```bash
# Replace <GROUP_ID> with the ID from the previous step
cargo run -- send-message --key alice --group-id <GROUP_ID> --message "ping"
```

### Step 4: Bob Lists and Accepts the Invitation

Bob checks for any pending group invitations and accepts the one from Alice.

```bash
# Bob lists his pending invites
cargo run -- list-invites --key bob

# Bob accepts the invitation
# Replace <GROUP_ID> with the ID from Step 2
cargo run -- accept-invite --key bob --group-id <GROUP_ID>
```

### Step 5: Bob Sends a Reply

Now that Bob has joined the group, he can send a message back to Alice.

```bash
# Replace <GROUP_ID> with the ID from Step 2
cargo run -- send-message --key bob --group-id <GROUP_ID> --message "pong"
```

You have now successfully completed an end-to-end test of the Dialog CLI! 