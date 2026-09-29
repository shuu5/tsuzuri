# 設計: 便 61 — 設計ノートの床が meta・節・図の行の未知の欄を数える（FR9 の未達の節・天井の 11 周目の実態 F-1 の是正）

- 要件: FR9（設計ノートの正本の形は構造の床で数える）/ FR5（3 値の判定）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-5.1 / P-4.1
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の実態 F-1（止める）= 要件書 FR9 の規範文「正本の形（重複キー・未知の欄・欄の非空）は構造の床（FR5）で数える。」に対し、設計ノートの写しに未知の欄を 1 つ足して folio check を撃つと、最上位と表の行は違反 1 になるが、meta・節（sections の行）・図の行（figures の行）は違反 0 の合格のまま（判断の記録の側は 3 か所とも落ちる）。欄の決まり design-intent/design-note/schema.yaml はこの 3 階層とも閉じた欄の集合（required と optional）として宣言している。
- 根拠の判断: 新しい判断は無い。欄の集合は実装の定数 DOC_META・SECTION・FIGURE_ENTRY（crates/folio/src/note.rs・欄の決まりの生成区間の正本）が既に持ち、本便はそれを数えるだけ。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bj が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 数える式。crates/folio/src/note.rs（正規化 1281 行・余地 219）の設計ノートの床で、次の 3 階層について、欄の表に在る鍵が required と optional の和集合に無ければ違反 1 件を出す: ① meta（check_meta・定数 DOC_META）② sections の各行（check_section・定数 SECTION。by_type の required と forbid は既存のまま・未知の欄は SECTION の和集合で見る）③ figures の各行（check_figures・定数 FIGURE_ENTRY）。種別は既存の「未知の欄」・文言は要件書の図の行の検査（crates/folio/src/check.rs の SRS_FIGURE_REQUIRED の側）と同じ形「{file}: {at} の未知の欄「{key}」」（{at} は meta / sections[n] / figures[id] の道・{ } は値）。同じ行に未知の欄が 2 つ在れば 2 件。鍵の照合は字面の一致（大文字小文字の揺れも未知）。表の行（parts / ports / fields / teeth / contract の rows）と最上位は既に数えているので触らない。実の設計ノート 3 本（example・figures・schema の見本）は 2026-09-21 の実測で未知の欄 0 なので、本便で床の結果は変わらない。

(b) 歯（crates/folio/tests/note.rs・正規化 411 行・関数名は note_unknown_field_ で始める・今この語で始まる歯は無い）。写しは既存の Work::new（実の design-intent の写し）と mutate_file（字の置換）で作り、新しい fixture file は置かない。
1. note_unknown_field_in_meta_fails: 写しの design-note/example.yaml の meta に鍵 mystery: x を 1 つ足す → 終了 1・違反はその 1 件だけ（assert_single_violation・種別 note・文言に「meta の未知の欄「mystery」」を含む）。
2. note_unknown_field_in_section_fails: 同じ写しの sections の行 1 つに mystery: x を足す → 終了 1・違反 1 件・文言に「未知の欄「mystery」」と sections の道を含む。
3. note_unknown_field_in_figure_fails: 写しの design-note/figures.yaml の figures の行 1 つに mystery: x を足す → 終了 1・違反 1 件・文言に「figures」と「未知の欄「mystery」」を含む。
4. note_unknown_field_none_on_the_canonical_copy: 変えない写しが合格（違反 0）= 実の 3 本に未知の欄が無いことを、式を足した後の床で確かめる。
5. 回帰（期待不変・verify の 2 行目）: 既存の note_ の歯すべて（正本の写しの合格・節の型・器の導出 file・契約表の欄）。
6. tests/schema.rs の歯（filter schema・本文は不変・verify の 3 行目）が緑 = 欄の集合を変えないので欄の決まりの生成区間が正本と一致のまま。

(c) 大きさと接続。新規 file は無い。note.rs（+約 25 行・余地 219 → 約 194）・tests/note.rs（+約 50 行）・tests/schema.rs は本文不変（検証の範囲に入るので write-set に置く）。size S。check.rs・adr.rs・schema の生成区間（欄の集合は変えないので導出物は不変 = folio schema --check は一致のまま）・design-intent・fixture・CI は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: meta・節・図の行の未知の欄の数え・歯 4 本。
- 入れない: 欄の集合の変更（生成区間の改訂は別の判断）・判断の記録の側（既に数える）・要件書 FR9 の注（席の一括）・要件書の側の同じ検査（check.rs は既に数える）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| meta | 式 ① | check_meta に DOC_META の和集合で未知の欄 |
| section | 式 ② | check_section に SECTION の和集合で未知の欄 |
| figure | 式 ③ | check_figures に FIGURE_ENTRY の和集合で未知の欄 |
| teeth | 歯 | tests/note.rs に note_unknown_field_ の 4 本 |

## 4. 検査（歯）

§1 (b) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bj"
title = "設計ノートの床（note.rs）が meta・節（sections の行）・図（figures の行）の未知の欄を、欄の決まりの定数 DOC_META / SECTION / FIGURE_ENTRY の required と optional の和集合で数えて違反にする（種別 未知の欄・文言は要件書の図の行と同形・FR9 の未達の節・天井の 11 周目の実態 F-1）"
req = ["FR9", "FR5"]
section = "1"
write-set = ["crates/folio/src/note.rs", "crates/folio/tests/note.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test note note_unknown_field_", "cargo nextest run -p folio --test note note_", "cargo nextest run -p folio --test schema schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "note_unknown_field_ の歯 4 本（meta・節・図の行に未知の欄を 1 つ足した写しがそれぞれ違反 1 件で落ちる・変えない写しは合格）が緑、既存の note_ の歯が全部緑、schema の歯が緑（欄の決まりの生成区間は一致のまま）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
