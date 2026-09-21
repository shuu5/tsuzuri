# 設計: 便 71 — 設計ノートの面の名札 3 か所（赤くなる条件・契約表の大きさの凡例・脚注の正本の file 名）と判断の記録の面の表紙の撤退条件の 1 文（天井の 12 周目の読みやすさ F-6 / F-7 / F-8 / F-11 の是正・FR16）

- 要件: FR16（判断の記録の面）/ FR4 / GOAL2（非エンジニアが読める）
- 条: P-2.1 / P-6.1
- 出所: 天井の 12 周目（2026-09-21・main bbd2a05）の読みやすさ F-6（直す）= 設計ノートの面の歯の表の行が red_when（この検査が赤くなる条件）の文を名札なしで出すので、題の直下で正反対のことを述べた文に見える（同じ行の「固定の材料」には名札が在る）。F-7（直す）= 契約表の行が「M」の札だけを出し、設計ノートの面には凡例が 1 つも無い。F-8（直す）= 設計ノートの面の脚注が「正本 design-note/<文書 id>.yaml」と差し込みの合図をそのまま出す（入口・憲法・要件書の脚注は実際の file 名）。F-11（参考）= 判断の記録の面の表紙が「撤退条件 数えた値」と種別の名だけを出し、何をもって捨てるのかが表紙から分からない。
- 根拠の判断: 見た目の直しであり判断は無い。文言は正本の欄の決まり（design-note/schema.yaml の row_note = red_when は「何を壊せば落ちるか（1 文）」・契約表の size の値域 S / M）から取る。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bt が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 設計ノートの面。crates/folio/src/face_note.rs（正規化 約 925 行・余地 約 575・便 65 の後の行数は席が受付時に測り直す）で: (1) 歯の表の行の red_when の文に名札「赤くなる条件」を付ける（「固定の材料」と同じ形 = 名札 + 本文）。(2) 契約表の章（§6 相当・rows が契約表の行のとき）の先頭に凡例 1 行「大きさ: S = 小さい便（src の余地 100 行の見積）／M = 中くらいの便（300 行の見積）」を置く（字面は憲法の面の凡例と同じ部品 legend-line・値は契約表の size の値域 S / M の順・部品目録に無い class は使わない）。(3) 脚注の正本の file 名を Frame の source に実際の id（meta.id）から `design-note/{id}.yaml` で入れる（「<文書 id>」の字を出さない）。

(b) 判断の記録の面。crates/folio/src/face_adr.rs（正規化 約 748 行・余地 約 750・便 65 の後は席が測り直す）の表紙の meta_span「撤退条件」の値を、種別の名だけから 1 文にする: 小さな試し → 「小さな試しで確かめて、外れたら捨てる」／数えた値 → 「数えた値が条件を超えたら捨てる」／持ち主の裁定 → 「持ち主の裁定で捨てる」（retreat_kind_label と同じ網羅の場合分け・値が増えれば組み立てが通らない）。章 04 の「撤退条件（種別）」の見出しは触らない。

(c) 凍結の面の写し。tests/fixtures/face/expected-note.html は (a) で、expected-adr.html と expected-site-adr-2.html は (b) で変わるので、同じ着地で fixture の正本から生成し直して置き換える（手で直さない）。

(d) 歯（関数名は label_fix_ で始める・今この語で始まる歯は無い）。
1. label_fix_red_when_has_a_label（crates/folio/tests/face_note.rs・正規化 約 800 行）: 凍結の正本から --write で出した設計ノートの面に「赤くなる条件」が在り、歯の表の行の red_when の文の直前に付いている（同じ要素の中）。
2. label_fix_size_legend_is_present（同）: 同じ面の契約表の章に「大きさ: S = 」と「M = 」を持つ legend-line が 1 つ在る。
3. label_fix_footer_names_the_real_file（同）: 同じ面の脚注に「design-note/<文書 id>.yaml」が無く「design-note/{meta.id}.yaml」（凍結の正本の id）が在る。
4. label_fix_retreat_sentence_on_the_cover（crates/folio/tests/face_adr.rs・正規化 約 840 行）: 凍結の正本（retreat.kind = measure）から出した判断の記録の面の表紙に「数えた値が条件を超えたら捨てる」が在り、表紙の meta_span の中に「数えた値」だけの値が無い。
5. 回帰（期待不変・verify の 2 行目）: tests/face_note.rs・tests/face_adr.rs・tests/site.rs の既存の歯すべて（凍結の写し 3 本との byte 一致の歯は (c) で更新した写しで緑）。

(e) 大きさと接続。新規 file は無い。face_note.rs（+約 20 行）・face_adr.rs（+約 12 行）・tests/face_note.rs（+約 40 行）・tests/face_adr.rs（+約 15 行）・写し 3 本（再生成）。size S。外部 crate は増やさない。便 65（f2-648.97）と write-set が重なるので、器の受付はその着地の後（dispatcher の overlap）。

## 2. 範囲

- 入れる: 名札 3 か所・撤退条件の 1 文・写し 3 本・歯 4 本。
- 入れない: 憲法と要件書の面（便 70）・語彙への項の追加・面の構成の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| labels | 名札 | face_note.rs の赤くなる条件・大きさの凡例・脚注 |
| retreat | 撤退条件 | face_adr.rs の表紙の 1 文 |
| fixture | 写し | 3 本を再生成 |
| teeth | 歯 | tests/face_note.rs に 3 本・tests/face_adr.rs に 1 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bt"
title = "設計ノートの面に名札「赤くなる条件」と契約表の大きさの凡例を置き、脚注の正本の file 名を実際の id にし、判断の記録の面の表紙の撤退条件を 1 文にする（凍結の写し 3 本を再生成・天井の 12 周目の読みやすさ F-6 / F-7 / F-8 / F-11）"
req = ["FR16", "FR4"]
section = "1"
write-set = ["crates/folio/src/face_note.rs", "crates/folio/src/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/site.rs", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test face_note --test face_adr label_fix_", "cargo nextest run -p folio --test face_note --test face_adr --test site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "label_fix_ の歯 4 本（赤くなる条件の名札・大きさの凡例・脚注の file 名・表紙の撤退条件の 1 文）が緑、tests/face_note.rs・tests/face_adr.rs・tests/site.rs の既存の歯が全部緑（再生成した凍結の写し 3 本との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
