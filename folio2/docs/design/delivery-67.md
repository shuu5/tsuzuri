# 設計: 便 67 — 図の本体の凡例の見出し語 Legend と言語の宣言を日本語にする（天井の 11 周目の読みやすさ F-3 の是正・FR15）

- 要件: FR15（図の道具の境界・図の本体だけを部品として埋める）/ GOAL2
- 条: P-2.1 / P-6.3
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の読みやすさ F-3（直す）= 図の道具（archify）が描く図の凡例の見出しが英語の Legend のまま面に出る（要件書・設計ノート 2 面・判断の記録 3 面の計 6 面 10 か所）。凡例の中身の名札は一括 4 で日本語にした。見出し語は道具の翻訳表（vendor/archify/renderers/shared/i18n.mjs の legend.title = 英語と中国語）が持ち、型付き記述からは替えられない。図の本体の言語の宣言も lang=en。
- 根拠の判断: 判断の記録 ADR-4 決定 (3) = folio は道具の出力から図の本体だけを取り出し、閲覧の仕掛けは捨てる。見出し語と言語の宣言の置き換えは、図の本体の意味の属性（箱の id・種別・名札・線）を変えない字の置き換えで、決定の範囲の内。道具の写し（vendor/archify・R-15 の要約値で凍結）は触らない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bp が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 置き換え。crates/folio/src/figure.rs（正規化 約 520 行・余地 約 980）の、道具の出力から図の本体を取り出す関数（body）の直後に、決定的な字の置き換えを 2 つ掛ける: ① 凡例の見出しの文字列の要素の中身 Legend を 凡例 に（要素の中身として現れる >Legend< の形だけ・属性や id の中の Legend は触らない）② 図の本体の根の要素の言語の宣言 lang=en を lang=ja に（1 つ目の svg の開始タグの中だけ）。置き換えは figure --write / --check と面の経路（render）の両方に掛かる（同じ関数を通る）。凍結 anchor の照合（anchor_holds・便 60）は道具の出力と写しを比べる段なので、置き換えの前の値で比べる（照合の入力は置き換え前・出力は置き換え後 = anchor の写し tests/fixtures/figure/anchor/body.svg は道具の生の出力のまま触らない）。

(b) 凍結の面の写し。tests/fixtures/face/expected-adr.html・expected-srs.html・expected-note.html・expected-site-adr-2.html（図を持つ面の凍結 anchor）は (a) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。tests/fixtures/figure/anchor/body.svg と spec.json は触らない（(a) の照合の順序による）。

(c) 歯（crates/folio/tests/figure.rs・正規化 約 300 行・関数名は legend_ja_ で始める・今この語で始まる歯は無い）。
1. legend_ja_replaces_the_heading_word: 凍結 anchor の型付き記述（fig-anchor・凡例を持つ）を --write で出した図の本体に >凡例< が在り >Legend< が無い。
2. legend_ja_sets_the_language_to_ja: 同じ出力の 1 つ目の svg の開始タグに lang=ja が在り lang=en が無い。
3. legend_ja_keeps_the_anchor_check_green: 置き換えを掛けても anchor_holds は合う = 改変しない写しで --write が終了 0（便 60 の歯 anchor_holds_on_the_untouched_copy と同じ判定・置き換えの前の値で照合していることの証）。
4. 回帰（期待不変・verify の 2 行目と 3 行目）: tests/figure.rs の既存の歯すべて（凍結 anchor との byte 一致の歯 figure_write_matches_the_frozen_anchor は、比べる相手を「道具の生の出力」でなく「folio の出力」にしている場合は (a) で赤になる。その場合は歯の比べ方を変えず、置き換えの後の出力と body.svg の差が >Legend< と lang= の 2 か所だけであることを確かめる形に、同じ file の中で直す = 契約の範囲の内）・tests/face.rs・tests/face_adr.rs・tests/face_note.rs・tests/site.rs（凍結の写しは (b) で更新）。

(d) 大きさと接続。新規 file は無い。figure.rs（+約 15 行）・tests/figure.rs（+約 45 行）・fixture の面 4 本（再生成）。size S。vendor/archify・design-intent・parts.json は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 見出し語と言語の宣言の置き換え・凍結の面の写しの更新・歯 3 本。
- 入れない: 道具の翻訳表への日本語の追加（道具の写しは R-15 で凍結・道具の版上げの便で扱う）・凡例の中身の名札（一括 4 で正本の側に済み）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| swap | 置き換え | body の直後に >Legend< → >凡例< と lang=en → lang=ja |
| order | 順序 | anchor の照合は置き換えの前の値 |
| fixture | 写し | 図を持つ面の凍結の写し 4 本を再生成 |
| teeth | 歯 | tests/figure.rs に legend_ja_ の 3 本 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bp"
title = "図の本体の凡例の見出し語（要素の中身の Legend）を 凡例 に、言語の宣言 lang=en を lang=ja に決定的に置き換える（凍結 anchor の照合は置き換えの前の値・図を持つ面の凍結の写し 4 本を再生成・天井の 11 周目の読みやすさ F-3）"
req = ["FR15"]
section = "1"
write-set = ["crates/folio/src/figure.rs", "crates/folio/tests/figure.rs", "crates/folio/tests/face.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/site.rs", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test figure legend_ja_", "cargo nextest run -p folio --test figure", "cargo nextest run -p folio --test face --test face_adr --test face_note --test site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "legend_ja_ の歯 3 本（見出し語が 凡例・lang=ja・anchor の照合は緑のまま）が緑、tests/figure.rs の既存の歯が全部緑、図を持つ面の歯（face / face_adr / face_note / site）が全部緑（凍結の写し 4 本は再生成）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
