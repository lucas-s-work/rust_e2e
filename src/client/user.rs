use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    crypto::{KeyPair, Mode},
    friend::{self, Friend},
    message::{EncryptedMessage, Message},
};

#[derive(Debug)]
pub struct User {
    id: String,
    nickname: String,
    enc_key: KeyPair,
    sig_key: KeyPair,
    friends: HashMap<String, Friend>,
}

impl User {
    pub fn new(nickname: &str) -> Result<User> {
        let enc_key = KeyPair::generate(Mode::EncryptDecrypt)?;
        let sig_key = KeyPair::generate(Mode::VerifySign)?;
        let uuid = Uuid::new_v4();

        Ok(User {
            id: uuid.to_string(),
            nickname: nickname.to_string(),
            enc_key: enc_key,
            sig_key: sig_key,
            friends: HashMap::new(),
        })
    }

    pub fn add_friend(&mut self, friend: Friend) -> Result<()> {
        if self.friends.get(&friend.id).is_some() {
            bail!("duplicate friend added");
        };

        self.friends.insert(friend.id.clone(), friend);
        Ok(())
    }

    pub fn get_friend_ids(&self) -> HashMap<String, String> {
        let mut friend_ids: HashMap<String, String> = HashMap::new();

        for (id, f) in &self.friends {
            friend_ids.insert(id.to_string(), f.nickname.to_string());
        }

        friend_ids
    }

    pub fn get_friend(&self, id: &str) -> Option<&Friend> {
        self.friends.get(id)
    }

    pub fn create_message(&self, friend_id: &str, msg: &str) -> Result<EncryptedMessage> {
        let friend = self
            .friends
            .get(friend_id)
            .ok_or(anyhow!("no friend with Id"))?;

        let enc_msg = friend.encrypt(msg)?;
        let msg_sig = self.sig_key.sign(&enc_msg)?;

        Ok(EncryptedMessage {
            id: Uuid::new_v4().to_string(),
            source_id: self.id.clone(),
            target_id: friend_id.to_string(),
            enc_content: enc_msg,
            created_at: Utc::now().timestamp(),
            sig: msg_sig,
        })
    }

    pub fn receive_message(&mut self, enc_msg: EncryptedMessage) -> Result<()> {
        if enc_msg.target_id != self.id {
            bail!("received message for another id");
        };

        let friend = self
            .friends
            .get_mut(&enc_msg.source_id)
            .ok_or(anyhow!("no friend with Id"))?;

        friend.verify(&enc_msg)?;

        let msg = self.enc_key.decrypt(&enc_msg.enc_content)?;

        let message = Message {
            id: enc_msg.id,
            source_id: enc_msg.source_id,
            target_id: enc_msg.target_id,
            content: msg,
            created_at: enc_msg.created_at,
        };

        friend.add_message(message);
        Ok(())
    }

    pub fn to_friend(&self) -> Result<Friend> {
        let enc_key = self.enc_key.to_public()?;
        let sig_key = self.sig_key.to_verify()?;

        Ok(Friend {
            id: self.id.clone(),
            nickname: self.nickname.clone(),
            enc_key,
            sig_key,
            messages: Vec::new(),
        })
    }
}
