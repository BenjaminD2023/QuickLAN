use crate::{
    error::{Error, Result},
    routes::validate_subnet,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Serialize, Deserialize, Zeroize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    pub fn generate() -> Result<Self> {
        Ok(Self(random_hex(32)?))
    }
    pub fn from_encoded(value: String) -> Result<Self> {
        let secret = Self(value);
        secret.validate()?;
        Ok(secret)
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
    pub fn validate(&self) -> Result<()> {
        if valid_hex(&self.0, 64) {
            Ok(())
        } else {
            Err(Error::InvalidInvitation)
        }
    }
}
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl Drop for Secret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub fn random_hex(bytes: usize) -> Result<String> {
    let mut data = vec![0u8; bytes];
    OsRng
        .try_fill_bytes(&mut data)
        .map_err(|_| Error::RandomUnavailable)?;
    let encoded = data.iter().map(|b| format!("{b:02x}")).collect();
    data.zeroize();
    Ok(encoded)
}
pub fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn validate_label(value: &str) -> Result<()> {
    if value.trim() != value
        || value.is_empty()
        || value.chars().count() > 64
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
    {
        Err(Error::InvalidLabel)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    Manual,
    Assisted,
    DirectOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Bootstrap {
    pub endpoint: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub id: String,
    pub label: String,
    pub subnet: String,
    pub policy: Policy,
    pub bootstrap: Vec<Bootstrap>,
}
impl Network {
    pub fn validate(&self) -> Result<()> {
        if !valid_hex(&self.id, 32) {
            return Err(Error::InvalidInvitation);
        }
        validate_label(&self.label)?;
        validate_subnet(&self.subnet)?;
        if self.policy == Policy::DirectOnly {
            return Err(Error::UnsupportedPolicy);
        }
        if self.bootstrap.len() > 4 {
            return Err(Error::InvalidEndpoint);
        }
        if self.policy == Policy::Assisted && self.bootstrap.is_empty() {
            return Err(Error::AssistanceConsentRequired);
        }
        let mut seen = std::collections::HashSet::new();
        for node in &self.bootstrap {
            validate_label(&node.operator)?;
            crate::invitation::validate_endpoint(&node.endpoint, self.policy)?;
            if !seen.insert(&node.endpoint) {
                return Err(Error::InvalidEndpoint);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-CN")]
    Chinese,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub nickname: String,
    pub theme: Theme,
    pub language: Language,
    pub onboarding_complete: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            nickname: "My computer".into(),
            theme: Theme::System,
            language: Language::English,
            onboarding_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SavedState {
    pub networks: Vec<Network>,
    pub preferences: Preferences,
}
impl SavedState {
    pub fn validate(&self) -> Result<()> {
        validate_label(&self.preferences.nickname)?;
        let mut ids = std::collections::HashSet::new();
        for network in &self.networks {
            network.validate()?;
            if !ids.insert(&network.id) {
                return Err(Error::InvalidStorage);
            }
        }
        Ok(())
    }
}
