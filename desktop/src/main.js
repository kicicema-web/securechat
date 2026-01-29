const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

// State
let currentConversation = null;
let conversations = [];
let contacts = [];

// Initialize
async function init() {
  // Check if account exists
  const hasAccount = await invoke('has_account');
  
  if (hasAccount) {
    showLogin();
  } else {
    showCreateAccount();
  }
  
  // Listen for events
  listen('message-received', (event) => {
    console.log('Message received:', event);
    loadConversations();
    if (currentConversation && event.payload.conversation_id === currentConversation.id) {
      loadMessages(currentConversation.id);
    }
  });
  
  listen('contact-online', (event) => {
    console.log('Contact online:', event);
    updateContactStatus(event.payload.contact_id, true);
  });
  
  listen('contact-offline', (event) => {
    console.log('Contact offline:', event);
    updateContactStatus(event.payload.contact_id, false);
  });
  
  listen('error', (event) => {
    console.error('Error:', event);
    showError(event.payload.message);
  });
}

// Auth functions
async function createAccount() {
  const displayName = document.getElementById('display-name').value.trim();
  const password = document.getElementById('new-password').value;
  const confirmPassword = document.getElementById('confirm-password').value;
  
  if (!displayName) {
    showError('Please enter a display name');
    return;
  }
  
  if (password.length < 8) {
    showError('Password must be at least 8 characters');
    return;
  }
  
  if (password !== confirmPassword) {
    showError('Passwords do not match');
    return;
  }
  
  try {
    await invoke('create_account', { password, displayName });
    await startApp();
  } catch (e) {
    showError(e);
  }
}

async function unlockAccount() {
  const password = document.getElementById('password').value;
  
  if (!password) {
    showError('Please enter your password');
    return;
  }
  
  try {
    await invoke('unlock_account', { password });
    await startApp();
  } catch (e) {
    showError(e);
  }
}

async function startApp() {
  // Hide auth, show app
  document.getElementById('auth-screen').style.display = 'none';
  document.getElementById('app').classList.add('show');
  
  // Load user profile
  const profile = await invoke('get_profile');
  if (profile) {
    document.getElementById('user-name').textContent = profile.display_name;
    document.getElementById('user-avatar').textContent = profile.display_name.charAt(0).toUpperCase();
  }
  
  // Load conversations
  await loadConversations();
  
  // Start network
  try {
    await invoke('start_network');
    console.log('Network started');
  } catch (e) {
    console.error('Failed to start network:', e);
  }
}

// UI functions
function showLogin() {
  document.getElementById('create-account-form').style.display = 'none';
  document.getElementById('login-form').style.display = 'block';
  document.getElementById('error-message').classList.remove('show');
}

function showCreateAccount() {
  document.getElementById('login-form').style.display = 'none';
  document.getElementById('create-account-form').style.display = 'block';
  document.getElementById('error-message').classList.remove('show');
}

function showError(message) {
  const errorEl = document.getElementById('error-message');
  errorEl.textContent = message;
  errorEl.classList.add('show');
}

// Conversation functions
async function loadConversations() {
  try {
    conversations = await invoke('get_conversations');
    renderConversations();
  } catch (e) {
    console.error('Failed to load conversations:', e);
  }
}

function renderConversations() {
  const container = document.getElementById('conversations-list');
  container.innerHTML = '';
  
  for (const conv of conversations) {
    const item = document.createElement('div');
    item.className = 'conversation-item' + (currentConversation?.id === conv.id ? ' active' : '');
    item.onclick = () => selectConversation(conv);
    
    const contact = contacts.find(c => c.id === conv.contact_id);
    const name = contact?.display_name || 'Unknown';
    const initial = name.charAt(0).toUpperCase();
    
    const time = conv.updated_at 
      ? new Date(conv.updated_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      : '';
    
    const preview = conv.last_message_preview || 'No messages yet';
    
    item.innerHTML = `
      <div class="conversation-avatar">${initial}</div>
      <div class="conversation-info">
        <div class="conversation-header">
          <div class="conversation-name">${escapeHtml(name)}</div>
          <div class="conversation-time">${time}</div>
        </div>
        <div class="conversation-preview">
          ${escapeHtml(preview)}
          ${conv.unread_count > 0 ? `<span class="unread-badge">${conv.unread_count}</span>` : ''}
        </div>
      </div>
    `;
    
    container.appendChild(item);
  }
}

async function selectConversation(conv) {
  currentConversation = conv;
  
  // Update UI
  document.getElementById('empty-state').style.display = 'none';
  document.getElementById('active-chat').style.display = 'flex';
  
  // Get contact info
  const contact = contacts.find(c => c.id === conv.contact_id);
  const name = contact?.display_name || 'Unknown';
  
  document.getElementById('chat-name').textContent = name;
  document.getElementById('chat-avatar').textContent = name.charAt(0).toUpperCase();
  
  // Load messages
  await loadMessages(conv.id);
  
  // Update conversation list highlighting
  renderConversations();
}

async function loadMessages(conversationId) {
  try {
    const messages = await invoke('get_messages', { conversationId, limit: 50 });
    renderMessages(messages);
  } catch (e) {
    console.error('Failed to load messages:', e);
  }
}

function renderMessages(messages) {
  const container = document.getElementById('messages-container');
  container.innerHTML = '';
  
  for (const msg of messages) {
    const messageEl = document.createElement('div');
    messageEl.className = `message ${msg.is_outgoing ? 'outgoing' : 'incoming'}`;
    
    const content = formatMessageContent(msg.content);
    const time = new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    const status = msg.is_outgoing ? getStatusIcon(msg) : '';
    
    messageEl.innerHTML = `
      ${content}
      <div class="message-meta">
        ${time}
        ${status}
      </div>
    `;
    
    container.appendChild(messageEl);
  }
  
  // Scroll to bottom
  container.scrollTop = container.scrollHeight;
}

function formatMessageContent(content) {
  if (content.Text) {
    return escapeHtml(content.Text.text);
  } else if (content.Image) {
    return '📷 Image' + (content.Image.caption ? ': ' + escapeHtml(content.Image.caption) : '');
  } else if (content.File) {
    return '📎 ' + escapeHtml(content.File.filename);
  } else if (content.Voice) {
    return '🎤 Voice message (' + content.Voice.duration_secs + 's)';
  } else if (content.Location) {
    return '📍 Location';
  }
  return 'Unknown message type';
}

function getStatusIcon(msg) {
  if (msg.read) return '✓✓';
  if (msg.delivered) return '✓✓';
  if (msg.sent) return '✓';
  return '⏳';
}

async function sendMessage() {
  const input = document.getElementById('message-input');
  const text = input.value.trim();
  
  if (!text || !currentConversation) return;
  
  try {
    await invoke('send_text_message', { 
      conversationId: currentConversation.id, 
      text 
    });
    
    input.value = '';
    input.rows = 1;
    
    // Reload messages
    await loadMessages(currentConversation.id);
    await loadConversations();
  } catch (e) {
    console.error('Failed to send message:', e);
    showError('Failed to send message: ' + e);
  }
}

function handleKeyDown(event) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    sendMessage();
  }
  
  // Auto-resize textarea
  const textarea = event.target;
  textarea.rows = 1;
  const lines = Math.min(5, Math.ceil(textarea.scrollHeight / 24));
  textarea.rows = lines;
}

// Contact functions
async function showNewContactModal() {
  const modal = document.getElementById('new-contact-modal');
  modal.classList.add('show');
  
  // Get public key
  try {
    const publicKey = await invoke('get_public_key');
    const keyStr = btoa(String.fromCharCode(...publicKey));
    document.getElementById('my-public-key').value = keyStr;
    document.getElementById('qr-code').textContent = keyStr.substring(0, 20) + '...';
  } catch (e) {
    console.error('Failed to get public key:', e);
  }
}

function closeModal() {
  document.getElementById('new-contact-modal').classList.remove('show');
}

async function addContact() {
  const name = document.getElementById('contact-name').value.trim();
  const keyStr = document.getElementById('contact-key').value.trim();
  
  if (!name || !keyStr) {
    showError('Please enter both name and public key');
    return;
  }
  
  try {
    // Decode base64 key
    const keyBytes = Uint8Array.from(atob(keyStr), c => c.charCodeAt(0));
    
    const contact = await invoke('add_contact', { 
      publicKey: Array.from(keyBytes), 
      displayName: name 
    });
    
    // Create conversation
    await invoke('get_or_create_conversation', { contactId: contact.id });
    
    // Reload
    await loadConversations();
    closeModal();
    
    // Clear form
    document.getElementById('contact-name').value = '';
    document.getElementById('contact-key').value = '';
  } catch (e) {
    console.error('Failed to add contact:', e);
    showError('Failed to add contact: ' + e);
  }
}

function updateContactStatus(contactId, online) {
  // Update UI if needed
  if (currentConversation) {
    const contact = contacts.find(c => c.id === contactId);
    if (contact) {
      document.getElementById('chat-status').textContent = online ? 'Online' : 'Offline';
    }
  }
}

// Utility functions
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Make functions global for onclick handlers
window.createAccount = createAccount;
window.unlockAccount = unlockAccount;
window.showLogin = showLogin;
window.showCreateAccount = showCreateAccount;
window.sendMessage = sendMessage;
window.handleKeyDown = handleKeyDown;
window.showNewContactModal = showNewContactModal;
window.closeModal = closeModal;
window.addContact = addContact;

// Start
init();
