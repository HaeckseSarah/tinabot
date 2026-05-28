// https://github.com/open-source-cooperative/keyring-rs/blob/main/src/lib.rs

use keyring_core::{Entry, Error, Result, set_default_store};
use twitch_api::twitch_oauth2::UserToken;

pub fn use_native_store() -> Result<()> {
    #[cfg(target_os = "windows")]
    use_windows_native_store()?;
    #[cfg(target_os = "linux")]
    use_linux_keyutils_store()?;
    #[cfg(not(any(target_os = "linux", target_os = "windows",)))]
    return Err(Error::NotSupportedByStore("Unsupported OS".to_string()));

    Ok(())
}

/// Use the Linux Keyutils store.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
#[allow(unused_variables)]
pub fn use_linux_keyutils_store() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use dbus_secret_service_keyring_store::Store;

        set_default_store(Store::new()?);
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::NotSupportedByStore(
            "The keyutils store is only available on Linux".to_string(),
        ))
    }
}

/// Use the Windows Credential store.
///
/// Fails with a `NotSupportedByStore` error on other platforms.
pub fn use_windows_native_store() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows_native_keyring_store::Store;
        set_default_store(Store::new_with_configuration(config)?);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(Error::NotSupportedByStore(
            "The Windows credential store is only available on Windows".to_string(),
        ))
    }
}

pub fn store_user_token(token: &UserToken) -> Result<()> {
    let refresh_token = token.refresh_token.clone().unwrap();
    self::store_token(&token.login.as_str(), &refresh_token.as_str())?;
    self::store_token(&token.user_id.as_str(), &refresh_token.as_str())?;

    Ok(())
}

pub fn store_token(user: &str, token: &str) -> Result<()> {
    self::use_native_store()?;
    let entry = Entry::new("tinaBot", user)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token(user: &str) -> Result<String> {
    self::use_native_store()?;
    let entry = Entry::new("tinaBot", user)?;
    entry.get_password()
}
