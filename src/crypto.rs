use std::{path::Path, fs::File, io::Read};
use aes_gcm::{Key, Aes256Gcm, aead::{AeadCore, KeyInit, OsRng, Aead, Nonce}};
use rand::RngCore;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, digest::{Digest, FixedOutput}};

macro_rules! unwrap {
    (Opt $expr:expr) => {
        if let Some(x) = $expr { x } else { return None }
    };
    (Err $expr:expr) => {
        if let Ok(x) = $expr { x } else { return None }
    };
}

macro_rules! get_aes_key {
    ($password:expr) => {
        &Key::<Aes256Gcm>::from_slice(&hash($password.as_bytes()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub salt: [u8; 32],
    pub hash: [u8; 32],
}

impl Password {
    #[inline]
    pub fn verify_password(&self, password: &str) -> bool {
        let mut hasher: Sha256 = Digest::new();

        hasher.update(&self.salt);
        hasher.update(password.as_bytes());

        let hash = hasher.finalize_fixed();
        &self.hash[..] == &hash[..]
    }
}

#[inline]
pub fn hash_password(password: &str) -> Password {
    let mut hasher: Sha256 = Digest::new();
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    hasher.update(&salt);
    hasher.update(password.as_bytes());

    let hash = hasher.finalize_fixed();
    Password {
        salt,
        hash: hash.to_vec().try_into().unwrap(),
    }
}

#[inline]
pub fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher: Sha256 = Digest::new();

    hasher.update(bytes);

    let result = hasher.finalize_fixed();
    result.to_vec().try_into().unwrap()
}

#[inline]
pub fn hash_file(path: impl AsRef<Path>) -> Result<[u8; 32], std::io::Error> {
    let mut hasher: Sha256 = Digest::new();
    let mut file = File::open(path)?;
    let mut buffer = vec![0; 4 * 1024 * 1024].into_boxed_slice(); // 4MiB buffer
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        } hasher.update(&buffer[..n]);
    }
    
    let result = hasher.finalize_fixed();
    Ok(result.to_vec().try_into().unwrap())
}

#[inline]
pub fn encrypt(password: &str, bytes: &[u8]) -> Option<Box<[u8]>> {
    let cipher = Aes256Gcm::new(get_aes_key!(password));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    cipher.encrypt(
        &nonce,
        bytes
    ).map(|mut x| {
        x.append(&mut nonce.to_vec());
        Some(x.into_boxed_slice())
}).unwrap_or(None)
}

#[inline]
pub fn decrypt(password: &str, encrypted: &[u8]) -> Option<Box<[u8]>> {
    let cipher = Aes256Gcm::new(get_aes_key!(password));
    let this = unwrap!(Err cipher.decrypt(Nonce::<Aes256Gcm>::from_slice(&encrypted[encrypted.len()-12..]), &encrypted[..encrypted.len()-12]));
    Some(this.into_boxed_slice())
}