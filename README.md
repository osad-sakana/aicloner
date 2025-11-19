# aicloner

コーディング CLI を複数タスクで同時に走らせる際、毎回 clone する手間をコマンド 1 つで完了させるためのツールです。タスクを追加するたびにリモートの最新ブランチから新しい clone を作成し、ワークスペースを素早く用意・破棄できます。

単一のリモート Git リポジトリから各タスク専用の clone を生成し、`workspaces/<task>/` 配下で管理します。`add` 実行時にはタスク名と同じ Git ブランチを自動作成するため、タスクごとの作業内容をそのままブランチとして扱えます。

## バイナリ配布版のインストール

GitHub Release から OS に対応したアーカイブを取得し、解凍したバイナリをお好みの場所へ配置してください。ファイル名は以下の通りです。

- `aicloner-linux-x86_64.tar.gz`
- `aicloner-macos-arm64.tar.gz`
- `aicloner-windows-x86_64.zip`

典型的な利用例:

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
./target/release/aicloner init --repo-url git@github.com:owner/repo.git \
    --workspaces-dir ./workspaces \
    --config ./.aicloner.toml
```

- `--repo-url` は必須。`workspaces`・`.aicloner.toml` がデフォルトです。
- ワークスペースディレクトリが無い場合は自動作成します。
- 設定ファイルは常に上書き保存されます。

## タスク clone 追加 `add`

```bash
./target/release/aicloner add login-ui --from main --config ./.aicloner.toml
```

- リモートリポジトリから `--from`（デフォルト `main`）を `--single-branch` で clone します。
- clone 完了後にタスク名（この例では `login-ui`）のブランチを `git checkout -b <task_name>` で自動作成し、直ちに切り替えます。
- 同名ディレクトリが存在する場合はエラーになります。

## タスク clone 削除 `rm`

```bash
./target/release/aicloner rm login-ui --config ./.aicloner.toml
```

- `--force` を付けない場合は `y` で確認が必要です。
- `workspaces/<task>` ディレクトリが無い場合はエラーになります。

## 一覧表示 `list`

```bash
./target/release/aicloner list --config ./.aicloner.toml
```

- `workspaces` 直下の各タスク名とディレクトリパス、現在のブランチ（取得できた場合）を表形式で出力します。

## 設定ファイル

`.aicloner.toml` 例:

```toml
repo_url = "git@github.com:owner/repo.git"
workspaces_dir = "workspaces"
```

相対パスは設定ファイルの設置場所を起点に解決されます。複数の設定ファイルを用意して別々のリポジトリを管理することも可能です。

## ディレクトリ移動の例

タスク用 clone へ移動し作業する場合:

```bash
cd workspaces/<task_name>
```

作業後にワークスペースルートへ戻る場合:

```bash
cd ..
```

任意のシェルエイリアス（例: `ws(){ cd workspaces/$1; }`）を設定すると移動を簡略化できます。

## 開発者向け情報

### ソースからのビルド

```bash
cargo build --release
```

生成物は `target/release/aicloner` です。カレントディレクトリ直下のバイナリを利用する想定です。

### ローカルインストール

```bash
cargo install --path . --locked
```

`~/.cargo/bin` に配置されます。別の場所に置きたい場合はビルド済みバイナリをコピーし、`PATH` に追加してください。

### 開発版アップデート

```bash
git pull origin main  # 必要に応じて
cargo install --path . --locked --force
```

手動配置の場合も再ビルド後に新しいバイナリで上書きしてください。

### リリース (GitHub Actions)

`v*` 形式のタグを push すると GitHub Actions が自動的に実行され、Linux/macOS/Windows 向けのバイナリをビルドして GitHub Release に添付します。成果物は以下のファイル名でアップロードされます。

- `aicloner-linux-x86_64.tar.gz`
- `aicloner-macos-arm64.tar.gz`
- `aicloner-windows-x86_64.zip`

実行手順の例:

```bash
# バージョンを更新したコミットを push 済みと仮定
git tag v0.1.0
git push origin v0.1.0
```

タグが作成されると `Release` Workflow が走り、成果物付きの Release が自動的に公開されます。
