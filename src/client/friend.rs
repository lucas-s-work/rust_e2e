use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Friend {
    pub id: String,
    pub nickname: String,
    pub enc_key: KeyPair,
    pub sig_key: KeyPair,
    messages: Vec<Message>,
}

use base64::{engine::general_purpose, Engine as _};

use super::{
    crypto::{KeyPair, Mode},
    message::{EncryptedMessage, Message},
};

#[derive(Serialize, Deserialize)]
struct FriendJson {
    id: String,
    nickname: String,
    pub_key: String,
    ver_key: String,
}

impl Friend {
    pub fn encrypt(&self, msg: &str) -> Result<String> {
        self.enc_key.encrypt(msg)
    }

    pub fn verify(&self, msg: &EncryptedMessage) -> Result<()> {
        self.sig_key.verify(&msg.enc_content, &msg.sig)
    }

    pub fn to_string(&self) -> Result<String> {
        let json = FriendJson {
            id: self.id.clone(),
            nickname: self.nickname.clone(),
            pub_key: self.enc_key.pub_key_pem()?,
            ver_key: self.sig_key.pub_key_pem()?,
        };

        let json = serde_json::to_string(&json)?;
        Ok(general_purpose::STANDARD_NO_PAD.encode(&json))
    }

    pub fn from_string(msg: &str) -> Result<Friend> {
        let decoded_msg = general_purpose::STANDARD_NO_PAD.decode(msg)?;
        let friendJson: FriendJson = serde_json::from_slice(&decoded_msg)?;

        if friendJson.id.is_empty() {
            return Err(anyhow!("id is empty"));
        };

        let enc_key = KeyPair::from_pub_pem(&friendJson.pub_key, Mode::Encrypt)?;
        let sig_key = KeyPair::from_pub_pem(&friendJson.ver_key, Mode::Verify)?;

        Ok(Friend {
            id: friendJson.id,
            nickname: friendJson.nickname,
            enc_key: enc_key,
            sig_key: sig_key,
            messages: Vec::new(),
        })
    }

    pub fn print_messages(&mut self) {
        self.messages
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));
        for message in self.messages {
            println!("{}|{}: {}", message.id, message.created_at, message.content);
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}
