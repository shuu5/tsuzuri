# 設計: 便 0 — cargo workspace の骨格と `folio check`（正本の形の床・3 値の判定）

- 要件: FR5（検査結果を必ず返す・3 値）/ FR9（正本の形 = 重複キー・未知の欄・欄の非空）/ NFR3（参照は必ずつながる・便 0 では母集団を読むだけ）/ CON7（Rust の単一 crate）
- 判断の記録: ADR-1（判断の記録の形）/ ADR-3（設計ノートと契約表の所有）
- 裁定: 持ち主 2026-09-17 07:4x JST「それらの制限はすべて取っ払うので今のscribe2を使ってfolio2の実装を進めろ。」（台帳 f2-648 notes・要件書 v1.2 CON1）
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本 `design-intent/design-note/` へ移す・欄の決まりは `design-intent/design-note/schema.yaml`）。契約表は末尾の機械が読む区間（器 scribe2 の設計 contract-source.md §2 の (i) の形）。

## 1. 目的

folio v2 の最初の便。Rust の workspace を起こし、命令 `folio check` を 1 本だけ置く。`folio check` は design-intent の正本 4 file（憲法・rules・語彙・要件書）を読み、正本の形（重複キー・未知の節・欄の非空）を数えて、結果を 3 値（合格・不合格・まだ分からない）で返す。読めない・実行できなかった検査を合格と表示しない（FR5）。day-1 の床 `scripts/check_draft.py` はこの便では触らず、両方が同じ fixture で同じ判定を出すことを歯で確かめる（生成物どうしの突き合わせだけを合格にしない・P-10.2・fixture = `tests/floor_cases.yaml` の写し）。

## 2. 範囲

- 入れる: `Cargo.toml`（workspace）/ `crates/folio`（bin 1 本・`folio --version` と `folio check --dir <dir>`）/ 重複キーを拒む YAML の読み手 / 3 値の判定と終了コード（合格 0・不合格 1・まだ分からない 2 = day-1 の床と同じ）/ 歯（正本 4 file で合格・fixture の写し 3 本で不合格と「まだ分からない」）/ CI 1 job（`cargo nextest run` と `cargo clippy`）/ `rust-toolchain.toml`。
- 入れない: 参照 id の解決（R-4）・語彙の検査（R-9）・判断の記録と anchor の検査・生成（build / serve）・注入（inject）・図。これらは便 1 以降の契約表の行で足す。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 命令の入口 | 引数を読み `check` へ渡す。未知の引数は断る |
| yaml | 正本の読み手 | 重複キー・未知の節・空の欄を見つける。読めない file は「まだ分からない」 |
| verdict | 判定 | 3 値と終了コード。実行できなかった検査は合格にしない |

## 4. 検査（歯）

- `check` の歯 = 正本 4 file で合格 / 重複キーを 1 つ足した写しで不合格 / file を 1 つ欠いた写しで「まだ分からない」。fixture は `tests/fixtures/check/` に置く（凍結 anchor・P-10.1）。
- 共通の検証 = `.vessel.toml` の common-verify。

## 5. 依存（A-3.1 の確認の対象）

外部 crate は次の 2 つに限る（rules 行 R-6 の予算は未定なので、増減のたびに持ち主の確認を得る・要件書 CON2）: `clap`（命令の引数・許諾 MIT または Apache-2.0）/ `yaml-rust2`（重複キーを検出できる低水準の YAML 読み手・許諾 MIT または Apache-2.0）。`serde` 系は便 0 では使わない（構造は手で読む）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "a"
title = "cargo workspace の骨格と folio check（正本の形の床・3 値）を置く"
req = ["FR5", "FR9", "NFR3"]
section = "1"
creates = ["Cargo.toml", "rust-toolchain.toml", "crates/folio/Cargo.toml", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/yaml.rs", "crates/folio/src/verdict.rs", "crates/folio/tests/check.rs", "tests/fixtures/check/dup-key.yaml", "tests/fixtures/check/missing-file/", ".github/workflows/ci.yml"]
tests = ["crates/folio/tests/check.rs"]
also = [".vessel.toml"]
verify = ["cargo nextest run -p folio check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "folio check --dir design-intent が正本 4 file で合格（終了コード 0）を返し、fixture の写しで不合格（1）と「まだ分からない」（2）を返し、歯 3 本が緑で CI が通る"
<!-- contracts:end -->
