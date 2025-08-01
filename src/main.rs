mod client;

use crate::client::{execute_request, load_config_file, BasicAuthConfig, Config, ProxyConfig};
use clap::{arg, Parser};
use std::error::Error;

use crate::client::{DEFAULT_METHOD, DEFAULT_TIMEOUT_SECS, DEFAULT_RETRY_COUNT, DEFAULT_RETRY_DELAY};

// エラーメッセージ定数
const ERROR_MISSING_URL: &str = "URL is required. Use -u/--url option or specify in config file.";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "BASIC_USER")]
    basic_user: Option<String>,

    #[arg(long, env = "BASIC_PASS")]
    basic_pass: Option<String>,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(long, action = clap::ArgAction::Append)]
    cookies: Option<Vec<String>>,

    #[arg(long, default_value_t = false)]
    dry_run: bool,

    #[arg(short, long)]
    form_data: Option<String>,

    #[arg(long, action = clap::ArgAction::Append)]
    form: Option<Vec<String>>,

    #[arg(long, action = clap::ArgAction::Append)]
    headers: Option<Vec<String>>,

    #[arg(short, long)]
    json: Option<String>,

    #[arg(long)]
    json_filter: Option<String>,

    #[arg(short, long, default_value = DEFAULT_METHOD)]
    method: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    preset: Option<String>,

    #[arg(long, default_value_t = false)]
    pretty_json: bool,

    #[arg(long, env = "PROXY_HOST")]
    proxy_host: Option<String>,

    #[arg(long, env = "PROXY_PORT")]
    proxy_port: Option<String>,

    #[arg(long, env = "PROXY_USER")]
    proxy_user: Option<String>,

    #[arg(long, env = "PROXY_PASS")]
    proxy_pass: Option<String>,

    #[arg(long, default_value_t = DEFAULT_RETRY_COUNT)]
    retry: u32,

    #[arg(long, default_value_t = DEFAULT_RETRY_DELAY)]
    retry_delay: f64,

    #[arg(short, long, default_value_t = false)]
    silent: bool,

    #[arg(short, long, default_value_t = DEFAULT_TIMEOUT_SECS)]
    timeout: u64,

    #[arg(long, default_value_t = false)]
    timing: bool,

    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // 設定ファイルの読み込み
    let mut config = if let Some(config_path) = &args.config {
        load_config_file(config_path, args.preset.as_deref())?
    } else {
        Config::default()
    };

    // コマンドライン引数で設定ファイルの値をオーバーライド
    if let Some(basic_user) = args.basic_user {
        if let Some(basic_pass) = args.basic_pass {
            config.basic_auth = Some(BasicAuthConfig {
                user: basic_user,
                pass: basic_pass,
            });
        }
    }

    if let Some(cookies) = args.cookies {
        config.cookies = Some(cookies);
    }

    if args.dry_run {
        config.dry_run = args.dry_run;
    }

    if let Some(form_data) = args.form_data {
        config.form_data = Some(form_data);
    }

    if let Some(form) = args.form {
        config.form = Some(form);
    }

    if let Some(headers) = args.headers {
        config.headers = Some(headers);
    }

    if let Some(json) = args.json {
        config.json = Some(json);
    }

    if let Some(json_filter) = args.json_filter {
        config.json_filter = Some(json_filter);
    }

    if args.method != DEFAULT_METHOD {
        config.method = args.method;
    }

    if let Some(output) = args.output {
        config.output = Some(output);
    }

    if args.pretty_json {
        config.pretty_json = args.pretty_json;
    }

    // プロキシ設定
    if let Some(proxy_host) = args.proxy_host {
        if let Some(proxy_port) = args.proxy_port {
            let proxy_config = ProxyConfig {
                host: proxy_host,
                port: proxy_port,
                user: args.proxy_user,
                pass: args.proxy_pass,
            };
            config.proxy = Some(proxy_config);
        }
    }

    if args.retry > 0 {
        config.retry = args.retry;
    }

    if args.retry_delay != 1.0 {
        config.retry_delay = args.retry_delay;
    }

    if args.silent {
        config.silent = args.silent;
    }

    if args.timeout != 30 {
        config.timeout = args.timeout;
    }

    if args.timing {
        config.timing = args.timing;
    }

    if let Some(url) = args.url {
        config.url = url;
    }

    if args.verbose {
        config.verbose = args.verbose;
    }

    // URLが設定されていない場合はエラー
    if config.url.is_empty() {
        return Err(ERROR_MISSING_URL.into());
    }

    execute_request(config)?;

    Ok(())
}