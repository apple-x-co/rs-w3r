use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;
use std::error::Error;
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
    pub form_data: Option<String>,
    pub json: Option<String>,
    pub method: String,
    pub proxy: Option<ProxyConfig>,
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

    let client = client_builder.build()?;

    // リクエスト実行
    let method = Method::from_bytes(config.method.as_bytes())?;
    let mut request_builder = match method {
        Method::GET => client.get(config.url),
        Method::POST => client.post(config.url),
        Method::PUT => client.put(config.url),
        Method::DELETE => client.delete(config.url),
        Method::HEAD => client.head(config.url),
        Method::PATCH => client.patch(config.url),
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
            "{:?} {} {}",
            response.version(),
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );
        for (name, value) in response.headers() {
            println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
        println!();
    }

    // ボディの表示
    let body = response.text()?;
    if config.verbose {
        println!("{}", body);
    }

    Ok(())
}
