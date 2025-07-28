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

## ✨ 主な特徴

⚡ **高速処理** - Rustによる最適化されたパフォーマンス  
🔐 **セキュア通信** - rustls-tlsによる安全なHTTPS接続  
🍪 **クッキー管理** - 自動的なクッキーの送受信  
📋 **カスタムヘッダー** - 柔軟なHTTPヘッダー設定  
🌍 **プロキシ対応** - HTTP プロキシサーバー経由でのリクエスト  
📊 **詳細出力** - レスポンスのステータス、ヘッダー、実行時間の表示  
🔇 **サイレントモード** - スクリプト用の静寂実行  
⏱️ **タイムアウト設定** - カスタマイズ可能なリクエストタイムアウト  
🔧 **環境変数対応** - 設定の環境変数による管理  
📁 **ファイル出力** - レスポンスの直接ファイル保存

## 🛠️ 技術スタック

- **言語**: Rust 2021 Edition
- **HTTPクライアント**: reqwest (0.12) - JSON、クッキー、ブロッキング、rustls-tls、HTTP/2対応
- **CLI**: clap (4.5) - derive、環境変数機能付き
- **最適化**: LTO、コード生成最適化、シンボル削除による小さなバイナリサイズ
- **クロスコンパイル**: cross対応（Linux musl target）

## 📖 使用例

### 基本的なGETリクエスト
```bash
rs-w3r -u https://api.example.com/users
```

### JSONデータをPOST
```bash
rs-w3r -m POST -u https://api.example.com/users -j '{"name": "田中", "email": "tanaka@example.com"}'
```

### Basic認証付きリクエスト
```bash
rs-w3r --basic-user myuser --basic-pass mypass -u https://api.example.com/private
```

### カスタムヘッダー付きリクエスト
```bash
rs-w3r -u https://api.example.com/data --headers "Authorization: Bearer token123" --headers "Content-Type: application/json"
```

### フォームデータの送信
```bash
rs-w3r -m POST -u https://api.example.com/form -f "name=田中&email=tanaka@example.com"
```

### プロキシ経由でのリクエスト
```bash
rs-w3r -u https://api.example.com/data --proxy-host proxy.example.com --proxy-port 8080
```

### 詳細出力とファイル保存
```bash
rs-w3r -v -u https://api.example.com/data -o response.json
```

### 環境変数を使用した設定
```bash
export BASIC_USER=myuser
export BASIC_PASS=mypass
export PROXY_HOST=proxy.example.com
export PROXY_PORT=8080

rs-w3r -u https://api.example.com/secure-data
```