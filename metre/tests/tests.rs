use metre::Config;
use metre::ConfigLoader;
use metre::Format;
use metre::PartialConfig;
use std::collections::HashMap;

#[test]
fn test() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(default = "default".into())]
    default: String,

    #[config(nested)]
    nested: Nested,

    optional: Option<String>,

    #[config(parse_env = metre::parse::comma_separated::<String>)]
    list: Vec<String>,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    a: String,
    b: u8,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader.defaults().unwrap();

  loader
    .code(
      r#"
        nested.a = "a"
        nested.b = 1
        list = ["item"]
        "#,
      Format::Toml,
    )
    .unwrap();

  let config = loader.finish().unwrap();

  assert_eq!(
    config,
    Conf {
      default: "default".into(),
      list: vec!["item".into()],
      nested: Nested {
        a: "a".into(),
        b: 1
      },
      optional: None,
    }
  );
}

#[test]
fn from_fixed_env() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(env = "MY_APP_PORT")]
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("MY_APP_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader.env_with_provider(&env).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[test]
fn from_env_with_prefix() {
  #[derive(Config, Debug, Eq, PartialEq)]
  #[config(env_prefix = "{}CONF_")]
  struct Conf {
    #[config(env = "PORT")]
    port: u16,
    #[config(env = "{}LIST_RENAMED", parse_env = metre::parse::comma_separated::<String>)]
    list: Vec<String>,
    #[config(rename = "opt")]
    optional: Option<String>,
  }

  let mut env = HashMap::new();
  env.insert("PORT", "3000");
  env.insert("MY_APP_CONF_LIST_RENAMED", "item1,item2");
  env.insert("MY_APP_CONF_OPT", "optional");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .env_with_provider_and_prefix(&env, "MY_APP_")
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
  assert_eq!(config.list, vec!["item1".to_string(), "item2".to_string()]);
  assert_eq!(config.optional, Some("optional".into()));
}

#[cfg(feature = "json")]
#[test]
fn from_json_code() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        {
          "port": 3000
        }
        "#,
      Format::Json,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "jsonc")]
#[test]
fn should_load_jsonc_code() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        {
          // this is a comment
          "port": 3000
        }
        "#,
      Format::Jsonc,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "toml")]
#[test]
fn should_load_toml_code() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port = 3000
        "#,
      Format::Toml,
    )
    .unwrap();

  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "yaml")]
#[test]
fn should_load_yaml_code() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "env")]
#[test]
fn should_load_env() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader.env_with_provider(&env).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "yaml")]
#[test]
fn should_accumulate_partial_states() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();
  let partial_state = loader.partial_state();
  assert_eq!(partial_state.port, Some(3000));

  loader
    .code(
      r#"
        port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();
  let partial_state = loader.partial_state();
  assert_eq!(partial_state.port, Some(3001));
}

#[cfg(feature = "yaml")]
#[test]
fn should_merge_partal_states() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        addr: "addr"
        "#,
      Format::Yaml,
    )
    .unwrap();

  loader
    .code(
      r#"
        port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();

  let partial_state = loader.partial_state();

  assert_eq!(partial_state.port, Some(3001));
  assert_eq!(partial_state.addr, Some("addr".into()));

  let config = loader.finish().unwrap();
  assert_eq!(config.port, 3001);
  assert_eq!(config.addr, "addr");
}

#[cfg(feature = "yaml")]
#[test]
fn should_error_on_missing_properties() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();

  let err = loader.finish().unwrap_err();
  assert!(err.to_string().contains("missing"));
}

#[cfg(feature = "yaml")]
#[test]
fn should_list_missing_properties_and_error() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();

  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, ["addr"]);

  assert!(loader.finish().is_err());
}

#[cfg(feature = "yaml")]
#[test]
fn should_not_list_missing_properties_that_are_optional() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: Option<String>,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();

  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, Vec::<String>::new());
  assert!(loader.finish().is_ok());
}

#[cfg(feature = "env")]
#[test]
fn should_skip_env() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(skip_env)]
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader.env_with_provider(&env).unwrap();
  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, vec!["port"]);

  loader.finish().unwrap_err();
}

#[cfg(feature = "env")]
#[test]
fn should_skip_env_for_nested() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(nested)]
    nested: Nested,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    #[config(skip_env)]
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("NESTED_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader.env_with_provider(&env).unwrap();
  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, ["nested.port"]);

  loader.finish().unwrap_err();
}

#[cfg(feature = "env")]
#[test]
fn should_skip_env_for_nested_with_prefix() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(nested)]
    nested: Nested,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    #[config(skip_env)]
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("MY_APP_N_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .env_with_provider_and_prefix(&env, "MY_APP_")
    .unwrap();
  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, ["nested.port"]);

  loader.finish().unwrap_err();
}


#[cfg(all(feature = "yaml", feature = "env"))]
#[test]
fn should_override_with_env() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();
  loader.env_with_provider(&env).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(all(feature = "yaml", feature = "env"))]
#[test]
fn should_override_with_env_with_prefix() {
  #[derive(Config, Debug, Eq, PartialEq)]
  #[config(env_prefix = "{}CONF_")]
  struct Conf {
    port: u16,
  }

  let mut env = HashMap::new();
  env.insert("MY_APP_CONF_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();
  loader
    .env_with_provider_and_prefix(&env, "MY_APP_")
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(all(feature = "yaml", feature = "env"))]
#[test]
fn should_override_with_env_with_prefix_and_rename() {
  #[derive(Config, Debug, Eq, PartialEq)]
  #[config(env_prefix = "{}CONF_")]
  struct Conf {
    #[config(rename = "port")]
    port_renamed: u16,
  }

  let mut env = HashMap::new();
  env.insert("MY_APP_CONF_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();
  loader
    .env_with_provider_and_prefix(&env, "MY_APP_")
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port_renamed, 3000);
}

#[cfg(all(feature = "yaml", feature = "env"))]
#[test]
fn should_override_with_env_with_prefix_and_rename_and_nested() {
  #[derive(Config, Debug, Eq, PartialEq)]
  #[config(env_prefix = "{}CONF_")]
  struct Conf {
    #[config(nested)]
    nested: Nested,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    #[config(rename = "port")]
    port_renamed: u16,
  }

  let mut env = HashMap::new();
  env.insert("MY_APP_CONF_NESTED_PORT", "3000");

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        nested:
          port: 3001
        "#,
      Format::Yaml,
    )
    .unwrap();
  loader
    .env_with_provider_and_prefix(&env, "MY_APP_")
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.nested.port_renamed, 3000);
}

#[cfg(feature = "json")]
#[test]
fn should_error_on_invalid_type() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        {
          "port": "3001"
        }
        "#,
      Format::Json,
    )
    .unwrap_err();
}

#[test]
fn should_not_list_as_missing_optional_types() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: Option<u16>,
  }

  let loader = ConfigLoader::<Conf>::new();
  let missing = loader.partial_state().list_missing_properties();
  assert_eq!(missing, Vec::<String>::new());
}

#[cfg(feature = "yaml")]
#[test]
fn should_work_for_nested_optional_types() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(nested)]
    nested: Option<Nested>,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        nested:
          port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(
    config,
    Conf {
      nested: Some(Nested { port: 3000 })
    }
  );
}

#[test]
fn should_work_for_nested_optional_missing_values() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(nested)]
    nested: Option<Nested>,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    port: u16,
  }

  let loader = ConfigLoader::<Conf>::new();
  let config = loader.finish().unwrap();

  assert_eq!(config, Conf { nested: None });
}

#[test]
fn should_respect_defaults_from_attrs() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(default = 3000)]
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader.defaults().unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[test]
fn should_respect_defaults_for_nested_configs() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(nested)]
    nested: Nested,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    #[config(default = 3000)]
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader.defaults().unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(
    config,
    Conf {
      nested: Nested { port: 3000 }
    }
  );
}

#[cfg(feature = "toml")]
#[test]
fn should_work_with_custom_merge_functions() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    #[config(merge = metre::merge::append_vec, skip_env)]
    list: Vec<String>,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        list = ["item1"]
        "#,
      Format::Toml,
    )
    .unwrap();
  loader
    .code(
      r#"
        list = ["item2"]
        "#,
      Format::Toml,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.list, ["item1", "item2"]);
}

#[cfg(feature = "yaml")]
#[test]
fn should_error_on_unkown_extra_properties() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        addr: "addr"
        "#,
      Format::Yaml,
    )
    .unwrap_err();
}

#[cfg(feature = "yaml")]
#[test]
fn should_not_error_on_unkown_extra_properties_with_allow_unkown_fields_attr() {
  #[derive(Config, Debug, Eq, PartialEq)]
  #[config(allow_unknown_fields)]
  struct Conf {
    port: u16,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        addr: "addr"
        "#,
      Format::Yaml,
    )
    .unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(all(feature = "yaml", feature = "json"))]
#[test]
fn partial_config_should_not_serialize_missing_properties() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();
  let partial = loader.partial_state();

  let serialized = serde_json::to_string(&partial).unwrap();
  assert_eq!(serialized, "{\"port\":3000}");
}

#[cfg(all(feature = "yaml", feature = "json"))]
#[test]
fn partial_config_should_not_serialize_empty_nested_configs() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    #[config(nested)]
    nested: Nested,
  }

  #[derive(Config, Debug, Eq, PartialEq)]
  struct Nested {
    prop: String,
  }

  let mut loader = ConfigLoader::<Conf>::new();
  loader
    .code(
      r#"
        port: 3000
        "#,
      Format::Yaml,
    )
    .unwrap();
  let partial = loader.partial_state();

  let serialized = serde_json::to_string(&partial).unwrap();
  assert_eq!(serialized, "{\"port\":3000}");
}

#[cfg(feature = "json")]
#[test]
fn should_load_json_file() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
  }

  let path = std::env::temp_dir()
    .as_path()
    .join("metre-test-config.json");

  std::fs::write(&path, "{\"port\": 3000}").unwrap();

  let mut loader = ConfigLoader::<Conf>::new();
  loader.file(path.to_str().unwrap(), Format::Json).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
}

#[cfg(feature = "jsonc")]
#[test]
fn should_load_jsonc_file() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let path = std::env::temp_dir()
    .as_path()
    .join("metre-test-config.jsonc");
  std::fs::write(
    &path,
    r#"
      {
        // this is a comment
        "port": 3000,
        "addr": "addr"
      }
      "#,
  )
  .unwrap();

  let mut loader = ConfigLoader::<Conf>::new();
  loader.file(path.to_str().unwrap(), Format::Jsonc).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
  assert_eq!(config.addr, "addr");
}

#[cfg(feature = "toml")]
#[test]
fn should_load_toml_file() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let path = std::env::temp_dir()
    .as_path()
    .join("metre-test-config.toml");
  std::fs::write(
    &path,
    r#"
      port = 3000
      addr = "addr"
      "#,
  )
  .unwrap();

  let mut loader = ConfigLoader::<Conf>::new();
  loader.file(path.to_str().unwrap(), Format::Toml).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
  assert_eq!(config.addr, "addr");
}

#[cfg(feature = "yaml")]
#[test]
fn should_load_yaml_file() {
  #[derive(Config, Debug, Eq, PartialEq)]
  struct Conf {
    port: u16,
    addr: String,
  }

  let path = std::env::temp_dir()
    .as_path()
    .join("metre-test-config.yaml");
  std::fs::write(
    &path,
    r#"
      port: 3000
      addr: "addr"
      "#,
  )
  .unwrap();

  let mut loader = ConfigLoader::<Conf>::new();
  loader.file(path.to_str().unwrap(), Format::Yaml).unwrap();
  let config = loader.finish().unwrap();

  assert_eq!(config.port, 3000);
  assert_eq!(config.addr, "addr");
}
