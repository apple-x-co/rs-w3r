use clap::{arg, Parser};
use reqwest::blocking::Client;
use reqwest::Method;
use std::error::Error;
use std::time::Duration;

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

    let url = args.url;
    let timeout = args.timeout;

    // カスタムクライアントの作成
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .user_agent("rs-w3r/1.0");

    if proxy_host != "" && proxy_port != "" {
        let proxy_url = format!("https://{}:{}", proxy_host, proxy_port);
        let mut proxy = reqwest::Proxy::http(proxy_url)?;

        if proxy_user != "" && proxy_password != "" {
            proxy = proxy.basic_auth(proxy_user.as_str(), proxy_password.as_str());
        }

        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder.build()?;

    // リクエスト実行
    let method = Method::from_bytes(args.method.as_bytes())?;
    let mut request_builder = match method {
        Method::GET => client.get(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::DELETE => client.delete(url),
        Method::HEAD => client.head(url),
        Method::PATCH => client.patch(url),
        _ => panic!("unknown method"),
    };

    if basic_user != "" && basic_password != "" {
        request_builder = request_builder.basic_auth(basic_user, Some(basic_password));
    }

    let response = request_builder.send()?;

    // レスポンス情報の表示
    if args.verbose {
        println!(
            "Status: {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );
        println!("Headers:");
        for (name, value) in response.headers() {
            println!("  {}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
        println!();
    }

    // ボディの表示
    let body = response.text()?;
    if args.verbose {
        println!("{}", body);
    }

    Ok(())
}
