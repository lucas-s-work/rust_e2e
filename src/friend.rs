use crate::{
    crypto::{self, KeyPair, KeyPairError, Mode},
    message::EncryptedMessage,
};

use serde::{Deserialize, Serialize};

pub enum FriendError {
    JsonError(serde_json::Error),
    CryptoError(KeyPairError),
    InvalidShareString(String),
}

impl From<serde_json::Error> for FriendError {
    fn from(value: serde_json::Error) -> Self {
        FriendError::JsonError(value)
    }
}

impl From<KeyPairError> for FriendError {
    fn from(value: KeyPairError) -> Self {
        FriendError::CryptoError(value)
    }
}

#[derive(Debug)]
pub struct Friend {
    pub id: String,
    pub enc_key: crypto::KeyPair,
    pub sig_key: crypto::KeyPair,
}

#[derive(Serialize, Deserialize)]
struct FriendJson {
    id: String,
    pub_key: String,
    ver_key: String,
}

impl Friend {
    pub fn encrypt(&self, msg: &str) -> Result<String, KeyPairError> {
        self.enc_key.encrypt(msg)
    }

    pub fn verify(&self, msg: &EncryptedMessage) -> Result<(), KeyPairError> {
        self.sig_key.verify(&msg.enc_content, &msg.sig)
    }

    pub fn to_string(&self) -> Result<String, FriendError> {
        let json = FriendJson {
            id: self.id.clone(),
            pub_key: self.enc_key.pub_key_pem()?,
            ver_key: self.sig_key.pub_key_pem()?,
        };

        Ok(serde_json::to_string(&json)?)
    }

    pub fn from_string(msg: &str) -> Result<Friend, FriendError> {
        let friendJson: FriendJson = serde_json::from_str(msg)?;

        if friendJson.id.is_empty() {
            return Err(FriendError::InvalidShareString("id is empty".to_string()));
        };

        let enc_key = KeyPair::from_pub_pem(&friendJson.pub_key, Mode::Encrypt)?;
        let sig_key = KeyPair::from_pub_pem(&friendJson.ver_key, Mode::Verify)?;

        Ok(Friend {
            id: friendJson.id,
            enc_key: enc_key,
            sig_key: sig_key,
        })
    }
}
