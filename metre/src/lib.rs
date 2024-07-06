#![cfg_attr(docsrs, feature(doc_cfg))]
//! # **metre**. The configuration loader for Rust.
//!   
//! #### AKA: The `#[derive(Config)]` macro 
//! 
//! **metre** is a configuration loader for Rust that allows you to load configurations from a variety of formats such as **toml**, **json**, **jsonc** and **yaml**
//! It also supports a variety of sources such as **program defaults**, **env variables**, **files**, and **urls**.   
//! &nbsp;
//! &nbsp; 
//! ## Focus
//! 
//! **metre** focus is to provide a **declarative** and **type-safe** way to load configurations in Rust.
//! &nbsp;
//! &nbsp;
//! ## How?
//!
//! **metre** works by defining a struct that implements the `Config` trait, usually via the `#[derive(Config)]` macro. 
//! 
//! Under the hood metre creates deep partial version of the struct to accumulate the configuration from different sources.
//!
//! Once all the configuration is accumulated, you can access the final configuration as the defined struct. If the sum of all sources does not comply with the required properties, metre will return an error.
//! &nbsp;
//! &nbsp;
//! ## Show me the code
//! 
//! The following code shows how to load configurations 
//! from different sources and in different formats 
//! with descriptions for most macro attributes 
//! 
//! To see all macro attributes available see the [Config](macro@Config) derive macro documentation.
//! 
//! ```
//! # fn load_config() -> Result<(), Box<dyn std::error::Error>> {
//! use metre::{Config, ConfigLoader, Format};
//! 
//! #[derive(Config)]
//! struct MyConfig {
//!   foo: u16,
//!   bar: String,
//! }
//! 
//! let mut loader = ConfigLoader::<MyConfig>::new();
//!
//! loader.file("config.json", Format::Json)?;
//! loader.env()?;
//! loader.defaults()?;
//!
//! // config have the type MyConfig here, or loader.finish() returns an error
//! let config = loader.finish()?;
//! 
//! # Ok(())
//! # }
//! ``` 

use owo_colors::*;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;
#[cfg(feature = "env")]
use std::{env::VarError, collections::{BTreeMap, HashMap}};
#[allow(unused)]
use std::convert::Infallible;

pub mod error;
pub mod merge;
pub mod parse;
#[doc(hidden)]
pub mod util;

pub use error::Error;
/// Derive macro for [`Config`] trait
///
/// This macro will implement the [`Config`] trait for the given struct
/// and define the associated [`PartialConfig`] struct
///
/// The generated [`PartialConfig`] struct will have the name `Partial{StructName}` by default
/// and is accesible through the `Partial` associated type of the generated [`Config`] trait
///
/// The generated [`PartialConfig`] will have the same visibility as the [`Config`] (eg: pub, pub(crate), private, etc).
///
/// The [`PartialConfig`] generated type is a deep-partial version of the struct
///
/// See the [`Config`] and [`PartialConfig`] documentation for more information on the available methods.
///
/// # Container Attributes
/// | Attribute | Description | Default | Example | Observations |
/// | --- | --- | --- | --- | --- |
/// | rename_all | The case conversion to apply to all fields | none | `#[config(rename_all = "snake_case")]` | This will apply `#[serde(rename_all)]` to the PartialConfig struct |
/// | skip_env | If applied, this struct will not load anything from env variables | false | `#[config(skip_env)]` |
/// | env_prefix | The prefix to use for all fields environment variables | "{}" | `#[config(env_prefix = "{}MY_APP_")]` | Almost always you'll want to include the `{}` placeholder like `"{}MY_APP"` to allow auto generated prefixes to work, if not the env key will be fixed to the value of the attribute |
/// | allow_unknown_fields | Allow unknown fields in deserialization of the PartialConfig type | false | `#[config(allow_unknown_fields)]` | By default metre will add a `#[serde(deny_unknown_fields)]` to the Partial definition, use this attribute if you want to override this behavior |
/// | parial_name | The name of the generated PartialConfig struct | `Partial{StructName}` | `#[config(partial_name = PartialMyConfig)] | rename the PartialConfig generated struct, the PartialConfig struct will have the same visibility as the struct |
/// | crate | Rename the metre crate in the generated derive code | `metre` | `#[config(crate = other)]` | This is almost only useful for internal unit tests |
///
/// # Field Attributes
/// | Attribute | Description | Default | Example | Observations |
/// | --- | --- | --- | --- | --- |
/// | env | The name of the environment variable to use for this field | `"{}PROPERTY_NAME"` | `#[config(env = "{}PORT")]` | The default value of the attribute is the SCREAMING_SNAKE_CASE version of the field name after applying rename and rename_all configurations, and the `{}` placeholder is filled with the auto calculated env prefix |
/// | skip_env | If applied, this field will not load from env variables | false | `#[config(skip_env)]` | This attribute has precedence over the skip_env attribute in the container |
/// | parse_env | The name of the function to use to parse the value from the environment variable | `FromStr::from_str` | `#[config(parse_env = parse_fn)]` | The function must have the signature `fn(&str) -> Result<Option<T>, E>` where `T` is the type of the field and `E` is any error that implements Display, see the [`parse`] module to see utility functions that can be used here |
/// | merge | The name of the function to use to merge two values of this field | - | `#[config(merge = merge_fn)]` | The function must have the signature `fn(&mut Option<T>, Option<T>) -> Result<(), E>` where `T` is the type of the field and `E` is any error that implements Display, see the [`merge`] module to find utility functions that can be used here, the default implementation replaces the previous value with the next, if it is present in the new added stage |
/// | default | The default value to use for this field | none | `#[config(default = 3000)]` | The default value must be of the same type as the field, if the field is an Option, the default value must be of the same type as the inner type of the Option, the [`Default::default`] implementation of the Partial struct will not use this value, to get the values defined with this attribute use [`PartialConfig::defaults`] |
/// | flatten | If applied, this field will be merged with the previous stage instead of replacing it | false | `#[config(flatten)]` | This attribute will apply a `#[serde(flatten)]` to the PartialConfig struct, it will also modify the calculated env key prefix for nested fields |
/// | nested | If applied, this field will be treated as a nested configuration | false | `#[config(nested)]` | This attrbute indicates that this field is a nested partial configuration, the nested field must also implement the [`Config`] trait |
/// | rename | The rename the field in the partial configuration | - | `#[config(rename = "other_name")]` | This will apply a `#[serde(rename)]` attribute to the Partial struct, it will also modify the auto calculated env key for the field |
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use metre_macros::Config;

use error::{FromPartialError, MergeError};

#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
use error::FromEnvError; 

/// The Config trait that is implemented from the [`Config`](macro@Config) derive macro
///
/// see the [`Config`](macro@Config) derive macro for more information
/// and the [`PartialConfig`] trait for more information on the methods available
pub trait Config: Sized {
  /// The Partial type generated by the [`Config`](macro@Config) derive macro
  ///
  /// This is a deep-partial version of the struct
  type Partial: PartialConfig;

  /// Tries to create a configuration from a partial configuration
  ///
  /// This will error if the partial configuration is missing required properties
  fn from_partial(partial: Self::Partial) -> Result<Self, FromPartialError>;
}

/// The partial configuration trait that is automatically implemented by the [`Config`](macro@Config) derive macro.
///
/// You should almost never want to implement this trait manually.
///
/// Note that this trait is implemented for the [Config::Partial] associated type and not for the struct itself.
///
/// The [Config::Partial] associated type is a auto generated struct definition that is a deep partial version of target struct
pub trait PartialConfig: DeserializeOwned + Default {
  /// Get the default values for this partial configuration as defined with the `#[config(default = value)]` attributes
  ///
  /// Note that the [Default::default] implementation will differ from this method, as it will return a totally empty struct
  fn defaults() -> Self;

  /// Deep merge this partial configuration with another
  fn merge(&mut self, other: Self) -> Result<(), MergeError>;

  /// List of missing properties in this partial configuration that are required in the final configuration
  fn list_missing_properties(&self) -> Vec<String>;

  /// Returns true if this partial configuration has no values
  fn is_empty(&self) -> bool;

  /// Create a partial configuration from environment variables
  /// [`EnvProvider`] is specially usefull for unit tests and is already implemented for several
  /// types of [HashMap]'s and [BTreeMap]'s from the standard library
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env_with_provider_and_optional_prefix<E: EnvProvider>(
    env: &E,
    prefix: Option<&str>,
  ) -> Result<Self, FromEnvError>;

  /// Forwards to [`Self::from_env_with_provider_and_optional_prefix`]
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env_with_provider_and_prefix<E: EnvProvider, P: AsRef<str>>(
    env: &E,
    prefix: P,
  ) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(env, Some(prefix.as_ref()))
  }

  /// Forwards to [`Self::from_env_with_provider_and_optional_prefix`]
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env_with_provider<E: EnvProvider>(env: &E) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(env, None)
  }

  /// Forwards to [`Self::from_env_with_provider_and_optional_prefix`] with the standard library's [`std::env::var`] as the [`EnvProvider`]
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env_with_prefix<P: AsRef<str>>(prefix: P) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(&StdEnv, Some(prefix.as_ref()))
  }

  /// Forwards to [`Self::from_env_with_provider_and_optional_prefix`] with the standard library's [`std::env::var`] as the [`EnvProvider`]
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env() -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(&StdEnv, None)
  }
}

impl<T: Config> Config for Option<T> {
  type Partial = Option<T::Partial>;
  fn from_partial(partial: Self::Partial) -> Result<Self, FromPartialError> {
    match partial {
      None => Ok(None),
      Some(inner) => {
        if inner.is_empty() {
          Ok(None)
        } else {
          let v = T::from_partial(inner)?;
          Ok(Some(v))
        }
      }
    }
  }
}

impl<T: PartialConfig> PartialConfig for Option<T> {
  fn defaults() -> Self {
    let inner = T::defaults();
    if inner.is_empty() {
      None
    } else {
      Some(inner)
    }
  }

  fn merge(&mut self, other: Self) -> Result<(), MergeError> {
    match (self.as_mut(), other) {
      (None, Some(other)) => *self = Some(other),
      (Some(me), Some(other)) => me.merge(other)?,
      (Some(_), None) => {}
      (None, None) => {}
    };

    Ok(())
  }

  fn list_missing_properties(&self) -> Vec<String> {
    match self {
      None => vec![],
      Some(me) => {
        if !me.is_empty() {
          me.list_missing_properties()
        } else {
          vec![]
        }
      }
    }
  }

  fn is_empty(&self) -> bool {
    match self {
      None => true,
      Some(me) => me.is_empty(),
    }
  }

  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  fn from_env_with_provider_and_optional_prefix<E: EnvProvider>(
    env: &E,
    prefix: Option<&str>,
  ) -> Result<Self, FromEnvError> {
    let v = T::from_env_with_provider_and_optional_prefix(env, prefix)?;
    if v.is_empty() {
      Ok(None)
    } else {
      Ok(Some(v))
    }
  }
}

/// Implement this trait if you want to load a configuration from custom environment variables
/// that are not in [`std::env::var`]
///
/// This is speecially usefull for unit tests
///
/// This trait is already implemented for several kinds of [HashMap]'s and [BTreeMap]'s from the standard library
pub trait EnvProvider {
  type Error: Display;
  /// Read a variable from the enviroment
  ///
  /// This should fail if the variable is not UTF-8 encoded
  ///
  /// If the variable is not present, implementations should return `Ok(None)`
  fn get(&self, key: &str) -> Result<Option<String>, Self::Error>;
}

#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
macro_rules! impl_env_provider_for_map {
  ($ty:ty) => {
    impl EnvProvider for $ty {
      type Error = Infallible;
      fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        Ok(self.get(key).map(ToString::to_string))
      }
    }
  };
}

#[cfg(feature = "env")]
impl_env_provider_for_map!(HashMap<String, String>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(HashMap<&str, String>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(HashMap<String, &str>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(HashMap<&str, &str>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(BTreeMap<String, String>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(BTreeMap<&str, String>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(BTreeMap<String, &str>);
#[cfg(feature = "env")]
impl_env_provider_for_map!(BTreeMap<&str, &str>);

/// An implementation of [`EnvProvider`] that reads from the standard library's [`std::env::var`]
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
pub struct StdEnv;

#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
impl EnvProvider for StdEnv {
  type Error = VarError;
  fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
    match std::env::var(key) {
      Err(e) => match &e {
        VarError::NotPresent => Ok(None),
        VarError::NotUnicode(_) => Err(e),
      },
      Ok(v) => Ok(Some(v)),
    }
  }
}

/// A location from where a configuration was loaded
///
/// can be from Memory, File, or URL
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LoadLocation {
  Memory,
  File(String),
  #[cfg(any(feature = "url-blocking", feature = "url-async"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "url-blocking", feature = "url-async"))))]
  Url(String),
}

impl Display for LoadLocation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LoadLocation::*;
    match self {
      Memory => write!(f, "{}", "memory".yellow()),
      File(location) => write!(f, "file: {}", location.yellow()),
      #[cfg(any(feature = "url-blocking", feature = "url-async"))]
      Url(location) => write!(f, "url: {}", location.yellow()),
    }
  }
}

/// List of known configuration formats
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Format {
  #[cfg(feature = "json")]
  #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
  Json,
  #[cfg(feature = "jsonc")]
  #[cfg_attr(docsrs, doc(cfg(feature = "jsonc")))]
  Jsonc,
  #[cfg(feature = "toml")]
  #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
  Toml,
  #[cfg(feature = "yaml")]
  #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
  Yaml,
}

/// The configuration loader
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConfigLoader<T: Config> {
  partial: T::Partial,
}

impl<T: Config> ConfigLoader<T> {
  /// Create a new configuration loader with all fields set as empty
  pub fn new() -> Self {
    Self {
      partial: T::Partial::default(),
    }
  }

  /// Add a partial configuration from a file
  #[allow(clippy::result_large_err)]
  pub fn file(&mut self, path: &str, format: Format) -> Result<&mut Self, Error> {
    let code = std::fs::read_to_string(path).map_err(|e| Error::Io {
      path: path.into(),
      source: Arc::new(e),
    })?;

    self.code_with_location(&code, format, LoadLocation::File(path.to_string()))
  }

  /// Add a partial configuration from a file, if it exists
  #[allow(clippy::result_large_err)]
  pub fn file_optional(&mut self, path: &str, format: Format) -> Result<&mut Self, Error> {
    let exists = Path::new(path).try_exists().map_err(|e| Error::Io {
      path: path.into(),
      source: Arc::new(e),
    })?;

    if exists {
      self.file(path, format)
    } else {
      Ok(self)
    }
  }

  /// Add a partial configuration from enviroment varialbes
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn env(&mut self) -> Result<&mut Self, Error> {
    self._env(&StdEnv, None)
  }

  /// Add a partial configuration from enviroment variables with a prefix
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn env_with_prefix(&mut self, prefix: &str) -> Result<&mut Self, Error> {
    self._env(&StdEnv, Some(prefix))
  }

  /// Add a partial configuration from enviroment variables with a custom provider
  ///
  /// The provider must implement the [`EnvProvider`] trait
  ///
  /// The [`EnvProvider`] trait is already implemented for several kinds of Maps from the standard library
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn env_with_provider<E: EnvProvider>(&mut self, env: &E) -> Result<&mut Self, Error> {
    self._env(env, None)
  }

  /// See [`Self::env_with_provider`] and [`Self::env_with_prefix`]
  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn env_with_provider_and_prefix<E: EnvProvider>(
    &mut self,
    env: &E,
    prefix: &str,
  ) -> Result<&mut Self, Error> {
    self._env(env, Some(prefix))
  }

  /// Add a partial configuration from in-memory code
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn code<S: AsRef<str>>(&mut self, code: S, format: Format) -> Result<&mut Self, Error> {
    self._code(code.as_ref(), format, LoadLocation::Memory)
  }

  /// Add a partial configuration from in-memory code
  ///
  /// Specifying the [`LoadLocation`] of the in-memory code is useful for error reporting
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn code_with_location<S: AsRef<str>>(
    &mut self,
    code: S,
    format: Format,
    location: LoadLocation,
  ) -> Result<&mut Self, Error> {
    self._code(code.as_ref(), format, location)
  }

  /// Add a partial configuration from a url
  #[cfg(feature = "url-blocking")]
  #[cfg_attr(docsrs, doc(cfg(feature = "url-blocking")))]
  #[allow(clippy::result_large_err)]
  pub fn url(&mut self, url: &str, format: Format) -> Result<&mut Self, Error> {
    let map_err = |e| Error::Network {
      url: url.to_string(),
      source: Arc::new(e),
    };

    let code = reqwest::blocking::get(url)
      .map_err(map_err)?
      .text()
      .map_err(map_err)?;

    self._code(&code, format, LoadLocation::Url(url.to_string()))
  }

  #[cfg(feature = "url-async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "url-async")))]
  /// Add a partial configuration from a url, async version
  pub async fn url_async(&mut self, url: &str, format: Format) -> Result<&mut Self, Error> {
    let map_err = |e| Error::Network {
      url: url.to_string(),
      source: Arc::new(e),
    };

    let code = reqwest::get(url)
      .await
      .map_err(map_err)?
      .text()
      .await
      .map_err(map_err)?;

    self._code(&code, format, LoadLocation::Url(url.to_string()))
  }

  #[cfg(feature = "env")]
  #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  fn _env<E: EnvProvider>(&mut self, env: &E, prefix: Option<&str>) -> Result<&mut Self, Error> {
    let partial = T::Partial::from_env_with_provider_and_optional_prefix(env, prefix)?;
    self._add(partial)
  }

  #[allow(unused)]
  #[allow(clippy::result_large_err)]
  fn _code(
    &mut self,
    code: &str,
    format: Format,
    location: LoadLocation,
  ) -> Result<&mut Self, Error> {
    let partial = match format {
      #[cfg(feature = "json")]
      #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
      Format::Json => serde_json::from_str(code).map_err(|e| Error::Json {
        location,
        source: Arc::new(e),
      })?,

      #[cfg(feature = "jsonc")]
      #[cfg_attr(docsrs, doc(cfg(feature = "jsonc")))]
      Format::Jsonc => {
        let reader = json_comments::StripComments::new(code.as_bytes());
        serde_json::from_reader(reader).map_err(|e| Error::Json {
          location,
          source: Arc::new(e),
        })?
      }

      #[cfg(feature = "toml")]
      #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
      Format::Toml => toml::from_str(code).map_err(|e| Error::Toml {
        location,
        source: e,
      })?,

      #[cfg(feature = "yaml")]
      #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
      Format::Yaml => serde_yaml::from_str(code).map_err(|e| Error::Yaml {
        location,
        source: Arc::new(e),
      })?,
    };

    self._add(partial)
  }

  /// Add a partial configuration from the `#[config(default = value)]` attributes
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn defaults(&mut self) -> Result<&mut Self, Error> {
    self._add(T::Partial::defaults())
  }

  /// Add a pre generated partial configuration
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn partial(&mut self, partial: T::Partial) -> Result<&mut Self, Error> {
    self._add(partial)
  }

  #[inline(always)]
  #[allow(clippy::result_large_err)]
  fn _add(&mut self, partial: T::Partial) -> Result<&mut Self, Error> {
    self.partial.merge(partial)?;
    Ok(self)
  }

  /// Get a reference to the partial configuration
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn partial_state(&self) -> &T::Partial {
    &self.partial
  }

  /// Get a mutable reference to the partial configuration
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn partial_state_mut(&mut self) -> &mut T::Partial {
    &mut self.partial
  }

  /// Get the final Config from the sum of all previously added stages
  ///
  /// this function will error if there are missing required properties
  #[inline(always)]
  #[allow(clippy::result_large_err)]
  pub fn finish(self) -> Result<T, Error> {
    let v = T::from_partial(self.partial)?;
    Ok(v)
  }
}

impl<T: Config> Default for ConfigLoader<T> {
  fn default() -> Self {
    Self::new()
  }
}
