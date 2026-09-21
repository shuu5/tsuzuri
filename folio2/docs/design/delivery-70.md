# 設計: 便 70 — 要件書の面の章の数えの助数詞と、憲法の面 §5 の凡例の見出し語と「射影」の名札を直す（天井の 12 周目の読みやすさ F-2 / F-4 / F-5 の是正・FR4）

- 要件: FR4（3 面を単一の生成器と単一の design token で）/ GOAL2（非エンジニアが読める）
- 条: P-2.1 / P-6.3
- 出所: 天井の 12 周目（2026-09-21・main bbd2a05）の読みやすさ F-2（直す）= 要件書の面は目次と章の見出しで「19 の機能要件」「3 の非機能要件」「17 の受入基準」と助数詞を落とす（「3 の非機能要件」は日本語にならない・表紙は「19 件」と数える）。F-4（直す）= 憲法の面 §5 の規則の表の行が欄 projection を「射影」の名札で出すが、射影は語彙にも面の用語集にも無い数学の語。F-5（直す）= §5 の凡例が種別を英語の鍵（deny: … ／ build-check: … ／ detect: … ／ human-review: …）で並べるのに、表の種別の欄は日本語の名札（測って落とす・生成時の検査・記録のみ・人が守る作法）で出るので、凡例と表の字が噛み合わない。
- 根拠の判断: 見た目の直しであり判断は無い。数えの字は便 63 が憲法の面に置いた規則（9 までは「つ」・10 以上は「N の」）と同じ形にし、1 か所の関数から出す（P-6.3）。凡例の名札は表の種別の欄と同じ関数（face.rs の rule_kind_label）から出す。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bs が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 数えの関数を共有する。crates/folio/src/face_constitution.rs（正規化 約 1,130 行・余地 約 370）の private の関数 count_word（便 63・「{n} つの原則」／「{n} の原則」）を、名詞を引数に取る形 `pub fn count_word(n: usize, noun: &str) -> String`（9 までは「{n} つの{noun}」・10 以上は「{n} の{noun}」）として crates/folio/src/face.rs（正規化 約 1,256 行・余地 約 244）へ移し、憲法の面は count_word(n, "原則") を呼ぶ（憲法の面の出力は 1 byte も変わらない）。

(b) 要件書の面。crates/folio/src/face_srs.rs（正規化 約 1,292 行・余地 約 208）の章の見出しの 3 か所（「{} の機能要件 — …」「{} の非機能要件 — …」「{} の受入基準 — …」）を count_word(n, "機能要件") などに替える。目次と章の帯は同じ関数を通るので同じ字面になる（例: 20 の機能要件・3 つの非機能要件・18 の受入基準）。表紙の「N 件」は触らない。

(c) 憲法の面 §5。crates/folio/src/face_constitution.rs の (1) 規則の表の行の欄の名札の表引き ("projection", "射影") を ("projection", "写す範囲") に。(2) 種別の凡例（規則の表の生成区間 schema.kind_meaning の対を「{鍵}: {説明}」で並べる）を「{名札}（{鍵}）: {説明}」の形にし、名札は鍵を rules::RuleKind に解いて face.rs の rule_kind_label で引く（鍵が値域に無ければ Err・fail-closed）。

(d) 凍結の面の写し。tests/fixtures/face/expected.html（憲法の面）は (c) で、expected-srs.html は (b) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。ほかの写し（expected-adr / -note / -index / -index-sheet / -site-adr-2）は触らない。

(e) 歯（crates/folio/tests/face.rs・正規化 約 1,250 行・関数名は noun_count_ で始める・今この語で始まる歯は無い）。
1. noun_count_srs_uses_the_counter_word: 凍結の正本（tests/fixtures/face/srs.yaml）から --write で出した要件書の面の目次に「の機能要件」「の受入基準」（10 以上）と「つの非機能要件」（3）が在り、「3 の非機能要件」が無い。
2. noun_count_constitution_is_unchanged_by_the_move: 凍結の正本 tests/fixtures/face/constitution.yaml の meta.counts（always / ask_first / never の 3 つの数）を歯が自分で読み、憲法の面の出力の目次に、その数ごとに便 63 の規則で組んだ字（9 以下は「{n} つの原則」・10 以上は「{n} の原則」・歯の中で独立に組む）がすべて在り、「{n} つの原則」（n ≥ 10）の形が無いことを見る（oracle は正本の数と規則の字で、再生成する写しには依らない）。
3. noun_count_rule_kind_legend_uses_the_labels: 憲法の面 §5 の種別の凡例に「測って落とす（deny）」「生成時の検査（build-check）」「記録のみ（detect）」「人が守る作法（human-review）」が在り「deny: 」の形が無い。
4. noun_count_projection_label_is_plain: 憲法の面 §5 に「写す範囲」が在り「射影」が無い。
5. 回帰（期待不変・verify の 2 行目と 3 行目）: tests/face.rs の既存の歯すべて（憲法の面と要件書の面の凍結の写しとの byte 一致の歯は (d) で更新した写しで緑）・tests/site.rs（配信先の写しは憲法・要件書の面を含まない = 不変）。

(f) 大きさと接続。新規 file は無い。face.rs（+約 10 行）・face_constitution.rs（−8 +6 行）・face_srs.rs（3 行）・tests/face.rs（+約 45 行）・写し 2 本（再生成）。size S。外部 crate は増やさない。

## 2. 範囲

- 入れる: 数えの関数の共有・要件書の面の章の見出し・憲法の面 §5 の名札と凡例・写し 2 本・歯 4 本。
- 入れない: 表紙の「N 件」の形・語彙への「射影」の追加（面の側で言い換える）・判断の記録と設計ノートの面（便 71）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| counter | 数え | face.rs の count_word(n, noun) |
| srs | 要件書の面 | 章の見出し 3 か所 |
| legend | 凡例 | §5 の種別の凡例の名札と projection の名札 |
| teeth | 歯 | tests/face.rs に noun_count_ の 4 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bs"
title = "数えの関数 count_word(n, noun) を face.rs に共有して要件書の面の章の見出しの助数詞を直し（3 つの非機能要件・20 の機能要件）、憲法の面 §5 の種別の凡例を表と同じ名札（鍵は括弧）にし「射影」を「写す範囲」に（凍結の写し 2 本を再生成・天井の 12 周目の読みやすさ F-2 / F-4 / F-5）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-srs.html"]
verify = ["cargo nextest run -p folio --test face noun_count_", "cargo nextest run -p folio --test face", "cargo nextest run -p folio --test site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "noun_count_ の歯 4 本（要件書の面の助数詞・憲法の面の数えは正本の数と便 63 の規則から独立に組んだ字と一致・§5 の凡例の名札・写す範囲）が緑、tests/face.rs の既存の歯が全部緑（再生成した凍結の写し expected.html / expected-srs.html との byte 一致の歯を含む）、tests/site.rs が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
