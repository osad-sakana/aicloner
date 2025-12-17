
<img src="aicloner_logo.png" alt="aicloner logo" width="200"/>
**AI対応タスク別ワークスペース管理ツール**

![Rust](https://img.shields.io/badge/rust-2021-orange)
[![Release](https://img.shields.io/github/v/release/osad-sakana/aicloner)](https://github.com/osad-sakana/aicloner/releases)
[![Build](https://github.com/osad-sakana/aicloner/actions/workflows/release.yml/badge.svg)](https://github.com/osad-sakana/aicloner/actions)

---

## 目次

- [💡 概要](#-概要)
- [🚀 クイックスタート](#-クイックスタート)
- [📦 インストール](#-インストール)
- [📚 コマンドリファレンス](#-コマンドリファレンス)
  - [🏗️ init](#️-init---リポジトリの初期化)
  - [➕ add](#-add---タスクcloneの追加)
  - [✖️ rm](#️-rm---タスクcloneの削除)
  - [📋 list](#-list---ワークスペース一覧)
  - [🐛 issues](#-issues---issue一覧表示)
  - [▶️ start](#️-start---issue対応開始)
- [⚙️ 設定ファイル](#️-設定ファイル)
- [👨‍💻 開発向け情報](#-開発向け情報)

---

## 💡 概要

**aicloner** は、単一のリモート Git リポジトリからタスク専用の clone を量産し、`ws/<task>/` 配下で管理する Rust 製 CLI ツールです。

### 主な特徴

- ✨ タスクごとに独立したワークスペースを自動作成
- 🔀 タスク名と同名の Git ブランチを自動生成
- 🤖 GitHub Issue と AI ツール（Claude / Codex）を統合
- ⚡ 複数の AI コードツールを並列実行可能

---

## 🚀 クイックスタート

### なぜ aicloner が必要なのか？

複数の AI コードツールを並列で実行したいとき、同じリポジトリを何度もクローンするのは面倒です。
**aicloner** はリポジトリの準備からブランチ作成、AI ツールの起動までを自動化し、効率的なタスク管理を実現します。

### 最小限の使い方

```bash
aicloner init git@github.com:owner/repo.git
cd repo
aicloner start 1
```

これだけで、Claude が Issue #1 の内容を読んで実装を開始してくれます。
次からも `aicloner start 2` とすれば Issue #2 の対応を始められます。

### 必要環境

- **Git** - リポジトリ操作の基盤
- **GitHub CLI (`gh`)** - Issue 管理と確認用（`issues` / `start` コマンドで使用）
- **AI CLI** - `claude` または `codex-cli`（`start` コマンドで使用）

---

## 📦 インストール

### 📥 バイナリ配布のインストール

GitHub Release から OS に対応したアーカイブを取得し、解凍したバイナリを任意の場所へ配置してください。

- `aicloner-linux-x86_64.tar.gz`
- `aicloner-macos-arm64.tar.gz`
- `aicloner-windows-x86_64.zip`

```bash
# Linux / macOS
curl -L -o aicloner.tar.gz https://github.com/osad-sakana/aicloner/releases/download/v0.1.0/aicloner-linux-x86_64.tar.gz
tar -xf aicloner.tar.gz
install -m 755 aicloner ~/bin/aicloner

# Windows (PowerShell)
Invoke-WebRequest -OutFile aicloner.zip https://github.com/osad-sakana/aicloner/releases/download/v0.1.0/aicloner-windows-x86_64.zip
Expand-Archive aicloner.zip
Move-Item .\aicloner.exe C:\tools\aicloner.exe
```

`PATH` が通った場所へ配置したら `aicloner --help` で動作を確認できます。

### 🔨 ソースからのビルド

```bash
cargo build --release
```

生成物は `target/release/aicloner` です。

```bash
# ローカルインストール
cargo install --path . --locked
```

`~/.cargo/bin` に配置されます。別の場所に置きたい場合はビルド済みバイナリをコピーして `PATH` に追加してください。

---

## 📚 コマンドリファレンス

### 🏗️ init - リポジトリの初期化

```bash
aicloner init <repo_url> [--base-dir base] [--workspaces-dir ws] [--config .aicloner.toml]
```

- カレントディレクトリにリポジトリ名と同名のディレクトリを新規作成します（例: `repo/`）
- その配下に `base/`・`ws/`・`.aicloner.toml` をまとめて用意します
- `base/` にはリモートリポジトリの `main` ブランチを `--single-branch` で clone します
- ディレクトリ名は `--base-dir` / `--workspaces-dir` で変更できます。設定ファイル名は `--config` で指定します（相対パスは生成したリポジトリ名ディレクトリ基準）

**例:**
```bash
aicloner init git@github.com:owner/repo.git
```

---

### ➕ add - タスクcloneの追加

```bash
aicloner add <task_name> [--from main] [--config ./repo/.aicloner.toml]
```

- リモートリポジトリから `--from`（デフォルト `main`）を `--single-branch` で clone します
- 同名のリモートブランチが存在する場合はそれを clone し、存在しない場合は `--from` から `git checkout -b <task_name>` で新規作成します
- 同名ディレクトリが既にある場合はエラーになります

**例:**
```bash
aicloner add login-ui --from main --config ./repo/.aicloner.toml
```

---

### ✖️ rm - タスクcloneの削除

```bash
aicloner rm <task_name> [--config ./repo/.aicloner.toml] [--force]
```

- `--force` を付けない場合は `y` で確認が必要です
- `ws/<task>` ディレクトリが無い場合はエラーになります

**例:**
```bash
aicloner rm login-ui --config ./repo/.aicloner.toml
```

---

### 📋 list - ワークスペース一覧

```bash
aicloner list [--config ./repo/.aicloner.toml]
```

- `ws` 直下のタスク名とディレクトリパス、現在のブランチ（取得できた場合）を表形式で出力します

**例:**
```bash
aicloner list --config ./repo/.aicloner.toml
```

---

### 🐛 issues - Issue一覧表示

```bash
aicloner issues [--config ./repo/.aicloner.toml]
```

- GitHub の open issues を一覧表示します
- `gh` コマンドが必要です（[GitHub CLI](https://cli.github.com/)）
- リポジトリが aicloner で管理されている必要があります

**例:**
```bash
aicloner issues --config ./repo/.aicloner.toml
```

---

### ▶️ start - Issue対応開始

```bash
aicloner start <issue_number> [--config ./repo/.aicloner.toml] [--claude|--codex]
```

指定した番号の GitHub issue に対応するワークスペースを作成し、AI ツール対話セッションを起動します。

#### AI ツールの選択

デフォルトでは Claude を使用しますが、`--codex` フラグで Codex を選択できます：

```bash
# Claude を使用（デフォルト）
aicloner start 1

# Claude を明示的に指定
aicloner start 1 --claude

# Codex を使用
aicloner start 1 --codex
```

#### ワークフロー

1. `gh issue view <番号>` で issue の存在を確認
2. `aicloner-issue<番号>` の名前でブランチを作成（例: `aicloner-issue1`）
3. ベースブランチは `main`（存在しなければ `master`）
4. 既存ブランチがある場合はユーザーに確認
5. ワークスペースを `ws/aicloner-issue<番号>/` に作成
6. 選択した AI ツールのセッションを起動し、issue 対応を開始

#### 典型的な使い方

```bash
# 1. Open issues を確認
aicloner issues

# 2. 対応したい issue 番号を選んで作業開始（Claude を使用）
aicloner start 3

# 3. Claude が起動し、issue #3 の対応を開始
```

---

## ⚙️ 設定ファイル

`.aicloner.toml` の例:

```toml
repo_url = "git@github.com:owner/repo.git"
base_dir = "base"
workspaces_dir = "ws"
```

相対パスは設定ファイルの設置場所を起点に解決されます。複数の設定ファイルを用意して別のリポジトリを管理することも可能です。

---

## 👨‍💻 開発向け情報

### 🔨 ソースからのビルド

```bash
cargo build --release
```

生成物は `target/release/aicloner` です。カレントディレクトリ直下に置いたバイナリを利用する想定です。

### 📦 ローカルインストール

```bash
cargo install --path . --locked
```

`~/.cargo/bin` に配置されます。別の場所に置きたい場合はビルド済みバイナリをコピーして `PATH` に追加してください。

### 🔄 開発版アップデート

```bash
git pull origin main  # 必要に応じて
cargo install --path . --locked --force
```

手動配置の場合も再ビルド後に新しいバイナリで上書きしてください。

### 📤 リリース (GitHub Actions)

`v*` 形式のタグを push すると GitHub Actions が自動で実行され、Linux/macOS/Windows 向けのバイナリをビルドして GitHub Release に添付します。

```bash
# バージョンを更新したコミットを push 済みと仮定
git tag v0.1.0
git push origin v0.1.0
```

タグが作成されると `Release` Workflow が走り、成果物付きの Release が公開されます。

---

詳しい使い方は [`docs/USAGE.md`](docs/USAGE.md) を参照してください。
