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
    let mut config = load_config_if_specified(&args)?;

    // コマンドライン引数で設定ファイルの値をオーバーライド
    apply_args_to_config(&mut config, args);

    // URLが設定されていない場合はエラー
    validate_config(&config)?;

    // HTTP リクエスト実行
    execute_request(config)?;

    Ok(())
}

/// 設定ファイルが指定されている場合に読み込む
fn load_config_if_specified(args: &Args) -> Result<Config, Box<dyn Error>> {
    match &args.config {
        Some(config_path) => load_config_file(config_path, args.preset.as_deref()),
        None => Ok(Config::default()),
    }
}

/// 設定の有効性を検証
fn validate_config(config: &Config) -> Result<(), Box<dyn Error>> {
    if config.url.is_empty() {
        return Err(ERROR_MISSING_URL.into());
    }
    Ok(())
}

/// コマンドライン引数を設定に反映
fn apply_args_to_config(config: &mut Config, args: Args) {
    apply_auth_config(config, &args);
    apply_data_config(config, &args);
    apply_request_config(config, &args);
    apply_proxy_config(config, &args);
    apply_output_config(config, &args);
    apply_retry_config(config, &args);
    apply_flags(config, &args);
}

/// 認証設定の適用
fn apply_auth_config(config: &mut Config, args: &Args) {
    if let (Some(basic_user), Some(basic_pass)) = (&args.basic_user, &args.basic_pass) {
        config.basic_auth = Some(BasicAuthConfig {
            user: basic_user.clone(),
            pass: basic_pass.clone(),
        });
    }
}

/// データ送信設定の適用
fn apply_data_config(config: &mut Config, args: &Args) {
    if let Some(form_data) = &args.form_data {
        config.form_data = Some(form_data.clone());
    }

    if let Some(form) = &args.form {
        config.form = Some(form.clone());
    }

    if let Some(json) = &args.json {
        config.json = Some(json.clone());
    }

    if let Some(json_filter) = &args.json_filter {
        config.json_filter = Some(json_filter.clone());
    }
}

/// リクエスト設定の適用
fn apply_request_config(config: &mut Config, args: &Args) {
    if args.method != DEFAULT_METHOD {
        config.method = args.method.clone();
    }

    if let Some(headers) = &args.headers {
        config.headers = Some(headers.clone());
    }

    if let Some(cookies) = &args.cookies {
        config.cookies = Some(cookies.clone());
    }

    if let Some(url) = &args.url {
        config.url = url.clone();
    }

    if args.timeout != DEFAULT_TIMEOUT_SECS {
        config.timeout = args.timeout;
    }
}

/// プロキシ設定の適用
fn apply_proxy_config(config: &mut Config, args: &Args) {
    if let (Some(proxy_host), Some(proxy_port)) = (&args.proxy_host, &args.proxy_port) {
        config.proxy = Some(ProxyConfig {
            host: proxy_host.clone(),
            port: proxy_port.clone(),
            user: args.proxy_user.clone(),
            pass: args.proxy_pass.clone(),
        });
    }
}

/// 出力設定の適用
fn apply_output_config(config: &mut Config, args: &Args) {
    if let Some(output) = &args.output {
        config.output = Some(output.clone());
    }
}

/// リトライ設定の適用
fn apply_retry_config(config: &mut Config, args: &Args) {
    if args.retry != DEFAULT_RETRY_COUNT {
        config.retry = args.retry;
    }

    if args.retry_delay != DEFAULT_RETRY_DELAY {
        config.retry_delay = args.retry_delay;
    }
}

/// フラグの適用
fn apply_flags(config: &mut Config, args: &Args) {
    if args.dry_run {
        config.dry_run = true;
    }

    if args.pretty_json {
        config.pretty_json = true;
    }

    if args.silent {
        config.silent = true;
    }

    if args.timing {
        config.timing = true;
    }

    if args.verbose {
        config.verbose = true;
    }
}