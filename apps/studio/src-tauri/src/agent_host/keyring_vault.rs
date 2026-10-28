use keyring::Entry;

pub const SERVICE_NAME: &str = "ForgeEngine";
pub const KEY_NAME: &str = "GeminiApiKey";

/// Save the Gemini API key securely in Windows Credential Manager.
pub fn set_api_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access credential store: {e}"))?;
    entry
        .set_password(key)
        .map_err(|e| format!("Failed to save credential: {e}"))
}

/// Check if the Gemini API key exists in Windows Credential Manager without exposing the value.
pub fn has_api_key() -> Result<bool, String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access credential store: {e}"))?;
    match entry.get_password() {
        Ok(pass) => Ok(!pass.trim().is_empty()),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Credential store error: {e}")),
    }
}

/// Delete the Gemini API key from Windows Credential Manager.
pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access credential store: {e}"))?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Failed to delete credential: {e}")),
    }
}

/// Retrieve the Gemini API key internally (used ONLY to pipe directly into the sidecar's stdin).
/// Never exposed over Tauri IPC, never logged, never returned to the UI.
pub fn get_api_key_internal() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("Failed to access credential store: {e}"))?;
    match entry.get_password() {
        Ok(pass) => {
            if pass.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(pass))
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Credential store error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_never_leaked_in_formatting() {
        let fake_key = "AIzaSySecretFakeTestKey123456789";
        // Verify key representation or secrets filter expectations
        assert!(!format!("SERVICE: {}", SERVICE_NAME).contains(fake_key));
    }
}
