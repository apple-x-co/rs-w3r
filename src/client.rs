use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderName, CONTENT_TYPE};
use reqwest::{Method, Url};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

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
    pub method: String,
    pub output: Option<String>,
    pub proxy: Option<ProxyConfig>,
    pub silent: bool,
    pub timeout: u64,
    pub url: String,
    pub verbose: bool,
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

    if let Some(proxy) = config.proxy {
        let proxy_url = format!("https://{}:{}", proxy.host, proxy.port);
        let mut http_proxy = reqwest::Proxy::http(proxy_url)?;

        if let Some(proxy_user) = proxy.user {
            if let Some(proxy_pass) = proxy.pass {
                http_proxy = http_proxy.basic_auth(proxy_user.as_str(), proxy_pass.as_str());
            }
        }

        client_builder = client_builder.proxy(http_proxy);
    }

    if let Some(cookies) = config.cookies {
        let cookie_jar = Jar::default();
        let url = &Url::parse(config.url.as_str()).unwrap();

        for cookie in cookies {
            cookie_jar.add_cookie_str(cookie.as_str(), url);
        }

        client_builder = client_builder.cookie_provider(Arc::new(cookie_jar));
    }

    if let Some(headers) = config.headers {
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

    if let Some(basic_auth) = config.basic_auth {
        request_builder = request_builder.basic_auth(basic_auth.user, Some(basic_auth.pass));

        // デフォルトヘッダー追跡
        default_headers.insert(
            reqwest::header::AUTHORIZATION,
            "Basic <credentials>".to_string().parse().unwrap(),
        );
    }

    if let Some(form_data) = config.form_data {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_data);

        // デフォルトヘッダー追跡
        default_headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
    }

    if let Some(form) = config.form {
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

    if let Some(json) = config.json {
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

    // リクエストを実行しレスポンスを取得
    let response = client.execute(request)?;

    // レスポンス情報の表示
    if config.verbose {
        println!(
            "< {:?} {} {}",
            response.version(),
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );
        for (name, value) in response.headers() {
            println!("< {}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
        println!();
    }

    // レスポンスのボディの表示
    let body = response.text()?;
    if let Some(output) = config.output {
        write_file_bytes(output.as_str(), body.as_ref())?;
    } else {
        if !config.silent {
            println!("{}", body);
        }
    }

    Ok(())
}

fn write_file_bytes(file_path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(file_path)?;
    file.write_all(data)?;

    Ok(())
}
