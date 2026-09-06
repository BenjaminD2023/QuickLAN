use crate::{
    error::{Error, Result},
    model::{SavedState, Secret},
};
use serde::{Deserialize, Serialize};

pub trait Store: Send {
    fn load(&self) -> Result<SavedState>;
    fn save(&mut self, state: &SavedState) -> Result<()>;
    fn get_secret(&self, id: &str) -> Result<Secret>;
    fn put_secret(&mut self, id: &str, secret: &Secret) -> Result<()>;
    fn remove_secret(&mut self, id: &str) -> Result<()>;
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: u32,
    state: SavedState,
}

pub fn decode_state(bytes: &[u8]) -> Result<SavedState> {
    if bytes.len() > 1_048_576 {
        return Err(Error::InvalidStorage);
    }
    let envelope: Envelope = serde_json::from_slice(bytes).map_err(|_| Error::InvalidStorage)?;
    // Version 1 is the first schema. Refuse future/unknown versions without overwriting.
    // Add explicit, tested transforms here when a second schema actually exists.
    if envelope.schema != 1 {
        return Err(Error::InvalidStorage);
    }
    envelope
        .state
        .validate()
        .map_err(|_| Error::InvalidStorage)?;
    Ok(envelope.state)
}
pub fn encode_state(state: &SavedState) -> Result<Vec<u8>> {
    state.validate()?;
    serde_json::to_vec_pretty(&Envelope {
        schema: 1,
        state: state.clone(),
    })
    .map_err(|_| Error::InvalidStorage)
}

#[cfg(feature = "os-vault")]
pub struct OsStore {
    directory: std::path::PathBuf,
}
#[cfg(feature = "os-vault")]
impl OsStore {
    pub fn new(directory: std::path::PathBuf) -> Result<Self> {
        let existed = directory.try_exists().map_err(|_| Error::UnsafePath)?;
        if !existed {
            std::fs::create_dir_all(&directory).map_err(|_| Error::UnsafePath)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
                    .map_err(|_| Error::UnsafePath)?;
            }
        }
        let store = Self { directory };
        store.check_directory()?;
        Ok(store)
    }
    fn check_directory(&self) -> Result<()> {
        let metadata = std::fs::symlink_metadata(&self.directory).map_err(|_| Error::UnsafePath)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(Error::UnsafePath);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(Error::UnsafePath);
            }
        }
        Ok(())
    }
    fn entry(id: &str) -> Result<keyring::Entry> {
        if !crate::model::valid_hex(id, 32) {
            return Err(Error::InvalidStorage);
        }
        keyring::Entry::new("io.quicklan.desktop.network", id)
            .map_err(|_| Error::StorageUnavailable)
    }
}
#[cfg(feature = "os-vault")]
impl Store for OsStore {
    fn load(&self) -> Result<SavedState> {
        self.check_directory()?;
        let path = self.directory.join("networks-v1.json");
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(SavedState::default()),
            Err(_) => return Err(Error::InvalidStorage),
            Ok(meta) => {
                if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 1_048_576 {
                    return Err(Error::UnsafePath);
                }
            }
        }
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| Error::InvalidStorage)?
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::InvalidStorage)?;
        decode_state(&bytes)
    }
    fn save(&mut self, state: &SavedState) -> Result<()> {
        use std::io::Write;
        self.check_directory()?;
        let path = self.directory.join("networks-v1.json");
        if let Ok(meta) = std::fs::symlink_metadata(&path) {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(Error::UnsafePath);
            }
        }
        let bytes = encode_state(state)?;
        let temporary = self
            .directory
            .join(format!(".metadata-{}.tmp", crate::model::random_hex(16)?));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = (|| {
            let mut file = options.open(&temporary).map_err(|_| Error::UnsafePath)?;
            file.write_all(&bytes)
                .map_err(|_| Error::StorageUnavailable)?;
            file.sync_all().map_err(|_| Error::StorageUnavailable)?;
            drop(file);
            std::fs::rename(&temporary, &path).map_err(|_| Error::StorageUnavailable)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
    fn get_secret(&self, id: &str) -> Result<Secret> {
        let raw = Self::entry(id)?
            .get_password()
            .map_err(|_| Error::StorageUnavailable)?;
        Secret::from_encoded(raw).map_err(|_| Error::InvalidStorage)
    }
    fn put_secret(&mut self, id: &str, secret: &Secret) -> Result<()> {
        secret.validate()?;
        Self::entry(id)?
            .set_password(secret.expose())
            .map_err(|_| Error::StorageUnavailable)
    }
    fn remove_secret(&mut self, id: &str) -> Result<()> {
        match Self::entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(Error::StorageUnavailable),
        }
    }
}
