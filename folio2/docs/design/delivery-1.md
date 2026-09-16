# 設計: 便 1 — `folio check` に参照 id の解決（R-4）と憲法の件数を足す

- 要件: NFR3（参照は必ずつながる・内部 3 空間の未解決 0）/ AC6（未解決が 0）/ FR5（3 値・実行できなかった検査を合格にしない）
- 判断の記録: ADR-3 決定 (6)（参照 id の床の母集団は正本 4 file・内部 3 空間・広げない）
- 前提: 便 0（main 53b4c83・`folio check` = 正本の形の床）。裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 b が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）: 器の touches の閉包は閉じた enum + const slice + 網羅 match の形にしか効かず、folio の素の struct（Report）では 0 file になる（審査 FAIL 2026-09-17・run …T230924Z）。

## 1. 目的と中身

便 0 の `folio check`（正本 4 file の形の床・3 値・終了コード 合格 0 / 違反 1 / 読めない 2）に、検査を 3 つ足す。day-1 の床 `scripts/check_draft.py` の同じ検査と同じ式にし、この便では床の script を触らない。

(a) 参照 id の解決（rules 行 R-4 の前半・要件書 NFR3 / AC6）。母集団 = 正本 4 file の全欄の文字列（憲法は schema 節を除き、meta の changes_from_ で始まる欄を除く。rules は schema 節を除く。語彙と要件書は全欄）。参照 id の形 = 要件書の id（GOAL / FR / NFR / AC / CON + 数字）・条 id（P / A / N + 「-」+ 数字・枝番「.数字」付きも含む）・rules 行 id（R / D + 「-」+ 数字）で、英数字と「-」が前後に続かないもの。解決先 = 要件書の goals / requirements / nonfunctional / acceptance / constraints / actors / outputs の id、憲法の条 id と規範文 id、rules 行の id。解決できない id が 1 つでも在れば 違反（終了コード 1）。判断の記録の id（ADR-n）と凍結 anchor に在った過去の id は便 1 では解かない（便 2 以降・母集団は広げない）。

(b) rules 行の逆参照（R-4 の後半・双方向）。各 rules 行 id は、少なくとも 1 つの条の relations.rules か規範文の本文に現れること。各 rules 行の article 欄が指す条の relations.rules にその行 id が在ること。どちらかが欠ければ 違反。憲法の relations の名前空間は reqs / rules / articles / sections の 4 つだけで、他の名前空間が在れば 違反。

(c) 憲法の件数。meta.counts の always / ask-first / never が条の tier の実数と一致すること。違えば 違反。

読めない file・型が違う節は「まだ分からない」（終了コード 2・合格にしない）。

便 0 が置いた形（admin 席の実測・main 8f7f238）: `crates/folio/` は bin だけで library の入口の file は置かず、`src/main.rs` が `mod check; mod verdict; mod yaml;` を宣言する。便 1 は `mod refs;` を `main.rs` に足し、`check.rs` の `check_dir` から `refs` を呼ぶ。integration test は binary 経由（`Command::new` に `env!` の `CARGO_BIN_EXE_folio` を渡す形・`tests/check.rs` と `tests/parity.rs` と同じ形）で、`tests/refs.rs` も同じ形にする。外部 crate は増やさない（`Cargo.toml` の依存は clap と yaml-rust2 の 2 つのまま・`Cargo.toml` と `Cargo.lock` は触らない）。参照 id の形は正規表現を使わず文字の走査で判定する。新規の fixture dir（`tests/fixtures/refs/` の 3 組）は file を 1 本ずつ `+` で write-set に列挙してあり、器の受付はそれで通る（dir 項目は受けない）。

「正本で合格（終了コード 0）」を測る歯は、便 0 の parity の入力 (1)（変異なし → 床 0 / folio 0・`tests/parity.rs` の `parity_unmutated_passes`）で、便 1 でも同じ file の便 0 の歯 5 本（(1) 変異なし 0 / (2) rules.yaml の重複キー 1 / (3) constitution.yaml の未知の節 1 / (4) constitution.yaml の条の plain 空 1 / (5) constitution.yaml を欠く 2）をそのまま残す。便 1 が足す parity の入力 (6)〜(9) は期待がすべて 1 で、それだけでよい。

便 0 の fixture（`tests/fixtures/check/dup-key/constitution.yaml`・`tests/fixtures/check/unknown-section/constitution.yaml`・`tests/fixtures/check/empty-field/constitution.yaml`）は meta.counts と条の relations.rules を持たない最小形なので、便 1 の (b)(c) を通すと違反・まだ分からないが増え、便 0 の歯 `crates/folio/tests/check.rs`（各 1 違反・終了コード 1）が赤になる。便 1 はこの 3 file に meta.counts（tier の実数）と P-1 の relations: {rules: [R-1, D-1]}（fixture の rules 行 2 つの article が P-1）を足し、fixture を便 1 の正本の形に追随させる。便 0 の歯の期待は変えず、`crates/folio/tests/check.rs` は触らない（便 0 の 3 値の期待は不変）。

突き合わせの歯（`crates/folio/tests/parity.rs`・便 0 の 5 入力に足す）: 歯の中で design-intent の写し全部（adr/ と anchors/ を含む）を一時 dir に作り `git init` と 1 commit を行い、変異を 1 つだけ当てて `python3 scripts/check_draft.py --dir <写し>` と `folio check --dir <写し>` の終了コードが一致することを見る。足す入力と期待の終了コード: (6) `srs.yaml` の FR1 の basis に実在しない条 id を 1 つ足す → 1 / (7) `rules.yaml` の R-3 の article を実在しない条 id にする → 1 / (8) `constitution.yaml` の P-2 の relations.rules から R-3 を外す → 1 / (9) `constitution.yaml` の meta.counts.always を実数と違う値にする → 1。要件書と語彙の欄の非空には変異を当てない（床が数えないため）。

`folio check` 自身の歯（`crates/folio/tests/refs.rs`）の fixture は `tests/fixtures/refs/` の 4 file 形（正本 4 file の写しに変異 1 つ・parity の入力にはしない）。変異先と期待: `dangling-id/` = srs.yaml の FR1 の basis に実在しない条 id → 1 / `orphan-rule/` = constitution.yaml の P-2 の relations.rules から R-3 を外す → 1 / `bad-counts/` = constitution.yaml の meta.counts.always を実数と違う値に → 1。

## 2. 範囲

- 入れる: `crates/folio/src/refs.rs`（(a)(b)(c)）と `check.rs` からの呼び出し・歯 `refs.rs`・parity の 4 入力・fixture 3 組・便 0 の fixture 3 file への meta.counts / relations.rules の追記。
- 入れない: 判断の記録（adr/）の id の解決・anchor の列に在った過去の id・語彙の閉包（R-9）・relations.sections の実在。便 2 以降。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| refs | 参照 id の解決 | 母集団を歩いて id を集め、解決先の集合と突き合わせる |
| reverse | rules 行の逆参照 | 条 → 行・行 → 条 の双方向を数える |
| counts | 憲法の件数 | meta.counts と tier の実数 |

## 4. 検査（歯）

§1 のとおり（refs.rs の 3 組・parity の 4 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（便 0 の clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "b"
title = "folio check に参照 id の解決（R-4）と憲法の件数を足す"
req = ["NFR3", "FR5"]
section = "1"
write-set = ["+crates/folio/src/refs.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/verdict.rs", "+crates/folio/tests/refs.rs", "crates/folio/tests/parity.rs", "tests/fixtures/check/dup-key/constitution.yaml", "tests/fixtures/check/unknown-section/constitution.yaml", "tests/fixtures/check/empty-field/constitution.yaml", "+tests/fixtures/refs/dangling-id/constitution.yaml", "+tests/fixtures/refs/dangling-id/rules.yaml", "+tests/fixtures/refs/dangling-id/vocabulary.yaml", "+tests/fixtures/refs/dangling-id/srs.yaml", "+tests/fixtures/refs/orphan-rule/constitution.yaml", "+tests/fixtures/refs/orphan-rule/rules.yaml", "+tests/fixtures/refs/orphan-rule/vocabulary.yaml", "+tests/fixtures/refs/orphan-rule/srs.yaml", "+tests/fixtures/refs/bad-counts/constitution.yaml", "+tests/fixtures/refs/bad-counts/rules.yaml", "+tests/fixtures/refs/bad-counts/vocabulary.yaml", "+tests/fixtures/refs/bad-counts/srs.yaml"]
verify = ["cargo nextest run -p folio refs", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "folio check --dir design-intent が正本で合格（0）を返し、tests/fixtures/refs/ の 3 組で不合格（1）を返し、refs の歯 3 本と parity の歯（便 0 の 5 入力 + §1 の 4 入力 = 9 入力で day-1 の床と終了コードが一致）が緑で CI が通る"
<!-- contracts:end -->
