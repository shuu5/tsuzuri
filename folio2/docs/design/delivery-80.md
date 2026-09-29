# 設計: 便 80 — 憲法の面に条の欠番の行を置き、改訂来歴の行に前の文と理由を出す（天井 17 周目 読みやすさ F-4・19 周目 読みやすさ F-3・FR4）

- 要件: FR4（人が読むページを 1 つの生成器から出す）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-7.2（廃止は状態で表し、番号は空けたままにする）/ P-4.1（読めない入力を異常なしにしない）/ P-6.2（生成物を手で直さない）
- 出所: 一括 10 の仕分け C。天井の 17 周目 読みやすさ F-4「憲法の面の条の並びに『P-9 廃止』の行を置く（欠番の印）」と 19 周目 読みやすさ F-3「改訂の来歴の行が面で同一に見える。前の文と理由の欄を面へ出すか、行を区別する」。
- 位置: 便 79（同じ file の数値の表の 2 件）の後。便 79 が置く歯の file crates/folio/tests/face_constitution.rs に本便の歯を足すので、便 79 の着地の後に受け付ける。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cc が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ と tests/fixtures/face/ だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

憲法の面の条は段（いつも守る・確認してから変える・絶対にやらない）ごとの章に並ぶ。読み手が詰まる所が 2 つある。① 条の番号は P-8 の次が P-10 で、P-9 は空いている。条 P-7.2 は「廃止は状態（status）で表し、番号は空けたままにする」と定めるが、面にはその印が 1 つも出ないので、読み手には抜けなのか誤りなのか分からない。② 改訂来歴の行は、どの条でも「判断の記録の id + 日付 + 裁定 + 承認者」という同じ型の並びで、何がどう変わったのかが面から読めない。正本は前の文（previous_text）と理由（rationale）を欄に持っているのに、面はその 2 欄を読むだけ読んで捨てている。本便はこの 2 つを面の側だけで直す。正本 design-intent/constitution.yaml は 1 byte も触らない。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- 条の id は crates/folio/src/face_constitution.rs の関数 context（156〜222 行）が正本の articles をそのまま順に読んで作る。正本に無い id は列に現れず、欠番を埋める仕掛けは無い。
- 正本 design-intent/constitution.yaml の条は 27 本。id の接頭辞は P・A・N の 3 つで、実測の列は P-1 から P-8・P-10 から P-18（P-9 だけ欠け）・A-1 から A-4（欠けなし）・N-1 から N-6（欠けなし）。P-9 への言及は meta.changes_from_v0_1 の散文（86 行・P-9 を P-6 へ畳んだ旨）だけで、条の欄にも schema の節にも status の欄は無い。P-9 がどの段に属していたかは正本のどこにも無い（だから段の章の中には置けない）。
- 章 01（読み方）を組むのは同じ file の関数 reading（459〜489 行）。章の帯を置いた後に chapbody を開き、部品 section-lead-callout の中に段のカードを 3 枚並べ、callout と chapbody を閉じる。class legend-line の 1 行は同じ file の rules_chapter（722 行）と crates/folio/src/face_srs.rs の ac_legend_line（1072 行）で既に使っており、新しい class は要らない。
- 条への参照のリンクは同じ file の what_with_xref（802〜809 行）が class xref のアンカーで組む。アンカーの字は crates/folio/src/face.rs の anchor（337 行）。
- 改訂来歴を組むのは同じ file の item_row（510〜652 行）の 601〜634 行。部品 principle-amendment-history の中に、supersedes_v1 が在れば 1 行、amended_by の項ごとに 1 行を置く。amended_by の行は 「{adr} <span class=am-meta>{date} · {ruling} · 承認 {approved_by}</span>」 の形で、直前の 621〜622 行で am.ef(previous_text) と am.ef(rationale) を呼んで結果を捨てている（欄の実在だけを確かめている）。
- 実測の件数（design-intent/constitution.yaml）: supersedes_v1 を持つ条は 1 本、amended_by を持つ条は 3 本で amended_by の項は合わせて 4 つ。
- 小窓は face.rs の hint（358〜363 行）。様式 design-intent/preview/folio.css は小窓の本体と legend-line の規則を既に持つので、本便は様式を 1 byte も触らない。部品目録 design-intent/preview/parts.json も触らない（新しい部品も新しい class も作らない）。

### (a) 条の欠番の行

face_constitution.rs に、条の id の列から欠番を決定的に数える関数 1 本を足し、章 01 の末尾に 1 行を置く。

1. 関数 missing_numbers(ctx) -> R<Vec<String>>。ctx が持つ条の id を先頭から見て、id を最後のハイフンで接頭辞と数に分け（face::split_dash・342 行）、数が符号なしの整数に読めなければ Err（「条の id「<id>」の番号が読めない」・P-4.1）。接頭辞ごとに、その接頭辞で最初に現れた順を保ったまま、1 からその接頭辞の最大の数までのうち列に無い数を欠番として集め、「<接頭辞>-<数>」 の形で返す。接頭辞の並びは正本の初出の順、同じ接頭辞の中は数の小さい順。
2. reading の chapbody の中で、段のカードの callout を閉じた後に、欠番が 1 つ以上あるときだけ class legend-line の 1 行を置く。字面は 「欠番: <欠番を中黒で繋いだもの>（条の廃止は状態で表し、番号は空けたままにする — <P-7 への xref のリンク>）」。リンクの字は P-7、行き先は what_with_xref と同じ形の #<anchor(P-7)>。欠番が 0 なら 1 行も置かない（面の字は本便の前と同じ）。
3. 実の正本では欠番は P-9 の 1 つだけなので、面には 「欠番: P-9（条の廃止は状態で表し、番号は空けたままにする — P-7）」 の 1 行が出る。

### (b) 改訂来歴の行に前の文と理由

item_row の 616〜634 行の amended_by の行の末尾に、小窓を 2 つ足す。

1. am-meta の span を閉じた後、同じ am-row の span の中に hint(「前の文」, <previous_text の escape 済みの字>) と hint(「理由」, <rationale の escape 済みの字>) をこの順で置く。
2. 621〜622 行の捨てていた 2 回の読み（am.ef(previous_text) と am.ef(rationale)）は、1 で使う値の読みに置き換える（同じ欄を 2 度読まない）。欄が無い・空のときの断り方は今までどおり（ef が Err を返し、面は導出できない）。
3. supersedes_v1 の行（610〜618 行）は触らない（その行は理由を既に枡の中に出している）。

### (c) 歯（関数名は f80_ で始める。便 79 が置く crates/folio/tests/face_constitution.rs に足す）

1. f80_constitution_shows_the_missing_number: 実の置き場の写しで憲法の面を書く → 0 ∧ 章 01 の中に 「欠番: P-9」 が在る ∧ 「欠番:」 は面全体で 1 回だけ。本便の前の main では 「欠番」 が 0 回なので赤い歯。
2. f80_missing_numbers_match_an_independent_count: 歯の側で正本 design-intent/constitution.yaml の articles の id を直に読み（yaml-rust2）、接頭辞ごとに 1 から最大までの欠けを歯の側の規則で数え、その列（実測では P-9 の 1 つ）と面の行の列が字で一致する。生成器の数えを写さない物差し。
3. f80_no_missing_line_when_the_numbers_are_dense: 写しの constitution.yaml の条 P-10 から P-18 の id を P-9 から P-17 へ詰め替えた写しでは 「欠番:」 の行が 1 つも出ない、という向きは正本の関係の欄を壊すので採らない。代わりに、面の欠番の行に A- と N- で始まる id が 1 つも無いことを見る（A と N には欠けが無い＝欠番は接頭辞ごとに数えている）。
4. f80_amendment_rows_carry_the_previous_text: 同じ面で、部品 principle-amendment-history の中の amended_by の行それぞれに小窓 前の文 と 理由 が 1 つずつ在り、小窓の数の合計が正本の amended_by の項の総数（実測 4）と一致する（数は歯の側で正本から数える）。本便の前の main では 前の文 の小窓が 0 個なので赤い歯。
5. f80_amendment_previous_text_is_verbatim: 小窓 前の文 の中身が正本の previous_text の逐語（escape だけを掛けたもの）と一致し、小窓 理由 の中身が rationale の逐語と一致する（歯の側が正本を直に読み、escape の 5 字も歯の側で持つ）。
6. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/face_constitution.rs の f79_ の歯・tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯すべて。

### (d) 凍結の写し

面の字が変わるのは憲法の面だけなので、tests/fixtures/face/expected.html を同じ着地で生成し直して置き換える。手で直さず、歯と同じ経路（binary に face --face constitution --dir tests/fixtures/face --out <一時 file> --write を当て、その出力を写す）で作る。ほかの 6 本は 1 byte も変わらない。写しの正本 tests/fixtures/face/constitution.yaml は触らない（欠番も改訂来歴も持たない最小の手書きなら (a)(b) の字はこの写しには出ず、それでよい。実の置き場の写しで見るのが (c) の歯）。

### (e) 大きさ

src は crates/folio/src/face_constitution.rs 1 本だけ（幅 120 で正規化して 1130 行・余地 370・便 79 の着地で約 55 行増える見込みなので受付時の余地は約 315・本便の見積は + 約 45 行）。歯は crates/folio/tests/face_constitution.rs（+ 約 90 行）。size **S**（src の余地は 100 以上）。外部 crate は増やさない。face.rs・ほかの 4 面の生成器・図の道具・部品目録・様式・design-intent の下の正本・tests/floor_cases.yaml・CI の yml は触らない。

### (f) 本便が運ばないもの

正本 design-intent/constitution.yaml（条の欄に status を足すこと・欠番の一覧を正本に書くこと・meta の散文）。条の廃止の機構そのもの（条 P-7.2 の status の欄は憲法の schema の節に無く、足すには憲法の改訂が要る。本便は面に印を出すだけで、機構は足さない）。判断の記録の面と設計ノートの面の改訂来歴（それぞれ別の欄の決まりを持つ）。改訂の例の章（amendment_chapter・879 行〜）は既に前の文と理由を詳しく出しているので触らない。

## 2. 範囲

- 入れる: 欠番を数える関数 1 本・章 01 の 1 行・改訂来歴の行の小窓 2 つ・歯 5 本・凍結の写し 1 本の生成し直し。
- 入れない: 正本の欄と散文・条の status の機構・新しい class と新しい部品・部品目録・様式・ほかの 4 面・改訂の例の章。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| gaps | 欠番 | face_constitution.rs の missing_numbers（正本の id の列から数える） |
| line | 欠番の行 | reading の chapbody の末尾の legend-line 1 行 |
| why | 前の文と理由 | item_row の am-row の小窓 2 つ |
| teeth | 歯 | crates/folio/tests/face_constitution.rs の f80_ 5 本 |
| frozen | 凍結 | tests/fixtures/face/expected.html の生成し直し |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cc"
title = "憲法の面の章 01 に条の欠番の行（正本の id の列から接頭辞ごとに数える・実測は P-9 の 1 つ・P-7 へのリンク付き）を置き、改訂来歴の行に正本が既に持つ 前の文 と 理由 を小窓で出す（正本は 1 byte も触らない・凍結の写し 1 本を生成し直す・天井 17 周目 読みやすさ F-4 と 19 周目 読みやすさ F-3）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_constitution.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs", "tests/fixtures/face/expected.html"]
verify = ["cargo nextest run -p folio --test face_constitution f80_", "cargo nextest run -p folio --test face_constitution --test face --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f80_ の歯 5 本（章 01 の欠番の行が 1 回だけ・歯の側の独立な数えと一致・接頭辞ごとに数えている・改訂来歴の小窓の数が正本の項の総数と一致・小窓の中身が正本の逐語）が緑、tests/face_constitution.rs の f79_ と tests/face.rs と tests/site.rs と tests/badge.rs の既存の歯が全部緑（生成し直した凍結の写し expected.html との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
