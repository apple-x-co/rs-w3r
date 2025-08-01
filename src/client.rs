use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderName, CONTENT_TYPE};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// アプリケーション情報
const USER_AGENT: &str = "rs-w3r/1.0";

// デフォルト値
pub(crate) const DEFAULT_RETRY_COUNT: u32 = 0;
pub(crate) const DEFAULT_RETRY_DELAY: f64 = 1.0;
pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub(crate) const DEFAULT_METHOD: &str = "GET";

// リトライ関連
const RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;

// ファイルサイズ計算
const BYTES_PER_KB: f64 = 1024.0;

// HTTPステータスコード
const SERVER_ERROR_START: u16 = 500;
const SERVER_ERROR_END: u16 = 599;
const TOO_MANY_REQUESTS: u16 = 429;
const REQUEST_TIMEOUT: u16 = 408;

// Content-Type
const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";
const CONTENT_TYPE_JSON: &str = "application/json; charset=utf-8";

// 認証プレースホルダー
const BASIC_AUTH_PLACEHOLDER: &str = "Basic <credentials>";

// JSONフィルタ関連
const JSON_PATH_ROOT: &str = ".";

// エラーメッセージ
const ERROR_REQUEST_CLONE: &str = "Failed to clone request for retry";
const ERROR_PRESET_NOT_FOUND: &str = "Preset '{}' not found in config file";
const ERROR_NO_PRESETS: &str = "No presets found in config file";
const ERROR_UNKNOWN_METHOD: &str = "Unknown HTTP method";

// 表示メッセージ
const TIMING_HEADER: &str = "--- Timing Information ---";
const RETRY_ATTEMPT_PREFIX: &str = "--- Retry Attempt {} ---";
const RESPONSE_RECEIVED_MSG: &str = "Response received: {}";
const BODY_READ_TIME_MSG: &str = "Body read time: {}";
const TOTAL_TIME_MSG: &str = "Total time: {}";
const RESPONSE_SIZE_MSG: &str = "Response size: {1} bytes ({2} KB)";
const THROUGHPUT_MSG: &str = "Throughput: {} KB/s";
const HTTP_RETRY_MSG: &str = "HTTP {} - retrying after delay...";
const REQUEST_ERROR_RETRY_MSG: &str = "Request error: {} - retrying after delay...";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub host: String,
    pub port: String,
    pub user: Option<String>,
    pub pass: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub basic_auth: Option<BasicAuthConfig>,
    pub cookies: Option<Vec<String>>,
    pub dry_run: bool,
    pub form_data: Option<String>,
    pub form: Option<Vec<String>>,
    pub headers: Option<Vec<String>>,
    pub json: Option<String>,
    pub json_filter: Option<String>,
    pub method: String,
    pub output: Option<String>,
    pub pretty_json: bool,
    pub proxy: Option<ProxyConfig>,
    pub retry: u32,
    pub retry_delay: f64,
    pub silent: bool,
    pub timeout: u64,
    pub timing: bool,
    pub url: String,
    pub verbose: bool,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    preset: HashMap<String, ConfigPreset>,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigPreset {
    url: Option<String>,
    method: Option<String>,
    headers: Option<Vec<String>>,
    timeout: Option<u64>,
    pretty_json: Option<bool>,
    timing: Option<bool>,
    verbose: Option<bool>,
    silent: Option<bool>,
    retry: Option<u32>,
    retry_delay: Option<f64>,
    json: Option<String>,
    json_filter: Option<String>,
    form_data: Option<String>,
    form: Option<Vec<String>>,
    cookies: Option<Vec<String>>,
    output: Option<String>,
    dry_run: Option<bool>,
    basic_auth: Option<BasicAuthConfig>,
    proxy: Option<ProxyConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            basic_auth: None,
            cookies: None,
            dry_run: false,
            form_data: None,
            form: None,
            headers: None,
            json: None,
            json_filter: None,
            method: DEFAULT_METHOD.to_string(),
            output: None,
            pretty_json: false,
            proxy: None,
            retry: DEFAULT_RETRY_COUNT,
            retry_delay: DEFAULT_RETRY_DELAY,
            silent: false,
            timeout: DEFAULT_TIMEOUT_SECS,
            timing: false,
            url: String::new(),
            verbose: false,
        }
    }
}

#[derive(Debug)]
struct ResponseInfo {
    status: reqwest::StatusCode,
    version: reqwest::Version,
    headers: reqwest::header::HeaderMap,
}

#[derive(Debug)]
struct TimingInfo {
    response_time: Duration,
    body_read_time: Duration,
    total_time: Duration,
}

#[derive(Debug)]
struct RequestContext {
    client: Client,
    request: reqwest::blocking::Request,
    default_headers: reqwest::header::HeaderMap,
}

impl ResponseInfo {
    pub fn new(
        status: reqwest::StatusCode,
        version: reqwest::Version,
        headers: reqwest::header::HeaderMap,
    ) -> Self {
        Self {
            status,
            version,
            headers,
        }
    }

    pub fn status(&self) -> reqwest::StatusCode {
        self.status
    }

    pub fn version(&self) -> reqwest::Version {
        self.version
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }
}

impl TimingInfo {
    pub fn new(response_time: Duration, body_read_time: Duration, total_time: Duration) -> Self {
        Self {
            response_time,
            body_read_time,
            total_time,
        }
    }
}

/// 設定ファイルを読み込んでConfigを作成
pub fn load_config_file(
    config_path: &str,
    preset_name: Option<&str>,
) -> Result<Config, Box<dyn Error>> {
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config_file: ConfigFile = toml::from_str(&contents)?;

    let preset = get_preset(&config_file, preset_name)?;
    Ok(create_config_from_preset(preset))
}

/// プリセットを取得
fn get_preset<'a>(
    config_file: &'a ConfigFile,
    preset_name: Option<&'a str>,
) -> Result<&'a ConfigPreset, Box<dyn Error>> {
    match preset_name {
        Some(name) => config_file
            .preset
            .get(name)
            .ok_or_else(|| format!("{}", ERROR_PRESET_NOT_FOUND.replace("{}", name)).into()),
        None => config_file
            .preset
            .values()
            .next()
            .ok_or(ERROR_NO_PRESETS.into()),
    }
}

/// プリセットからConfigを作成
fn create_config_from_preset(preset: &ConfigPreset) -> Config {
    Config {
        basic_auth: preset.basic_auth.clone(),
        cookies: preset.cookies.clone(),
        dry_run: preset.dry_run.unwrap_or(false),
        form_data: preset.form_data.clone(),
        form: preset.form.clone(),
        headers: preset.headers.clone(),
        json: preset.json.clone(),
        json_filter: preset.json_filter.clone(),
        method: preset
            .method
            .clone()
            .unwrap_or_else(|| DEFAULT_METHOD.to_string()),
        output: preset.output.clone(),
        pretty_json: preset.pretty_json.unwrap_or(false),
        proxy: preset.proxy.clone(),
        retry: preset.retry.unwrap_or(DEFAULT_RETRY_COUNT),
        retry_delay: preset.retry_delay.unwrap_or(DEFAULT_RETRY_DELAY),
        silent: preset.silent.unwrap_or(false),
        timeout: preset.timeout.unwrap_or(DEFAULT_TIMEOUT_SECS),
        timing: preset.timing.unwrap_or(false),
        url: preset.url.clone().unwrap_or_default(),
        verbose: preset.verbose.unwrap_or(false),
    }
}

/// HTTPリクエストを実行
pub fn execute_request(config: Config) -> Result<(), Box<dyn Error>> {
    let request_context = create_request_context(&config)?;

    display_request_info(&config, &request_context);

    if config.dry_run {
        return Ok(());
    }

    let (response_info, response_body, timing_info) = execute_request_with_retry(
        &request_context.client,
        request_context.request,
        &config,
    )?;

    handle_response(response_info, response_body, timing_info, &config)?;

    Ok(())
}

/// HTTPクライアントとリクエストを作成
fn create_request_context(config: &Config) -> Result<RequestContext, Box<dyn Error>> {
    let (client, default_headers) = create_http_client(config)?;
    let request = build_request(&client, config)?;

    Ok(RequestContext {
        client,
        request,
        default_headers,
    })
}

/// HTTPクライアントを作成
fn create_http_client(
    config: &Config,
) -> Result<(Client, reqwest::header::HeaderMap), Box<dyn Error>> {
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .user_agent(USER_AGENT);

    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert(reqwest::header::USER_AGENT, USER_AGENT.parse().unwrap());

    client_builder = setup_proxy(client_builder, config)?;
    client_builder = setup_cookies(client_builder, config)?;
    let (client_builder, headers) = setup_default_headers(client_builder, config, default_headers)?;

    Ok((client_builder.build()?, headers))
}

/// プロキシ設定を適用
fn setup_proxy(
    mut client_builder: reqwest::blocking::ClientBuilder,
    config: &Config,
) -> Result<reqwest::blocking::ClientBuilder, Box<dyn Error>> {
    if let Some(proxy_config) = &config.proxy {
        let proxy_url = format!("https://{}:{}", proxy_config.host, proxy_config.port);
        let mut http_proxy = reqwest::Proxy::http(proxy_url)?;

        if let (Some(proxy_user), Some(proxy_pass)) = (&proxy_config.user, &proxy_config.pass) {
            http_proxy = http_proxy.basic_auth(proxy_user, proxy_pass);
        }

        client_builder = client_builder.proxy(http_proxy);
    }

    Ok(client_builder)
}

/// クッキー設定を適用
fn setup_cookies(
    mut client_builder: reqwest::blocking::ClientBuilder,
    config: &Config,
) -> Result<reqwest::blocking::ClientBuilder, Box<dyn Error>> {
    if let Some(cookie_list) = &config.cookies {
        let cookie_jar = Jar::default();
        let parsed_url = &Url::parse(&config.url)?;

        for cookie_str in cookie_list {
            cookie_jar.add_cookie_str(cookie_str, parsed_url);
        }

        client_builder = client_builder.cookie_provider(Arc::new(cookie_jar));
    }

    Ok(client_builder)
}

/// デフォルトヘッダーを設定
fn setup_default_headers(
    mut client_builder: reqwest::blocking::ClientBuilder,
    config: &Config,
    mut default_headers: reqwest::header::HeaderMap,
) -> Result<
    (
        reqwest::blocking::ClientBuilder,
        reqwest::header::HeaderMap,
    ),
    Box<dyn Error>,
> {
    if let Some(header_list) = &config.headers {
        let mut header_map = reqwest::header::HeaderMap::new();

        for header_entry in header_list {
            if let Some((key, value)) = header_entry.split_once(':') {
                if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                    let header_value = value.trim();
                    if let Ok(parsed_value) = header_value.parse() {
                        header_map.insert(header_name.clone(), parsed_value);
                        default_headers.insert(header_name, header_value.parse().unwrap());
                    }
                }
            }
        }

        if !header_map.is_empty() {
            client_builder = client_builder.default_headers(header_map);
        }
    }

    Ok((client_builder, default_headers))
}

/// HTTPリクエストを構築
fn build_request(client: &Client, config: &Config) -> Result<reqwest::blocking::Request, Box<dyn Error>> {
    let method = Method::from_bytes(config.method.as_bytes())?;
    let mut request_builder = create_request_builder(client, &method, &config.url)?;

    request_builder = apply_authentication(request_builder, config);
    request_builder = apply_request_body(request_builder, config)?;

    Ok(request_builder.build()?)
}

/// リクエストビルダーを作成
fn create_request_builder(
    client: &Client,
    method: &Method,
    url: &str,
) -> Result<reqwest::blocking::RequestBuilder, Box<dyn Error>> {
    let request_builder = match *method {
        Method::GET => client.get(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::DELETE => client.delete(url),
        Method::HEAD => client.head(url),
        Method::PATCH => client.patch(url),
        _ => return Err(ERROR_UNKNOWN_METHOD.into()),
    };

    Ok(request_builder)
}

/// 認証設定を適用
fn apply_authentication(
    mut request_builder: reqwest::blocking::RequestBuilder,
    config: &Config,
) -> reqwest::blocking::RequestBuilder {
    if let Some(auth_config) = &config.basic_auth {
        request_builder = request_builder.basic_auth(&auth_config.user, Some(&auth_config.pass));
    }

    request_builder
}

/// リクエストボディを適用
fn apply_request_body(
    mut request_builder: reqwest::blocking::RequestBuilder,
    config: &Config,
) -> Result<reqwest::blocking::RequestBuilder, Box<dyn Error>> {
    if let Some(form_data_body) = &config.form_data {
        request_builder = request_builder
            .header(CONTENT_TYPE, CONTENT_TYPE_FORM)
            .body(form_data_body.clone());
    } else if let Some(form_params) = &config.form {
        let param_pairs = parse_form_params(form_params);
        request_builder = request_builder
            .header(CONTENT_TYPE, CONTENT_TYPE_FORM)
            .form(&param_pairs);
    } else if let Some(json_data) = &config.json {
        request_builder = request_builder
            .header(CONTENT_TYPE, CONTENT_TYPE_JSON)
            .json(json_data);
    }

    Ok(request_builder)
}

/// フォームパラメータを解析
fn parse_form_params(form_params: &[String]) -> Vec<(String, String)> {
    form_params
        .iter()
        .filter_map(|param| {
            param
                .split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
        })
        .collect()
}

/// リクエスト情報を表示
fn display_request_info(config: &Config, context: &RequestContext) {
    if !config.verbose {
        return;
    }

    println!("> {} {}", config.method, config.url);

    for (name, value) in &context.default_headers {
        let display_value = if name == reqwest::header::AUTHORIZATION {
            BASIC_AUTH_PLACEHOLDER
        } else {
            value.to_str().unwrap_or("<binary>")
        };
        println!("> {}: {}", name, display_value);
    }

    for (name, value) in context.request.headers() {
        if !context.default_headers.contains_key(name) {
            println!("> {}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
    }

    println!();
}

/// リトライ機能付きでリクエストを実行
fn execute_request_with_retry(
    client: &Client,
    request: reqwest::blocking::Request,
    config: &Config,
) -> Result<(ResponseInfo, String, TimingInfo), Box<dyn Error>> {
    let mut current_attempt: u32 = 0;
    let max_attempts: u32 = config.retry + 1;
    let overall_start = Instant::now();

    loop {
        current_attempt += 1;

        let retry_request = request.try_clone().ok_or(ERROR_REQUEST_CLONE)?;

        if config.verbose && current_attempt > 1 {
            println!(
                "{}",
                RETRY_ATTEMPT_PREFIX.replace("{}", &current_attempt.saturating_sub(1).to_string())
            );
        }

        let request_start = Instant::now();

        match client.execute(retry_request) {
            Ok(response) => {
                let status = response.status();

                if should_retry_for_status(status.as_u16()) && current_attempt < max_attempts {
                    handle_retry_delay(config, current_attempt, status.as_u16());
                    continue;
                }

                return handle_successful_response(response, request_start, overall_start);
            }
            Err(e) => {
                if current_attempt < max_attempts {
                    handle_request_error_retry(config, current_attempt, &e);
                    continue;
                }
                return Err(e.into());
            }
        }
    }
}

/// 成功したレスポンスを処理
fn handle_successful_response(
    response: reqwest::blocking::Response,
    request_start: Instant,
    overall_start: Instant,
) -> Result<(ResponseInfo, String, TimingInfo), Box<dyn Error>> {
    let response_received_time = request_start.elapsed();

    let status_code = response.status();
    let version = response.version();
    let headers = response.headers().clone();

    let body_start = Instant::now();
    let response_body = response.text()?;
    let body_read_time = body_start.elapsed();

    let total_time = overall_start.elapsed();

    let response_info = ResponseInfo::new(status_code, version, headers);
    let timing_info = TimingInfo::new(response_received_time, body_read_time, total_time);

    Ok((response_info, response_body, timing_info))
}

/// リトライ遅延を処理
fn handle_retry_delay(config: &Config, current_attempt: u32, status_code: u16) {
    if config.verbose {
        println!(
            "{}",
            HTTP_RETRY_MSG.replace("{}", &status_code.to_string())
        );
    }

    let backoff_delay = config.retry_delay
        * RETRY_BACKOFF_MULTIPLIER.powi(current_attempt.saturating_sub(1) as i32);
    thread::sleep(Duration::from_secs_f64(backoff_delay));
}

/// リクエストエラーのリトライを処理
fn handle_request_error_retry(config: &Config, current_attempt: u32, error: &reqwest::Error) {
    if config.verbose {
        println!(
            "{}",
            REQUEST_ERROR_RETRY_MSG.replace("{}", &error.to_string())
        );
    }

    let backoff_delay = config.retry_delay
        * RETRY_BACKOFF_MULTIPLIER.powi(current_attempt.saturating_sub(1) as i32);
    thread::sleep(Duration::from_secs_f64(backoff_delay));
}

/// ステータスコードによるリトライ判定
fn should_retry_for_status(status_code: u16) -> bool {
    matches!(
        status_code,
        SERVER_ERROR_START..=SERVER_ERROR_END | TOO_MANY_REQUESTS | REQUEST_TIMEOUT
    )
}

/// レスポンスを処理
fn handle_response(
    response_info: ResponseInfo,
    response_body: String,
    timing_info: TimingInfo,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    display_response_info(&response_info, config);
    display_timing_info(&timing_info, response_body.len(), config);

    let processed_response = format_response_body(&response_body, config)?;
    output_response(&processed_response, config)?;

    Ok(())
}

/// レスポンス情報を表示
fn display_response_info(response_info: &ResponseInfo, config: &Config) {
    if !config.verbose {
        return;
    }

    println!(
        "< {:?} {} {}",
        response_info.version(),
        response_info.status().as_u16(),
        response_info
            .status()
            .canonical_reason()
            .unwrap_or("")
    );

    for (name, value) in response_info.headers() {
        println!("< {}: {}", name, value.to_str().unwrap_or("<binary>"));
    }

    println!();
}

/// タイミング情報を表示
fn display_timing_info(timing_info: &TimingInfo, response_size: usize, config: &Config) {
    if !config.timing {
        return;
    }

    println!("{}", TIMING_HEADER);
    println!(
        "{}",
        RESPONSE_RECEIVED_MSG.replace("{}", &format!("{:?}", timing_info.response_time))
    );
    println!(
        "{}",
        BODY_READ_TIME_MSG.replace("{}", &format!("{:?}", timing_info.body_read_time))
    );
    println!(
        "{}",
        TOTAL_TIME_MSG.replace("{}", &format!("{:?}", timing_info.total_time))
    );
    println!(
        "{}",
        RESPONSE_SIZE_MSG
            .replace("{1}", &response_size.to_string())
            .replace("{2}", &format!("{:.2}", response_size as f64 / BYTES_PER_KB))
    );

    if response_size > 0 && timing_info.total_time.as_secs_f64() > 0.0 {
        let throughput =
            response_size as f64 / timing_info.total_time.as_secs_f64() / BYTES_PER_KB;
        println!(
            "{}",
            THROUGHPUT_MSG.replace("{}", &format!("{:.2}", throughput))
        );
    }

    println!();
}

/// レスポンスボディをフォーマット
fn format_response_body(body: &str, config: &Config) -> Result<String, Box<dyn Error>> {
    let json_value = match from_str::<Value>(body) {
        Ok(value) => value,
        Err(_) => return Ok(body.to_string()),
    };

    let mut result = json_value;

    if let Some(filter_path) = &config.json_filter {
        result = extract_json_path(result, filter_path)?;
    }

    let formatted = if config.pretty_json {
        serde_json::to_string_pretty(&result)?
    } else {
        serde_json::to_string(&result)?
    };

    Ok(formatted)
}

/// JSONパスを抽出
fn extract_json_path(mut json: Value, path: &str) -> Result<Value, Box<dyn Error>> {
    let path = path.trim();

    if path == JSON_PATH_ROOT {
        return Ok(json);
    }

    let path = path.strip_prefix('.').unwrap_or(path);
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();

    for part in parts {
        json = process_json_path_part(json, part)?;
    }

    Ok(json)
}

/// JSONパスの一部を処理
fn process_json_path_part(json: Value, part: &str) -> Result<Value, Box<dyn Error>> {
    if let Some(bracket_pos) = part.find('[') {
        let field_name = &part[..bracket_pos];
        let index_part = &part[bracket_pos + 1..];

        if let Some(close_bracket) = index_part.find(']') {
            let index_str = &index_part[..close_bracket];
            if let Ok(index) = index_str.parse::<usize>() {
                let mut result = json;

                if !field_name.is_empty() {
                    result = result.get(field_name).cloned().unwrap_or(Value::Null);
                }

                return Ok(result.get(index).cloned().unwrap_or(Value::Null));
            }
        }
    }

    Ok(json.get(part).cloned().unwrap_or(Value::Null))
}

/// レスポンスを出力
fn output_response(processed_response: &str, config: &Config) -> Result<(), Box<dyn Error>> {
    match &config.output {
        Some(output_file) => save_response_to_file(output_file, processed_response.as_bytes()),
        None if !config.silent => {
            println!("{}", processed_response);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// レスポンスをファイルに保存
fn save_response_to_file(file_path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(file_path)?;
    file.write_all(data)?;
    Ok(())
}