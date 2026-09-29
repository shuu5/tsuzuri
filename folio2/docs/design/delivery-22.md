# 設計: 便 22 — 入口の面の「支度表」の節で、持つ文書の付録（with）を入口の正本の annexes に解く（便 20 の穴の是正）

- 要件: FR1（支度表 1 枚）/ FR4（3 面を 1 つの生成器から）/ NFR2（部品目録に無い class は 0）
- 条: P-4.1（実行できなかった生成を異常なしにしない＝実の正本で通らない生成器を直す）/ P-6.3（同じ内容は一方を正本に他方は導出＝付録の型は入口の正本の annexes が正本）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない＝fixture だけでなく実の正本で 1 周）
- 判断の記録: ADR-6（決定 (2) 行き先は入口の棚の文書の id と注入の 5 つに閉じる＝付録は行き先の行にならない／決定 (3) 支度表の人が読む面は入口の面の節）。便 21（f2-648.35）の後に直列で置く。
- 裁定: 持ち主 2026-09-18 の裁定と ADR-6 の発効「承認する」（f2-648.31 notes）。契約の穴の是正であり新しい裁定は要らない。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 w が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

便 20 の契約 (d) は、支度表の documents の各行の with（付録の id）を intake.yaml の targets の行に解くと定めた。だが実の相談窓口の正本 intake.yaml v0.1 は、付録（vocabulary・rules）を targets の行に持たず constitution の行の with の中にだけ持つ（ADR-6 決定 (2) のとおり行き先は棚の文書 4 つと注入に閉じる）。このため実の正本で `folio intake --write`（便 19）が書いた支度表を `folio face --face index`（便 20）が読むと「行き先の id「vocabulary」が intake.yaml の targets に無い」で導出できない（2）になる（admin と planner が 2026-09-18 に main 2231a63 で再現）。便 20 の凍結 fixture は vocabulary を targets の行として持っていたので歯は緑のままだった。本便は読み手側を直す（正本は変えない・案 A）: with の id は入口の正本 index.yaml の shelf.annexes の id に解き、表示の型（語彙・rules）も annexes の type から写す。これは便 18 の床（intake.yaml の targets の with を index.yaml の annexes に解く）と同じ規則で、読み手と床が揃う。

planner の実測（2026-09-18・main 2231a63）: `face_index.rs` の支度表の読みは documents の各行の id と with の各要素を同じ関数 target_type（intake.yaml の targets の行から type を引く）で解いている。入口の正本 index.yaml の shelf.annexes は vocabulary（type 語彙・inside constitution）と rules（type rules・inside constitution）の 2 行で、入口の面の生成器は既にこれを読んでいる（棚の付録の chip）。凍結 fixture `tests/fixtures/face/intake.yaml` の targets は constitution〔with = vocabulary〕・vocabulary・inject の 3 行で、fixture の index.yaml の annexes は vocabulary と rules の 2 行。fixture の支度表 `intake-sheet.yaml` の documents は constitution〔with = [vocabulary]〕の 1 行。実の正本で intake --write（answers-5）した支度表の documents は constitution〔with = vocabulary・rules〕・adr・design-note の 3 行。

(a) 読み手の直し（`crates/folio/src/face_index.rs`）: 支度表の documents の各行の id は今のまま intake.yaml の targets に解く（type も targets から）。with の各要素は index.yaml の shelf.annexes の id に解き、表示の型は annexes の行の type（escape して逐語）。解けなければ導出できない = 2・文言「intake-sheet.yaml.documents[<n>].with[<m>]: 付録の id「<値>」が index.yaml の annexes に無い」（出力先に 1 byte も書かない）。持つ文書の行の字面（「<type>（付録の<型を・で>）」）は変えない。他の節・部品・class は変えない（部品 12 種のまま・AC2）。

(b) 凍結 fixture（`tests/fixtures/face/`）: `intake.yaml` の targets から vocabulary の行を外す（実の正本と同じ形・constitution の with = [vocabulary] はそのまま）。`intake-sheet.yaml` はそのまま。期待の面 `expected-index-sheet.html` は with の型が annexes の 語彙 で今と同じ字面なので変わらない見込み（変わるなら生成物を写して更新し、歯が固定する）。`expected-index.html`（支度表なし）は変わらない。

(c) 歯 `crates/folio/tests/face.rs`（既存の歯の file に足す・関数名はすべて face を含める）:
- 実の正本で 1 周（P-10.2・本便の主の歯）: `design-intent/` を丸ごと一時 dir へ写し、`folio intake --dir <写し> --answers tests/fixtures/intake/answers-5.yaml --write` = 0 → `folio face --face index --dir <写し> --out <一時 file> --write` = 0 ∧ 出力に「憲法（付録の語彙・rules）」∧ `folio parts --check --dir design-intent --page index=<一時 file>` = 0。回答なしの `--write`（全部おすすめ）でも同じく face = 0 ∧ 推奨で進めた項目の行に 5 つの ask。
- 凍結 fixture（直した intake.yaml）で支度表あり／なしの byte 一致は既存の歯のまま緑（期待が変わるなら更新）。
- 導出できない: fixture の支度表の写しの with を annexes に無い id（nowhere）にする → 2 ∧「annexes に無い」。便 20 の「documents の id が targets に無い → 2」の歯はそのまま。
- 便 20 の字面の 2 case（with の有無・承認の空／あり）はそのまま緑。

(d) 便 21 までの形との接続: 変えるのは `crates/folio/src/face_index.rs`（with の解き方 1 か所と文言）・歯 `crates/folio/tests/face.rs`・fixture `tests/fixtures/face/intake.yaml`（期待の面 2 本は変わらない見込み・変わるなら `expected-index-sheet.html` を更新）。`sheet.rs`（便 19）・`intake.rs`（便 18）・`face.rs`・他の src・`design-intent/`・`tests/site.rs`・`tests/serve.rs`・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`face_index.rs` の増分は 20 行未満・歯の増分は 4 本）。

## 2. 範囲

- 入れる: with を annexes に解く読み手の直し・fixture の intake.yaml の形の是正・実の正本で 1 周の歯・導出できない 1 つ。
- 入れない: intake.yaml の版上げ（正本は変えない）・支度表の形の床・folio2 自身の支度表の生成（持ち主の手番の後）・便 21。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| annex | 付録の解決 | with を index.yaml の annexes に解き type を写す |
| roundtrip | 1 周の歯 | 実の正本で intake → face → parts |
| fixture | 凍結 fixture | intake.yaml を実の形に |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 21 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "w"
title = "入口の面の支度表の節: 付録（with）を index.yaml の annexes に解く（便 20 の穴の是正・実の正本で 1 周の歯）"
req = ["FR1", "FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face_index.rs", "crates/folio/tests/face.rs", "tests/fixtures/face/intake.yaml", "tests/fixtures/face/expected-index-sheet.html"]
verify = ["cargo nextest run -p folio --test face face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（実の正本で intake → face → parts の 1 周が回答あり／なしで緑・凍結 fixture の byte 一致 2 本・annexes に無い with で 2・便 20 の歯は期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
