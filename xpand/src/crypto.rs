use std::{path::Path, fs::File, io::Read};
use aes_gcm::{Key, Aes256Gcm, aead::{AeadCore, KeyInit, OsRng, Aead, Nonce}};
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

#[inline]
fn hash(bytes: &[u8]) -> [u8; 32] {
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

#[derive(Serialize, Deserialize)]
pub struct Encrypted([u8; 12], Box<[u8]>);
#[inline]
pub fn encrypt(password: &str, bytes: &[u8]) -> Option<Encrypted> {
    let cipher = Aes256Gcm::new(get_aes_key!(password));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    cipher.encrypt(
        &nonce,
        bytes
    ).map(|x|
        Some(Encrypted(nonce.to_vec().try_into().unwrap(), x.into_boxed_slice()))
    ).unwrap_or(None)
}

#[inline]
pub fn decrypt(password: &str, encrypted: &Encrypted) -> Option<Box<[u8]>> {
    let cipher = Aes256Gcm::new(get_aes_key!(password));
    let this = unwrap!(Err cipher.decrypt(Nonce::<Aes256Gcm>::from_slice(&encrypted.0), encrypted.1.as_ref()));
    Some(this.into_boxed_slice())
}