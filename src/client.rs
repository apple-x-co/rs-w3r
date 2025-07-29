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
    pub form_data: Option<String>,
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

pub fn execute_request(config: Config) -> Result<(), Box<dyn Error>> {
    // カスタムクライアントの作成
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .user_agent("rs-w3r/1.0");

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
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
            }

            client_builder = client_builder.default_headers(header_map);
        }
    }

    let client = client_builder.build()?;

    // リクエスト実行
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
    }

    if let Some(form_data) = config.form_data {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_data);
    }

    if let Some(json) = config.json {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/json; charset=utf-8")
            .json(&json);
    }

    let response = request_builder.send()?;

    // レスポンス情報の表示
    if config.verbose {
        println!(
            "> {} {}",
            method.as_ref(),
            config.url.as_str(),
        );
        println!();
        // TODO: リクエストのヘッダー情報を表示

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

    // ボディの表示
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
