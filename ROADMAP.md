# Roadmap

このドキュメントは endringer の今後の開発方針とバージョン計画を示します。
バージョン番号は [Semantic Versioning](https://semver.org/) に従います。

---

## リリース管理方針

### バージョニングルール

| 変更の種類 | バージョンの上げ方 |
|---|---|
| 公開 API の破壊的変更（型・メソッドの削除・シグネチャ変更） | major（ただし v1.0 到達前は minor） |
| 後方互換の機能追加 | minor |
| バグ修正・ドキュメント修正・内部リファクタリング | patch |

v1.0 到達前（現在）は、minor バージョンアップでも破壊的変更が含まれる場合があります。
CHANGELOG で `Breaking change:` として明示します。

### タアーボール命名規則

```
dist/endringer-{version}.tar.gz   # 例: dist/endringer-0.9.0.tar.gz
```

ファイル名に `v` プレフィックスは付与しません。
git タグは引き続き `v{version}` 形式（例: `v0.9.0`）を使用します。

### リリース手順

```sh
# 1. Cargo.toml の version を更新
# 2. CHANGELOG.md に [x.y.z] セクションを追加
# 3. ROADMAP.md のリリース履歴を更新
# 4. テスト・ビルドが通ることを確認
cargo test --lib

# 5. リリーススクリプトを実行
./scripts/release.sh

# 6. リモートへ push
git push origin master
git push origin v{version}

# 7. crates.io へ公開（任意）
cargo publish
```

---

## リリース履歴

| バージョン | リリース日 | 概要 |
|---|---|---|
| [v0.7.1] | 2025 | 初回公開。ブランチ一覧・コミット履歴・ステータスダイジェスト |
| [v0.8.0] | 2026-05-04 | `CommitId` newtype、タグ操作、`log_since`、公開 API の整合 |
| [v0.8.1] | 2026-05-04 | バグ修正（repo_name・current_branch・timestamp 型安全性・author 一貫性）、derive 整合、tarball 命名変更 |

---

## v0.9.0（次期リリース）

### Annotated タグのサポート

`create_tag` は現在、軽量タグのみを作成します。
リリースワークフローで一般的な annotated タグ（タッガー・メッセージ付き）を追加します。

```rust
// 提案 API
repo.create_annotated_tag("v1.0.0", "Release v1.0.0")?;
```

### `CommitId::from_hex`

既知の SHA-1 hex 文字列から `CommitId` を構築し、履歴をたどらずに特定コミットを参照できるようにします。

```rust
let id = CommitId::from_hex("a1b2c3d4...")?;
```

### `list_tags` / `list_commits` のソートオプション

現在は ref ストア順で返します。タイムスタンプ順・名前順の並べ替えオプションを追加します。

```rust
// 提案 API
repo.list_commits_sorted(SortOrder::NewestFirst)?;
repo.list_tags_sorted(SortOrder::ByName)?;
```

### `CommitId` の `PartialOrd` / `Ord`

コレクション操作（ソート・重複排除）に備えて `Ord` を実装します。

---

## v0.10.0

### `CommitInfo` のコミッター情報

author とは別に committer identity を公開します。
cherry-pick・rebase 後のコミットで両者が異なる場合の情報取得に対応します。

```rust
pub struct CommitInfo {
    pub commit_id: CommitId,
    pub author: String,
    pub committer: String,          // new
    pub summary: String,
    pub timestamp: SystemTime,      // author time
    pub committer_timestamp: SystemTime,  // new
}
```

### `Repository::find_commit(id)`

`CommitId` を指定して単一の `CommitInfo` を返します（全履歴の走査なし）。

```rust
let info = repo.find_commit(some_id)?;
```

### Diff サマリ

2つの `CommitId` 間で変更されたファイルパス（追加・変更・削除）を返します。
パッチテキストは含みません。

```rust
let diff = repo.diff(from_id, to_id)?;
// diff.added, diff.modified, diff.deleted: Vec<PathBuf>
```

### リモート URL の取得

`origin` など設定済みリモートの URL をネットワーク I/O なしで読み取ります。

```rust
let url = repo.remote_url("origin")?;
```

---

## v1.0.0（安定 API）

複数の minor バージョンを経て公開 API が安定したと判断したタイミングで v1.0.0 をリリースします。
v1.0.0 以降は major バージョンなしに破壊的変更を行いません。

目安となる条件：

- `find_commit` / `diff` / annotated タグが揃っている
- 実際のダウンストリームクレートで 2 バージョン以上の運用実績がある
- 公開 API に「将来削除したいもの」が残っていない

---

## 長期 / 探索的

### 非同期ファサード（`async` feature flag）

`tokio::task::spawn_blocking` を用いて非同期コンテキスト向けのラッパーを提供します。
デフォルト off で、既存の同期 API を維持します。

### Jujutsu (jj) バックエンド

拡張依頼書が言及する `vcs::jj` アダプタ。
jj の普及度を見ながら、`Repository` の同一インタフェースで切り替えられる形を目指します。

---

## 設計上スコープ外

| 項目 | 理由 |
|---|---|
| コミット・マージ・プッシュ | endringer は読み取り主体。タグ操作以外の書き込みは対象外 |
| 設定ファイルの永続化 | アプリケーション層の責務 |
| UI / i18n | ライブラリの責務ではない |
| 定期ポーリング | 呼び出し側の責務（例: iced の `Subscription`） |
| 認証・クレデンシャル管理 | gix を通じて OS クレデンシャルストアに委任 |

[v0.7.1]: https://github.com/example/endringer/releases/tag/v0.7.1
[v0.8.0]: https://github.com/example/endringer/releases/tag/v0.8.0
[v0.8.1]: https://github.com/example/endringer/releases/tag/v0.8.1
