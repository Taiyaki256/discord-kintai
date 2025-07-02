# --- ビルドステージ ---
FROM rust:1.87-slim as builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# --- ランタイムステージ ---
FROM debian:bookworm-slim

# ランタイム依存関係のインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーとグループを作成
RUN groupadd --system --gid 1001 appgroup && \
    useradd --system --uid 1001 --gid appgroup --create-home --shell /bin/false appuser

# アプリケーションバイナリをコピー
COPY --from=builder /app/target/release/discord-kintai /usr/local/bin/discord-kintai

# バイナリの所有権と実行権限をrootで設定
RUN chown appuser:appgroup /usr/local/bin/discord-kintai && \
    chmod +x /usr/local/bin/discord-kintai

# ユーザーを非rootに切り替え
USER appuser

# ワーキングディレクトリをユーザーのホームディレクトリに設定
WORKDIR /home/appuser

# アプリケーションの実行
CMD ["discord-kintai"]