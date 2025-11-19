# aicloner

コーディング CLI を複数タスクで同時に走らせる際、毎回 clone する手間やディスク容量を抑えたいという課題から生まれたツールです。1 つのベース clone を共有しつつ、タスクごとに独立したワークスペースを素早く用意・破棄できるように設計されています。

単一のリモート Git リポジトリからベース clone を 1 度だけ作成し、各タスク用に複製ディレクトリを管理する CLI です。
`base-repo/` と `workspaces/<task>/` を構築し、タスク単位で clone を増減できます。

## ビルド

```bash
cargo build --release
```

生成物は `target/release/aicloner` です。以降の例ではカレントディレクトリ直下のバイナリを実行すると想定しています。

## インストール

ローカルにインストールする場合は、Cargo の `install` を使うと `~/.cargo/bin` に配置されます。

```bash
cargo install --path . --locked
```

独自の場所に置きたい場合は、`cargo build --release` 後に `target/release/aicloner` を任意のディレクトリへコピーし、`PATH` に追加してください。

## アップデート

最新版に差し替えるには、作業ツリーで最新のソースを取得したうえで同じコマンドを再実行します。

```bash
git pull origin main  # 必要に応じて
cargo install --path . --locked --force
```

すでにバイナリを手動配置している場合は、再ビルド後に新しいバイナリで上書きしてください。

## 初期化 `init`

```bash
./target/release/aicloner init --repo-url git@github.com:owner/repo.git \
    --base-dir ./base-repo \
    --workspaces-dir ./workspaces \
    --config ./.aicloner.toml
```

- `--repo-url` は必須。デフォルトでは `base-repo`・`workspaces`・`.aicloner.toml` が使われます。
- ベースディレクトリが無ければ `git clone`、あれば `git -C base-repo fetch --all` で更新します。
- 設定ファイルは常に上書き保存されます。

## タスク clone 追加 `add`

```bash
./target/release/aicloner add login-ui --from main --config ./.aicloner.toml
```

- `workspaces/login-ui` を clone 先として作成し、まず `git clone --reference base-repo ...` を試み、失敗時は通常 clone にフォールバックします。
- clone 後に `git checkout <branch>` を実行します（デフォルト `main`）。
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
base_dir = "base-repo"
workspaces_dir = "workspaces"
```

相対パスは設定ファイルの設置場所を起点に解決されます。複数の設定ファイルを用意して別々のリポジトリを管理することも可能です。

## ディレクトリ移動の例

タスク用 clone へ移動し作業する場合:

```bash
cd workspaces/<task_name>
```

作業後にベース clone へ戻る場合:

```bash
cd ../../base-repo
```

任意のシェルエイリアス（例: `ws(){ cd workspaces/$1; }`）を設定すると移動を簡略化できます。
