# Discord-Kintai

Discord上でコマンドを使って勤怠記録ができるDiscord Botです。

## 技術スタック

### 主要技術
- **言語**: Rust 🦀
- **Discord API**: [Serenity](https://github.com/serenity-rs/serenity) v0.12+
- **コマンドフレームワーク**: [Poise](https://github.com/serenity-rs/poise)
- **データベース**: SQLite (SQLx使用)
- **日時処理**: chrono
- **設定管理**: dotenv
- **ログ**: tracing + tracing-subscriber

### 依存関係
```toml
[dependencies]
poise = {git = "https://github.com/serenity-rs/poise.git"}
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

## 機能

### 基本的な勤怠コマンド
- `/start` - 勤務開始
- `/end` - 勤務終了
- `/status` - 現在の勤務状況確認・修正

### 修正機能（statusコマンド内）
- 🔧 **時間修正**: 開始・終了時間の修正
- 🔧 **終了忘れ対応**: 終了し忘れた場合の後からの終了登録
- 🔧 **削除機能**: 誤った記録の削除

### レポート機能
- `/daily` - 日次勤怠レポート
- `/weekly` - 週次勤怠レポート
- `/monthly` - 月次勤怠レポート

### 管理機能
- `/admin_report <user>` - 指定ユーザーのレポート（管理者のみ）
- `/admin_export` - 全体データのエクスポート（管理者のみ）

## データ構造

### データベーステーブル

#### users テーブル
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    discord_id TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### attendance_records テーブル
```sql
CREATE TABLE attendance_records (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    record_type TEXT NOT NULL, -- 'start', 'end'
    timestamp DATETIME NOT NULL,
    is_modified BOOLEAN DEFAULT FALSE, -- 修正されたかどうか
    original_timestamp DATETIME, -- 修正前の元の時間
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
```

#### work_sessions テーブル（集計用）
```sql
CREATE TABLE work_sessions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    total_minutes INTEGER, -- 自動計算
    date DATE NOT NULL, -- 日付での検索用
    is_completed BOOLEAN DEFAULT FALSE, -- 終了済みかどうか
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
```

## セットアップ

### 前提条件
- Rust 1.70以上
- Discord Developer Portal でのBot作成
- SQLite

### 環境設定
1. `.env` ファイルを作成：
```env
DISCORD_TOKEN=your_bot_token_here
DATABASE_URL=sqlite:attendance.db
RUST_LOG=info
ADMIN_ROLE_ID=your_admin_role_id
```

2. 依存関係のインストール：
```bash
cargo build
```

3. データベースの初期化：
```bash
cargo run --bin setup_db
```

4. Botの起動：
```bash
cargo run
```

### Discord Bot設定
1. [Discord Developer Portal](https://discord.com/developers/applications) でアプリケーションを作成
2. Bot権限を設定：
   - `applications.commands` (スラッシュコマンド用)
   - `bot` (基本的なBot機能)
3. OAuth2 URLでサーバーに招待

## 使用方法

### 基本的な勤怠記録
```
/start       # 勤務開始
/end         # 勤務終了
/status      # 現在の状況確認・修正メニュー
```

### Status コマンドでの修正操作
`/status`コマンド実行後、インタラクティブなUI要素で操作：

#### ボタンメニュー
- 🕐 **時間修正** → モーダル入力で時間変更
- ✅ **開始・終了登録** → 確認ボタンで開始・終了時間指定追加
- 🗑️ **記録削除** → 確認ダイアログで削除
- 📝 **履歴表示** → セレクトメニューで日付選択

#### UI実装詳細
- **時間修正**: モーダルダイアログ（`HH:MM`形式入力）
- **開始・終了**: モーダルダイアログ（`HH:MM`形式入力）開始終了の時間指定追加
- **削除確認**: 2段階ボタン（削除 → 本当に削除？）
- **履歴選択**: セレクトメニューで過去7日間から選択

### レポート確認
```
/daily       # 今日の勤怠
/weekly      # 今週の勤怠
/monthly     # 今月の勤怠
```

## アーキテクチャ

```
src/
├── main.rs              # エントリーポイント
├── bot/
│   ├── mod.rs          # Botモジュール
│   ├── commands/       # コマンド実装
│   │   ├── mod.rs
│   │   ├── attendance.rs   # 開始・終了コマンド
│   │   ├── status.rs       # 状況確認・修正コマンド
│   │   ├── reports.rs      # レポート機能
│   │   └── admin.rs        # 管理者機能
│   ├── handlers/       # イベントハンドラー
│   │   ├── mod.rs
│   │   └── ready.rs
│   └── interactions/   # インタラクション処理
│       ├── mod.rs
│       └── status_buttons.rs  # Status修正ボタン処理
├── database/
│   ├── mod.rs          # データベース関連
│   ├── models.rs       # データモデル
│   ├── queries.rs      # SQLクエリ
│   └── migrations.rs   # マイグレーション
├── utils/
│   ├── mod.rs
│   ├── time.rs         # 時間計算ユーティリティ
│   ├── format.rs       # フォーマット関数
│   └── validation.rs   # バリデーション
└── config.rs           # 設定管理
```

## 開発

### ローカル開発
```bash
# 開発モードで実行
cargo run

# テスト実行
cargo test

# フォーマット
cargo fmt

# Linting
cargo clippy
```

## TODO

- [ ] データベース設計と実装（マイグレーション含む）
- [ ] 基本的な勤怠コマンドの実装（start/end）
- [ ] Statusコマンドでの修正機能実装
  - [ ] インタラクティブボタンUI
  - [ ] 時間修正機能
  - [ ] 開始・終了時間指定追加
  - [ ] 記録削除機能
- [ ] レポート機能の実装
- [ ] 管理者機能の実装
- [ ] エラーハンドリングの改善
- [ ] バリデーション強化
- [ ] テストの追加
- [ ] Docker対応
- [ ] CI/CD設定

## 実装予定の修正機能詳細

### Status コマンドの動作フロー
1. `/status` 実行
2. 現在の勤務状況表示
3. 以下のインタラクションを提供：

#### メインメニュー（ボタン）
```
[🕐 時間修正] [✅ 終了登録] [🗑️ 削除] [📋 履歴]
```

#### 時間修正フロー
1. 🕐ボタン → 修正方法選択
2. **モーダル入力**: `HH:MM`形式で直接入力
3. 修正確認 → 履歴保存

#### 削除フロー
1. 🗑️ボタン → 削除対象選択
2. **今日の開始時刻** / **今日の終了時刻** / **全て**
3. 確認ダイアログ：`本当に削除しますか？` [はい] [いいえ]

#### UI技術仕様
- **ボタン**: `serenity::ComponentType::Button`
- **モーダル**: `serenity::CreateModal` + `TextInputStyle::Short`
- **セレクト**: `serenity::CreateSelectMenu`
- **タイムアウト**: 5分間（自動無効化）

