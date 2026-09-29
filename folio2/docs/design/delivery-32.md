# 設計: 便 32 — 図の本体の class を道具の 5 型の実測で 39 語に閉じ、見本の図の数の pin を外す（NFR2・ADR-4 決定 (3)）

- 要件: NFR2（部品目録に無い class は rules 行 R-3 の値 0・色は design token 由来 100%）/ FR15（図の本体の意味の属性と class を落とさない）
- 条: P-2.3（色の元の値は 1 か所）/ P-2.4（閉じた一覧）/ P-10.1（凍結 anchor）
- 判断の記録: ADR-4 決定 (3)（図の本体の意味を表す class を部品目録に載せ、色・線の太さは folio の design token で塗る）。便 31（f2-648.46）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「進めて」（図の見本の後の残りの整理）・「手順図が非エンジニア二わかりやすいが順序図はエンジニアにわかりやすいのでそのように書いて両方あったらいいんじゃない？」（図の対・欄の決まりの figures_note に指針として記した・床は数えない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ag が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

便 31 で図の本体の意味の class 33 語を部品目録（figure_body_classes）と folio.css に載せたが、その一覧は構成図 1 本の実測から採ったもので、道具の他の型（手順図・順序図・流れ図・状態図）が出す class を含まない。本便は (1) 道具の 5 型の見本の出力から class を実測し、一覧に無い 6 語を部品目録と folio.css に足して 39 語で閉じる、(2) 見本の設計ノート example.yaml の図の数を 1 に固定している歯を外し、図の数を正本から数える形にする、の 2 つ。生成器（face_note.rs・figure.rs）は触らない。

planner の実測（2026-09-19・main b564552）: 道具の写し vendor/archify の見本 11 本（手順図 3・順序図 2・流れ図 2・状態図 2・構成図 2・道具の repo の examples/）と folio2 の見本の図 3 枚（手順図・順序図・構成図）を deliver（showcase）に掛け、出力の svg の class を集めると 39 語で、部品目録の 33 語に無いのは 6 語: c-region（構成図の領域の枠）・c-security-group（手順図の帯や群の variant security / exception の枠）・t-frontend・t-external・t-cloud・t-database（文字の種別の色）。道具の template.html の様式は `.c-region` が fill 薄い黄・stroke cloud の色・stroke-dasharray 8,4、`.c-security-group` が fill transparent・stroke security の色・stroke-dasharray 4,4、`.t-<種別>` が fill 種別の stroke の色。folio2 の見本の手順図は帯の variant exception を付けると c-security-group を出すので便 31 の時点では外した（本便の後に戻せる）。tests/face_note.rs（正規化 859）の歯 face_note_on_the_real_source_uses_only_catalogued_figure_classes は parts.json の 5 一覧の和を 33 と固定して数え（362 行）、歯 face_note_census_on_the_real_source_counts_and_verbatims と face_note_on_the_real_source_embeds_the_figures は example.yaml の図の数を 1 と固定して数える（281 行・424 行）。folio.css は正規化 858・parts.json は 367。凍結目録 tests/fixtures/floor/parts-catalog.json は figure_body_classes を持たず、build.rs も読まない（部品目録の faces・figure_types・shelf_types・style_props だけ）。

(a) 部品目録（`design-intent/preview/parts.json`）。figure_body_classes の node_kind の末尾に c-region・c-security-group の 2 語、text_role の末尾に t-frontend・t-external・t-cloud・t-database の 4 語を足す（この順）。source の字に「+ 道具の 5 型の見本 11 本と folio2 の見本 3 枚の出力（2026-09-19・39 語で閉じる・足した 6 語）」を添える。他の鍵は不変。

(b) 様式（`design-intent/preview/folio.css`）。便 31 の図の規則の区間に足す（token は便 31 の `--fig-*` を使い新しい token は足さない・色の直書き 0）: `.c-region` は fill が cloud-fill・stroke が cloud・stroke-dasharray 8,4（`.c-mask` の規則の直後）／`.c-security-group` は fill none・stroke が security・stroke-dasharray 4,4（その直後）／`.t-frontend` `.t-external` `.t-cloud` `.t-database` は fill が種別の token（`.t-security` の規則の直後・この順）。選択子は便 31 と同じ figure-panel の属性選択子の下。

(c) 歯（`crates/folio/tests/face_note.rs`・関数名は face_note を含める・`--test face_note` の scope）。
1. 図の class の一覧の歯: 5 一覧の和の数を 33 でなく 39 と数え、6 語が在ることを名指しで見る。
2. 図の数の pin: 例の 281 行と 424 行の「1」を外し、図の数は正本 example.yaml の figures の長さで数える（1 以上であることだけ固定・図を足しても歯が落ちない）。census の figure-panel の数・svg の数は既に figures.len() で数えているので不変。
3. 他の歯は本文も期待も不変（凍結 expected-note.html は不変・css の追加は面の byte に出ない）。

(d) 便 31 までの形との接続: 新規 file は無い。`design-intent/preview/parts.json` は 6 語と source の字・`design-intent/preview/folio.css` は 6 規則・`crates/folio/tests/face_note.rs` は数の直し 3 か所。`face_note.rs`・`figure.rs`・`parts.rs`・凍結目録・`expected-note.html`・`example.yaml`・`figures.yaml`・`vendor/` は触らない。外部 crate は増えない。size は S。

## 2. 範囲

- 入れる: 6 語の追加（目録と様式）・歯の pin の解除。
- 入れない: 生成器の変更・見本の図の追加（planner の別 PR・手順図の帯の variant exception を戻す）・道具の版上げ・図の対の床の検査（指針のみ・欄の決まりの figures_note）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| list | 一覧 | 39 語 |
| rules | 規則 | 6 規則 |
| unpin | 歯 | 図の数と語の数を正本から |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ag"
title = "図の本体の class を道具の 5 型の実測で 39 語に閉じ（目録 + 様式）、見本の図の数と語の数の pin を外す"
req = ["NFR2", "FR15"]
section = "1"
write-set = ["design-intent/preview/parts.json", "design-intent/preview/folio.css", "crates/folio/tests/face_note.rs"]
verify = ["cargo nextest run -p folio --test face_note face_note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_note の歯（39 語と 6 語の名指し・図の数は正本から・他は期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
