use sha256::digest;
use std::string::FromUtf8Error;

use openssl::{
    error,
    pkey::{Private, Public},
    rsa::{Padding, Rsa},
};

use base64::{engine::general_purpose, DecodeError, Engine as _};

#[derive(Debug)]
pub enum KeyPairError {
    ModeError(String),
    OpenSSLError(openssl::error::ErrorStack),
    Base64Error(DecodeError),
    Utf8Error(FromUtf8Error),
    VerifyError,
    NoKeyError(String),
}

impl From<FromUtf8Error> for KeyPairError {
    fn from(value: FromUtf8Error) -> Self {
        KeyPairError::Utf8Error(value)
    }
}

impl From<DecodeError> for KeyPairError {
    fn from(value: DecodeError) -> Self {
        KeyPairError::Base64Error(value)
    }
}

impl From<error::ErrorStack> for KeyPairError {
    fn from(value: error::ErrorStack) -> Self {
        KeyPairError::OpenSSLError(value)
    }
}

#[derive(Debug)]
pub enum Mode {
    Encrypt,
    EncryptDecrypt,
    Verify,
    VerifySign,
}

#[derive(Debug)]
pub struct KeyPair {
    public: Option<Rsa<Public>>,
    private: Option<Rsa<Private>>,
    mode: Mode,
}

impl KeyPair {
    fn can_encrypt(&self) -> Result<&Rsa<Public>, KeyPairError> {
        match self.mode {
            Mode::Encrypt | Mode::EncryptDecrypt => Ok(self.public.as_ref().unwrap()),
            _ => Err(KeyPairError::ModeError(String::from(
                "cannot encrypt with non encrypt mode key",
            ))),
        }
    }

    fn can_decrypt(&self) -> Result<&Rsa<Private>, KeyPairError> {
        match self.mode {
            Mode::EncryptDecrypt => Ok(self.private.as_ref().unwrap()),
            _ => Err(KeyPairError::ModeError(String::from(
                "cannot decrypt with non decrypt mode key",
            ))),
        }
    }

    pub fn encrypt(&self, msg: &str) -> Result<String, KeyPairError> {
        let key = self.can_encrypt()?;

        let mut buf = vec![0; key.size() as usize];
        key.public_encrypt(msg.as_bytes(), &mut buf, Padding::PKCS1)?;
        let buf_b64 = general_purpose::STANDARD_NO_PAD.encode(buf);
        Ok(buf_b64)
    }

    pub fn decrypt(&self, enc_msg_str: &str) -> Result<String, KeyPairError> {
        let key = self.can_decrypt()?;

        let enc_msg = general_purpose::STANDARD_NO_PAD.decode(enc_msg_str)?;
        let mut buf = vec![0; key.size() as usize];
        key.private_decrypt(&enc_msg, &mut buf, Padding::PKCS1)?;

        // remove the trailing zeros by finding the index of the first trailing zero
        let mut trailing_index = 0;
        for (i, c) in buf.iter().rev().enumerate() {
            if *c != 0u8 {
                trailing_index = i;
                break;
            }
        }
        buf.truncate(buf.len() - trailing_index);

        let decrypted_str = String::from_utf8(buf)?;
        Ok(decrypted_str.trim().to_string())
    }

    fn can_sign(&self) -> Result<&Rsa<Private>, KeyPairError> {
        match self.mode {
            Mode::VerifySign => Ok(self.private.as_ref().unwrap()),
            _ => Err(KeyPairError::ModeError(String::from(
                "cannot sign with non signing mode key",
            ))),
        }
    }

    fn can_verify(&self) -> Result<&Rsa<Public>, KeyPairError> {
        match self.mode {
            Mode::VerifySign | Mode::Verify => Ok(self.public.as_ref().unwrap()),
            _ => Err(KeyPairError::ModeError(String::from(
                "cannot verify with non verify mode key",
            ))),
        }
    }

    pub fn sign(&self, msg: &str) -> Result<String, KeyPairError> {
        let key = self.can_sign()?;

        let mut buf = vec![0; key.size() as usize];
        let hash_msg = digest(msg);
        key.private_encrypt(hash_msg.as_bytes(), &mut buf, Padding::PKCS1)?;

        let buf_64 = general_purpose::STANDARD_NO_PAD.encode(buf);
        Ok(buf_64)
    }

    pub fn verify(&self, msg: &str, sig: &str) -> Result<(), KeyPairError> {
        let key = self.can_verify()?;

        let mut buf = vec![0; key.size() as usize];
        let hash_msg = digest(msg);
        key.public_decrypt(msg.as_bytes(), &mut buf, Padding::PKCS1)?;

        if hash_msg == String::from_utf8(buf)? {
            Ok(())
        } else {
            Err(KeyPairError::VerifyError)
        }
    }

    pub fn generate(mode: Mode) -> Result<KeyPair, KeyPairError> {
        let key = Rsa::generate(4096)?;
        let pub_pem = key.public_key_to_pem()?;
        let priv_pem = key.private_key_to_pem()?;
        let pub_key = Rsa::public_key_from_pem(&pub_pem)?;
        let priv_key = Rsa::private_key_from_pem(&priv_pem)?;

        Ok(KeyPair {
            public: Some(pub_key),
            private: Some(priv_key),
            mode: mode,
        })
    }

    pub fn to_public(&self) -> Result<KeyPair, KeyPairError> {
        let key = self.can_encrypt()?;

        Ok(KeyPair {
            public: Some(key.clone()),
            private: None,
            mode: Mode::Encrypt,
        })
    }

    pub fn to_verify(&self) -> Result<KeyPair, KeyPairError> {
        let key = self.can_verify()?;

        Ok(KeyPair {
            public: Some(key.clone()),
            private: None,
            mode: Mode::Verify,
        })
    }

    pub fn pub_key_pem(&self) -> Result<String, KeyPairError> {
        match &self.public {
            Some(key) => {
                let pem_bytes = key.public_key_to_pem()?;
                let b64_key = general_purpose::STANDARD_NO_PAD.encode(pem_bytes);
                Ok(b64_key)
            }
            None => Err(KeyPairError::NoKeyError(String::from("No public key"))),
        }
    }

    pub fn from_pub_pem(msg: &str, mode: Mode) -> Result<KeyPair, KeyPairError> {
        match mode {
            Mode::EncryptDecrypt => {
                return Err(KeyPairError::ModeError(
                    "cannot load only public key for decrypt mode".to_string(),
                ))
            }
            Mode::VerifySign => {
                return Err(KeyPairError::ModeError(
                    "cannot load only public key for sign mode".to_string(),
                ))
            }
            _ => (),
        };

        let key = Rsa::public_key_from_pem(msg.as_bytes())?;
        Ok(KeyPair {
            public: Some(key),
            private: None,
            mode: mode,
        })
    }
}
