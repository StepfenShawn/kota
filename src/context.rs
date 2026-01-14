use anyhow::{Context, Result};
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 对话上下文管理器
#[derive(Debug, Clone)]
pub struct ContextManager {
    storage_dir: PathBuf,
    session_id: String,
    messages: Vec<Message>,
    max_messages: usize,
}

/// 会话元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub last_updated: String,
    pub message_count: usize,
}

/// 持久化的会话数据
#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    pub metadata: SessionMetadata,
    pub messages: Vec<SerializableMessage>,
}

/// 可序列化的消息类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for SerializableMessage {
    fn from(msg: &Message) -> Self {
        match msg {
            Message::User { content, .. } => Self {
                role: "user".to_string(),
                content: content
                    .iter()
                    .map(|c| match c {
                        rig::completion::message::UserContent::Text(text) => text.text.clone(),
                        _ => "[non-text content]".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            },
            Message::Assistant { content, .. } => Self {
                role: "assistant".to_string(),
                content: content
                    .iter()
                    .map(|c| match c {
                        rig::completion::message::AssistantContent::Text(text) => text.text.clone(),
                        rig::completion::message::AssistantContent::ToolCall(_) => {
                            "[tool call]".to_string()
                        }
                        rig::completion::message::AssistantContent::Reasoning(_) => {
                            "[reasoning]".to_string()
                        }
                        rig::completion::message::AssistantContent::Image(_) => {
                            "[image]".to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            },
        }
    }
}

impl From<SerializableMessage> for Message {
    fn from(msg: SerializableMessage) -> Self {
        match msg.role.as_str() {
            "user" => Message::user(msg.content),
            "assistant" => Message::assistant(msg.content),
            _ => Message::user(msg.content),
        }
    }
}

impl ContextManager {
    pub fn new<P: AsRef<Path>>(storage_dir: P, session_id: String) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir).with_context(|| {
                format!("Failed to create storage directory: {:?}", storage_dir)
            })?;
        }
        Ok(Self {
            storage_dir,
            session_id,
            messages: Vec::new(),
            max_messages: 100,
        })
    }

    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn get_messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn save(&self) -> Result<()> {
        let file_path = self.get_session_file_path();
        let now = chrono::Utc::now().to_rfc3339();
        let metadata = SessionMetadata {
            session_id: self.session_id.clone(),
            created_at: now.clone(),
            last_updated: now,
            message_count: self.messages.len(),
        };
        let serializable_messages: Vec<SerializableMessage> = self
            .messages
            .iter()
            .map(SerializableMessage::from)
            .collect();
        let session_data = SessionData {
            metadata,
            messages: serializable_messages,
        };
        let json_data = serde_json::to_string_pretty(&session_data)
            .context("Failed to serialize session data")?;
        fs::write(&file_path, json_data)
            .with_context(|| format!("Failed to write session file: {:?}", file_path))?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<bool> {
        let file_path = self.get_session_file_path();
        if !file_path.exists() {
            return Ok(false);
        }
        let json_data = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read session file: {:?}", file_path))?;
        let session_data: SessionData =
            serde_json::from_str(&json_data).context("Failed to deserialize session data")?;
        self.messages = session_data
            .messages
            .into_iter()
            .map(Message::from)
            .collect();
        Ok(true)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();
        if !self.storage_dir.exists() {
            return Ok(sessions);
        }
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json_data) = fs::read_to_string(&path) {
                    if let Ok(session_data) = serde_json::from_str::<SessionData>(&json_data) {
                        sessions.push(session_data.metadata);
                    }
                }
            }
        }
        sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        Ok(sessions)
    }

    pub fn delete_session(&self) -> Result<bool> {
        let file_path = self.get_session_file_path();
        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("Failed to delete session file: {:?}", file_path))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_session_file_path(&self) -> PathBuf {
        self.storage_dir.join(format!("{}.json", self.session_id))
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn switch_session(&mut self, new_session_id: String) {
        self.session_id = new_session_id;
        self.messages.clear();
    }
}
