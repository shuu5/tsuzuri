# 設計: 便 95 — 索引の欄の決まりの正本 design-intent/graph.yaml を起こし、閉じた一覧 2 本を生成区間へ導出する（FR19 / NFR3）

**改訂 b（2026-09-22 15:1x JST・検証役の report d94-verify.md への応答）。** 直したのは文だけで、生成区間の逐語・行数・byte 数・要約値・write-set・size は 1 つも動かない。

- 要件: FR19（文書の決まりの部分を、機械の中の決まりから各 file の生成区間へ写す）/ NFR3（参照は必ずつながる）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを人が読める型付きデータとして設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.4（2 つの面を人が書き一致を検査で強制する設計を採らない）/ P-10.1（独立した凍結 anchor）/ P-13.3（設計文書の変更は判断の記録か版付きの文書として残す）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）
- 出所: 判断の記録 ADR-13 決定 (1)（節点の種類と辺の型の閉じた一覧の写しを受ける file は、便が新しく起こす索引の欄の決まりの正本 design-intent/graph.yaml の生成区間とする・要件書 FR19 の対象に足す）。設計ノート docs/design/graph-and-incremental-ceiling.md §2.3・§8 の便の列の 4 本目（G1）の後半。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cr が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は design-intent/graph.yaml と凍結 anchor の 2 本で先頭に + を付ける・縮む file は無い）。**本便の行は便 94 が着地するまで器へ出さない。** 便 94 が作る module は本便の起草の時点の base に無いので、いま `scribe2 contracts check` を当てると本便の行だけに 4 件（write-set-item-unresolved 1・name-unresolved 3・どれも同じ module を指す）が出る。**便 94 が着地した base では 0 件になる**（席が便 94 の枝に module の殻を置いて実測し、findings が着手前の 6 件＝ほかの便の古い行だけに戻ることを確かめた）。
- 門: 本便は design-intent の下に正本を 1 file 新設するので天井の門の対象である。席が実測した `folio ceiling --gate` は 2（まだ分からない・印が古い）を返す。持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・逐語「一通り完成を目指して速度を上げたいので、あまりにも無駄に似たような審査を繰り返しまくっているならやめてどんどん進めて」・台帳 f2-648 notes）を規則の表の開発規律行 D-12 が言う持ち主の裁定として受け、設計文書を触る便は印が古くても門を経ずに出す。
- 前の便: 便 94（行 cq・docs/design/delivery-94.md）。本便は便 94 が置いた閉じた一覧 2 本（`crates/folio/src/graph.rs` の `NODE_KINDS` と `EDGE_TYPES`）の写しを設計文書の置き場へ導出する。**2 本は続けて出す**＝そのあいだだけ、実装の型付きの定数が規則の正本でありながら写しが設計文書の置き場に無い窓（条 P-5.6）が開く。

## 1. 目的と中身

便 94 は索引を組む口を置いたが、節点の種類 11 と辺の型 17 の閉じた一覧は実装の型付きの定数の中にしか無い。条 P-5.6 は、実装の定数を規則の正本とするあいだ、その写しを人が読める型付きデータとして設計文書の置き場へ決定的に導出することを求める。本便はその受け皿 `design-intent/graph.yaml` を起こし、`folio schema --write` の対象を 8 file から 9 file へ増やす。**索引の中身そのものはこの file に置かない**（毎回組み直す導出物・ADR-13 決定 (1) の末尾）。

### (a) 実測（2026-09-22・便 94 の着地後）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **graph.rs 306・余地 1194**／**schema.rs 474・余地 1026**／main.rs 563・余地 937。歯 = tests/schema.rs 624・余地 876／tests/schema_docs.rs 1029・余地 471。**3 本とも余地が S の見積 100 を上回る。**
- 対象の file の一覧の正本は `crates/folio/src/schema.rs` の定数 `TARGETS`（33 行）で、今は 8 対（`adr/schema.yaml`・`design-note/schema.yaml`・`ceiling.yaml`・`rules.yaml`・`index.yaml`・`srs.yaml`・`vocabulary.yaml`・`intake.yaml`）である。**9 本目として、`graph.yaml` と、その正本になる `graph.rs` の床の木を指す 1 対を末尾に足す。**
- 床の木の型 `Floor` と導出の口 `derive` と突き合わせの口 `floor_diff` は同じ file に在り、`ceiling.rs` と `rules.rs` と `entrance.rs` が同じ形で床の木を持っている。**本便の床の木も同じ形で `graph.rs` に置く**（閉じた一覧 2 本は既に定数なので、床の木はその 2 本を指すだけ＝同じ一覧を 2 回書かない）。
- 導出の体裁は `schema.rs` の module の頭が持つ 5 つの規則（幅 W = 100 字・一覧は flow が W 以内なら flow さもなくば block・表は子が全部 字と数と真偽かその一覧で W 以内なら flow）。席はこの 5 つの規則を写した独立の実装を書き、`schema.rs` の中の単体の歯 `schema_layout_switches_flow_and_block_at_the_width` の期待値と 1 字も違わない出力を出すことを確かめた（2026-09-22）。
- 合格の標準出力の行数を数える定数は **3 か所**に在る。`crates/folio/tests/schema.rs` の `TARGETS`（48 行・値 8）と `crates/folio/tests/schema_docs.rs` の `TARGETS`（111 行・値 8）に加えて、同じ file の歯 `f77_check_covers_the_three_files`（722 行〜）の中に直の 8 が 1 つ（727 行の `assert_eq!(lines.len(), 8, ...)`）ある。**3 か所とも 9 に直す。** 直の 8 を残すと verify の 2 行目で落ちる。
- `folio check` は `graph.yaml` を読まない（`check.rs` の `FILES` は 7 file の固定の一覧で、design-intent の下の知らない file を落とす検査は無い＝席が実測した）。天井の材料の束も読まない（束の中身は天井の正本の `documents` と観点の `reads` が決め、本便はそのどちらも触らない）。**`folio check` と `folio ceiling` と面の生成器は 1 字も変わらない。**
- 歯の関数名の接頭辞。`grep -rn 'fn f95_' crates/folio/tests` は今 **0 本**。

### (b) 新しい正本 design-intent/graph.yaml

人が書く節は `meta` の 1 つだけで、**索引の欄も節点の一覧も辺の一覧も人は書かない**（条 P-6.4）。最上位の節は 2 つ（`meta` と生成区間 `schema`）で、生成区間は file の末尾に 1 対とする（`ceiling.yaml`・`index.yaml`・`srs.yaml` と同じ置き方）。

人が書く部分は次のとおり（file の頭の注釈 4 行と `meta`）。

```
# folio2 索引の欄の決まり — 正本（v0.1 起草・orchestrator 席）
# 位置: 判断の記録 ADR-13 決定 (1)(2)。設計文書の索引（節点と辺）の 種類と型の閉じた一覧 を人が読める形で持つ。
# 索引の中身そのものはここに置かない＝毎回 folio graph --print が正本から組み直す導出物である（ADR-13 決定 (4)・P-6.3 / P-6.4）。
# 状態: v0.1 は起草（便 95・行 cr）。閉じた一覧の正本は実装の型付きの定数 crates/folio/src/graph.rs で、下の生成区間はその写しである（P-5.1・P-5.6）。

meta:
  id: folio2-graph
  title: 設計文書の索引の欄の決まり
  version: v0.1
  status: draft
  approval:
    - {role: 作成, who: orchestrator 席（AI・fable 5.1）, when: 2026-09-22, stamp: 起草（便 95・行 cr・判断の記録 ADR-13 決定 (1)(2)）, version: v0.1}
```

### (c) 生成区間と凍結 anchor

`folio schema --write` が書く生成区間は **28 行・2106 byte**で、逐語は次のとおり。凍結 anchor `tests/fixtures/schema/graph-region.txt` はこれと同じ byte を持つ。

```
schema:
  top_level: [meta, schema]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。meta は人が書き、schema は生成区間。索引の中身そのものはこの file に置かない＝毎回 folio graph --print が正本から組み直す導出物である（判断の記録 ADR-13 決定 (4)・P-6.3 / P-6.4）
  node: {required: [id, kind, file, title]}
  node_note: 索引の節点 1 つの欄。id は設計文書の全体で 1 つに定まる id、kind は node_kinds の値、file は正本の置き場からの相対の path、title は空白を 1 つに畳んで Unicode の字で 36 に切った 1 行の題。欄の要約値は天井の印と同じ便で足す（ADR-13 決定 (1)(8)）
  node_kinds: [条, 規範文, 規則行, 目的, 要件, 非機能要件, 受入基準, 制約, 登場人物, 出力, 判断の記録]
  node_kinds_note: 節点の種類の閉じた一覧（順も固定・増減は判断の記録が要る＝P-2.4 と同じ扱い）。正本は実装の型付きの定数 crates/folio/src/graph.rs の NODE_KINDS で、この節はその写しである（P-5.1・P-5.6）
  edge: {required: [from, to, type]}
  edge_note: 索引の辺 1 つの欄。from と to は節点の id、type は edge_types の値。両端が節点のときだけ表に出し、端が節点でない参照（図の名・改訂の範囲の節名）は数だけ要約の 1 行に出す（P-4.2）
  edge_types:
    - in-article
    - relations.articles
    - relations.reqs
    - relations.rules
    - relations.sections
    - amended_by
    - article
    - refs
    - basis
    - goals
    - rules
    - adrs
    - verify.ac
    - verifies
    - figures
    - produced
    - amends
  edge_types_note: 辺の型の閉じた一覧（順も固定・ADR-13 決定 (2)）。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_TYPES で、この節はその写しである。figures は伝播に使わない（決定 (2)）。観点が読む文書の欄（reads）と入口の棚の関係は、文書を節点にする便で足す
```

- 新しい anchor の行数 = **28**・byte 数 = **2106**・要約値（sha256）= **c4385eb354b96cd2979400572d406bd190a54318e35673fa13895bb06adeeccc**
- **本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**
- 席はこの区間を、`schema.rs` の導出の 5 つの規則を写した独立の実装（folio の code を 1 行も呼ばない）で組み、byte 数と要約値を OS の道具 `sha256sum` で独立に出した（P-10.2）。

### (d) 床の木（crates/folio/src/graph.rs に足す）

床の木 `FLOOR` は `Floor::Map` の 10 対で、閉じた一覧の 2 つの葉は便 94 が置いた定数 `NODE_KINDS` と `EDGE_TYPES` をそのまま指す（同じ一覧を 2 回書かない）。欄の順は (c) の逐語のとおり＝`top_level`・`top_level_note`・`node`・`node_note`・`node_kinds`・`node_kinds_note`・`edge`・`edge_note`・`edge_types`・`edge_types_note`。`_note` で終わる 5 つは人が読む説明で、`floor_diff` は読まない。

`crates/folio/src/schema.rs` は `TARGETS` の末尾に 1 対と、その注釈の 1 行（9 本目は索引の欄の決まり）を足すだけで、ほかは 1 字も触らない。`crates/folio/src/main.rs` は副命令 `Schema` の `--dir` の説明の字（生成区間を持つ file 8 本 の並び）を 9 本に直す 1 行だけで、**名の並びの末尾に `・graph.yaml` を足す形にする**。`crates/folio/tests/check.rs` の歯 `r11_schema_help_names_the_schema_files` が 8 本の名の連なりを部分文字列で見ており、末尾に足すぶんには残るが、順を変えたり語を挟んだりすると落ちるためである（席が `--help` の折り返しが無いことを実測した）。

### (e) 本便が触らないもの・次の一括の 🔴

- **入口の棚（design-intent/index.yaml）と天井の正本（design-intent/ceiling.yaml）は 1 字も触らない。** 判断の記録 ADR-13 決定 (1) は「この正本は入口の棚と天井の正本の文書の一覧に載るので、その 2 file の版上げを伴う」と書くが、席は本便でそれを運ばない。理由は 2 つ。
  - **入口の棚は載せない向きに書かれている。** 棚の説明の逐語は 「この棚には、相談窓口が決める文書の型（道具自身の正本 = 相談窓口の正本と天井の正本は載せない）と、型どうしのつながりが出る。」 で、索引の欄の決まりは相談窓口が決める文書の型ではなく道具自身の正本である。載せるには持ち主が承認した v0.3 のこの 1 文を書き換えることになる。
  - **天井の正本の文書の一覧を 9 から 10 にすると、床の定数が連れて動く。** `crates/folio/src/ceiling.rs` の `DOCUMENT_IDS` は `[&str; 9]` の固定長で、`crates/folio/src/bundle.rs` の `FACE_NAMES` も `[(&str, Option<FaceName>); 9]` の固定長である。さらに天井の正本の生成区間と凍結 anchor `tests/fixtures/schema/ceiling-region.txt` と束の凍結 anchor が同時に動く。**別の便 1 本ぶんの大きさ**であり、どの観点がこの文書を読むのかを決めるのは天井の正本の版上げ（持ち主の承認欄）である。
  - 🔴 **次の一括の承認要求に載せる**: ADR-13 決定 (1) の 2 file の版上げをどうするか。席の推奨は **入口の棚には載せず、天井の正本の文書の一覧にだけ足す**（索引の欄の決まりは道具自身の正本で、棚は相談窓口が決める文書の型を並べる面だから）。この推奨を採るなら、判断の記録の決定 (1) の末尾の 1 句を訂正する改訂（条 A-2 の手順）が要る。
- 🔴 **要件書 FR19 の対象の一覧**: FR19 の規範文は対象の file を字で並べており（判断の記録と設計ノートの欄の決まりの file と天井の正本と規則の表と入口の正本と、要件書・語彙・相談窓口の正本）、注が 「対象の file は増えるたびにこの要件の文に足す（憲法 P-5.6・P-13.3）」 と書く。**本便は要件書を 1 字も触らない**（起草を重ねない）。索引の欄の決まりを対象に足した版上げを、便 94 の 🔴 1 点目（索引の要件そのもの）と同じ一括の承認要求に載せる。
- `check.rs`・`refs.rs`・`link.rs`・`mentions.rs`・`adr.rs`・`note.rs`・`rules.rs`・`entrance.rs`・`intake.rs`・`ceiling.rs`・`bundle.rs`・`stamp.rs`・`gate.rs`・面の生成器・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・欄の決まりの写し 17 本は 1 字も触らない。

### (f) 歯（関数名は f95_ で始める・置き場は crates/folio/tests/schema_docs.rs）

置き場を `schema_docs.rs` にするのは、この file が欄の決まりの 2 本を除いた正本の側の歯を持つからである（file の頭の逐語）。`Work` は実の design-intent を一時 dir へ写す口を既に持つ。

1. **f95_the_graph_schema_region_matches_the_anchor** — 実の `design-intent/graph.yaml` の生成区間が **28 行・2106 byte**で、凍結 anchor `tests/fixtures/schema/graph-region.txt` と byte 一致し、要約値が (c) の値と一致すること。**赤い歯**（本便の前の main にこの file が無い）。
2. **f95_the_command_now_sees_nine_files** — 写しに `folio schema --check` を当てて終了コード 0・標準出力が **9 行**で、9 行目が `graph.yaml` の行であること。同じ理由で **赤い歯**。
3. **f95_a_drift_in_the_graph_region_fails** — 写しの生成区間の 1 byte（`node_kinds` の行の 判断の記録 の末尾の 1 字）を変えて `--check` を当て、終了コード 1 でその file の名が理由の行に出ること。`--write` を当てると元の byte に戻ること。同じ理由で **赤い歯**。
4. **f95_the_closed_lists_are_the_same_as_the_index** — 写しの生成区間の `node_kinds` と `edge_types` の値が、同じ写しに `folio graph --print` を当てた出力に現れる種類と型の集合を漏れなく覆うこと（索引が出す種類と型が、写しの側の閉じた一覧の外に 1 つも無いこと）。同じ理由で **赤い歯**。

4 本とも席が独立の実装の出力で期待値を出した。回帰は verify の 2 行目（`tests/schema.rs` と `tests/schema_docs.rs` の既存の歯・どちらも `TARGETS` の値が 8 から 9 に変わる）と common-verify が見る。

### (g) 大きさ

src は `crates/folio/src/graph.rs`（306・余地 1194・床の木 `FLOOR` で +38 行の見込み）と `crates/folio/src/schema.rs`（474・余地 1026・+6 行）と `crates/folio/src/main.rs`（563・余地 937・説明の 1 行）。歯は `crates/folio/tests/schema_docs.rs`（1029・余地 471・+70 行の見込みと `TARGETS` の 1 行）と `crates/folio/tests/schema.rs`（624・余地 876・`TARGETS` の 1 行だけ）。正本は新しい file `design-intent/graph.yaml`（人が書く 12 行 + 生成区間 28 行 + 印 2 行）。凍結 anchor は新しい file `tests/fixtures/schema/graph-region.txt`（28 行・2106 byte）。size **M**（触る src 3 本の余地はどれも M の見積 300 を上回る）。新しい dir は作らず、縮む file も無い。外部 crate は増やさない。

### (h) 本便が運ばないもの・撤退条件

- 入口の棚と天井の正本の文書の一覧の版上げ（(e) の 🔴 1 点目）。
- 要件書 FR19 の対象の一覧の版上げ（(e) の 🔴 2 点目）。
- 節点ごとの要約値の欄（便 94 の 🔴 3 点目）・全体像の短い出力・注意の先・周の引き金・門の単位。
- `folio check` が `graph.yaml` の `meta` を数えること。本便は生成区間だけを `folio schema` に見せる。`meta` の欄の検査を足すかは、この正本が版を上げて発効するとき（持ち主の承認欄が要るとき）に決める。
- 撤退条件: 閉じた一覧 2 本の正本を実装の定数から設計文書の側へ移す（つまり `graph.yaml` を人が書く面にする）ことになったときは、本便を捨てて `schema.rs` の `TARGETS` を 8 に戻し、`design-intent/graph.yaml` と 2 つの anchor を消す。**便 1 本で戻せる。** 同じ一覧を 2 つの面が人の手で持つ形は取らない（条 P-6.4）。

## 2. 範囲

- 入れる: 新しい正本 `design-intent/graph.yaml`（人が書く `meta` と生成区間 28 行）・`crates/folio/src/graph.rs` の床の木 `FLOOR` 1 本・`crates/folio/src/schema.rs` の `TARGETS` に 1 対と注釈の 1 行・`crates/folio/src/main.rs` の説明の 1 行・凍結 anchor `tests/fixtures/schema/graph-region.txt` 1 本・`crates/folio/tests/schema.rs` と `crates/folio/tests/schema_docs.rs` の `TARGETS` を 9 へ・`schema_docs.rs` の f95_ 4 本。
- 入れない: 入口の棚・天井の正本・要件書・憲法・規則の表・語彙・相談窓口・判断の記録の本文・`check.rs` と `refs.rs` と `link.rs` と `mentions.rs` と `adr.rs` と `note.rs` と `rules.rs` と `entrance.rs` と `intake.rs` と `ceiling.rs` と `bundle.rs` と `stamp.rs` と `gate.rs` と面の生成器・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・欄の決まりの写し 17 本・新しい dir・外部 crate。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| source | 新しい正本 | `design-intent/graph.yaml`（人が書く `meta` と生成区間・索引の中身は置かない） |
| floor | 床の木 | `crates/folio/src/graph.rs` の `FLOOR`（閉じた一覧 2 本を指す 10 対） |
| target | 対象の一覧 | `crates/folio/src/schema.rs` の `TARGETS` の 9 本目 |
| region | 生成区間 | `graph.yaml` の 28 行（`folio schema --write` が書く） |
| anchor | 凍結 anchor | `tests/fixtures/schema/graph-region.txt`（28 行・2106 byte） |
| pins | 凍結の定数 | `tests/schema.rs` と `tests/schema_docs.rs` の `TARGETS`（8 → 9） |
| teeth | 歯 | `crates/folio/tests/schema_docs.rs` の f95_ 4 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 94（行 cq）の着地が前提である（閉じた一覧 2 本と module がそこで置かれるため）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cr"
title = "索引の欄の決まりの正本 design-intent/graph.yaml を新設し、便 94 が置いた閉じた一覧 2 本（節点の種類 11・辺の型 17）と索引の節点と辺の欄の決まりを、その生成区間へ folio schema --write で導出する。人が書く節は meta の 1 つだけで、索引の中身そのものは置かない（毎回組み直す導出物）。対象の file の一覧 TARGETS を 8 対から 9 対へ増やし、2 つの歯の file の凍結の定数も 8 から 9 へ直す。入口の棚と天井の正本の文書の一覧と要件書は 1 字も触らず、判断の記録 ADR-13 決定 (1) が言うその 2 file の版上げは次の一括の承認要求に回す"
req = ["FR19", "NFR3"]
section = "1"
write-set = ["+design-intent/graph.yaml", "crates/folio/src/graph.rs", "crates/folio/src/schema.rs", "crates/folio/src/main.rs", "+tests/fixtures/schema/graph-region.txt", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test schema_docs f95_", "cargo nextest run -p folio --test schema_docs --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f95_ の歯 4 本（graph.yaml の生成区間が 28 行 2106 byte で凍結 anchor と byte 一致し要約値も一致／folio schema --check が終了コード 0 で 9 行を出し 9 行目が graph.yaml／生成区間を 1 byte 変えると終了コード 1 でその file の名が出て --write で戻る／生成区間の閉じた一覧 2 本が folio graph --print の出す種類と型を漏れなく覆う）が全部緑、tests/schema_docs.rs と tests/schema.rs の既存の歯が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
