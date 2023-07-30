use metre::Config;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Asd {
  Asd
}

fn parse_enum(_: &str) -> Result<Option<Asd>, Box<dyn std::error::Error>> {
  todo!();
}

#[test]
fn simple() {
  #[derive(Config)]
  #[config(env_prefix = "{prefix}PREFIX_", rename_all = "snake_case")]
  pub struct Conf {
    #[config(rename = "renamed", env = "{prefix}ASD")]
    pub field: String,
    #[config(parse_env = parse_enum)]
    pub kind: Asd,
    #[config(nested)]
    pub nested: Nested,

    pub asd: Option<String>,
  }

  #[derive(Config)]
  #[config(env_prefix = "NESTED_")]
  pub struct Nested {
    pub n: usize
  }
}
