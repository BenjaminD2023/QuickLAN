use crate::platform::AndroidPlatform;
use quicklan_core::{
    error::{Error, Result},
    model::{SavedState, Secret},
    storage::{decode_state, encode_state, Store},
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

pub struct AndroidStore {
    directory: PathBuf,
    platform: AndroidPlatform,
}

impl AndroidStore {
    pub fn new(directory: PathBuf, platform: AndroidPlatform) -> Result<Self> {
        let existed = directory.try_exists().map_err(|_| Error::UnsafePath)?;
        if !existed {
            fs::create_dir_all(&directory).map_err(|_| Error::UnsafePath)?;
        }
        let store = Self {
            directory,
            platform,
        };
        store.check_directory()?;
        Ok(store)
    }

    fn check_directory(&self) -> Result<()> {
        let metadata = fs::symlink_metadata(&self.directory).map_err(|_| Error::UnsafePath)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(Error::UnsafePath);
        }
        Ok(())
    }
}

impl Store for AndroidStore {
    fn load(&self) -> Result<SavedState> {
        self.check_directory()?;
        let path = self.directory.join("networks-v1.json");
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(SavedState::default()),
            Err(_) => return Err(Error::InvalidStorage),
            Ok(meta) => {
                if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 1_048_576 {
                    return Err(Error::UnsafePath);
                }
            }
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| Error::InvalidStorage)?
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::InvalidStorage)?;
        decode_state(&bytes)
    }

    fn save(&mut self, state: &SavedState) -> Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        self.check_directory()?;
        let path = self.directory.join("networks-v1.json");
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(Error::UnsafePath);
            }
        }
        let bytes = encode_state(state)?;
        let temporary = self.directory.join(format!(
            ".metadata-{}.tmp",
            quicklan_core::model::random_hex(16)?
        ));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)
                .map_err(|_| Error::UnsafePath)?;
            file.write_all(&bytes)
                .map_err(|_| Error::StorageUnavailable)?;
            file.sync_all().map_err(|_| Error::StorageUnavailable)?;
            drop(file);
            fs::rename(&temporary, &path).map_err(|_| Error::StorageUnavailable)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn get_secret(&self, id: &str) -> Result<Secret> {
        Secret::from_encoded(self.platform.read_secret(id)?).map_err(|_| Error::InvalidStorage)
    }

    fn put_secret(&mut self, id: &str, secret: &Secret) -> Result<()> {
        secret.validate()?;
        self.platform.write_secret(id, secret.expose())
    }

    fn remove_secret(&mut self, id: &str) -> Result<()> {
        self.platform.remove_secret(id)
    }
}
