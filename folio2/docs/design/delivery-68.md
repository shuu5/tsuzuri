# 設計: 便 68 — 密度 profile の閉じた一覧を部品目録に載せ、設計ノートの床はそこから読む（天井の 12 周目の整合 F-2 の是正・P-2.4・FR9）

- 要件: FR9（設計ノートを 1 つの型 = 密度 profile の 1 行で生成する）/ GOAL1
- 条: P-2.4 / P-5.1 / P-6.3
- 出所: 天井の 12 周目（2026-09-21・main bbd2a05）の整合 F-2（止める・反証 支持）= 規範文 P-2.4 は「部品・図の型・密度 profile は閉じた一覧（部品目録）で持ち」と置き場を名指すが、密度 profile の閉じた一覧は部品目録（design-intent/preview/parts.json とそこから組み立て時に導出する catalog）に無く、設計ノートの床の定数（crates/folio/src/note.rs の PROFILE_ENUM = design-note の 1 つ）だけが持つ。条が名指す置き場の外に型の一覧が在る。
- 根拠の判断: 判断は無い（条 P-2.4 のとおりの置き場へ移すだけ・条の改訂は要らない）。値は変えない（design-note の 1 つ・ADR-3 決定 (1)）。一覧の正本を部品目録に 1 つにし、設計ノートの床の値域と欄の決まりの生成区間（design-note/schema.yaml の profile_enum）はそこから導出する（P-6.3）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bq が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 部品目録。design-intent/preview/parts.json（部品目録の正本・組み立て時に build.rs が読む）の最上位に鍵 profile_enum を足し、値は ["design-note"] の 1 つ（figure_type_enum・shelf_type_enum と同じ形の文字列の一覧）。同じ file の note の末尾に 1 文「2026-09-21（便 68・P-2.4）: 密度 profile の閉じた一覧 profile_enum を載せた（値は設計ノートの床の定数から移した・1 つ）」を足す。ほかの鍵と値は触らない。

(b) 導出。crates/folio/build.rs（正規化 約 415 行）の parts_catalog は profile_enum を string_list で読み、catalog に `pub const PROFILES: &[&str]`（目録の順・注釈「密度 profile の閉じた一覧（部品目録の profile_enum・目録の順）」）を出す。鍵が無い・文字列の一覧でないは Err（ほかの一覧と同じ・fail-closed）。

(c) 読む側。crates/folio/src/note.rs（正規化 約 1,295 行・余地 約 205）の定数 PROFILE_ENUM を消し、床の木の profile_enum と meta.profile の値域の判定の 2 か所は crate::parts::catalog::PROFILES を読む。値は同じなので design-intent/design-note/schema.yaml の生成区間は 1 byte も変わらない（folio schema --check は合格のまま・凍結 anchor tests/fixtures/schema/note-region.txt は触らない・その不変を測る歯の置き場 crates/folio/tests/schema.rs は本文不変のまま write-set に置く）。crates/folio/src/parts.rs（正規化 約 630 行）の print_catalog は末尾に "profiles":[...] を足し（style_props の後）、catalog_matches（parts --check）は parts.json の profile_enum と PROFILES の一致も見る（ほかの 3 つの一覧と同じ形）。

(d) 歯（関数名は profiles_ で始める・今この語で始まる歯は無い）。
1. profiles_are_in_the_catalog_print（crates/folio/tests/parts.rs・正規化 約 430 行）: folio parts --print の出力に "profiles":["design-note"] が在る。
2. profiles_print_matches_the_parts_json（同）: 出力の profiles の一覧と design-intent/preview/parts.json の profile_enum（歯が file を読んで取る）が同じ順で同じ。
3. profiles_gate_the_note_meta（crates/folio/tests/note.rs・正規化 約 461 行）: 設計ノートの写しの meta.profile を一覧に無い値（design-notex）に変えて folio check を当てると 1 で終わり、標準出力に「meta.profile」と「一覧に無い」が在る（一覧を部品目録へ移しても床が同じに落ちる証）。
4. 回帰（期待不変・verify の 2 行目）: tests/parts.rs・tests/note.rs・tests/schema.rs の既存の歯すべて。

(e) 大きさと接続。新規 file は無い。crates/folio/tests/schema.rs は本文不変のまま write-set に置く（verify の --test schema の scope・生成区間の不変を測る歯の置き場）。parts.json（+2 行）・build.rs（+約 10 行）・parts.rs（+約 12 行）・note.rs（−3 行 +2 行）・tests/parts.rs（+約 30 行）・tests/note.rs（+約 15 行）。size S。外部 crate は増やさない。憲法 P-2 の機構の注と語彙の部品目録の項（「部品目録には載っていない」の文）は設計文書の側で席が一括 6 で直す（この便では触らない）。

## 2. 範囲

- 入れる: 部品目録の鍵・導出の定数・読む側の付け替え・parts --print / --check への追加・歯 3 本。
- 入れない: 密度 profile の値の追加（1 つのまま）・面の生成器の変更・設計ノートの欄の決まりの生成区間の変更（値が同じなので変わらない）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| key | 鍵 | parts.json の profile_enum |
| derive | 導出 | build.rs → catalog::PROFILES |
| read | 読む側 | note.rs の値域と床の木・parts.rs の print / check |
| teeth | 歯 | tests/parts.rs に 2 本・tests/note.rs に 1 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bq"
title = "密度 profile の閉じた一覧 profile_enum を部品目録 parts.json に載せ、build.rs が catalog::PROFILES に導出し、設計ノートの床の値域と生成区間・parts --print / --check はそこから読む（値は design-note の 1 つのまま・天井の 12 周目の整合 F-2）"
req = ["FR9"]
section = "1"
write-set = ["design-intent/preview/parts.json", "crates/folio/build.rs", "crates/folio/src/parts.rs", "crates/folio/src/note.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/note.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test parts --test note profiles_", "cargo nextest run -p folio --test schema --test parts --test note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "profiles_ の歯 3 本（print に在る・parts.json と同じ・meta.profile の門が同じに落ちる）が緑、tests/parts.rs・tests/note.rs・tests/schema.rs の既存の歯が全部緑（design-note/schema.yaml は不変）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
