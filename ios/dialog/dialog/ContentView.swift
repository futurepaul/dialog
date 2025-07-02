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
    // @State private var client: DialogClient?
    @State private var secretKey = "Hidden for security"
    @State private var relayUrl = "ws://127.0.0.1:7979"
    @State private var connectionError: String?
    @State private var isConnected = false
    @State private var publicKey = "Test Key - Dialog Client Integration"
    
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
        // Temporary: Comment out DialogClient initialization for testing
        // do {
        //     client = try DialogClient()
        //     publicKey = client?.getPublicKey() ?? "Error"
        //     connectionError = nil
        // } catch {
        //     connectionError = "Failed to initialize client: \(error.localizedDescription)"
        // }
        publicKey = "Demo Public Key - 1234567890abcdef"
        connectionError = nil
    }
    
    private func generateNewKeys() {
        // Temporary: Mock key generation
        // do {
        //     client = try DialogClient()
        //     publicKey = client?.getPublicKey() ?? "Error"
        //     connectionError = nil
        // } catch {
        //     connectionError = "Failed to generate new keys: \(error.localizedDescription)"
        // }
        publicKey = "New Demo Key - \(UUID().uuidString.prefix(16))"
        connectionError = nil
    }
    
    private func toggleConnection() {
        // Temporary: Mock connection toggle
        // guard let client = client else {
        //     connectionError = "Client not initialized"
        //     return
        // }
        // 
        // do {
        //     if isConnected {
        //         try client.disconnectFromRelay()
        //         isConnected = false
        //     } else {
        //         try client.connectToRelay(relayUrl: relayUrl)
        //         isConnected = true
        //     }
        //     connectionError = nil
        // } catch {
        //     connectionError = "Connection error: \(error.localizedDescription)"
        // }
        
        isConnected.toggle()
        connectionError = isConnected ? nil : "Mock disconnection"
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
    // @State private var client: DialogClient?
    @State private var newMessage = ""
    @State private var showingCompose = false
    @State private var errorMessage: String?
    @State private var notes: [MockNoteData] = []
    
    var body: some View {
        NavigationView {
            VStack {
                List {
                    Section("Notes") {
                        ForEach(notes) { note in
                            MockNoteRow(note: note)
                        }
                    }
                }
                
                if notes.isEmpty {
                    VStack {
                        Image(systemName: "message.circle")
                            .font(.system(size: 60))
                            .foregroundColor(.gray)
                        Text("No messages yet")
                            .font(.headline)
                            .foregroundColor(.gray)
                        Text("Publish your first note!")
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
                ComposeNoteView(
                    newMessage: $newMessage,
                    onSend: publishNote
                )
            }
        }
        .onAppear {
            initializeClient()
            fetchMessages()
        }
    }
    
    private func initializeClient() {
        // Temporary: Mock initialization
        // do {
        //     client = try DialogClient()
        //     try client?.connectToRelay(relayUrl: "ws://127.0.0.1:7979")
        //     errorMessage = nil
        // } catch {
        //     errorMessage = "Failed to initialize client: \(error.localizedDescription)"
        // }
        errorMessage = nil
    }
    
    private func fetchMessages() {
        // Temporary: Mock message fetching
        // guard let client = client else {
        //     errorMessage = "Client not initialized"
        //     return
        // }
        // 
        // do {
        //     notes = try client.fetchNotes(limit: 50)
        //     errorMessage = nil
        // } catch {
        //     errorMessage = "Failed to fetch messages: \(error.localizedDescription)"
        // }
        
        notes = [
            MockNoteData(id: "1", content: "Hello from whitenoise!", author: "user123", createdAt: UInt64(Date().timeIntervalSince1970)),
            MockNoteData(id: "2", content: "Testing the app integration", author: "user456", createdAt: UInt64(Date().timeIntervalSince1970 - 3600))
        ]
        errorMessage = nil
    }
    
    private func publishNote() {
        // Temporary: Mock note publishing
        // guard let client = client else {
        //     errorMessage = "Client not initialized"
        //     return
        // }
        // 
        // do {
        //     try client.publishNote(content: newMessage)
        //     newMessage = ""
        //     showingCompose = false
        //     fetchMessages() // Refresh after publishing
        // } catch {
        //     errorMessage = "Failed to publish note: \(error.localizedDescription)"
        // }
        
        let newNote = MockNoteData(
            id: UUID().uuidString,
            content: newMessage,
            author: "currentuser",
            createdAt: UInt64(Date().timeIntervalSince1970)
        )
        notes.insert(newNote, at: 0)
        newMessage = ""
        showingCompose = false
        errorMessage = nil
    }
}

struct GroupsView: View {
    // @State private var dialogClient: DialogClient?
    @State private var groups: [String] = []
    @State private var showingCreateGroup = false
    @State private var errorMessage: String?
    
    var body: some View {
        NavigationView {
            VStack {
                List(groups, id: \.self) { groupId in
                    GroupRowSimple(groupId: groupId)
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
            initializeClient()
            fetchGroups()
        }
    }
    
    private func initializeClient() {
        // Temporary: Mock client initialization
        // do {
        //     dialogClient = try DialogClient()
        //     try dialogClient?.connectToRelay(relayUrl: "ws://127.0.0.1:7979")
        //     errorMessage = nil
        // } catch {
        //     errorMessage = error.localizedDescription
        // }
        errorMessage = nil
    }
    
    private func fetchGroups() {
        // Temporary: Mock group fetching
        // guard let client = dialogClient else {
        //     errorMessage = "Client not initialized"
        //     return
        // }
        // 
        // do {
        //     groups = try client.fetchGroups()
        //     errorMessage = nil
        // } catch {
        //     errorMessage = error.localizedDescription
        // }
        
        groups = ["whitenoise-group-1", "demo-group-abc123"]
        errorMessage = nil
    }
    
    private func createGroup(name: String, members: [String]) {
        // Temporary: Mock group creation
        // guard let client = dialogClient else {
        //     errorMessage = "Client not initialized"
        //     return
        // }
        // 
        // do {
        //     let groupId = try client.createGroup(groupName: name, memberPubkeys: members)
        //     groups.append(groupId)
        //     showingCreateGroup = false
        //     errorMessage = nil
        // } catch {
        //     errorMessage = error.localizedDescription
        // }
        
        let newGroupId = "\(name.lowercased())-\(UUID().uuidString.prefix(8))"
        groups.append(newGroupId)
        showingCreateGroup = false
        errorMessage = nil
    }
}

// MARK: - Supporting Views

struct NoteRow: View {
    let note: NoteData
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(note.shortPubkey)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Text(note.formattedDate)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(note.content)
                .font(.body)
        }
        .padding(.vertical, 2)
    }
}

struct MockNoteRow: View {
    let note: MockNoteData
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(note.shortPubkey)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Text(note.formattedDate)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(note.content)
                .font(.body)
        }
        .padding(.vertical, 2)
    }
}

struct GroupRowSimple: View {
    let groupId: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Group")
                    .font(.headline)
                Spacer()
                Text("ID: \(groupId.prefix(8))...")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text("Group ID: \(groupId)")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 2)
    }
}

struct ComposeNoteView: View {
    @Binding var newMessage: String
    let onSend: () -> Void
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                VStack(alignment: .leading) {
                    Text("Note Content")
                        .font(.headline)
                    TextEditor(text: $newMessage)
                        .frame(minHeight: 150)
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(Color.gray.opacity(0.3), lineWidth: 1)
                        )
                }
                
                Spacer()
            }
            .padding()
            .navigationTitle("New Note")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .confirmationAction) {
                    Button("Publish") {
                        onSend()
                    }
                    .disabled(newMessage.isEmpty)
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
// Using UniFFI DialogClient with whitenoise integration

// Temporary mock data structure for testing
struct MockNoteData: Identifiable {
    let id: String
    let content: String
    let author: String
    let createdAt: UInt64
    
    var shortPubkey: String {
        return String(author.prefix(8)) + "..."
    }
    
    var formattedDate: String {
        let date = Date(timeIntervalSince1970: TimeInterval(createdAt))
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// Extensions for NoteData to provide UI helper properties (commented out for now)
// extension NoteData: Identifiable {
//     public var shortPubkey: String {
//         let pubkey = author
//         return String(pubkey.prefix(8)) + "..."
//     }
//     
//     public var formattedDate: String {
//         let date = Date(timeIntervalSince1970: TimeInterval(createdAt))
//         let formatter = DateFormatter()
//         formatter.dateStyle = .short
//         formatter.timeStyle = .short
//         return formatter.string(from: date)
//     }
// }

#Preview {
    ContentView()
}
