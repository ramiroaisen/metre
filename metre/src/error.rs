use owo_colors::*;
use std::convert::Infallible;
use std::sync::Arc;
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LoadLocation {
  Memory,
  File(String),
  Url(String),
}

impl Display for LoadLocation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LoadLocation::*;
    match self {
      Memory => write!(f, "{}", "memory".yellow()),
      File(location) => write!(f, "file: {}", location.yellow()),
      Url(location) => write!(f, "url: {}", location.yellow()),
    }
  }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
  #[error("Network error loading config from {}", url.yellow())]
  Network{
    url: String,
    #[source]
    source: Arc<reqwest::Error>,
  },

  #[error("I/O error loading config from {}", path.yellow())]
  Io {
    path: String,
    #[source]
    source: Arc<std::io::Error>
  },

  #[error("JSON error loading config from {}", location)]
  Json {
    #[source]
    source: Arc<serde_json::Error>,
    location: LoadLocation,
  },

  #[error("TOML error loading config from {}", location)]
  Toml {
    #[source]
    source: toml::de::Error,
    location: LoadLocation,
  },

  #[error("YAML error loading config from {}", location)]
  Yaml{
    #[source]
    source: Arc<serde_yaml::Error>,
    location: LoadLocation,
  },

  #[error(transparent)]
  FromEnv(#[from] FromEnvError),

  #[error(transparent)]
  Merge(#[from] MergeError),

  #[error(transparent)]
  FromPartial(#[from] FromPartialError),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("error merging config field {}: {}", field.yellow(), message)]
pub struct MergeError {
  pub field: String,
  pub message: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("error getting var {} from env for field: {}: {}", key.yellow(), field.yellow(), message)]
pub struct FromEnvError {
  pub key: String,
  pub field: String,
  pub message: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("missing properties {} in finished config", missing_properties.iter().map(|name| name.yellow().to_string()).collect::<Vec<_>>().join(", ") )]
pub struct FromPartialError {
  pub missing_properties: Vec<String>,
}

macro_rules! impl_from_infallible {
  ($($ty:ty)*) => {
    $(
      impl From<Infallible> for $ty {
        fn from(e: Infallible) -> Self {
          match e {}
        }
      }
    )*
  }
}

impl_from_infallible!(
  Error
  MergeError
  FromEnvError
  FromPartialError
);