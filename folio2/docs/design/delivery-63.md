# 設計: 便 63 — 憲法の面に強度の凡例を置き、10 以上の数えを「N の原則」にする（天井の 11 周目の読みやすさ F-1・F-2 の是正・FR4）

- 要件: FR4（3 面を単一の生成器と単一の design token で）/ GOAL2（非エンジニアが読める）
- 条: P-2.1 / P-6.1
- 出所: 天井の 11 周目（2026-09-21・main de3ce19）の読みやすさ F-1（直す）= 憲法の面は規範文 67 本の末尾に MUST か MUST NOT の英語の札を出すが、その意味を言う凡例が憲法の面に無い（要件書の面 §3 には凡例が在るが札は出ない）。F-2（直す）= 憲法の面の章見出しが「17 つの原則」で、日本語の「つ」は 9 までにしか付かない（要件書の面は「19 の機能要件」と「つ」無し）。
- 根拠の判断: 見た目の直しであり判断は無い。凡例の字面は要件書の面が既に持つ字（面の生成器 face.rs の強度の名札の表 = 必ず守る／決してしない／強い推奨（外すなら理由が要る））をそのまま使う（1 か所の正本・P-6.3）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bl が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 数えの字。crates/folio/src/face_constitution.rs（正規化 1107 行・余地 393）の章見出しと目次の「{n} つの原則」（2 か所・tier_count の値を入れる format）を、n が 1〜9 のときは「{n} つの原則」、10 以上のときは「{n} の原則」にする関数 1 つ（count_word の名・face_constitution.rs の中に置く）に寄せる。今の実の憲法は 17・4・6 なので、面は「17 の原則」「4 つの原則」「6 つの原則」になる。

(b) 強度の凡例。同じ file の、規範文の札（MUST / MUST NOT / SHOULD）を出す 3 つの段の章（いつも守る・確認してから・絶対にやらない）の最初の章の帯の直後に、凡例を 1 行出す: 「MUST = 必ず守る／MUST NOT = 決してしない／SHOULD = 強い推奨（外すなら理由が要る）」。字は面の生成器 face.rs の強度の名札の表（strength_label の 3 値の日本語）から組み、手書きの写しを face_constitution.rs に置かない。部品目録の class は既存の要件書の面の凡例（face_srs.rs が §3 の頭に出す同じ凡例）と同じ部品と class を使う（部品目録 parts.json に新しい部品も class も足さない = folio parts --check は変わらない）。凡例は 3 つの段の章の最初の 1 か所だけ（3 か所に繰り返さない）。

(c) 凍結の面の写し。tests/fixtures/face/expected.html（憲法の面の凍結 anchor・fixture の正本から生成）は (a)(b) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない・歯 face_ の凍結の一致がその写しと合う）。fixture の憲法（tests/fixtures/face/constitution.yaml）の条の数は実の憲法と違ってよい（数えの字は n で決まる）。

(d) 歯（crates/folio/tests/face.rs・正規化 1205 行・関数名は tier_legend_ と count_word_ で始める・今この語で始まる歯は無い）。
1. count_word_uses_tsu_only_up_to_nine: fixture の憲法の写しで、いつも守るの条の数を 10 以上にした写し（条を複製して id を変える・既存の歯 edit / mutated と同じ形）の面に「{n} の原則」が出て「{n} つの原則」が出ない。もう 1 つ、条が 9 以下の段の見出しは「{n} つの原則」のまま。
2. tier_legend_appears_once_in_the_constitution_face: 実の design-intent の写しから生成した憲法の面に「MUST = 必ず守る」を含む凡例が 1 回だけ出る（2 回以上でも 0 回でも赤）。要件書の面の凡例と同じ class を持つ。
3. tier_legend_words_come_from_the_shared_table: 凡例の 3 つの語（必ず守る・決してしない・強い推奨（外すなら理由が要る））が要件書の面の凡例の語と 1 字も違わない（2 面を生成して同じ 3 語を含む）。
4. 回帰（期待不変・verify の 2 行目）: tests/face.rs の既存の歯すべて（凍結の写しは (c) で更新）。

(e) 大きさと接続。新規 file は無い。face_constitution.rs（+約 25 行）・tests/face.rs（+約 60 行）・tests/fixtures/face/expected.html（再生成）。size S。face.rs（余地 252）・face_srs.rs・parts.json・design-intent は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 数えの字の関数・強度の凡例 1 行・凍結の写しの更新・歯 3 本。
- 入れない: 要件書の面の変更・用語集の章への強度の項（語彙の field_terms を面に出す設計は別）・他の面の凡例。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| count | 数え | count_word: 1〜9 は「つ」・10 以上は無し |
| legend | 凡例 | 段の最初の章の帯の直後に強度の凡例 1 行（face.rs の表から） |
| anchor | 写し | expected.html を fixture から再生成 |
| teeth | 歯 | tests/face.rs に count_word_ 1 本と tier_legend_ 2 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bl"
title = "憲法の面の章見出しの数えを 10 以上は「N の原則」にし、規範文の札 MUST / MUST NOT / SHOULD の凡例を段の最初の章に 1 行置く（字は面の生成器の強度の名札の表から・凍結の写し expected.html を再生成・天井の 11 周目の読みやすさ F-1 / F-2）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_constitution.rs", "crates/folio/tests/face.rs", "tests/fixtures/face/expected.html"]
verify = ["cargo nextest run -p folio --test face count_word_", "cargo nextest run -p folio --test face tier_legend_", "cargo nextest run -p folio --test face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "count_word_ の歯 1 本（10 以上は「の原則」・9 以下は「つの原則」）と tier_legend_ の歯 2 本（凡例が 1 回だけ・語が要件書の面と同じ）が緑、tests/face.rs の既存の歯が全部緑（凍結の写しは再生成）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
