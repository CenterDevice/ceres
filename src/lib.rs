extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_sts;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate spectral;
extern crate toml;
extern crate tabwriter;

pub mod config;
pub mod modules;
pub mod output;
pub mod provider;
