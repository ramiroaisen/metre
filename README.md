# **metre**: The rust configuration loader
### AKA:  `#[derive(Config)]`

[![crates.io](https://img.shields.io/crates/v/metre.svg)](https://crates.io/crates/metre)
[![build](https://github.com/ramiroaisen/metre/actions/workflows/build.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/build.yml)
[![unit-tests](https://github.com/ramiroaisen/metre/actions/workflows/unit-tests.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/unit-tests.yml)
[![integration-tests](https://github.com/ramiroaisen/metre/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/integration-tests.yml)
[![docs](https://github.com/ramiroaisen/metre/actions/workflows/docs.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/docs.yml)



## What is it?
**metre** is a configuration loader for Rust that allows you to load configurations from a variety of formats such as **toml**, **json**, **jsonc** and **yaml**

It also supports a variety of sources such as **program defaults**, **env variables**, **files**, and **urls**.   

## Usage
```rust
use metre::{Config, ConfigLoader, Format};

#[derive(Config)]
struct MyConfig {
  #[config(default = 8000)]
  port: u16,
  foo: String
}

let mut loader = ConfigLoader::<MyConfig>::new();

loader.defaults()?;
loader.file("config.json", Format::Json)?;
loader.env()?;

// config have the type MyConfig here
let config = loader.finish()?;  
```

## Focus

**metre** focus is to provide a **declarative** and **type-safe** way to load configurations in Rust.


## How?

**metre** works by defining a struct that implements the `Config` trait, usually via the `#[derive(Config)]` macro. 

Under the hood metre creates deep partial version of the struct to accumulate the configuration from different sources.

Once all the configuration is accumulated, you can access the final configuration as the defined struct. If the sum of all sources does not comply with the required properties, metre will return an error.


## Install
```sh
cargo add metre
```

## Docs
[https://docs.rs/metre](https://docs.rs/metre)