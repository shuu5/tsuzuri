# 設計: 便 81 — 要件書の面の図の名札を読める字にし、受入基準の英字の欄名を日本語に揃え、承認欄の来歴を折りたたむ（天井 17 周目 読みやすさ F-1・18 周目 読みやすさ F-4 / F-6・FR4）

- 要件: FR4（人が読むページを 1 つの生成器から出す）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-4.1（読めない入力を異常なしにしない）/ P-6.2（生成物を手で直さない）
- 出所: 一括 10 の仕分け C。天井の 17 周目 読みやすさ F-1「要件のカードの図の名札『図2-4』を『図 2 の 4 段目』と読める形にするか、凡例に 1 行足す」、18 周目 読みやすさ F-4「受入基準のカードの英字の欄名（fixture）を設計ノートの面と同じ日本語の名札に揃える」、18 周目 読みやすさ F-6「承認欄の来歴の段落を折りたたむ。要旨の 1 行は A で先頭に置いた」。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cd が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 1 本）。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ と tests/fixtures/face/ だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

要件書の面で読み手が詰まる所が 3 つある。① 要件のカードは、その要件が縛る図の段へのリンクを出すが、字が 「図 2 4 3 枚を出す」 のように数字が続くので、どこまでが図の番号でどこからが段の名なのか読めない。② 受入基準のカードの小窓は、正本の欄の名 fixture を英字のまま本文に混ぜて出す。設計ノートの面は同じ欄を 「固定の材料」 の日本語の名札で出しているので、面ごとに字が違う。③ 承認欄の帯の 1 行に、正本の meta.status_note がまるごと流れ込む。この欄は先頭の 1 文が要旨で、その後ろに版ごとの来歴が 4000 字以上続く（一括 10 で先頭の 1 文を正本に置いた）。面はその全部を 1 行の lead に出すので、要旨が埋もれる。本便はこの 3 つを面の側だけで直す。正本 design-intent/srs.yaml は 1 byte も触らない。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- 図の参照を解くのは crates/folio/src/face_srs.rs の関数 resolve（427〜458 行）。正本の参照の字（図2-<n>・図1・図3・全段）を parse_fig（405〜422 行）が読み、段の参照のときだけ 長い字 = 「図 2 {n} {what}」・短い字 = 「図 2 {n}」 を組む（438〜441 行）。図 1・図 3・全段 の字は数字が続かないので今のままでよい。
- 長い字を使うのは同じ file の item_row（967〜1071 行）の 1044〜1050 行で、class rq-where のリンクとして要件のカードの meta-chips に出す。短い字を使うのは対応表（部品 rtm-grid）で、crates/folio/src/face_srs_rtm.rs が Where の short を読む。
- 正本 design-intent/srs.yaml の rail の段は 7 段（n = 1 から 7）。凍結の写し tests/fixtures/face/expected-srs.html には今 「図 2 1 相談を受ける」「図 2 3 面を出す」 の形で焼き込まれている。
- 受入基準の章を組むのは同じ file の ac_chapter（1076〜1135 行）。1103〜1109 行の hint の名札は 「RED の歯」（日本語）だが、本体が 「{sentence}／fixture: {fixture}」 の形で英字の欄名をそのまま出す。凍結の写しでの実測は 「／fixture: tests/fixtures/intake/answers.yaml」。この章で英字の欄名を出しているのはこの 1 か所だけ（milestone は値だけを pill に出し、欄名を出さない）。
- 設計ノートの面 crates/folio/src/face_note.rs は同じ欄を 861 行で 「固定の材料」 の名札に置き換えて出す（英字の fixture を面に出さない）。
- 承認欄を組むのは同じ file の approval（1212〜1247 行）。1214〜1217 行で meta.status_note が在れば 「{status} — {status_note}」 を lead にし、frame の approval_band（crates/folio/src/face.rs 960〜969 行）が 1 行の lead として出す。chapbody は部品 approval-block の中に meta.approval の行を並べる。
- 実測の status_note（design-intent/srs.yaml 15 行）: 最初の 「。」 までが 「いま発効しているのは 第 1.24 版（2026-09-21）で、この欄の残りは版ごとの来歴です（同じ来歴は下の承認欄の行にも在ります）。」 の 84 字で、その後ろに 4343 字が続く。
- 折りたたみの字面は crates/folio/src/face_index.rs の 1168〜1174 行に在る details（class は note・summary の次に div・その中に p）と同じ形を使う。新しい class は作らない。部品目録 design-intent/preview/parts.json と様式 design-intent/preview/folio.css は 1 byte も触らない。

### (a) 図の名札を読める字に

resolve（438〜441 行）の段の参照の 2 つの字を次に変える。ほかの 3 つの形（図 1・図 3・全段）と parse_fig と正本の参照の字は触らない。

- 長い字 = 「図 2 の {n} 段目（{what}）」
- 短い字 = 「図 2 の {n} 段目」

これで要件のカードのリンクは 「図 2 の 4 段目（3 枚を出す）」 になり、対応表の枡は 「図 2 の 4 段目」 になる。数字が 2 つ続く形は面から消える。

### (b) 受入基準の英字の欄名

ac_chapter の 1106〜1108 行の hint の本体の字 「／fixture: 」 を 「／固定の材料: 」 に置き換える。名札（RED の歯）と値（正本の fixture の逐語）は変えない。これで受入基準の章に英字の欄名の字面 「／fixture: 」 が 1 つも残らない（値の path に含まれる英字は正本の逐語なのでそのまま）。

### (c) 承認欄の来歴の折りたたみ

approval（1212〜1220 行）を次の形にする。

1. status_note が在れば、その字を最初の 「。」 で 2 つに分ける（「。」 は前半に含める）。前半を要旨、後半を来歴とする。
2. lead は 「{status} — {要旨}」。status_note が無ければ今までどおり status だけ（字は変わらない）。
3. chapbody を開いた直後（部品 approval-block の div より前）に、来歴が空でないときだけ折りたたみを 1 つ置く。字面は face_index.rs 1168〜1174 行の details と同じ形で、summary の字は 「版ごとの来歴」、中の p が来歴の字（escape 済み）。
4. 「。」 が 1 つも無い、または後ろが空のときは折りたたみを置かない（面の字は本便の前と同じ）。
5. 部品 approval-block の中身（meta.approval の行）は触らない。

### (d) 歯（関数名は f81_ で始める。`grep -rn 'fn f81_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

歯は新しい file crates/folio/tests/face_srs.rs（新規・+）に置く。要件書の面の歯の置き場が今は crates/folio/tests/face.rs（幅 120 で正規化して 1637 行）に相乗りしており、入口・判断の記録・設計ノートの 3 面が既に持つ 1 面 1 file の形に揃える。既存の歯は 1 本も移さない（tests/face.rs は 1 byte も変えない）。新しい file は tests/face_adr.rs と同じ作り（binary を CARGO_BIN_EXE_folio で起動し、実の置き場 design-intent/ と図の道具 vendor/archify/ を一時 dir へ写して面を書く）で始める。

1. f81_figure_label_reads_as_a_step: 実の置き場の写しで要件書の面を書く → 0 ∧ 面に 「図 2 の 」 が 1 回以上在る ∧ 面に 「図 2 1」「図 2 2」「図 2 3」「図 2 4」「図 2 5」「図 2 6」「図 2 7」 のどれも 0 回。本便の前の main では旧字面が在るので赤い歯。
2. f81_every_referenced_step_has_a_label: 歯の側で正本 design-intent/srs.yaml の rail の段（実測 7 段）と、要件と非機能要件の figures 欄の参照を直に読み（yaml-rust2）、参照された段 n のそれぞれについて 「図 2 の {n} 段目」 が面に在る。生成器の字を写さない物差し。
3. f81_rtm_cell_uses_the_short_label: 同じ面の対応表（部品 rtm-grid）の枡に 「図 2 の 」 が在り、対応表の枡に 「図 2 1」 の形が 0 回。
4. f81_acceptance_has_no_english_field_name: 受入基準の章（章の帯から次の章の帯まで）に欄名の字面 「／fixture: 」 が 0 回 ∧ 「／固定の材料: 」 が在り、その回数が正本の acceptance の行の数と一致する（数は歯の側で正本から数える）。物差しは欄名の字面だけで、値の側の逐語（tests/fixtures/intake/answers.yaml のように fixture を含む path）は数えない。本便の前の main では 「／fixture: 」 が在るので赤い歯。
5. f81_approval_lead_is_only_the_summary: 承認欄の帯の lead の字が 「{status} — 」 の後ろに status_note の最初の 「。」 までだけを持ち、lead に 「v1.0 = 」 が 0 回（要旨と来歴の切り方は歯の側で正本から独立に組む）。本便の前の main では lead に来歴が在るので赤い歯。
6. f81_approval_history_is_folded: 承認欄の chapbody に summary の字が 「版ごとの来歴」 の details が 1 つ在り、その中の p の字が status_note の最初の 「。」 の後ろの逐語（escape だけを掛けたもの）と一致する。
7. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯すべて。

### (e) 凍結の写し

面の字が変わるのは要件書の面だけなので、tests/fixtures/face/expected-srs.html を同じ着地で生成し直して置き換える。手で直さず、歯と同じ経路（binary に face --face srs --dir tests/fixtures/face --out <一時 file> --write を当て、その出力を写す）で作る。ほかの 6 本（expected.html・expected-index.html・expected-index-sheet.html・expected-adr.html・expected-note.html・expected-site-adr-2.html）は 1 byte も変わらない。写しの正本 tests/fixtures/face/srs.yaml は触らない。

### (f) 大きさ

src は crates/folio/src/face_srs.rs 1 本だけ（幅 120 で正規化して 1305 行・余地 195・見積は + 約 30 行）。歯は新規の crates/folio/tests/face_srs.rs（+ 約 220 行・helper を含む）。size **S**（src の余地は 100 以上）。外部 crate は増やさない。face.rs・face_srs_rtm.rs・ほかの 4 面の生成器・図の道具・部品目録・様式・design-intent の下の正本・tests/floor_cases.yaml・CI の yml は触らない。

### (g) 本便が運ばないもの

正本 design-intent/srs.yaml（参照の字 図2-<n>・受入の欄名 fixture・status_note の中身・版と承認欄）。設計ノートの面の同じ欄（既に日本語）。要件書の面のほかの章。承認欄の行（meta.approval）の字。部品目録と様式。要件書 FR4 の規範文と版。

## 2. 範囲

- 入れる: 段の参照の 2 つの字・受入の小窓の欄名の字・承認欄の lead の切り方と折りたたみ 1 つ・歯 6 本と新しい歯の file・凍結の写し 1 本の生成し直し。
- 入れない: 正本の字・parse_fig と参照の形・図 1 / 図 3 / 全段 の字・新しい class と新しい部品・部品目録・様式・ほかの 4 面。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| step | 段の名札 | face_srs.rs の resolve の長い字と短い字 |
| field | 欄の名札 | ac_chapter の小窓の本体の 固定の材料 |
| fold | 折りたたみ | approval の lead の要旨と details の来歴 |
| teeth | 歯 | 新規 crates/folio/tests/face_srs.rs の f81_ 6 本 |
| frozen | 凍結 | tests/fixtures/face/expected-srs.html の生成し直し |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cd"
title = "要件書の面で、図の段の参照を 図 2 の n 段目 の読める字にし（対応表の短い字も同じ）、受入基準の小窓の英字の欄名 fixture を設計ノートの面と同じ 固定の材料 に揃え、承認欄の帯は status_note の最初の 1 文だけを出して残りの来歴を折りたたむ（正本は 1 byte も触らない・凍結の写し 1 本を生成し直す・天井 17 周目 読みやすさ F-1 と 18 周目 読みやすさ F-4 / F-6）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_srs.rs", "+crates/folio/tests/face_srs.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs", "tests/fixtures/face/expected-srs.html"]
verify = ["cargo nextest run -p folio --test face_srs f81_", "cargo nextest run -p folio --test face_srs --test face --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f81_ の歯 6 本（段の名札が読める字で旧字面が 0 回・正本から数えた参照の段が全部面に在る・対応表の短い字・受入の章に欄名の字面 ／fixture: が 0 回で日本語の名札が行の数だけ・承認欄の lead が要旨だけ・来歴が折りたたみの中で逐語一致）が緑、tests/face.rs と tests/site.rs と tests/badge.rs の既存の歯が全部緑（生成し直した凍結の写し expected-srs.html との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
