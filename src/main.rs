mod client;

use crate::client::{execute_request, BasicAuthConfig, Config, ProxyConfig};
use clap::{arg, Parser};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "BASIC_USER")]
    basic_user: Option<String>,

    #[arg(long, env = "BASIC_PASS")]
    basic_pass: Option<String>,

    #[arg(short, long)]
    form_data: Option<String>,

    #[arg(short, long)]
    json: Option<String>,

    #[arg(short, long, default_value = "GET")]
    method: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long, env = "PROXY_HOST")]
    proxy_host: Option<String>,

    #[arg(long, env = "PROXY_PORT")]
    proxy_port: Option<String>,

    #[arg(long, env = "PROXY_USER")]
    proxy_user: Option<String>,

    #[arg(long, env = "PROXY_PASS")]
    proxy_pass: Option<String>,

    #[arg(short, long, default_value_t = false)]
    silent: bool,

    #[arg(short, long, default_value = "30")]
    timeout: u64,

    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config = Config {
        basic_auth: if let Some(basic_user) = args.basic_user {
            if let Some(basic_pass) = args.basic_pass {
                Some(BasicAuthConfig {
                    user: basic_user,
                    pass: basic_pass,
                })
            } else {
                None
            }
        } else {
            None
        },
        form_data: args.form_data,
        json: args.json,
        method: args.method,
        output: args.output,
        proxy: if let Some(proxy_host) = args.proxy_host {
            if let Some(proxy_port) = args.proxy_port {
                if let Some(proxy_user) = args.proxy_user {
                    if let Some(proxy_pass) = args.proxy_pass {
                        Some(ProxyConfig {
                            host: proxy_host,
                            port: proxy_port,
                            user: Some(proxy_user),
                            pass: Some(proxy_pass),
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
                    Some(ProxyConfig {
                        host: proxy_host,
                        port: proxy_port,
                        user: None,
                        pass: None,
                    })
                }
            } else {
                None
            }
        } else {
            None
        },
        silent: args.silent,
        timeout: args.timeout,
        url: args.url,
        verbose: args.verbose,
    };

    execute_request(config)?;

    Ok(())
}
