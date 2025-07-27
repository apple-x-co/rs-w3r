mod client;

use crate::client::{execute_request, BasicAuthConfig, Config, ProxyConfig};
use clap::{arg, Parser};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "GET")]
    method: String,

    #[arg(short, long, default_value = "30")]
    timeout: u64,

    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let proxy_host = std::env::var("PROXY_HOST").unwrap_or("".to_string());
    let proxy_port = std::env::var("PROXY_PORT").unwrap_or("".to_string());
    let proxy_user = std::env::var("PROXY_USER").unwrap_or("".to_string());
    let proxy_password = std::env::var("PROXY_PASSWORD").unwrap_or("".to_string());

    let basic_user = std::env::var("BASIC_USER").unwrap_or("".to_string());
    let basic_password = std::env::var("BASIC_PASSWORD").unwrap_or("".to_string());

    let args = Args::parse();

    let config = Config {
        basic_auth: if basic_user != "" && basic_password != "" {
            Some(BasicAuthConfig {
                user: basic_user,
                pass: basic_password,
            })
        } else {
            None
        },
        method: args.method,
        proxy: if proxy_host != "" && proxy_port != "" {
            if proxy_user != "" && proxy_password != "" {
                Some(ProxyConfig {
                    host: proxy_host,
                    port: proxy_port,
                    user: Some(proxy_user),
                    pass: Some(proxy_password),
                })
            } else {
                Some(ProxyConfig {
                    host: proxy_host,
                    port: proxy_port,
                    user: None,
                    pass: None,
                })
            }
        } else {
            None
        },
        timeout: args.timeout,
        url: args.url,
        verbose: args.verbose,
    };

    execute_request(config)?;

    Ok(())
}
