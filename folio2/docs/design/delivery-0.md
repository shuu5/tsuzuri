# 設計: 便 0 — cargo workspace の骨格と `folio check`（正本の形の床・3 値の判定）

- 要件: FR5（検査結果を必ず返す・3 値）/ FR9（正本の形 = 重複キー・未知の欄・欄の非空）/ CON7（Rust の単一 crate）。NFR3（参照 id の解決）は便 1 で足す（便 0 は正本の形だけ）。
- 判断の記録: ADR-1（判断の記録の形）/ ADR-3（設計ノートと契約表の所有）
- 裁定: 持ち主 2026-09-17 07:4x JST「それらの制限はすべて取っ払うので今のscribe2を使ってfolio2の実装を進めろ。」（台帳 f2-648 notes・要件書 v1.2 CON1）
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本 `design-intent/design-note/` へ移す・欄の決まりは `design-intent/design-note/schema.yaml`）。契約表は末尾の機械が読む区間（器 scribe2 の設計 contract-source.md §2 の (i) の形）。

## 1. 目的

folio v2 の最初の便。Rust の workspace を起こし、命令 `folio check` を 1 本だけ置く。`folio check` は design-intent の正本 4 file（憲法・rules・語彙・要件書）を読み、正本の形（重複キー・未知の節・欄の非空）を数えて、結果を 3 値（合格・不合格・まだ分からない）で返す。読めない・実行できなかった検査を合格と表示しない（FR5）。day-1 の床 `scripts/check_draft.py` はこの便では触らず、両方が同じ入力で同じ終了コードを出すことを突き合わせの歯（parity）で確かめる（生成物どうしの突き合わせだけを合格にしない・P-10.2）。day-1 の床の振る舞い（admin 席の実測 2026-09-17・main 9bc577e）: `python3 scripts/check_draft.py --dir <写し>` で受け、写しは design-intent の全部（adr/ と anchors/ を含む）でかつ版管理（git の 1 commit）の中に在ること。4 file だけの写し・版管理の無い写しは「まだ分からない」（終了コード 2）。終了コードは 合格 0 / 違反 1 / 読めない・測れない 2。床が数える「未知の節・欄の非空」は憲法と判断の記録の側で、要件書・語彙の未知の節や空の欄は数えない。

突き合わせの歯（`crates/folio/tests/parity.rs`）の 5 入力と期待の終了コードはこの節で確定する: 歯の中で design-intent の写し全部（adr/ と anchors/ を含む）を一時 dir に作り `git init` と 1 commit を行い（`tests/run_floor_cases.py` と同じ作り）、変異を 1 つだけ当てる。(1) 変異なし → 0 / (2) `rules.yaml` に重複キーを 1 つ足す → 1 / (3) `constitution.yaml` に schema の top_level に無い節を 1 つ足す → 1 / (4) `constitution.yaml` の条 1 つの plain を空にする → 1 / (5) `constitution.yaml` を欠く → 2。各入力に `python3 scripts/check_draft.py --dir <写し>` と `folio check --dir <写し>` を掛け、終了コードが一致することだけを見る。要件書と語彙には変異を当てない（床が数えないため）。

`folio check` 自身の歯（`crates/folio/tests/check.rs`）の fixture は `tests/fixtures/check/` の 4 file 形で、parity の入力にはしない（床は 4 file の写しを受けない）。変異先: `dup-key/` = rules.yaml に重複キー → 1 / `unknown-section/` = srs.yaml に top_level に無い節 → 1（folio は要件書の未知の節も数える・床は数えない） / `empty-field/` = vocabulary.yaml の 1 語の short を空 → 1（同じく folio だけが数える） / `missing-file/` = constitution.yaml を欠く → 2。

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

- `check` の歯 = 正本 4 file で合格 / `tests/fixtures/check/` の 4 組（変異先と期待は §1 のとおり: dup-key 1 / unknown-section 1 / empty-field 1 / missing-file 2）。fixture は file 1 本ずつ名指す（器の受付は新規 dir を受けない・凍結 anchor・P-10.1）。
- 突き合わせの歯（`crates/folio/tests/parity.rs`）の 5 入力・期待の終了コード・写しの作り方は §1 のとおり（§1 が正本・ここには写さない）。
- 共通の検証 = `.vessel.toml` の common-verify。

## 5. 依存（A-3.1 の確認の対象）

外部 crate は次の 2 つに限る（rules 行 R-6 の予算は未定なので、増減のたびに持ち主の確認を得る・要件書 CON2）: `clap`（命令の引数・許諾 MIT または Apache-2.0）/ `yaml-rust2`（重複キーを検出できる低水準の YAML 読み手・許諾 MIT または Apache-2.0）。`serde` 系は便 0 では使わない（構造は手で読む）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "a"
title = "cargo workspace の骨格と folio check（正本の形の床・3 値）を置く"
req = ["FR5", "FR9"]
section = "1"
creates = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates/folio/Cargo.toml", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/yaml.rs", "crates/folio/src/verdict.rs", "crates/folio/tests/check.rs", "crates/folio/tests/parity.rs", "tests/fixtures/check/dup-key/constitution.yaml", "tests/fixtures/check/dup-key/rules.yaml", "tests/fixtures/check/dup-key/vocabulary.yaml", "tests/fixtures/check/dup-key/srs.yaml", "tests/fixtures/check/unknown-section/constitution.yaml", "tests/fixtures/check/unknown-section/rules.yaml", "tests/fixtures/check/unknown-section/vocabulary.yaml", "tests/fixtures/check/unknown-section/srs.yaml", "tests/fixtures/check/empty-field/constitution.yaml", "tests/fixtures/check/empty-field/rules.yaml", "tests/fixtures/check/empty-field/vocabulary.yaml", "tests/fixtures/check/empty-field/srs.yaml", "tests/fixtures/check/missing-file/rules.yaml", "tests/fixtures/check/missing-file/vocabulary.yaml", "tests/fixtures/check/missing-file/srs.yaml", ".github/workflows/ci.yml"]
tests = ["crates/folio/tests/check.rs", "crates/folio/tests/parity.rs"]
also = [".vessel.toml", ".gitignore"]
verify = ["cargo nextest run -p folio check", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "folio check --dir design-intent が正本で合格（終了コード 0）を返し、tests/fixtures/check/ の写し 3 組で不合格（1）・憲法を欠いた写しで「まだ分からない」（2）を返し、check の歯 5 本と parity の歯（§1 の 5 入力で day-1 の床 scripts/check_draft.py --dir と終了コード 0 / 1 / 1 / 1 / 2 が一致）が緑で CI が通り、target/ が版管理に入らない"
<!-- contracts:end -->
