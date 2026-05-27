// https://github.com/open-source-cooperative/keyring-rs/blob/main/src/lib.rs

use keyring_core::{Error, Result, get_default_store, set_default_store, unset_default_store};
use std::collections::HashMap;

pub fn use_native_store() -> Result<()> {
    #[cfg(target_os = "windows")]
    use_windows_native_store()?;
    #[cfg(target_os = "linux")]
    use_linux_keyutils_store();
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
        use linux_keyutils_keyring_store::Store;
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
