use nostr_sdk::PublicKey;
use crossterm::event::KeyEvent;
use super::state::{ActivePane, ConversationId, ContactId, Contact, PendingInvite, PowerToolsMode, LogEntry};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: ConversationId,
    pub sender: PublicKey,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_own: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    // User Input
    KeyPress(KeyEvent),
    SendMessage,
    
    // Navigation
    SelectContact(ContactId),
    SelectConversation(ConversationId),
    SelectInvite(usize),
    SwitchPane(ActivePane),
    ScrollUp(u16),
    ScrollDown(u16),
    
    // Contact Actions
    ShowAddContactDialog,
    AddContact(String, String), // pubkey, petname
    ContactAdded(Contact),
    
    // Conversation Actions
    ShowCreateConversationDialog,
    CreateConversationWithContact(ContactId),
    ConversationCreated(ConversationId),
    
    // MLS Actions
    ShowPublishKeypackageDialog,
    PublishKeypackage,
    KeypackagePublished,
    
    // Invite Actions
    ShowAcceptInviteDialog,
    AcceptInvite(usize),
    InviteAccepted,
    InviteReceived(PendingInvite),
    
    // Dialog Actions
    DialogInput(char),
    DialogBackspace,
    DialogSubmit,
    DialogCancel,
    DialogNextField,
    
    // Network Events
    WebSocketConnected,
    WebSocketDisconnected,
    MessageReceived(ChatMessage),
    MessageSent(String),
    
    // UI Events
    TerminalResized(u16, u16),
    ToggleHelp,
    TogglePowerTools,
    PowerToolsSelect(usize),
    PowerToolsAction,
    PowerToolsModeSwitch(PowerToolsMode),
    LogMessage(LogEntry),
    
    // System Events
    Tick,
    FetchNewMessages,
    Quit,
}

#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    Batch(Vec<Cmd>),
    
    // Contact Commands
    SaveContact(Contact),
    LoadContacts,
    
    // Conversation Commands
    CreateMlsGroup(ContactId),
    
    // Network Commands
    ConnectWebSocket,
    SendMessage(String, ConversationId),
    PublishKeypackageToRelay,
    FetchPendingInvites,
    AcceptPendingInvite(usize),
    FetchNewMessages,
    
    // Power Tools Commands
    ResetAllState,
    DeleteAllContacts,
    DeleteAllConversations,
    RescanRelays,
    RepublishKeypackage,
    
    // Storage Commands
    SaveMessage(ChatMessage),
    LoadConversationHistory(ConversationId),
    SaveConversation(super::state::Conversation),
    LoadConversations,
    
    // System Commands
    Exit,
}