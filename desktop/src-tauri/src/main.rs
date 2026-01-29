// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use securechat_core::{SecureChat, ChatEvent, protocol::{Contact, Conversation, LocalMessage, UserProfile}};
use std::sync::Arc;
use tauri::{State, Manager, Window};
use tokio::sync::{Mutex, mpsc};
use anyhow::Result;

// App state
struct AppState {
    chat: Arc<Mutex<Option<SecureChat>>>,
    event_tx: Mutex<Option<mpsc::Sender<AppEvent>>>,
}

#[derive(Clone)]
enum AppEvent {
    ChatEvent(ChatEvent),
    AuthSuccess,
    AuthFailed(String),
}

// Commands

#[tauri::command]
async fn create_account(
    state: State<'_, AppState>,
    password: String,
    display_name: String,
    window: Window,
) -> Result<bool, String> {
    let data_dir = get_data_dir()?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let db_path = data_dir.join("securechat.db");
    
    let chat = SecureChat::new(None);
    match chat.create_account(&db_path, &password, &display_name).await {
        Ok(_) => {
            *state.chat.lock().await = Some(chat);
            
            // Start event listener
            if let Err(e) = start_event_listener(&state, window).await {
                return Err(e);
            }
            
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn unlock_account(
    state: State<'_, AppState>,
    password: String,
    window: Window,
) -> Result<bool, String> {
    let data_dir = get_data_dir()?;
    let db_path = data_dir.join("securechat.db");
    
    if !db_path.exists() {
        return Err("No account found. Please create one first.".to_string());
    }
    
    let chat = SecureChat::new(None);
    match chat.unlock_account(&db_path, &password).await {
        Ok(_) => {
            *state.chat.lock().await = Some(chat);
            
            // Start event listener
            if let Err(e) = start_event_listener(&state, window).await {
                return Err(e);
            }
            
            Ok(true)
        }
        Err(_) => Err("Invalid password".to_string()),
    }
}

#[tauri::command]
async fn has_account() -> Result<bool, String> {
    let data_dir = get_data_dir()?;
    let db_path = data_dir.join("securechat.db");
    Ok(db_path.exists())
}

#[tauri::command]
async fn get_conversations(state: State<'_, AppState>) -> Result<Vec<Conversation>, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_conversations().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_messages(
    state: State<'_, AppState>,
    conversation_id: String,
    limit: usize,
) -> Result<Vec<LocalMessage>, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_messages(&conversation_id, limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_text_message(
    state: State<'_, AppState>,
    conversation_id: String,
    text: String,
) -> Result<String, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.send_text_message(&conversation_id, &text).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_contacts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_contact(
    state: State<'_, AppState>,
    public_key: Vec<u8>,
    display_name: String,
) -> Result<Contact, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    
    if public_key.len() != 32 {
        return Err("Invalid public key length".to_string());
    }
    
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&public_key);
    
    chat.add_contact(key_array, &display_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_or_create_conversation(
    state: State<'_, AppState>,
    contact_id: String,
) -> Result<Conversation, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_or_create_conversation(&contact_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_profile(state: State<'_, AppState>) -> Result<Option<UserProfile>, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_profile().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_profile(
    state: State<'_, AppState>,
    display_name: Option<String>,
    status_message: Option<String>,
) -> Result<(), String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.update_profile(display_name.as_deref(), status_message.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_public_key(state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    chat.get_public_key().await.map_err(|e| e.to_string()).map(|k| k.to_vec())
}

#[tauri::command]
async fn start_network(state: State<'_, AppState>) -> Result<(), String> {
    use securechat_core::network::NetworkConfig;
    
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    
    let config = NetworkConfig::default();
    chat.start_network(config).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

// Helper functions

fn get_data_dir() -> Result<std::path::PathBuf, String> {
    let dirs = directories::ProjectDirs::from("com", "securechat", "SecureChat")
        .ok_or("Failed to get project directories")?;
    Ok(dirs.data_dir().to_path_buf())
}

async fn start_event_listener(state: &AppState, window: Window) -> Result<(), String> {
    let chat_guard = state.chat.lock().await;
    let chat = chat_guard.as_ref().ok_or("Not authenticated")?;
    
    use securechat_core::network::NetworkConfig;
    let config = NetworkConfig::default();
    let mut event_rx = chat.start_network(config).await.map_err(|e| e.to_string())?;
    
    // Spawn event handler
    tauri::async_runtime::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let event_name = match &event {
                ChatEvent::MessageReceived { .. } => "message-received",
                ChatEvent::MessageSent { .. } => "message-sent",
                ChatEvent::MessageDelivered { .. } => "message-delivered",
                ChatEvent::MessageRead { .. } => "message-read",
                ChatEvent::ContactOnline { .. } => "contact-online",
                ChatEvent::ContactOffline { .. } => "contact-offline",
                ChatEvent::ContactRequestReceived { .. } => "contact-request",
                ChatEvent::SyncCompleted => "sync-completed",
                ChatEvent::Error { .. } => "error",
            };
            
            if let Err(e) = window.emit(event_name, &event) {
                log::error!("Failed to emit event: {}", e);
            }
        }
    });
    
    Ok(())
}

fn main() {
    let state = AppState {
        chat: Arc::new(Mutex::new(None)),
        event_tx: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_account,
            unlock_account,
            has_account,
            get_conversations,
            get_messages,
            send_text_message,
            get_contacts,
            add_contact,
            get_or_create_conversation,
            get_profile,
            update_profile,
            get_public_key,
            start_network,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
