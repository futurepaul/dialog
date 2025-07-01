//
//  ContentView.swift
//  dialog
//
//  Created by Paul Miller on 7/1/25.
//

import SwiftUI
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

struct ContentView: View {
    @State private var selectedTab = 0
    
    var body: some View {
        TabView(selection: $selectedTab) {
            KeysView()
                .tabItem {
                    Image(systemName: "key")
                    Text("Keys")
                }
                .tag(0)
            
            MessagesView()
                .tabItem {
                    Image(systemName: "message")
                    Text("Messages")
                }
                .tag(1)
            
            GroupsView()
                .tabItem {
                    Image(systemName: "person.3")
                    Text("Groups")
                }
                .tag(2)
        }
    }
}

struct KeysView: View {
    @State private var publicKey = "Loading..."
    @State private var secretKey = "Loading..."
    @State private var relayUrl = "ws://127.0.0.1:7979"
    @State private var isConnected = false
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Identity")
                        .font(.headline)
                    
                    VStack(alignment: .leading, spacing: 5) {
                        Text("Public Key:")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Text(publicKey)
                            .font(.system(.caption, design: .monospaced))
                            .padding(8)
                            .background(Color.gray.opacity(0.1))
                            .cornerRadius(8)
                            .onTapGesture {
                                copyToClipboard(publicKey)
                            }
                    }
                    
                    VStack(alignment: .leading, spacing: 5) {
                        Text("Secret Key:")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Text(secretKey)
                            .font(.system(.caption, design: .monospaced))
                            .padding(8)
                            .background(Color.gray.opacity(0.1))
                            .cornerRadius(8)
                            .onTapGesture {
                                copyToClipboard(secretKey)
                            }
                    }
                }
                
                VStack(alignment: .leading, spacing: 10) {
                    Text("Relay Connection")
                        .font(.headline)
                    
                    HStack {
                        TextField("Relay URL", text: $relayUrl)
                            .textFieldStyle(RoundedBorderTextFieldStyle())
                        
                        Button(isConnected ? "Disconnect" : "Connect") {
                            toggleConnection()
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    
                    HStack {
                        Circle()
                            .fill(isConnected ? Color.green : Color.red)
                            .frame(width: 8, height: 8)
                        Text(isConnected ? "Connected" : "Disconnected")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                Spacer()
                
                Button("Generate New Keys") {
                    generateNewKeys()
                }
                .buttonStyle(.bordered)
            }
            .padding()
            .navigationTitle("Dialog Client")
        }
        .onAppear {
            initializeClient()
        }
    }
    
    private func initializeClient() {
        // TODO: Initialize DialogClient via UniFFI
        // For now, use mock data
        publicKey = "c4bbc395bf52bffb9e1b30a49a5f71142ac202f143e4db5556abc1b33c965227"
        secretKey = "d6d579ad23f4ddefe9a9db062be4f3afe4dc200e60ebf5249117003704ce56a8"
    }
    
    private func generateNewKeys() {
        // TODO: Call DialogClient.new() via UniFFI
        // For now, use mock data
        publicKey = "5fec84110135b91092aaac6bc7bb1a915c946ef8dce19017ecaee8eda9bc5fea"
        secretKey = "7a199e5bc77481b7f6b4738005f01afb4abe14136f4951517edea7e782c0a46c"
    }
    
    private func toggleConnection() {
        // TODO: Call DialogClient.connect_to_relay() via UniFFI
        isConnected.toggle()
    }
    
    private func copyToClipboard(_ text: String) {
        #if canImport(UIKit)
        UIPasteboard.general.string = text
        #elseif canImport(AppKit)
        NSPasteboard.general.setString(text, forType: .string)
        #endif
    }
}

struct MessagesView: View {
    @State private var messages: [EncryptedMessageData] = []
    @State private var newMessage = ""
    @State private var recipientPubkey = ""
    @State private var showingCompose = false
    
    var body: some View {
        NavigationView {
            VStack {
                List(messages) { message in
                    MessageRow(message: message)
                }
                
                if messages.isEmpty {
                    VStack {
                        Image(systemName: "message.circle")
                            .font(.system(size: 60))
                            .foregroundColor(.gray)
                        Text("No messages yet")
                            .font(.headline)
                            .foregroundColor(.gray)
                        Text("Send your first encrypted message!")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .navigationTitle("Messages")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showingCompose = true
                    } label: {
                        Image(systemName: "plus")
                    }
                }
                
                ToolbarItem(placement: .secondaryAction) {
                    Button("Refresh") {
                        fetchMessages()
                    }
                }
            }
            .sheet(isPresented: $showingCompose) {
                ComposeMessageView(
                    recipientPubkey: $recipientPubkey,
                    newMessage: $newMessage,
                    onSend: sendMessage
                )
            }
        }
        .onAppear {
            fetchMessages()
        }
    }
    
    private func fetchMessages() {
        // TODO: Call DialogClient.get_encrypted_messages() via UniFFI
        // For now, use mock data
        messages = [
            EncryptedMessageData(
                id: "1",
                content: "Hello! This is an encrypted message.",
                sender: "5fec8411...",
                timestamp: Date().addingTimeInterval(-3600)
            ),
            EncryptedMessageData(
                id: "2", 
                content: "Hi there! I got your message.",
                sender: "96088dbf...",
                timestamp: Date().addingTimeInterval(-1800)
            )
        ]
    }
    
    private func sendMessage() {
        // TODO: Call DialogClient.send_encrypted_message() via UniFFI
        let newMsg = EncryptedMessageData(
            id: UUID().uuidString,
            content: newMessage,
            sender: "You",
            timestamp: Date()
        )
        messages.append(newMsg)
        newMessage = ""
        showingCompose = false
    }
}

struct GroupsView: View {
    @State private var groups: [GroupData] = []
    @State private var showingCreateGroup = false
    
    var body: some View {
        NavigationView {
            VStack {
                List(groups) { group in
                    GroupRow(group: group)
                }
                
                if groups.isEmpty {
                    VStack {
                        Image(systemName: "person.3.fill")
                            .font(.system(size: 60))
                            .foregroundColor(.gray)
                        Text("No groups yet")
                            .font(.headline)
                            .foregroundColor(.gray)
                        Text("Create your first encrypted group!")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .navigationTitle("Groups")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showingCreateGroup = true
                    } label: {
                        Image(systemName: "plus")
                    }
                }
            }
            .sheet(isPresented: $showingCreateGroup) {
                CreateGroupView(onCreate: createGroup)
            }
        }
        .onAppear {
            fetchGroups()
        }
    }
    
    private func fetchGroups() {
        // TODO: Fetch groups from relay
        // For now, use mock data
        groups = [
            GroupData(
                id: "1",
                name: "Test Group",
                memberCount: 3,
                lastMessage: "Welcome to the group!"
            )
        ]
    }
    
    private func createGroup(name: String, members: [String]) {
        // TODO: Call DialogClient.create_group() via UniFFI
        let newGroup = GroupData(
            id: UUID().uuidString,
            name: name,
            memberCount: members.count + 1,
            lastMessage: "Group created"
        )
        groups.append(newGroup)
        showingCreateGroup = false
    }
}

// MARK: - Supporting Views

struct MessageRow: View {
    let message: EncryptedMessageData
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(message.sender)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Text(message.timestamp, style: .time)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(message.content)
                .font(.body)
        }
        .padding(.vertical, 2)
    }
}

struct GroupRow: View {
    let group: GroupData
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(group.name)
                    .font(.headline)
                Spacer()
                Text("\(group.memberCount) members")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(group.lastMessage)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 2)
    }
}

struct ComposeMessageView: View {
    @Binding var recipientPubkey: String
    @Binding var newMessage: String
    let onSend: () -> Void
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                VStack(alignment: .leading) {
                    Text("Recipient Public Key")
                        .font(.headline)
                    TextField("Enter public key...", text: $recipientPubkey)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .font(.system(.body, design: .monospaced))
                }
                
                VStack(alignment: .leading) {
                    Text("Message")
                        .font(.headline)
                    TextEditor(text: $newMessage)
                        .frame(minHeight: 100)
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(Color.gray.opacity(0.3), lineWidth: 1)
                        )
                }
                
                Spacer()
            }
            .padding()
            .navigationTitle("New Message")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .confirmationAction) {
                    Button("Send") {
                        onSend()
                    }
                    .disabled(recipientPubkey.isEmpty || newMessage.isEmpty)
                }
            }
        }
    }
}

struct CreateGroupView: View {
    @State private var groupName = ""
    @State private var memberPubkeys = ""
    let onCreate: (String, [String]) -> Void
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                VStack(alignment: .leading) {
                    Text("Group Name")
                        .font(.headline)
                    TextField("Enter group name...", text: $groupName)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                }
                
                VStack(alignment: .leading) {
                    Text("Member Public Keys")
                        .font(.headline)
                    Text("Enter one public key per line")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextEditor(text: $memberPubkeys)
                        .frame(minHeight: 150)
                        .font(.system(.body, design: .monospaced))
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(Color.gray.opacity(0.3), lineWidth: 1)
                        )
                }
                
                Spacer()
            }
            .padding()
            .navigationTitle("New Group")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .confirmationAction) {
                    Button("Create") {
                        let members = memberPubkeys
                            .components(separatedBy: .newlines)
                            .compactMap { $0.trimmingCharacters(in: .whitespaces) }
                            .filter { !$0.isEmpty }
                        onCreate(groupName, members)
                    }
                    .disabled(groupName.isEmpty)
                }
            }
        }
    }
}

// MARK: - Data Models

struct EncryptedMessageData: Identifiable {
    let id: String
    let content: String
    let sender: String
    let timestamp: Date
}

struct GroupData: Identifiable {
    let id: String
    let name: String
    let memberCount: Int
    let lastMessage: String
}

#Preview {
    ContentView()
}
