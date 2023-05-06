use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct EncryptedMessage {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub enc_content: String,
    pub created_at: i64,
    pub sig: String,
}

#[derive(Debug)]
pub struct Message {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub content: String,
    pub created_at: i64,
}
