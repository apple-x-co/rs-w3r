# rs-w3r

Rustで作られた高速・軽量なWebリクエストコマンドラインツール

## 🚀 概要

rs-w3rは、開発者やシステム管理者向けに設計されたパワフルなHTTPクライアントツールです。Rustの高いパフォーマンスと安全性を活かし、API開発・テスト、Webサービスのデバッグ、HTTP操作の自動化を効率的に行うことができます。シンプルなコマンドラインインターフェースで、複雑なHTTPリクエストを簡単に実行できます。

## 📦 対応フォーマット

- **リクエスト形式**: JSON、フォームデータ（application/x-www-form-urlencoded）
- **HTTPメソッド**: GET、POST、PUT、DELETE、HEAD、PATCH
- **認証方式**: Basic認証
- **プロキシ**: HTTP プロキシ（認証付き対応）
- **出力形式**: プレーンテキスト、ファイル出力
- **JSON処理**: 自動美化表示、jq風パスフィルタリング

## ✨ 主な特徴

⚡ **高速処理** - Rustによる最適化されたパフォーマンス  
🔐 **セキュア通信** - rustls-tlsによる安全なHTTPS接続  
🍪 **クッキー管理** - 自動的なクッキーの送受信  
📋 **カスタムヘッダー** - 柔軟なHTTPヘッダー設定  
🌍 **プロキシ対応** - HTTP プロキシサーバー経由でのリクエスト  
📊 **詳細出力** - レスポンスのステータス、ヘッダー、実行時間の表示  
⏱️ **パフォーマンス測定** - レスポンス時間、転送速度、サイズの詳細分析  
🎨 **JSON美化・フィルタ** - 自動的なJSON整形表示とjq風パスフィルタリング  
🔄 **自動リトライ** - 指数バックオフによるスマートな再試行機能  
🔇 **サイレントモード** - スクリプト用の静寂実行  
⏰ **タイムアウト設定** - カスタマイズ可能なリクエストタイムアウト  
🔧 **環境変数対応** - 設定の環境変数による管理  
📁 **ファイル出力** - レスポンスの直接ファイル保存  
🧪 **ドライラン** - 実際にリクエストを送信せずにリクエスト内容を確認

## 🛠️ 技術スタック

- **言語**: Rust 2021 Edition
- **HTTPクライアント**: reqwest (0.12) - JSON、クッキー、ブロッキング、rustls-tls、HTTP/2対応
- **CLI**: clap (4.5) - derive、環境変数機能付き
- **JSON処理**: serde_json (1.0) - 美化表示、パスフィルタリング
- **最適化**: LTO、コード生成最適化、シンボル削除による小さなバイナリサイズ
- **クロスコンパイル**: cross対応（Linux musl target）

## 📖 使用例

### 基本的なGETリクエスト

```bash
rs-w3r -u https://httpbin.org/get
```

### JSONデータをPOST

```bash
rs-w3r -m POST -u https://httpbin.org/post -j '{"name": "田中", "email": "tanaka@example.com"}'
```

### Basic認証付きリクエスト

```bash
rs-w3r --basic-user myuser --basic-pass mypass -u https://httpbin.org/headers
```

### カスタムヘッダー付きリクエスト

```bash
rs-w3r -u https://httpbin.org/headers --headers "Authorization: Bearer token123" --headers "Content-Type: application/json"
```

### フォームデータの送信

```bash
rs-w3r -m POST -u https://httpbin.org/post -f "name=田中&email=tanaka@example.com"
```

```bash
rs-w3r -m POST -u https://httpbin.org/post --form "name=田中" --form "email=tanaka@example.com"
```

### プロキシ経由でのリクエスト

```bash
rs-w3r -u https://httpbin.org/get --proxy-host proxy.example.com --proxy-port 8080
```

### 詳細出力とファイル保存

```bash
rs-w3r -v -u https://httpbin.org/ip -o response.json
```

### パフォーマンス測定

```bash
# 基本的なタイミング測定
rs-w3r -u https://httpbin.org/get --timing

# 詳細出力と組み合わせ
rs-w3r -u https://api.github.com/users/octocat --timing -v
```

**出力例:**

```
--- Timing Information ---
Response received: 187ms
Body read time: 12ms
Total time: 199ms
Response size: 1843 bytes (1.80 KB)
Throughput: 9.05 KB/s
```

### JSON美化・フィルタリング

```bash
# JSONの美化表示
rs-w3r -u https://api.github.com/users/apple-x-co --pretty-json

# 特定フィールドの抽出
rs-w3r -u https://api.github.com/users/apple-x-co --json-filter ".name"

# 配列の最初の要素
rs-w3r -u https://api.github.com/repos/apple-x-co/rocket/releases/latest --json-filter ".assets[0].browser_download_url"

# 美化とフィルタの組み合わせ
rs-w3r -u https://api.github.com/users/apple-x-co --pretty-json --json-filter ".public_repos"
```

**出力例:**

```bash
# 通常の出力
{"login":"apple-x-co","id":1,"name":"DUMMY","public_repos":8}

# --pretty-json適用後
{
  "login": "apple-x-co",
  "id": 1,
  "name": "DUMMY",
  "public_repos": 8
}

# --json-filter ".name"適用後
"DUMMY"
```

### リクエスト内容の確認（ドライラン）

```bash
rs-w3r --dry-run -v -m POST -u https://httpbin.org/post -j '{"test": "data"}'
```

### 環境変数を使用した設定

```bash
export BASIC_USER=myuser
export BASIC_PASS=mypass
export PROXY_HOST=proxy.example.com
export PROXY_PORT=8080

rs-w3r -u https://www.example.com/secure-data
```

### オプション一覧

#### 基本オプション

- `-u, --url <URL>` - リクエスト先のURL（必須）
- `-m, --method <METHOD>` - HTTPメソッド（デフォルト: GET）
- `-o, --output <FILE>` - レスポンスをファイルに保存
- `-t, --timeout <SECONDS>` - タイムアウト時間（デフォルト: 30秒）
- `-v, --verbose` - 詳細な出力を表示
- `-s, --silent` - 出力を抑制
- `--dry-run` - 実際にリクエストを送信せず、リクエスト内容のみ表示
- `--timing` - パフォーマンス測定情報を表示（レスポンス時間、転送速度など）
- `--pretty-json` - JSONレスポンスの美化表示（整形されたインデント付き）
- `--json-filter <PATH>` - jq風JSONパスフィルタリング（例：`.name`, `.[0].title`, `.data.items[0]`）
- `--retry <回数>` - リトライ回数の設定（デフォルト: 0、5xx/429/408エラー時に自動リトライ）
- `--retry-delay <秒数>` - 初回リトライ遅延時間、指数バックオフ適用（デフォルト: 1.0秒）

#### データ送信

- `-j, --json <JSON>` - JSON形式でデータを送信
- `-f, --form-data <DATA>` - 手動エンコード済みのフォームデータを送信（例："name=value&key=data"）
- `--form <KEY=VALUE>` - キー・バリューペアからフォームデータを自動生成（複数指定可能）

#### 認証・セキュリティ

- `--basic-user <USER>` - Basic認証のユーザー名
- `--basic-pass <PASS>` - Basic認証のパスワード
- `--headers <HEADER>` - カスタムヘッダー（複数指定可能）
- `--cookies <COOKIE>` - クッキーを送信（複数指定可能）

#### プロキシ設定

- `--proxy-host <HOST>` - プロキシサーバーのホスト
- `--proxy-port <PORT>` - プロキシサーバーのポート
- `--proxy-user <USER>` - プロキシ認証のユーザー名
- `--proxy-pass <PASS>` - プロキシ認証のパスワード

#### 環境変数

- `BASIC_USER`, `BASIC_PASS` - Basic認証の資格情報
- `PROXY_HOST`, `PROXY_PORT`, `PROXY_USER`, `PROXY_PASS` - プロキシ設定

## 🆚 比較

### Slack に投稿

**curl**  
`curl -X POST --data-urlencode 'payload={"channel": "#channel-name", "text": "HELLO"}' WEBHOOK_URL`

**rs-w3r**  
`rs-w3r --method POST --url WEBHOOK_URL --form 'payload={"channel": "#test-channel", "text": "HELLO"}'`