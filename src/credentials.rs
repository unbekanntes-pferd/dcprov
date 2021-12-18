use keytar::{delete_password, get_password, set_password};

use crate::provisioning::DRACOONProvisioningError;

const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn set_dracoon_env(dracoon_url: &str, service_token: &str) -> bool {
    match set_password(SERVICE_NAME, dracoon_url, service_token) {
        Ok(()) => true,
        Err(_) => false,
    }
}

pub fn get_dracoon_env(dracoon_url: &str) -> Result<String, DRACOONProvisioningError> {
    match get_password(SERVICE_NAME, dracoon_url) {
        Ok(pwd) => match pwd.success {
            true => Ok(pwd.password),
            false => return Err(DRACOONProvisioningError::InvalidAccount),
        },
        Err(_) => return Err(DRACOONProvisioningError::InvalidAccount),
    }
}

pub fn delete_dracoon_env(dracoon_url: &str) -> bool {
    match get_dracoon_env(dracoon_url) {
        Ok(_) => match delete_password(SERVICE_NAME, dracoon_url) {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false
    }
}
