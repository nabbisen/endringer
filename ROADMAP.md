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
| [v0.9.0] | 2026-05-04 | `CommitId::from_hex`、`SortOrder`、`list_commits_sorted`、`list_tags_sorted`、annotated タグ |
| [v0.10.0] | 2026-05-04 | `CommitInfo` コミッター情報、`find_commit`、`diff`、`remote_url` |
| [v0.11.0] | 2026-05-04 | `async` feature flag (`AsyncRepository`)、Jujutsu バックエンド (`JjBackend`)、`VcsBackend` trait |

---

## v0.9.0 ✅ リリース済み（2026-05-04）

### Annotated タグのサポート ✅

`create_annotated_tag(name, message)` を追加。tagger identity は git config から自動取得。

### `CommitId::from_hex` ✅

40文字の hex 文字列から `CommitId` を構築。失敗時は `CommitIdFromHexError` を返す。

### `list_commits_sorted` / `list_tags_sorted` ✅

`SortOrder::NewestFirst`、`OldestFirst`、`ByName` の3種類のソートを実装。

---

## v0.10.0 ✅ リリース済み（2026-05-04）

### `CommitInfo` のコミッター情報 ✅

`committer: String` と `committer_timestamp: SystemTime` フィールドを追加。
**Breaking change**: 直接構築コードは更新が必要。

### `Repository::find_commit(id)` ✅

`CommitId` → `CommitInfo` の O(1) オブジェクト DB ルックアップ（履歴走査なし）。

### Diff サマリ ✅

`Repository::diff(from, to)` が `DiffSummary { added, modified, deleted }` を返す。
リネームは delete + add として報告。パッチテキストなし。

### リモート URL の取得 ✅

`Repository::remote_url(name)` が `Option<String>` を返す（ネットワーク I/O なし）。

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

### 非同期ファサード（`async` feature flag） ✅ v0.11.0 にて実装済み

`tokio::task::spawn_blocking` を用いて非同期コンテキスト向けの `AsyncRepository` を提供。
`Cargo.toml` に `endringer = { version = "0.11", features = ["async"] }` を追記するだけで有効化。

### Jujutsu (jj) バックエンド ✅ v0.11.0 にて実装済み

`jj_repository(path)` で Jujutsu リポジトリを開ける。
`VcsBackend` trait により Git・jj どちらも同一の `Repository` API で操作可能。
jj バイナリが `$PATH` にあれば動作（`jj-lib` クレート依存なし）。

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
