# aicloner 取扱説明書

## 必要環境
- Git が利用可能で、対象リポジトリへアクセスできること（SSH 推奨）
- `aicloner` バイナリ（GitHub Release もしくは `cargo install --path . --locked` でインストール）

## 初期セットアップ
1. バイナリを `PATH` の通った場所に配置する。
2. 作業したいリポジトリ URL を指定して `init` を実行する。
   ```bash
   aicloner init <repo_url> \
     [--base-dir base] [--workspaces-dir ws] [--config .aicloner.toml]
   ```
   - カレントディレクトリ配下にリポジトリ名のディレクトリを作成し、その中に設定ファイル・`base/`・`ws/` を配置。
   - `base/` にはリモートの `main` ブランチを `--single-branch` で clone。
   - `--config` で相対パスを渡した場合は生成されるリポジトリ名ディレクトリを基準に保存。

## タスク clone の追加
- リモートに同名ブランチがあればそれを clone、無ければ `--from` から新規ブランチを作成。
- `ws/<task_name>/` に作成される。
```bash
aicloner add <task_name> [--from main] [--config ./repo/.aicloner.toml]
```

## タスク clone の削除
- デフォルトでは確認プロンプトが出る。`--force` で無確認削除。
```bash
aicloner rm <task_name> [--config ./repo/.aicloner.toml] [--force]
```

## ワークスペース一覧
- `ws` 直下のディレクトリと現在のブランチ名を表形式で表示（取得失敗時は `-` 表示）。
```bash
aicloner list [--config ./repo/.aicloner.toml]
```

## 設定ファイル
`.aicloner.toml` の主な項目:
```toml
repo_url = "git@github.com:owner/repo.git"
base_dir = "base"
workspaces_dir = "ws"
```
- 相対パスは設定ファイルの位置を基準に解決。
- ディレクトリが存在しない場合は自動で作成される。

## 運用メモ
- `base/` が既に存在する状態で `init` するとエラーになるため、再初期化時は削除するか別名ディレクトリを指定する。
- `add` で clone したワークスペースは通常の Git 作業と同様に扱える。必要に応じて `git fetch` などで更新する。***
