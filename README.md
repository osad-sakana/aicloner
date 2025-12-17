# aicloner

単一のリモート Git リポジトリからタスク専用の clone を量産し、`ws/<task>/` 配下で管理する CLI です。`add` 実行時にはタスク名と同名の Git ブランチを自動で用意するため、作業単位をそのままブランチとして扱えます。

詳しい使い方は `docs/USAGE.md` を参照してください。

## バイナリ配布のインストール

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

## 初期化 `init`

```bash
./target/release/aicloner init git@github.com:owner/repo.git
```

- カレントディレクトリにリポジトリ名と同名のディレクトリを新規作成します（例: `repo/`）。
- その配下に `base/`・`ws/`・`.aicloner.toml` をまとめて用意します。
- `base/` にはリモートリポジトリの `main` ブランチを `--single-branch` で clone します。
- ディレクトリ名は `--base-dir` / `--workspaces-dir` で変更できます。設定ファイル名は `--config` で指定します（相対パスは生成したリポジトリ名ディレクトリ基準）。

## タスク clone 追加 `add`

```bash
./target/release/aicloner add login-ui --from main --config ./repo/.aicloner.toml
```

- リモートリポジトリから `--from`（デフォルト `main`）を `--single-branch` で clone します。
- 同名のリモートブランチが存在する場合はそれを clone し、存在しない場合は `--from` から `git checkout -b <task_name>` で新規作成します。
- 同名ディレクトリが既にある場合はエラーになります。

## タスク clone 削除 `rm`

```bash
./target/release/aicloner rm login-ui --config ./repo/.aicloner.toml
```

- `--force` を付けない場合は `y` で確認が必要です。
- `ws/<task>` ディレクトリが無い場合はエラーになります。

## 一覧表示 `list`

```bash
./target/release/aicloner list --config ./repo/.aicloner.toml
```

- `ws` 直下のタスク名とディレクトリパス、現在のブランチ（取得できた場合）を表形式で出力します。

## Issue一覧表示 `issues`

```bash
./target/release/aicloner issues --config ./repo/.aicloner.toml
```

- GitHub の open issues を一覧表示します。
- `gh` コマンドが必要です（[GitHub CLI](https://cli.github.com/)）。
- リポジトリが aicloner で管理されている必要があります。

## Issue対応開始 `start`

```bash
./target/release/aicloner start 1 --config ./repo/.aicloner.toml
```

- 指定した番号の GitHub issue に対応するワークスペースを作成し、Claude 対話セッションを起動します。
- `gh` および `claude` コマンドが必要です。
- ワークフロー：
  1. `gh issue view <番号>` で issue の存在を確認
  2. `aicloner-issue<番号>` の名前でブランチを作成（例: `aicloner-issue1`）
  3. ベースブランチは `main`（存在しなければ `master`）
  4. 既存ブランチがある場合はユーザーに確認
  5. ワークスペースを `ws/aicloner-issue<番号>/` に作成
  6. Claude セッションを起動し、issue 対応を開始

典型的な使い方：

```bash
# 1. Open issues を確認
./target/release/aicloner issues

# 2. 対応したい issue 番号を選んで作業開始
./target/release/aicloner start 3

# 3. Claude が起動し、issue #3 の対応を開始
```

## 設定ファイル

`.aicloner.toml` の例:

```toml
repo_url = "git@github.com:owner/repo.git"
base_dir = "base"
workspaces_dir = "ws"
```

相対パスは設定ファイルの設置場所を起点に解決されます。複数の設定ファイルを用意して別のリポジトリを管理することも可能です。

## 開発向け情報

### ソースからのビルド

```bash
cargo build --release
```

生成物は `target/release/aicloner` です。カレントディレクトリ直下に置いたバイナリを利用する想定です。

### ローカルインストール

```bash
cargo install --path . --locked
```

`~/.cargo/bin` に配置されます。別の場所に置きたい場合はビルド済みバイナリをコピーして `PATH` に追加してください。

### 開発版アップデート

```bash
git pull origin main  # 必要に応じて
cargo install --path . --locked --force
```

手動配置の場合も再ビルド後に新しいバイナリで上書きしてください。

### リリース (GitHub Actions)

`v*` 形式のタグを push すると GitHub Actions が自動で実行され、Linux/macOS/Windows 向けのバイナリをビルドして GitHub Release に添付します。

```bash
# バージョンを更新したコミットを push 済みと仮定
git tag v0.1.0
git push origin v0.1.0
```

タグが作成されると `Release` Workflow が走り、成果物付きの Release が公開されます。
