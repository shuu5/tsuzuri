# 設計: 便 65 — 判断の記録と設計ノートの面の「前 / 次」を隣の 1 本へつなぐ（天井の 11 周目の読みやすさ F-4 の是正・FR16）

- 要件: FR16（判断の記録の面・棚から各ページへ辿れる）/ FR4
- 条: P-2.1 / P-6.1
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の読みやすさ F-4（直す）= 判断の記録 11 面と設計ノート 2 面の下端の「前 / 次」が 13 面とも同じ（前 = 要件書・次 = 入口）で、ADR-5 で「次」を押しても ADR-6 へ行かず入口へ戻る。面の骨格（face.rs の Frame）の prev / next は今は静的な字（&'static str の組）で、判断の記録の面（face_adr.rs）と設計ノートの面（face_note.rs）は固定の組を置いている。
- 根拠の判断: 導線の直しであり判断は無い。並びは番号順（判断の記録は ADR-n の n の数の順・設計ノートは棚と同じ順 = 入口の生成器が棚に並べる順）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bn が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 骨格。crates/folio/src/face.rs（正規化 1248 行・余地 252）の Frame の prev と next の型を、静的な字の組（&'static str, &'static str）から所有する字の組（String, String）に変える。prevnext を組む関数はその値を出すだけで文言も class も変えない。3 面（入口 face_index.rs・憲法 face_constitution.rs・要件書 face_srs.rs）は今の固定の組を所有する字にするだけ（面の出力は 1 byte も変わらない）。

(b) 判断の記録の面。crates/folio/src/face_adr.rs（正規化 748 行・余地 752）で、生成する記録 ADR-n の前と次を、正本の置き場に在る判断の記録の id を数の順に並べた列から決める: 前 = 1 つ小さい n の記録（file は adr-{n}.html・名は「ADR-{n}」+ 半角空白 + 題）・無ければ（先頭）入口の棚（index.html・「入口」）。次 = 1 つ大きい n の記録・無ければ（末尾）入口（index.html・「入口」）。並びは既存の数の順の関数（figure.rs の注 render_adr_order_is_numeric_not_lexical と同じ性質 = 字の順でなく数の順）に従い、廃止（status retired）の記録も列に入れる（番号を空けたままにする P-7.2 と同じ考え・飛ばさない）。

(c) 設計ノートの面。crates/folio/src/face_note.rs（正規化 925 行・余地 575）で、前と次を設計ノートの id の列（入口の棚と同じ順 = 正本の置き場の design-note/ の id の字の順・schema の見本は含めない）から同じ規則で決める（先頭の前と末尾の次は入口）。

(d) 凍結の面の写し。tests/fixtures/face/expected-adr.html・expected-note.html・expected-site-adr-2.html の prevnext の字が変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。再生成した写しとの byte 一致は既存の歯（tests/face_adr.rs・tests/face_note.rs の凍結 fixture の歯・tests/site.rs の site_write_matches_the_frozen_fixture）が測る = verify の 3 行目（絞り無し）で回す。fixture の判断の記録が 1 本だけなら前も次も入口になる（それでよい）。

(e) 歯（関数名は neighbor_ で始める・今この語で始まる歯は無い）。
1. crates/folio/tests/face_adr.rs: neighbor_adr_links_go_to_the_adjacent_record = 実の design-intent の写しから ADR-5 の面を生成すると、prevnext の前が adr-4.html・次が adr-6.html を指し、名に「ADR-4」「ADR-6」を含む。neighbor_adr_first_and_last_fall_back_to_the_entrance = ADR-1 の前と、最大の n の記録の次が index.html。
2. crates/folio/tests/face_note.rs: neighbor_note_links_follow_the_shelf_order = 実の design-intent の写しの設計ノート 2 本（example・figures）で、字の順の 1 本目の次が 2 本目・2 本目の前が 1 本目・両端は入口。
3. crates/folio/tests/face.rs（filter face_ の回帰・本文は不変・write-set に置く）と tests/face_index.rs の回帰 = 3 面の出力が 1 byte も変わらない（凍結の写し expected.html・expected-srs.html・expected-index.html は触らない = 変わらないことの証）。

(f) 大きさと接続。新規 file は無い。face.rs（型の変更 約 +5 行）・face_index.rs / face_constitution.rs / face_srs.rs（各 2 行）・face_adr.rs（+約 30 行）・face_note.rs（+約 25 行）・tests/face_adr.rs（+約 40 行）・tests/face_note.rs（+約 30 行）・fixture 3 本（再生成）。size S（各 file の増分は 100 行未満）。design-intent・parts.json は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: prevnext の隣の 1 本への接続（判断の記録・設計ノート）・骨格の型・凍結の写しの更新・歯 3 本。
- 入れない: 面の上端の案内（nav）に判断の記録と設計ノートを足すこと（別の判断）・入口の読む順番（便 66）・並びの規則の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| frame | 骨格 | Frame の prev / next を所有する字に |
| adr | 隣 | 数の順で前後の記録・両端は入口 |
| note | 隣 | 字の順で前後の設計ノート・両端は入口 |
| anchor | 写し | expected-adr / expected-note / expected-site-adr-2 を再生成 |
| teeth | 歯 | neighbor_ の 3 本 + 3 面の不変 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bn"
title = "判断の記録の面と設計ノートの面の下端の「前 / 次」を隣の 1 本（判断の記録は ADR-n の数の順・設計ノートは棚の順・両端は入口）につなぎ、骨格 Frame の prev / next を所有する字にする（3 面の出力は不変・凍結の写し 3 本を再生成・天井の 11 周目の読みやすさ F-4）"
req = ["FR16", "FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/site.rs", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test face_adr neighbor_", "cargo nextest run -p folio --test face_note neighbor_", "cargo nextest run -p folio --test face_adr --test face_note --test site", "cargo nextest run -p folio --test face", "cargo nextest run -p folio --test face_index", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "neighbor_ の歯 3 本（ADR-5 の前後が ADR-4 / ADR-6・両端は入口・設計ノートは棚の順）が緑、tests/face_adr.rs・tests/face_note.rs・tests/site.rs の既存の歯が全部緑（再生成した凍結の写し 3 本 expected-adr / expected-note / expected-site-adr-2 との byte 一致の歯を含む）、tests/face.rs と tests/face_index.rs の既存の歯が全部緑（3 面の凍結の写しは不変）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
