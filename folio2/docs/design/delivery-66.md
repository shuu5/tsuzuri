# 設計: 便 66 — 入口の読む順番が判断の記録へ導けるようにし、支度表の承認欄の見せ方を直す（天井の 11 周目の読みやすさ F-5・F-7 の是正・FR13 / FR1）

- 要件: FR13（入口の面と棚）/ FR1（相談窓口の支度表）/ FR4
- 条: P-2.1 / P-12.2
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の読みやすさ F-5（直す）= 入口の「読む順番」の行き先は憲法と要件書の節だけで、判断の記録へ行く道が無い。実装（face.rs の stop_anchor）が行き先の文書を憲法と要件書に閉じている。F-7（直す）= 入口の支度表の「承認」の行が「2026-09-18 aで」と出る（持ち主の逐語をそのまま面に出す）。
- 根拠の判断: 導線と見せ方の直し。承認欄の逐語は記録として正本（intake-sheet.yaml）に残し、面には「承認済み（日付）」と出し、逐語は開いて読める形（details）で残す（P-12.2 の記帳は正本の側・面はその写し）。読む順番に判断の記録の行き先を実際に足すのは正本 index.yaml の側（席の一括・本便は口だけ）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bo が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 行き先の口。crates/folio/src/face.rs（正規化 1248 行・余地 252）の stop_anchor に、文書 adr を足す: at が ADR-{n}（n は 1 以上の整数）の形なら Ok。ほかの文書と形は今のまま（憲法 s0〜s8・要件書 s1〜s8 と 3 つの図）。

(b) 入口の面。crates/folio/src/face_index.rs（正規化 1243 行・余地 257）の読む順番（lanes）の stop の href を、文書 adr のときは adr-{n}.html（判断の記録の面の file・棚と同じ形）にする。名札（label）は正本の字のまま。判断の記録の面が無い n（正本に無い id）は既存の参照 id の解決（R-4）で落ちる（本便は新しい検査を足さない）。

(c) 支度表の承認欄。同じ file の支度表の節で、承認の行が在るときの字を「{when} {verbatim}」から「承認済み（{when}）」にし、逐語は同じ行の中の details（部品目録に既に在る折りたたみの部品・要約は「言葉どおりの記録」）に入れる。承認の行が無いときの字（SHEET_NO_APPROVAL）は変えない。部品目録に新しい部品も class も足さない（parts.json 不変・folio parts --check は変わらない）。

(d) 凍結の面の写し。tests/fixtures/face/expected-index-sheet.html（支度表つきの入口の凍結 anchor）は (c) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。expected-index.html（支度表なし）は変わらない = 触らない。

(e) 歯（crates/folio/tests/face_index.rs・正規化 1284 行・関数名は lane_adr_ と sheet_approval_ で始める・今この語で始まる歯は無い）。
1. lane_adr_stop_links_to_the_record_face: fixture の入口の正本の写しで、読む順番の 1 行に stop {doc: adr, at: ADR-2, label: なぜそう決めたか} を足す（fixture の判断の記録は ADR-2 が在る）と、面の読む順番に href が adr-2.html の行き先が出る。
2. lane_adr_stop_with_unknown_record_is_refused: 同じ形で at を ADR-99（無い id）にすると生成が Err（参照 id の解決で落ちる・面を書かない）。
3. lane_adr_stop_anchor_rejects_other_shapes: stop_anchor に doc adr で at が s1 や ADR-x の形を渡すと Err（unit の歯でよい・tests 側から呼べるなら tests 側）。
4. sheet_approval_shows_approved_with_date_and_hides_the_verbatim_in_details: fixture の支度表（intake-sheet.yaml・承認の行が在る）から生成した入口の面に「承認済み（」と日付が出て、逐語は details の中に在り、行の本文（details の外）には逐語が出ない。
5. 回帰（期待不変・verify の 2 行目）: tests/face_index.rs の既存の歯すべて（凍結の写し expected-index-sheet.html は (d) で更新・expected-index.html は不変）。

(f) 大きさと接続。新規 file は無い。face.rs（+約 5 行・余地 252 → 約 247）・face_index.rs（+約 30 行・余地 257 → 約 227）・tests/face_index.rs（+約 70 行）・expected-index-sheet.html（再生成）。size S。index.yaml（読む順番に実際に判断の記録の行を足すのは席の一括）・parts.json は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 行き先の口（adr）・入口の href・承認欄の見せ方・凍結の写しの更新・歯 4 本。
- 入れない: index.yaml の読む順番の中身（席の一括）・設計ノートへの行き先（面が 1 枚に定まらない扱いは別の判断）・承認の記帳の仕組み。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| anchor | 口 | stop_anchor に adr / ADR-n |
| href | 導線 | lanes の stop を adr-n.html へ |
| approval | 見せ方 | 承認済み（日付）+ details の逐語 |
| fixture | 写し | expected-index-sheet.html を再生成 |
| teeth | 歯 | tests/face_index.rs に lane_adr_ 3 本・sheet_approval_ 1 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bo"
title = "入口の読む順番の行き先に判断の記録（doc adr・at ADR-n → adr-n.html）を受け付け、支度表の承認欄を「承認済み（日付）」+ 折りたたみの逐語にする（正本 index.yaml の中身は席の一括・凍結の写し expected-index-sheet.html を再生成・天井の 11 周目の読みやすさ F-5 / F-7）"
req = ["FR13", "FR1", "FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_index.rs", "crates/folio/tests/face_index.rs", "tests/fixtures/face/expected-index-sheet.html"]
verify = ["cargo nextest run -p folio --test face_index lane_adr_", "cargo nextest run -p folio --test face_index sheet_approval_", "cargo nextest run -p folio --test face_index", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "lane_adr_ の歯 3 本（adr-2.html へ・無い id は断る・形の外は断る）と sheet_approval_ の歯 1 本（承認済み（日付）・逐語は details の中だけ）が緑、tests/face_index.rs の既存の歯が全部緑（expected-index.html は不変・expected-index-sheet.html は再生成）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
