use darling::util::SpannedValue;
use darling::FromAttributes;
use inflector::Inflector;
use proc_macro2::Ident;
use syn::{Expr, ExprPath, Meta, Path};

#[derive(Debug, Clone, Copy)]
pub enum Inflection {
  Lower,
  Upper,
  Snake,
  Camel,
  Pascal,
  Kebab,
  UpperSnake,
  UpperKebab,
}

impl Inflection {
  pub fn apply(self, src: &str) -> String {
    use Inflection::*;
    match self {
      Lower => src.to_lowercase(),
      Upper => src.to_uppercase(),
      Snake => src.to_snake_case(),
      Camel => src.to_camel_case(),
      Pascal => src.to_pascal_case(),
      Kebab => src.to_kebab_case(),
      UpperSnake => src.to_screaming_snake_case(),
      UpperKebab => src.to_kebab_case().to_uppercase(),
    }
  }
}

impl std::str::FromStr for Inflection {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, ()> {
    use Inflection::*;
    let v = match s {
      "lowercase" => Lower,
      "UPPERCASE" => Upper,
      "snake_case" => Snake,
      "camelCase" => Camel,
      "PascalCase" => Pascal,
      "kebab-case" => Kebab,
      "SCREAMING_SNAKE_CASE" => UpperSnake,
      "SCREAMING-KEBAB-CASE" => UpperKebab,
      _ => return Err(()),
    };

    Ok(v)
  }
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(config))]
pub struct FieldArgs {
  pub nested: bool,
  pub flatten: bool,
  pub env: Option<SpannedValue<String>>,

  #[darling(with = preserve_str_literal, map = Some)]
  pub default: Option<Expr>,

  #[darling(default)]
  pub skip_env: SpannedValue<bool>,

  pub parse_env: Option<ExprPath>,
  pub merge: Option<ExprPath>,
  pub rename: Option<String>,
}

#[derive(FromAttributes, Default)]
#[darling(default, attributes(config))]
pub struct ContainerAttrs {
  pub partial_name: Option<Ident>,
  pub env_prefix: Option<SpannedValue<String>>,
  #[darling(rename = "crate")]
  pub metre_crate: Option<Path>,
  #[darling(default)]
  pub skip_env: SpannedValue<bool>,
  pub rename_all: Option<SpannedValue<String>>,
  pub allow_unknown_fields: bool,
}

impl ContainerAttrs {
  pub fn rename_all_inflection(&self) -> Result<Option<Inflection>, syn::Error> {
    use std::str::FromStr;
    match &self.rename_all {
      None => Ok(None),
      Some(v) => {
        let span = v.span();
        let value: &str = v;
        let inflection = match Inflection::from_str(value) {
          Ok(inflection) => inflection,
          Err(()) => return Err(syn::Error::new(span, format!("unknown rename_all attribute value {}, valid alternatives are lowercase, UPPERCASE, snake_case, camelCase, PascalCase, kebab-case, SCREAMING_SNAKE_CASE and SCREAMING-KEBAB-CASE", value)))
        };

        Ok(Some(inflection))
      }
    }
  }
}

// copied from crates.io/schematic
pub fn preserve_str_literal(meta: &Meta) -> darling::Result<Expr> {
  match meta {
    Meta::Path(_) => Err(darling::Error::unsupported_format("path").with_span(meta)),
    Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
    Meta::NameValue(nv) => Ok(nv.value.clone()),
  }
}
