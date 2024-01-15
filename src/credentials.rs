use crate::cmd::DcProvError;
use keyring::Entry;

// service name to store 
pub const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");


pub fn set_dracoon_env(entry: &Entry, secret: &str) -> Result<(), DcProvError> {
    match entry.set_password(secret) {
        Ok(_) => Ok(()),
        Err(_) => Err(DcProvError::CredentialStorageFailed),
    }
}

pub fn get_dracoon_env(entry: &Entry) -> Result<String, DcProvError> {
    match entry.get_password() {
        Ok(pwd) => Ok(pwd),
        Err(_) => Err(DcProvError::InvalidAccount),
    }
}

pub fn delete_dracoon_env(entry: &Entry) -> Result<(), DcProvError> {
    if entry.get_password().is_err() {
        return Err(DcProvError::InvalidAccount);
    }

    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(_) => Err(DcProvError::CredentialDeletionFailed),
    }
}

