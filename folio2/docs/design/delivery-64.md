# 設計: 便 64 — 要件書の面の受入基準の章に「まだ分からない」の札の凡例を置く（天井の 11 周目の読みやすさ F-6 の是正・FR4）

- 要件: FR4 / GOAL2
- 条: P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-2.1
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の読みやすさ F-6（直す）= 要件書の面 §5 は受入基準 17 件すべての番号の横に「まだ分からない」の札を付けるが、何がまだ分からないのかを言う字が面に無い（§3 には「言葉」と「確かめ方」の凡例が在るが §5 には無い）。実装（face_srs.rs の ac_chapter）の注は「folio はまだ受入の結果を測らない（P-4.2）」。
- 根拠の判断: 見た目の直しであり判断は無い。札の意味 = 受入の合否を folio は数えていない（歯が測るのは便の検証で、その結果は面に写していない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bm が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 凡例。crates/folio/src/face_srs.rs（正規化 1285 行・余地 215）の受入基準の章（ac_chapter・帯 5）の帯の直後、基準の一覧の前に、凡例を 1 行出す: 「まだ分からない = この基準の合否を folio はまだ数えていません（合否を測るのは便の検証の歯で、その結果はこのページに写していません）」。部品と class は §3 の凡例（同じ file が既に出す「言葉」と「確かめ方」の凡例）と同じものを使う（部品目録 parts.json に新しい部品も class も足さない）。札の字「まだ分からない」と札の部品（AcStateChip）は変えない。

(b) 凍結の面の写し。tests/fixtures/face/expected-srs.html（要件書の面の凍結 anchor・fixture の正本から生成）は (a) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。

(c) 歯（crates/folio/tests/face.rs・正規化 1205 行・関数名は ac_legend_ で始める・今この語で始まる歯は無い）。
1. ac_legend_appears_once_in_the_acceptance_chapter: 実の design-intent の写しから生成した要件書の面で、「合否を folio はまだ数えていません」を含む凡例が受入基準の章（帯 5 の後・帯 6 の前）に 1 回だけ出る。
2. ac_legend_uses_the_same_parts_as_chapter_three: その凡例の部品の名札（data-component の値）が §3 の凡例と同じ。
3. 回帰（期待不変・verify の 2 行目）: tests/face.rs の既存の歯すべて（凍結の写しは (b) で更新）。

(d) 大きさと接続。新規 file は無い。face_srs.rs（+約 12 行・余地 215 → 約 203）・tests/face.rs（+約 40 行）・tests/fixtures/face/expected-srs.html（再生成）。size S。face.rs・parts.json・design-intent は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 受入基準の章の凡例 1 行・凍結の写しの更新・歯 2 本。
- 入れない: 受入の合否を面に写す仕組み（要件書の側の判断が要る）・札の字や部品の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| legend | 凡例 | ac_chapter の帯の直後に 1 行 |
| anchor | 写し | expected-srs.html を fixture から再生成 |
| teeth | 歯 | tests/face.rs に ac_legend_ の 2 本 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bm"
title = "要件書の面の受入基準の章に「まだ分からない」の札の凡例を 1 行置く（合否を folio はまだ数えていない旨・§3 の凡例と同じ部品・凍結の写し expected-srs.html を再生成・天井の 11 周目の読みやすさ F-6）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_srs.rs", "crates/folio/tests/face.rs", "tests/fixtures/face/expected-srs.html"]
verify = ["cargo nextest run -p folio --test face ac_legend_", "cargo nextest run -p folio --test face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "ac_legend_ の歯 2 本（凡例が受入基準の章に 1 回だけ・部品が §3 の凡例と同じ）が緑、tests/face.rs の既存の歯が全部緑（凍結の写しは再生成）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
