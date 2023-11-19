use std::net::IpAddr;

use clap::{ArgAction, Parser};
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Options {
    /// Increase logs verbosity (Error (default), Warn, Info, Debug, Trace).
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub log_level: u8,
    /// HTTP listening address.
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    pub address: IpAddr,
    /// HTTP listening port.
    #[arg(short = 'p', long, default_value = "8080")]
    pub port: u16,
}

impl Options {
    pub fn log_level(&self) -> LevelFilter {
        match self.log_level {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            4.. => LevelFilter::Trace,
        }
    }
}
