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

破壊的変更が含まれる場合、CHANGELOG で `Breaking change:` として明示します。

### リリース手順

```sh
# 1. Cargo.toml の version を更新
# 2. CHANGELOG.md に変更内容を記載
# 3. テスト・ビルドが通ることを確認
# 4. リリーススクリプトを実行
./scripts/release.sh

# 5. リモートへ push（スクリプトはローカルのみ）
git push origin master
git push origin v0.8.0

# 6. crates.io へ公開（任意）
cargo publish
```

リリース Tarball は `dist/endringer-v{バージョン}.tar.gz` に生成されます
（例: `dist/endringer-v0.8.0.tar.gz`）。

---

## リリース履歴

| バージョン | リリース日 | 概要 |
|---|---|---|
| [v0.7.1] | 2025 | 初回公開。ブランチ一覧・コミット履歴・ステータスダイジェスト |
| [v0.8.0] | 2026-05-04 | `CommitId` newtype、タグ操作、`log_since`、公開 API の整合 |

---

## v0.9.0（次期リリース）

### Annotated タグのサポート

`create_tag` は現在、軽量タグのみを作成します。
リリースワークフローで一般的な annotated タグ（タッガー・メッセージ付き）を追加します。

```rust
// 予定 API
repo.create_annotated_tag("v1.0.0", "Release v1.0.0")?;
```

### `CommitId::from_hex`

既知の SHA-1 hex 文字列から `CommitId` を構築し、履歴をたどらずに特定コミットを参照できるようにします。

```rust
let id = CommitId::from_hex("a1b2c3d4...")?;
```

### `list_tags` / `list_commits` のソート

現在は ref ストア順で返します。タイムスタンプ順・名前順の並べ替えオプションを追加します。

### `CommitId` の `PartialOrd` / `Ord`

コレクション操作（ソート・重複排除）に備えて `Ord` を実装します。

---

## v0.10.0

### `CommitInfo` の拡張

- committer identity を author とは別に公開
- コミットメッセージの body（件名以降）をオプションフィールドとして追加

### `Repository::find_commit(id)`

`CommitId` を指定して単一の `CommitInfo` を返します（全履歴の走査なし）。

### Diff サマリ

2つの `CommitId` 間で変更されたファイルパス（追加・変更・削除）を返します。
パッチテキストは含みません。

```rust
let diff = repo.diff(from_id, to_id)?;
// diff.added, diff.modified, diff.deleted: Vec<PathBuf>
```

### リモート URL の取得

`origin` など設定済みリモートの URL をネットワーク I/O なしで読み取ります。

---

## v1.0.0（安定 API）

複数の minor バージョンを経て公開 API が安定したと判断したタイミングで v1.0.0 をリリースします。

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
