use crate::{
    crypto::{self, KeyPair, KeyPairError, Mode},
    friend::Friend,
    message::{EncryptedMessage, Message},
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub enum UserError {
    CryptoError(crypto::KeyPairError),
    DuplicateFriend(String),
    NoSuchFriend(String),
    UnknownMessage(String),
}

impl From<KeyPairError> for UserError {
    fn from(value: KeyPairError) -> Self {
        UserError::CryptoError(value)
    }
}

#[derive(Debug)]
pub struct User {
    id: String,
    enc_key: crypto::KeyPair,
    sig_key: crypto::KeyPair,
    friends: HashMap<String, Friend>,
}

impl User {
    pub fn new() -> Result<User, UserError> {
        let enc_key = KeyPair::generate(Mode::EncryptDecrypt)?;
        let sig_key = KeyPair::generate(Mode::VerifySign)?;
        let uuid = Uuid::new_v4();

        Ok(User {
            id: uuid.to_string(),
            enc_key: enc_key,
            sig_key: sig_key,
            friends: HashMap::new(),
        })
    }

    pub fn add_friend(&mut self, friend: Friend) -> Result<(), UserError> {
        if self.friends.get(&friend.id).is_some() {
            return Err(UserError::DuplicateFriend(String::from(
                "duplicate friend added",
            )));
        };

        self.friends.insert(friend.id.clone(), friend);
        Ok(())
    }

    pub fn create_message(
        &self,
        friend_id: &str,
        msg: &str,
    ) -> Result<EncryptedMessage, UserError> {
        let friend = self
            .friends
            .get(friend_id)
            .ok_or(UserError::NoSuchFriend(String::from("no friend with Id")))?;

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

    pub fn receive_message(&self, enc_msg: EncryptedMessage) -> Result<Message, UserError> {
        if enc_msg.target_id != self.id {
            return Err(UserError::UnknownMessage(String::from(
                "received message for another id",
            )));
        };

        let friend = self
            .friends
            .get(&enc_msg.source_id)
            .ok_or(UserError::NoSuchFriend(String::from("no friend with Id")))?;

        friend.verify(&enc_msg)?;

        let msg = self.enc_key.decrypt(&enc_msg.enc_content)?;

        Ok(Message {
            id: enc_msg.id,
            source_id: enc_msg.source_id,
            target_id: enc_msg.target_id,
            content: msg,
            created_at: enc_msg.created_at,
        })
    }

    pub fn to_friend(&self) -> Result<Friend, UserError> {
        let enc_key = self.enc_key.to_public()?;
        let sig_key = self.sig_key.to_verify()?;

        Ok(Friend {
            id: self.id.clone(),
            enc_key,
            sig_key,
        })
    }
}
