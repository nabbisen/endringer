# endringer — 開発者向けドキュメント

このドキュメントは endringer の設計判断・アーキテクチャ・モジュール責務について記述します。
エンドユーザー向けの使用方法は [README.md](../README.md) を参照してください。

---

## 設計思想

endringer は UNIX 哲学に則り、**一つのことを正しく行う**ライブラリです。

- **VCS の状態を読み取る**（ブランチ・コミット・タグ・HEAD スナップショット）
- 書き込みは軽量タグの作成・削除のみ（コミット・マージ・プッシュは対象外）
- 設定ファイルの永続化・スケジューリング・UI・i18n は呼び出し側の責務

### 境界原則

> gix を含む VCS 実装は endringer 内に完全に閉じる。
> 下流クレートは `Repository` と `types::*` のみを知れば足りる。

この原則を守るために v0.8.0 で行った変更：

| 変更 | 理由 |
|---|---|
| `CommitId` newtype 導入 | `gix::ObjectId` が公開 API に漏れていた |
| `repository::branch` / `repository::commit` を `pub(crate)` に変更 | 内部関数が直接呼ばれていた |
| `Repository` に `create_tag` / `delete_tag` / `list_tags` / `log_since` を追加 | `VcsAdapter` レイヤーに相当する操作が未公開だった |

---

## モジュール構成

```
src/
  lib.rs                   公開エントリポイント、crate-level doc
  types.rs                 公開型定義 (CommitId, BranchInfo, CommitInfo, StatusDigest, TagInfo)
  util.rs                  内部ユーティリティ (pub(crate))
  repository.rs            Repository 構造体・公開メソッド・constructor
  repository/
    branch.rs              local_branches / remote_branches / list_commits / log_since (pub(crate))
    branch/util.rs         refs プレフィクス走査ヘルパー (pub(super))
    commit.rs              status_digest (pub(crate))
    tag.rs                 list_tags / create_tag / delete_tag (pub(crate))
```

### 可視性ルール

| スコープ | 用途 |
|---|---|
| `pub` | `types::*`、`repository::Repository`、`repository::repository()`、`commit_id_to_short_id()` |
| `pub(crate)` | `repository::{branch, commit, tag}` — VCS 実装の内部モジュール |
| `pub(super)` | `repository/branch/util` — サブモジュール内の共有ヘルパー |
| private | `util`、gix の生の操作 |

---

## 型設計

### `CommitId`

```rust
pub struct CommitId(pub(crate) gix::ObjectId);
```

- `gix::ObjectId` を内包するが、フィールドは `pub(crate)` なので外部から見えない
- `Display` → 40 文字 hex
- `CommitId::short()` → 先頭 7 文字（慣習的省略形）
- 下流クレートが `gix` に依存する必要がない

### `TagInfo`

軽量タグ・注釈付きタグのどちらも peel-to-commit して返す。タグオブジェクト自体（メッセージ・署名）はこのバージョンでは公開していない。

---

## 実装ノート

### タグの作成 (`create_tag`)

現在 HEAD を指す軽量タグを作成します。
`gix::refs::transaction::PreviousValue::MustNotExist` を指定しているため、同名タグが存在する場合はエラーになります。

### `log_since(since, until)`

Git の歴史は DAG であり、コミットタイムスタンプは著者が任意に設定できます。
そのため全祖先を走査し、`since <= timestamp <= until` を満たすコミットをフィルタリングします。
大規模リポジトリでは `O(n)` の走査コストがかかります。

### doctest の制限

Edition 2024 + rustdoc 1.91 では `--check-cfg` に `-Z unstable-options` が必要なため、
`cargo test` の doctest が失敗します。これは toolchain の既知制約です。
ライブラリユニットテストは `cargo test --lib` で全件通過します。

---

## バージョン方針

このライブラリは [Semantic Versioning](https://semver.org/) に従います。

v0.8.0 は `gix::ObjectId` の除去という破壊的変更を含むため、v0.7.x から直接移行するには
`commit_id: gix::ObjectId` を `commit_id: endringer::types::CommitId` に置き換えてください。

---

## 依存クレート

| クレート | 用途 |
|---|---|
| `gix` | Git リポジトリの読み書き（内部のみ） |
| `anyhow` | エラー型の統一 |

`gix` は公開 API に露出しないため、下流クレートの依存ツリーには直接現れません
（Cargo.lock には推移的依存として含まれます）。
