# Configuration Loader for Rust, aka #[derive(Config)]

[![build](https://github.com/ramiroaisen/metre/actions/workflows/build.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/build.yml)
[![integration-tests](https://github.com/ramiroaisen/metre/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/MasterworksIO/metre/actions/workflows/integration-tests.yml)
[![unit-tests](https://github.com/ramiroaisen/metre/actions/workflows/unit-tests.yml/badge.svg)](https://github.com/ramiroaisen/metre/actions/workflows/unit-tests.yml)

## What is it?

**metre** is a configuration loader for Rust that allows you to load configuration from a variety of formats such as **toml**, **json**, **jsonc** and **yaml** 
yaml,

It also supports a variety of sources such as **environment variables**, **files**, and **urls**.   

## Why?

**metre** focus is to provide a declarative and type-safe way to load configurations in Rust. 

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