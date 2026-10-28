use crate::{account::DeviceSession, config::Config};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;
fn target(c: &Config) -> String {
    format!(
        "Kortex/desktop/{}",
        format!(
            "{:x}",
            Sha256::digest(format!("{}|{}", c.api_origin, c.data_dir.to_string_lossy()).as_bytes())
        )
    )
}
#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
#[cfg(windows)]
pub fn save(c: &Config, session: &DeviceSession) -> Result<(), String> {
    use windows_sys::Win32::Security::Credentials::*;
    let mut data = Zeroizing::new(serde_json::to_vec(session).map_err(|_| "Session invalide.")?);
    if data.len() > 4096 {
        return Err("Session trop volumineuse.".into());
    }
    let mut name = wide(&target(c));
    let mut user = wide(&session.user_id);
    // Credential Manager protects the blob under this Windows user's vault; no plaintext file fallback.
    unsafe {
        let mut credential: CREDENTIALW = std::mem::zeroed();
        credential.Type = CRED_TYPE_GENERIC;
        credential.TargetName = name.as_mut_ptr();
        credential.CredentialBlobSize = data.len() as u32;
        credential.CredentialBlob = data.as_mut_ptr();
        credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
        credential.UserName = user.as_mut_ptr();
        if CredWriteW(&credential, 0) == 0 {
            return Err("Le coffre Windows a refusé la session.".into());
        }
    }
    Ok(())
}
#[cfg(windows)]
pub fn load(c: &Config) -> Result<Option<DeviceSession>, String> {
    use windows_sys::Win32::{
        Foundation::{ERROR_NOT_FOUND, GetLastError},
        Security::Credentials::*,
    };
    let name = wide(&target(c));
    unsafe {
        let mut pointer = std::ptr::null_mut();
        if CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) == 0 {
            return if GetLastError() == ERROR_NOT_FOUND {
                Ok(None)
            } else {
                Err("Coffre Windows indisponible.".into())
            };
        }
        let record = &*pointer;
        let result = if record.CredentialBlobSize > 4096 {
            Err("Session du coffre invalide.".into())
        } else {
            let data = Zeroizing::new(
                std::slice::from_raw_parts(
                    record.CredentialBlob,
                    record.CredentialBlobSize as usize,
                )
                .to_vec(),
            );
            serde_json::from_slice(&data)
                .map(Some)
                .map_err(|_| "Session du coffre invalide.".into())
        };
        CredFree(pointer as _);
        result
    }
}
#[cfg(windows)]
pub fn clear(c: &Config) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::{ERROR_NOT_FOUND, GetLastError},
        Security::Credentials::*,
    };
    let name = wide(&target(c));
    unsafe {
        if CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) == 0
            && GetLastError() != ERROR_NOT_FOUND
        {
            return Err("La session n’a pas pu être effacée du coffre.".into());
        }
    }
    Ok(())
}
#[cfg(not(windows))]
pub fn save(_: &Config, _: &DeviceSession) -> Result<(), String> {
    Err("Cette validation est limitée à Windows.".into())
}
#[cfg(not(windows))]
pub fn load(_: &Config) -> Result<Option<DeviceSession>, String> {
    Err("Cette validation est limitée à Windows.".into())
}
#[cfg(not(windows))]
pub fn clear(_: &Config) -> Result<(), String> {
    Err("Cette validation est limitée à Windows.".into())
}
