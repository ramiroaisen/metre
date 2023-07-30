use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::env::VarError;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

pub mod error;

pub use error::Error;
pub use metre_macros::Config;

#[doc(hidden)]
pub mod util;

use error::{FromEnvError, FromPartialError, LoadLocation, MergeError};

pub trait Config: Sized {
  type Partial: PartialConfig;
  fn from_partial(partial: Self::Partial) -> Result<Self, FromPartialError>;
}

pub trait PartialConfig: DeserializeOwned + Default {

  fn defaults() -> Self;

  fn merge(&mut self, other: Self) -> Result<(), MergeError>;

  fn list_missing_properties(&self) -> Vec<String>;

  fn is_empty(&self) -> bool;

  fn from_env_with_provider_and_optional_prefix<E: EnvProvider>(
    env: &E,
    prefix: Option<&str>,
  ) -> Result<Self, FromEnvError>;

  fn from_env_with_provider_and_prefix<E: EnvProvider, P: AsRef<str>>(
    env: &E,
    prefix: P,
  ) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(env, Some(prefix.as_ref()))
  }

  fn from_env_with_provider<E: EnvProvider>(env: &E) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(env, None)
  }

  fn from_env_with_prefix<P: AsRef<str>>(prefix: P) -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(&StdEnv, Some(prefix.as_ref()))
  }

  fn from_env() -> Result<Self, FromEnvError> {
    Self::from_env_with_provider_and_optional_prefix(&StdEnv, None)
  }
}

impl<T: Config> Config for Option<T> {
  type Partial = T::Partial;
  fn from_partial(partial: Self::Partial) -> Result<Self, FromPartialError> {
    if partial.is_empty() {
      Ok(None)
    } else {
      let v = T::from_partial(partial)?;
      Ok(Some(v))
    }
  }
}

impl<T: PartialConfig> PartialConfig for Option<T> {
  
  fn defaults() -> Self {
      Some(T::defaults())
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
      None => T::default().list_missing_properties(),
      Some(me) => me.list_missing_properties(),
    }
  }

  fn is_empty(&self) -> bool {
    match self {
      None => true,
      Some(me) => me.is_empty(),
    }
  }

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

pub trait EnvProvider {
  type Error: Display;
  fn get(&self, key: &str) -> Result<Option<String>, Self::Error>;
}

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

impl_env_provider_for_map!(HashMap<String, String>);
impl_env_provider_for_map!(HashMap<&str, String>);
impl_env_provider_for_map!(HashMap<String, &str>);
impl_env_provider_for_map!(HashMap<&str, &str>);
impl_env_provider_for_map!(BTreeMap<String, String>);
impl_env_provider_for_map!(BTreeMap<&str, String>);
impl_env_provider_for_map!(BTreeMap<String, &str>);
impl_env_provider_for_map!(BTreeMap<&str, &str>);

#[derive(Debug, Clone, Copy)]
pub struct StdEnv;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Format {
  Json,
  Jsonc,
  Toml,
  Yaml,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Stage {
  Env,
  Json { code: String, path: Option<String> },
  Jsonc { code: String, path: Option<String> },
  Toml { code: String, path: Option<String> },
  Yaml { code: String, path: Option<String> },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConfigLoader<T: Config> {
  partial: T::Partial,
}

impl<T: Config> ConfigLoader<T> {
  pub fn new() -> Self {
    Self {
      partial: T::Partial::default(),
    }
  }

  pub fn file(&mut self, path: &str, format: Format) -> Result<&mut Self, Error> {
    let code = std::fs::read_to_string(path).map_err(|e| Error::Io {
      path: path.into(),
      source: Arc::new(e),
    })?;

    self.code_with_location(&code, format, LoadLocation::File(path.to_string()))
  }

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

  #[inline(always)]
  pub fn env(&mut self) -> Result<&mut Self, Error> {
    self._env(&StdEnv, None)
  }

  #[inline(always)]
  pub fn env_with_prefix(&mut self, prefix: &str) -> Result<&mut Self, Error> {
    self._env(&StdEnv, Some(prefix))
  }

  #[inline(always)]
  pub fn env_with_provider<E: EnvProvider>(&mut self, env: &E) -> Result<&mut Self, Error> {
    self._env(env, None)
  }

  #[inline(always)]
  pub fn env_with_provider_and_prefix<E: EnvProvider>(
    &mut self,
    env: &E,
    prefix: &str,
  ) -> Result<&mut Self, Error> {
    self._env(env, Some(prefix))
  }

  #[inline(always)]
  fn _env<E: EnvProvider>(&mut self, env: &E, prefix: Option<&str>) -> Result<&mut Self, Error> {
    let partial = T::Partial::from_env_with_provider_and_optional_prefix(env, prefix)?;
    self._add(partial)
  }

  #[inline(always)]
  pub fn code<S: AsRef<str>>(&mut self, code: S, format: Format) -> Result<&mut Self, Error> {
    self._code(code.as_ref(), format, LoadLocation::Memory)
  }

  #[inline(always)]
  pub fn code_with_location<S: AsRef<str>>(
    &mut self,
    code: S,
    format: Format,
    location: LoadLocation,
  ) -> Result<&mut Self, Error> {
    self._code(code.as_ref(), format, location)
  }


  pub fn url(&mut self, url: &str, format: Format) -> Result<&mut Self, Error> {
    let map_err = |e| {
        Error::Network { url: url.to_string(), source: Arc::new(e) }
    };

    let code = reqwest::blocking::get(url).map_err(map_err)?
        .text().map_err(map_err)?;

    self._code(&code, format, LoadLocation::Url(url.to_string()))
  }

  pub async fn url_async(&mut self, url: &str, format: Format) -> Result<&mut Self, Error> {
    let map_err = |e| {
        Error::Network { url: url.to_string(), source: Arc::new(e) }
    };
    
    let code = reqwest::get(url).await.map_err(map_err)?
        .text().await.map_err(map_err)?;

    self._code(&code, format, LoadLocation::Url(url.to_string()))
  }

  fn _code(
    &mut self,
    code: &str,
    format: Format,
    location: LoadLocation,
  ) -> Result<&mut Self, Error> {

    let partial = match format {
      Format::Json => serde_json::from_str(code).map_err(|e| Error::Json {
        location,
        source: Arc::new(e),
      })?,

      Format::Jsonc => {
        let reader = json_comments::StripComments::new(code.as_bytes());
        serde_json::from_reader(reader).map_err(|e| Error::Json {
          location,
          source: Arc::new(e),
        })?
      }

      Format::Toml => {
        toml::from_str(code).map_err(|e| Error::Toml {
            location,
            source: e,
        })?
     },

      Format::Yaml => serde_yaml::from_str(code).map_err(|e| Error::Yaml {
        location,
        source: Arc::new(e),
      })?,
    };

    self._add(partial)
  }

  #[inline(always)]
  pub fn defaults(&mut self) -> Result<&mut Self, Error> {
    self._add(T::Partial::defaults())
  }

  #[inline(always)]
  pub fn partial(&mut self, partial: T::Partial) -> Result<&mut Self, Error> {
    self._add(partial)    
  }
  
  #[inline(always)]
  fn _add(&mut self, partial: T::Partial) -> Result<&mut Self, Error> {
    self.partial.merge(partial)?;
    Ok(self)
  }

  #[inline(always)]
  pub fn partial_state(&self) -> &T::Partial {
    &self.partial
  }

  #[inline(always)]
  pub fn partial_state_mut(&mut self) -> &mut T::Partial {
    &mut self.partial
  }

  #[inline(always)]
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
