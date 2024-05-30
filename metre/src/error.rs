//! List of errors that can happen during the config loading process

use owo_colors::*;
use std::convert::Infallible;
use std::sync::Arc;

#[allow(unused)]
use crate::LoadLocation;

/// An error that can happen anywhere in the config loading process
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
  /// A network error loading a configuration from a url
  #[error("Network error loading config from {}", url.yellow())]
  #[cfg(any(feature = "url_blocking", feature = "url_async"))]
  Network {
    url: String,
    #[source]
    source: Arc<reqwest::Error>,
  },

  /// An I/O error loading a configuration from a file
  #[error("I/O error loading config from {}", path.yellow())]
  Io {
    path: String,
    #[source]
    source: Arc<std::io::Error>,
  },

  /// A JSON or JSONC error when deserialzing a partial configuration
  #[cfg(any(feature = "json", feature = "jsonc"))]
  #[error("JSON error loading config from {}", location)]
  Json {
    #[source]
    source: Arc<serde_json::Error>,
    location: LoadLocation,
  },

  /// A TOML error when deserialzing a partial configuration
  #[cfg(feature = "toml")]
  #[error("TOML error loading config from {}", location)]
  Toml {
    #[source]
    source: toml::de::Error,
    location: LoadLocation,
  },

  /// A YAML error when deserialzing a partial configuration
  #[error("YAML error loading config from {}", location)]
  #[cfg(feature = "yaml")]
  Yaml {
    #[source]
    source: Arc<serde_yaml::Error>,
    location: LoadLocation,
  },

  /// An error loading a partial configuration from an environment variable
  #[cfg(feature = "env")]
  #[error(transparent)]
  FromEnv(#[from] FromEnvError),

  /// An error when merging two partial configurations
  #[error(transparent)]
  Merge(#[from] MergeError),

  /// An error when creating a configuration from a partial configuration
  #[error(transparent)]
  FromPartial(#[from] FromPartialError),
}

/// Error produced when merging two partial configurations
#[derive(Debug, Clone, thiserror::Error)]
#[error("error merging config field {}: {}", field.yellow(), message)]
pub struct MergeError {
  /// The deep path to the field that caused the error: eg: my_app.port
  pub field: String,
  /// The error message from the merge function
  pub message: String,
}

/// Error parsing a value from an environment variable
#[cfg(feature = "env")]
#[derive(Debug, Clone, thiserror::Error)]
#[error("error parsing var {} from env for field: {}: {}", key.yellow(), field.yellow(), message)]
pub struct FromEnvError {
  /// The env key that produced the error: eg: MY_APP_PORT
  pub key: String,
  /// The deep path to the property: eg: my_app.port
  pub field: String,
  /// The error message from the parsing function
  pub message: String,
}

/// Error produced when creating a config from a partial config
#[derive(Debug, Clone, thiserror::Error)]
#[error("missing properties {} in finished config", missing_properties.iter().map(|name| name.yellow().to_string()).collect::<Vec<_>>().join(", ") )]
pub struct FromPartialError {
  /// The list of properties that are required but missing
  ///
  /// This will include the full path to the properties: eg: ["my_app.port"] for nested configurations
  ///
  /// Or just ["port"] for not nested configurations
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
  FromPartialError
);

#[cfg(feature = "env")]
impl_from_infallible!(
  FromEnvError
);
