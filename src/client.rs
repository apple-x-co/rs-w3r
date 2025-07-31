use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderName, CONTENT_TYPE};
use reqwest::{Method, Url};
use serde_json::{Value, from_str};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct BasicAuthConfig {
    pub user: String,
    pub pass: String,
}

pub struct ProxyConfig {
    pub host: String,
    pub port: String,
    pub user: Option<String>,
    pub pass: Option<String>,
}

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

struct ResponseInfo {
    status: reqwest::StatusCode,
    version: reqwest::Version,
    headers: reqwest::header::HeaderMap,
}

struct TimingInfo {
    response_time: Duration,
    body_read_time: Duration,
    total_time: Duration,
}

impl ResponseInfo {
    fn status(&self) -> reqwest::StatusCode { self.status }
    fn version(&self) -> reqwest::Version { self.version }
    fn headers(&self) -> &reqwest::header::HeaderMap { &self.headers }
}

const USER_AGENT: &str = "rs-w3r/1.0";

pub fn execute_request(config: Config) -> Result<(), Box<dyn Error>> {
    // デフォルトヘッダーの追跡用
    let mut default_headers = reqwest::header::HeaderMap::new();

    // カスタムクライアントの作成
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .user_agent(USER_AGENT);

    // デフォルトヘッダー追跡
    default_headers.insert(reqwest::header::USER_AGENT, USER_AGENT.parse().unwrap());

    if let Some(ref proxy) = config.proxy {
        let proxy_url = format!("https://{}:{}", proxy.host, proxy.port);
        let mut http_proxy = reqwest::Proxy::http(proxy_url)?;

        if let Some(proxy_user) = &proxy.user {
            if let Some(proxy_pass) = &proxy.pass {
                http_proxy = http_proxy.basic_auth(proxy_user.as_str(), proxy_pass.as_str());
            }
        }

        client_builder = client_builder.proxy(http_proxy);
    }

    if let Some(ref cookies) = config.cookies {
        let cookie_jar = Jar::default();
        let url = &Url::parse(config.url.as_str()).unwrap();

        for cookie in cookies {
            cookie_jar.add_cookie_str(cookie.as_str(), url);
        }

        client_builder = client_builder.cookie_provider(Arc::new(cookie_jar));
    }

    if let Some(ref headers) = config.headers {
        if !headers.is_empty() {
            let mut header_map = reqwest::header::HeaderMap::new();

            for header in headers {
                if let Some((key, value)) = header.split_once(':') {
                    if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
                        let value_string = value.trim().to_string();
                        if let Ok(header_value) = value_string.parse() {
                            header_map.insert(header_name.clone(), header_value);

                            // デフォルトヘッダー追跡
                            default_headers.insert(header_name, value_string.parse().unwrap());
                        }
                    }
                }
            }

            client_builder = client_builder.default_headers(header_map);
        }
    }

    let client = client_builder.build()?;

    // リクエストの作成
    let method = Method::from_bytes(config.method.as_bytes())?;
    let mut request_builder = match method {
        Method::GET => client.get(config.url.as_str()),
        Method::POST => client.post(config.url.as_str()),
        Method::PUT => client.put(config.url.as_str()),
        Method::DELETE => client.delete(config.url.as_str()),
        Method::HEAD => client.head(config.url.as_str()),
        Method::PATCH => client.patch(config.url.as_str()),
        _ => panic!("unknown method"),
    };

    if let Some(ref basic_auth) = config.basic_auth {
        request_builder = request_builder.basic_auth(&basic_auth.user, Some(&basic_auth.pass));

        // デフォルトヘッダー追跡
        default_headers.insert(
            reqwest::header::AUTHORIZATION,
            "Basic <credentials>".to_string().parse().unwrap(),
        );
    }

    if let Some(ref form_data) = config.form_data {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_data.to_string());

        // デフォルトヘッダー追跡
        default_headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
    }

    if let Some(ref form) = config.form {
        let params: Vec<_> = form
            .into_iter()
            .filter_map(|arg| {
                if let Some((key, value)) = arg.split_once('=') {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();

        request_builder = request_builder
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params);

        // デフォルトヘッダー追跡
        default_headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
    }

    if let Some(ref json) = config.json {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .json(&json);

        // デフォルトヘッダー追跡
        default_headers.insert(
            CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(),
        );
    }

    // リクエストをビルドしてヘッダー情報を取得
    let request = request_builder.build()?;

    if config.verbose {
        println!("> {} {}", method.as_ref(), config.url.as_str(),);

        // デフォルトヘッダーの表示
        for (name, value) in &default_headers {
            println!("> {}: {}", name, value.to_str().unwrap_or("<binary>"));
        }

        // リクエストヘッダーの表示
        if !request.headers().is_empty() {
            for (name, value) in request.headers() {
                if !default_headers.contains_key(name) {
                    println!("> {}: {}", name, value.to_str().unwrap_or("<binary>"));
                }
            }
        }

        println!();
    }

    if config.dry_run {
        return Ok(());
    }

    // リトライ処理を含むリクエスト実行
    let (response_info, body, timing_info) = execute_with_retry(&client, request, &config)?;

    // レスポンス情報の表示
    if config.verbose {
        println!(
            "< {:?} {} {}",
            response_info.version(),
            response_info.status().as_u16(),
            response_info.status().canonical_reason().unwrap_or("")
        );
        for (name, value) in response_info.headers() {
            println!("< {}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
        println!();
    }

    // レスポンスサイズ
    let response_size = body.len();

    // タイミング情報の表示
    if config.timing {
        println!("--- Timing Information ---");
        println!("Response received: {:?}", timing_info.response_time);
        println!("Body read time: {:?}", timing_info.body_read_time);
        println!("Total time: {:?}", timing_info.total_time);
        println!(
            "Response size: {} bytes ({:.2} KB)",
            response_size,
            response_size as f64 / 1024.0
        );

        // スループット計算
        if response_size > 0 && timing_info.total_time.as_secs_f64() > 0.0 {
            let throughput = response_size as f64 / timing_info.total_time.as_secs_f64() / 1024.0;
            println!("Throughput: {:.2} KB/s", throughput);
        }
        println!();
    }

    // ボディの処理とJSON加工
    let processed_body = process_json_response(&body, &config)?;

    // ボディの表示・保存
    if let Some(output) = config.output {
        write_file_bytes(output.as_str(), processed_body.as_ref())?;
    } else {
        if !config.silent {
            println!("{}", processed_body);
        }
    }

    Ok(())
}

fn execute_with_retry(
    client: &Client,
    request: reqwest::blocking::Request,
    config: &Config,
) -> Result<(ResponseInfo, String, TimingInfo), Box<dyn Error>> {
    let mut attempt = 0;
    let max_attempts = config.retry + 1; // 初回実行 + リトライ回数
    let overall_start = Instant::now();

    loop {
        attempt += 1;

        // リクエストをクローン（再試行のため）
        let cloned_request = request.try_clone()
            .ok_or("Failed to clone request for retry")?;

        if config.verbose && attempt > 1 {
            println!("--- Retry Attempt {} ---", attempt - 1);
        }

        // 個別リクエストのタイミング測定開始
        let request_start = Instant::now();

        // リクエスト実行
        match client.execute(cloned_request) {
            Ok(response) => {
                let status = response.status();

                // リトライが必要かどうかをチェック
                let should_retry = should_retry_for_status(status.as_u16()) && attempt < max_attempts;

                if should_retry {
                    if config.verbose {
                        println!("HTTP {} - retrying after delay...", status.as_u16());
                    }

                    // 指数バックオフ遅延
                    let delay_secs = config.retry_delay * (2_f64.powi((attempt - 1) as i32));
                    thread::sleep(Duration::from_secs_f64(delay_secs));
                    continue;
                } else {
                    // 成功または最大試行回数に到達
                    let response_received_time = request_start.elapsed();

                    // responseの情報を保存してからbodyを読み取り
                    let status_code = response.status();
                    let version = response.version();
                    let headers = response.headers().clone();

                    // ボディ読み取り時間測定
                    let body_start = Instant::now();
                    let body = response.text()?;
                    let body_read_time = body_start.elapsed();

                    let total_time = overall_start.elapsed();

                    // レスポンス情報を作成
                    let response_info = ResponseInfo {
                        status: status_code,
                        version,
                        headers,
                    };

                    let timing_info = TimingInfo {
                        response_time: response_received_time,
                        body_read_time,
                        total_time,
                    };

                    return Ok((response_info, body, timing_info));
                }
            },
            Err(e) => {
                if attempt < max_attempts {
                    if config.verbose {
                        println!("Request error: {} - retrying after delay...", e);
                    }

                    // 指数バックオフ遅延
                    let delay_secs = config.retry_delay * (2_f64.powi((attempt - 1) as i32));
                    thread::sleep(Duration::from_secs_f64(delay_secs));
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        }
    }
}

fn should_retry_for_status(status_code: u16) -> bool {
    match status_code {
        // サーバーエラー (5xx) はリトライ
        500..=599 => true,
        // Too Many Requests はリトライ
        429 => true,
        // Request Timeout はリトライ
        408 => true,
        // それ以外はリトライしない
        _ => false,
    }
}

fn process_json_response(body: &str, config: &Config) -> Result<String, Box<dyn Error>> {
    // JSONかどうかチェック
    let is_json = if let Ok(json_value) = from_str::<Value>(body) {
        let mut result = json_value;

        // JSONフィルタ適用
        if let Some(filter_path) = &config.json_filter {
            result = apply_json_filter(result, filter_path)?;
        }

        // 美化表示
        if config.pretty_json {
            serde_json::to_string_pretty(&result)?
        } else {
            serde_json::to_string(&result)?
        }
    } else {
        // JSONではない場合はそのまま返す
        body.to_string()
    };

    Ok(is_json)
}

fn apply_json_filter(mut json: Value, path: &str) -> Result<Value, Box<dyn Error>> {
    let path = path.trim();

    // ルートを表す "." の場合はそのまま返す
    if path == "." {
        return Ok(json);
    }

    // "." で始まるパスから最初の "." を削除
    let path = if path.starts_with('.') {
        &path[1..]
    } else {
        path
    };

    let parts: Vec<&str> = path.split('.').collect();

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // 配列インデックスのチェック（例：items[0]）
        if let Some(bracket_pos) = part.find('[') {
            let field_name = &part[..bracket_pos];
            let index_part = &part[bracket_pos+1..];

            if let Some(close_bracket) = index_part.find(']') {
                let index_str = &index_part[..close_bracket];
                if let Ok(index) = index_str.parse::<usize>() {
                    // オブジェクトのフィールドにアクセス
                    if !field_name.is_empty() {
                        json = json.get(field_name).cloned().unwrap_or(Value::Null);
                    }
                    // 配列のインデックスにアクセス
                    json = json.get(index).cloned().unwrap_or(Value::Null);
                    continue;
                }
            }
        }

        // 通常のフィールドアクセス
        json = json.get(part).cloned().unwrap_or(Value::Null);
    }

    Ok(json)
}

fn write_file_bytes(file_path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(file_path)?;
    file.write_all(data)?;

    Ok(())
}