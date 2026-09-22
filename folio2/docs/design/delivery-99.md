# 設計: 便 99 — 節点ごとの要約値の式を定め、索引の 5 つ目の欄と天井の印の表に置く（FR20 / FR14）

- 要件: FR20（天井の門 — 天井の判定の印を生成物として書く ceiling --stamp / --gate）/ FR14（機械が読む id の索引を出す）
- 条: P-2.4（閉じた一覧の追加には判断の記録が要る）/ P-4.1（実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の定数を正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（独立した凍結 anchor）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-10.3（anchor が維持できなくなった検査は「まだ分からない」に落とす）/ N-1.1（管理下の対象を回復不能に削除する操作を拒む）/ N-3.1（規則の例外機構を足さない）
- 出所: 判断の記録 ADR-13 決定 (1)（節点の欄は 5 つ＝id・種類・所属 file・欄の要約値・1 行の題）・決定 (8)（天井の印に節点ごとの要約値の表を足す。辺を足すだけの変更は周の引き金に数えない。欄の分類は実装の型付きの定数に置き、生成区間へ導出する）・決定 (16) の ⑤。設計ノート docs/design/graph-and-incremental-ceiling.md §3.4・§4.3・§8 の便の列の G4。便 94 の 🔴 3 点目（docs/design/delivery-94.md §1 (f)）が「節点の要約値の欄は G4 で、印の側の凍結 anchor と一緒に定める」と決めている。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cu が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は 2 本で先頭に + を付ける・縮む file は無い・新しい dir は作らない）。
- 門: 本便は design-intent の下の正本 design-intent/graph.yaml の生成区間を書き換えるので、天井の門の対象である。席が本便の write-set をそのまま渡して `folio ceiling --gate --dir design-intent` を実測すると **2（まだ分からない・断りの字は 印が古い）**（2026-09-22）。持ち主の指示（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes・逐語「一通り完成を目指して速度を上げたいので、あまりにも無駄に似たような審査を繰り返しまくっているならやめてどんどん進めて」の (2)）により、**設計文書を触る便は印が古くても門を経ずに出し、その裁定を根拠に記帳する**。24 周目は一括 12 の前に回す。
- 改訂 b（2026-09-22 17:2x JST・検証役の report handoff-2026-09-22/d99-verify.md への応答）: (f) が凍結する生成区間の 3 行（node_note・edge_fields_note・digest_note）を逐語で置いた（blocker・これが無いと runner の書いた別の字の生成区間が歯の緑のまま着地する）。🔴 4 の直す先を判断の記録から設計ノート §3.4 へ改めた。字の直し 4 点（見本の digest の 1 行ずれ・印の大きさ 7,689・作り直す anchor は 3 本・設計ノート §4.3 の節点 285 の実測）と、式の曖昧な 3 点と、大きさの見積の取り直し（graph.rs は +300 前後）を入れた。**式・anchor の 4 対・契約表の行 cu の字は 1 つも変えていない。**
- 前の便: 便 94（行 cq・`folio graph --print`）・便 95（行 cr・`design-intent/graph.yaml`）・便 96（行 cs・`folio graph --digest` と `folio hello` の 1 行）。**3 本とも着地済み**で、本便の base は `main 0806433` である。§1 の余地と数はその base の実測。**便 97（行 ct・束の凍結 anchor の独立 script・検証中）とは write-set が 1 file も重ならない**（便 97 は `tests/fixtures/ceiling/bundle-anchor.py` と `bundle-anchor.txt` と `crates/folio/tests/bundle.rs`・本便は `tests/fixtures/ceiling/findings/stamp-expected.yaml` と `crates/folio/tests/stamp.rs`）。

## 1. 目的と中身

判断の記録 ADR-13 の決定 (8) は「天井の印に節点ごとの要約値の表を足し、周の引き金が 1 つも立っていない変更では周を要らないと出す」と定める。**本便はそのうち、要約値の式を定めて凍結し、索引と印にその値を置くところまでを運ぶ。**「周が要るか要らないか」を判定する口は本便に入れず、後続の便へ回す（理由は (f) の 🔴 1 点目）。

要約値は周の引き金の物差しなので、**式は正本を読む道具に依らず決まり、凍結 anchor を folio の実装と別に組めなければならない**（条 P-10.2）。便 94 が 🔴 3 点目でこの欄を本便へ回したのは、そこが理由である。

### (a) 実測（2026-09-22・main 0806433）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **graph.rs 409・余地 1091**／**stamp.rs 331・余地 1169**／**gate.rs 252・余地 1248**。**3 本とも余地が M の見積 300 を上回る。** 歯 = tests/graph.rs 456・余地 1044／tests/stamp.rs 412・余地 1088／tests/schema_docs.rs 1136・余地 364。
- 歯の関数名の接頭辞。`grep -rn 'f99_' crates/folio` は今 **0 本**。
- 索引の節点の数（`folio graph --print` の実測）: 実の正本 **198**／凍結した土台 `tests/fixtures/floor_base/design-intent` **189**／束の凍結 fixture `tests/fixtures/ceiling/bundle/source` **23**。
- **公開する命令も旗も 1 つも増やさない。** 本便は副命令も旗も足さず、既にある `folio graph --print` の節点の行に欄を 1 つ足し、`folio ceiling --stamp` が書く印に欄を 2 つ足すだけなので、`crates/folio/tests/check.rs` の歯 `p1_commands_closed_list` の閉じた一覧 12 本は 1 字も変わらない（席が逐語を読んで確かめた）。**`crates/folio/src/main.rs` と `crates/folio/tests/check.rs` は write-set に入れない。**
- **ほかの全数固定の歯も動かない**（席が全数を読んで確かめた）: `crates/folio/src/ceiling.rs` の `DOCUMENT_IDS` 9・`crates/folio/src/bundle.rs` の `FACE_NAMES` 9・`crates/folio/src/graph.rs` の `NODE_KINDS` 11 と `EDGE_TYPES` 17・欄の決まりの写し 17 本。**節点の種類も辺の型も 1 語も足さない。**
- **`folio graph --digest`（便 96）の出力は 1 byte も変わらない。** 短い出力は種類ごと・型ごと・file ごとの数と要約の 2 行だけで、節点の行そのものを出さない（席が `crates/folio/src/graph.rs` の畳む口を逐語で読んで確かめた）。よって凍結 anchor `tests/fixtures/schema/graph-digest-anchor.txt` は **1 字も動かさず、write-set に入れない**。`folio hello` の 1 行も変わらない。
- **床の判定は 1 つも変わらない。** `crates/folio/src/check.rs` の固定の 7 file の一覧に `graph.yaml` は無く、本便は `check.rs`・`refs.rs`・`link.rs`・`mentions.rs`・`adr.rs`・`rules.rs`・`entrance.rs`・`intake.rs`・`ceiling.rs`・`bundle.rs`・`site.rs`・面の生成器を 1 字も触らない。
- 門の実測: 本便の write-set 12 本を `folio ceiling --gate --dir design-intent --write-set …` に渡すと **2（まだ分からない・印が古い）**。`design-intent/graph.yaml` が設計文書の正本に当たるためで、判定は本便の中身によらない（印は一括 10 の後の 23 周目のまま）。

### (b) 要約値の式（本便が定める・凍結する）

**要約値 = 節点の本文の byte の sha256 の先頭 8 字**（16 進の小文字）。「本文」は次の 3 段で切り出す。**正本を YAML として読まず、行の逐語の byte だけで決める。**

1. **節点の block を取る。** 節点の頭の行は、正本の節（憲法は articles・規則の表は thresholds と discipline・要件書は goals / actors / outputs / requirements / nonfunctional / acceptance / constraints）の中で、字下げ 2（憲法の規範文は 6）の `- id: <id>` の行か `- {id: <id>,` で始まる行とする。block は頭の行から、**空行でなく、字下げが頭の行以下である最初の行の直前**まで。流れの形（`- {`）の頭の行は、その 1 行だけが block である。**判断の記録は file 1 本が 1 節点で、block は file の全行。**
2. **落とす。** block から ① 入れ子の節点の block（条の中の規範文） ② **辺の欄の行**（欄の行の字下げは 頭の行の字下げ + 2・判断の記録は 0。その行より深い字下げの続きの行も一緒に落とす） ③ 流れの形の行の中の**辺の欄の対**を落とす。
3. **畳む。** 末尾の空行を落とし、残った行を改行ごと（行の終わりの改行を含めて）そのまま連結した byte 列が本文である。**引用符も空白も字下げも、正本に書かれた byte のまま数える。**

**辺の欄の閉じた一覧**（本便が実装の型付きの定数 `crates/folio/src/graph.rs` の `EDGE_FIELDS` に置き、`design-intent/graph.yaml` の生成区間へ導出する）:

| 正本 | 辺の欄 |
| --- | --- |
| constitution.yaml | relations |
| rules.yaml | article・refs |
| srs.yaml | basis・goals・rules・adrs・verifies・verify.ac |
| adr（判断の記録） | basis・produced |

- **verify.ac は、要件の verify の欄の中の対 ac を指す**（`verify:` の行そのものは落とさない＝method と how は本文である）。実の正本の `ac` の対は 23 本あり、**23 本とも要件の verify の中である**（席が全数を数えた）。
- **図の欄（figures）と改訂の来歴の欄（amended_by・amends）は辺の欄に入れない。** 図は伝播に使わない辺で（判断の記録 ADR-13 決定 (2)）、改訂の来歴は前の文と理由という散文を持つ。どちらも本文の側に残し、変われば周の引き金になる（落とす側より止める側に倒す・条 P-4）。
- **一覧は落とす側（除外）で持つ。** 正本に新しい欄が増えたとき、採る側の一覧だと黙って本文の外へ落ちる（黙って通す）が、落とす側の一覧なら本文に入って周を呼ぶ（黙って止まらない）。条 P-4.1 の向きに揃える。

**流れの形の行から対を落とす式**（`- {id: R-1, article: N-5, …}` の形の行と、欄の値が `{` で始まる行）:

- 対の頭は **`{` の直後か `, ` の直後に現れる 欄の名 + コロン + 半角空白** とする。
- 対の終わりは、角括弧と波括弧の入れ子の深さを数え、**二重引用符（ASCII）の中を跳ばし**ながら右へ進み、**深さ 0 の最初のコンマか、閉じ括弧の直前**とする。二重引用符の中では逆斜線が次の 1 字を逃がす。
- 落とすときは対の区切りも 1 つだけ落とす＝直前が `, ` ならそれを、直前が `{` なら対の後ろの `, ` を落とす。
- **単引用符は引用符として数えない。** 実の正本と凍結した土台の両方で、単引用符も数える式と数えない式の出力は 1 字も違わなかった（席が両方で実測）。日本語の散文の中の 1 つだけのアポストロフィで式が崩れないほうを採る。

**式の細かい 3 点**（今の 3 つの置き場では出力を変えないが、runner の迷いを消すために字で閉じる）:

- **欄の決まりの file（`adr/schema.yaml`・`design-note/schema.yaml`）は節点でない。** 判断の記録の節点は `adr/ADR-<数>.yaml` の形の file だけである（索引が数えるのと同じ）。この 2 本は (d) の残差の母集団には入る（dir 形の文書の直下の .yaml だから）。誤ると節点が 190 になり、歯 1 と (c) の食い違いの断りが同時に落ちる。
- **対の頭を探すときも二重引用符の中は跳ばす。** 終わりの走査だけでなく頭の走査も跳ばす。今の正本では差が出ないが、`rules.yaml` の ruling や note の二重引用符の中に `, refs: ` の形の字が 1 度でも入れば、跳ばさない式はその日から静かに壊れる。
- **空白だけの行も空行とみなす**（block の終わりの判定と末尾の落としの両方で）。実の正本にも凍結した土台にも空白だけの行は 0 行である（席が全数を数えた）。

**要約値が 8 字である理由。** 印の今の束の要約値（`bundle: a9b8a6aa`）と同じ幅に揃える。198 節点での取り違え（別の本文が同じ 8 字になる）の見込みは約 4.6 × 10⁻⁶ で、起きても効くのは「その節点の変更を 1 度見落とす」ところまでである。撤退条件 6（周の引き金が漏れる）がそれを数える。

**実測（この式を独立の実装で当てた値）。** 母集団の全 byte は、本文・辺の欄・残差 の 3 つにちょうど 1 度ずつ入る。

| 置き場 | 節点 | 本文の byte | 辺の欄の byte | 残差の byte | 合計 |
| --- | --- | --- | --- | --- | --- |
| 実の design-intent | 198 | 416,811 | 6,490 | 189,058 | 612,359 |
| 凍結した土台 floor_base | 189 | 243,360 | 4,812 | 140,885 | 389,057 |
| 束の fixture ceiling/bundle/source | 23 | 5,972 | 369 | 9,831 | 16,172 |

**式が効くことの実測（席が独立の実装で確かめた）。**

- 凍結した土台の写しに、辺の欄だけの変更を 5 か所（憲法の条の relations に 1 つ・規則の行に refs の対を 1 つ・要件の basis に 1 つ・要件の verify の中の ac に 1 つ・判断の記録の basis に 1 つ）入れると、**要約値 189 本も残差も 1 字も動かない**（辺の欄の byte だけが 4,812 → 4,854 になる）。
- 実の正本の写しでも同じ 5 か所を足して、**要約値 198 本も残差も 1 字も動かない**（辺の欄の byte が 6,490 → 6,535）。
- 凍結した土台の写しで要件 FR1 の shall を 1 字変えると、**動く要約値は FR1 の 1 本だけ**。
- 生成区間（要件書の schema の節・判断の記録の欄の決まりの file）を 1 字変えると、**残差の要約値が変わる**（節点の要約値は 1 本も動かない）。判断の記録 ADR-13 決定 (8) の引き金 2（生成区間と欄の決まりの file）は残差が拾う。

**退けた案 1（正本を YAML として読み、本文の欄の値を並べて数える）**: 読む道具ごとに block の字の末尾の改行と引用符の扱いが割れるので、凍結 anchor を folio の実装と別に組めなくなる（条 P-10.2）。便 94 の 🔴 3 点目が挙げた理由そのもの。

**退けた案 2（行の単位だけで落とし、流れの形の行の中は落とさない）**: 規則の行 29 本と受入基準 18 本と目的 4 本は 1 行の流れの形なので、辺の欄がその行の中に入る。辺の後付け（判断の記録 ADR-13 決定 (3)）の受け皿 3 つのうち 2 つ（規則の行の refs・要件の adrs）はまさにその形で、案 2 だと**地図を良くする作業が毎回周を呼ぶ**。持ち主の裁定（2026-09-22 10:1x JST・「天井を高い頻度で走らせる種になるなら微妙」）の向きに反するので採らない。

**退けた案 3（節点の block から入れ子の規範文を落とさない）**: 条の要約値が規範文の字を含むので、規範文を 1 字直すと条も一緒に変わったことになる。変わった節点の一覧（後続の便 G5）が 2 倍に膨らみ、どちらが本当に変わったのか読めなくなる。

### (c) 索引の 5 つ目の欄（`folio graph --print`）

判断の記録 ADR-13 決定 (1) は節点の欄を 5 つ（id・種類・所属 file・欄の要約値・1 行の題）と定める。便 94 は 4 つで出した。**本便が 5 つ目を足す。**

- 節点の行は **id / 種類 / file / 要約値 8 字 / 題 36 字** のタブ区切りになる（要約値は題の前＝決定 (1) の並びのまま）。
- 見出しの行を `# 節点（1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り）` に替える。
- 辺の表・要約の 1 行・端が節点でない参照の数え方は 1 字も変えない。
- **索引の節点の集合と、行の逐語から切り出した節点の集合が食い違ったら、表を 1 行も出さずに「まだ分からない」（終了コード 2）で終える**（条 P-4.1 / P-4.2）。索引は欄の値を読んで組み、要約値は行の byte を読んで組むので、2 つの読みが別の答えを出したときに黙って片方を採らない。実の正本・凍結した土台・束の fixture の 3 つとも、2 つの集合は **198 / 189 / 23 で 1 つの狂いもなく一致する**（席が実測）。
- 出力の大きさ: 実の正本で 34,297 byte → **36,097 byte**（978 行は変わらない）。凍結した土台で 29,558 byte → **31,277 byte**（768 行）。

### (d) 天井の印の 2 つの欄（`folio ceiling --stamp`）

印（`design-intent/preview/ceiling-stamp.yaml`・生成物）の欄の閉じた一覧の**末尾に 2 つ足す**。前に在る 8 欄（round・at・verdict・sources・faces・viewpoints・refutes・reads）の字と順は 1 つも動かさない。

```
rest: sha256 <16 進 64 字>
nodes:
  - {id: P-1, digest: 8b0e9bb3}
  - {id: P-1.1, digest: 03796068}
```

- **nodes = 節点ごとの要約値の表**（索引と同じ並び＝id の byte 順）。実の正本で 198 行・6,635 byte。
- **rest = 残差の要約値**（「sha256 」に 16 進 64 字・sources と faces と同じ形）。**残差 = 母集団の全 byte から、節点の本文と辺の欄を除いた残り。** 母集団は、索引の正本（憲法・規則の表・要件書・判断の記録の各 file）と、天井の正本 ceiling.yaml の観点の reads が指した文書の file（dir 形の文書は直下の .yaml）の**和集合**を、置き場からの相対 path の byte 順に並べたもの。
- 印の大きさ: 976 byte → **7,689 byte**（976 + nodes の 1 行と 198 行で 6,635 + rest の 1 行で 78）。生成物 1 file の上書きなので、版管理の履歴は行単位の差分として読める（条 P-6.2）。
- **印に種類と file と題は写さない。** その 3 つは索引が毎回正本から組み直す導出物で、前の周から覚えておく必要があるのは要約値だけである（条 P-6.3）。設計ノート §4.3 は 4 列・約 16 キロバイトと見積もっていたので、その字の直しを (f) の 🔴 2 点目に上げる。
- **全部か無しか。** 表か残差を組めないときは、印を 1 byte も書かずに「まだ分からない」（終了コード 2）で終える（今の印の持ち方と同じ・条 P-4.1）。
- **印を書き直すのは次の周である。** 本便は `design-intent/preview/ceiling-stamp.yaml` を write-set に入れない（24 周目の `--stamp` が新しい形で書く）。面の天井の名札は印の at と viewpoints だけを読むので、欄が 2 つ増えても名札の字は変わらない（席が `crates/folio/src/face.rs` の読み口を逐語で読んで確かめた）。

### (e) 凍結 anchor（P-10.1 / P-10.2）

**独立の実装を 1 本置く。** `tests/fixtures/schema/node-digest.py` は **python3 の標準 library だけ**（YAML の読み口も folio の code も 1 行も呼ばない）で (b) の式を実装し、引数に取った置き場の節点の要約値と残差を標準出力へ出す。出力の形は次のとおり（1 行目は見出し・以降は id の byte 順・最後の 2 行）。

```
# 節点の要約値（1 行 = id / 要約値 8 字・タブ区切り・id の byte 順）
A-1	6d545583
A-1.1	df732d5b
…
# 残差 sha256 b23c6edf86905587739ad71ac9334836545c77705b0afcc345297e670fb239c2
# 節点 189・本文の byte 243360・辺の欄の byte 4812・残差の byte 140885・合計 389057
```

凍結 anchor は **`tests/fixtures/schema/node-digest-anchor.txt`** の 1 本で、**凍結した土台 `tests/fixtures/floor_base/design-intent` に当てた出力そのもの**（anchor の byte の範囲は file の全部＝見出しの行・189 の行・最後の 2 行・末尾の改行 1 つを含む）。

- anchor の行数 = **192**・byte 数 = **3,007**・要約値（sha256）= **e71740609965710c1994257d9454ad8485b65b55c75cb5c30a74cfd6d411e21f**
- **本便の前の main にはこの file が無いので、これを見る歯は赤い歯である。**
- 席はこの値を、folio の code を 1 行も呼ばない python3 の標準 library だけの実装で出し、byte 数と要約値は OS の道具 `sha256sum` と `wc` で独立に測った（2026-09-22）。
- 突き合わせは唯一の合格判定ではない（条 P-10.2）。anchor の中身の出所は独立の実装で、folio の出力との一致は歯が見る事実の側である。**python3 を起動できない環境では、要約値を測れないときと同じ形で「まだ分からない」の 1 行を標準エラーへ出して歯を落とさない**（条 P-10.3・便 97 と同じ持ち方）。

**作り直す anchor は 3 本、足す anchor は 2 本。**

| anchor | 今 | 本便の後 |
| --- | --- | --- |
| tests/fixtures/schema/graph-anchor.txt（便 94） | 節点の表 d2a7280c…・辺の表 a09c3f55…・出力全体 7b2fbfe4…（768 行・29,558 byte） | 節点の表 **79d4c248118598ffba90aa1db87fd6bc23bbb59bb12314786683c2481c61a8d2**・辺の表 **a09c3f558bcd16a6be1999eba9cbdedf1df116020ea5437eeae6ef5d86f4945c（不変）**・出力全体 **939fe249003aecb49e5f646f9527913bfd4d6f3023e7c231265b71e5fc66fa88**（768 行・**31,277 byte**）。要約の 1 行 `# 節点 189・辺 576・型 10・端が節点でない参照 26` は不変 |
| tests/fixtures/schema/graph-region.txt（便 95） | 28 行・2,106 byte・c4385eb3… | **35 行・3,607 byte・7e2515a7727d84e4df0449d54577f842778f4e244701622986f3098c51eb947b** |
| tests/fixtures/ceiling/findings/stamp-expected.yaml（便 72） | 10 行・559 byte | **34 行・1,342 byte**（末尾に nodes の 1 行と 23 行が付く） |
| +tests/fixtures/schema/node-digest.py ／ +tests/fixtures/schema/node-digest-anchor.txt | 無い | 独立の実装と 192 行・3,007 byte の anchor |

**辺の表の要約値が 1 字も変わらないこと**が、節点の行に欄を 1 つ足しただけで索引のほかの面を壊していないことの裏である（席が独立の実装で新しい 3 値を組み、辺の表の値が今の anchor と一致することを確かめた）。

**印の凍結 anchor に足す 23 行**（束の fixture `tests/fixtures/ceiling/bundle/source` の節点・id の byte 順・歯が写しの ceiling.yaml の reads を書き替えても 1 字も動かない値）:

```
nodes:
  - {id: A-1, digest: a82cf170}
  - {id: A-1.1, digest: ea5b6120}
  - {id: AC1, digest: 9bd1e10b}
  - {id: ADR-1, digest: 11a999d6}
  - {id: ADR-2, digest: fd8ac49d}
  - {id: CON1, digest: 56aa54e5}
  - {id: D-1, digest: 86b0b390}
  - {id: FR1, digest: a81a54e7}
  - {id: FR2, digest: 4aac2ab1}
  - {id: GOAL1, digest: fc191415}
  - {id: GOAL2, digest: 40d0b256}
  - {id: N-1, digest: 565c9a03}
  - {id: N-1.1, digest: 5543ff04}
  - {id: NFR1, digest: 6af464d5}
  - {id: P-1, digest: a4fc0435}
  - {id: P-1.1, digest: 0019a1f7}
  - {id: P-1.2, digest: 8e2b9724}
  - {id: R-1, digest: 81e06388}
  - {id: ai-session, digest: a65dd1fe}
  - {id: folio, digest: 68d4f1d6}
  - {id: owner, digest: 1eaacd04}
  - {id: prep-sheet, digest: de2c6e1c}
  - {id: verdict, digest: e68a2fbb}
```

**残差の要約値は印の anchor に凍結しない。** 歯は写しの ceiling.yaml の reads を 4 観点で揃えてから周を組むので、残差の母集団が写しの上で動く。`crates/folio/tests/stamp.rs` の anchor を作る関数（今 at と sources と faces と行の中の bundle と at を落としている）に **rest の行も落とす**のを足す。節点の要約値はこの書き替えで動かない（ceiling.yaml に節点が無いため）ので、表の 23 行はそのまま凍結する。

### (f) 生成区間（design-intent/graph.yaml）

判断の記録 ADR-13 決定 (1) と (8) は、節点の欄と欄の分類の写しを索引の欄の決まりの正本の生成区間へ導出せよと定める。**正本は実装の型付きの定数**（`crates/folio/src/graph.rs` の床の木）で、`folio schema --write` が写しを書く（条 P-5.1・P-5.6・要件書 FR19）。本便が替えるのは次の 3 つだけで、閉じた一覧 2 本（node_kinds 11・edge_types 17）は 1 語も動かさない。

1. `node` の欄の一覧に **digest** を足す（`node: {required: [id, kind, file, digest, title]}`）。
2. `node_note` の末尾の 1 文（欄の要約値は天井の印と同じ便で足す）を、digest の言い換えに替える。
3. 末尾に **edge_fields**（辺の欄の閉じた一覧・file ごと）と **edge_fields_note** と **digest_note**（(b) の式）を足す。

生成区間は 28 行・2,106 byte → **35 行・3,607 byte** になる。**替える行と足す行の逐語は次の 8 行で、この 8 行がそのまま 3,607 byte と要約値 7e2515a7… を決める**（席が導出の規則を写して組み、`sha256sum` と `wc` で測った。床の木の側の字がこの 8 行と 1 字でも違えば、生成区間の byte 数も要約値も §1 が凍結した値にならない）。

```
  node_note: 索引の節点 1 つの欄。id は設計文書の全体で 1 つに定まる id、kind は node_kinds の値、file は正本の置き場からの相対の path、digest は節点の本文の要約値（式は digest_note）、title は空白を 1 つに畳んで Unicode の字で 36 に切った 1 行の題
  edge_fields:
    constitution.yaml: [relations]
    rules.yaml: [article, refs]
    srs.yaml: [basis, goals, rules, adrs, verifies, verify.ac]
    adr: [basis, produced]
  edge_fields_note: 辺の欄の閉じた一覧（正本の file ごと・欄の名）。節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は周の引き金にならない（判断の記録 ADR-13 決定 (8)）。正本は実装の型付きの定数 crates/folio/src/graph.rs の EDGE_FIELDS で、この節はその写しである（P-5.1・P-5.6）。verify.ac は要件の verify の中の対を指す。図の欄（figures）と改訂の来歴の欄（amended_by・amends）は散文を持つので本文の側に残す
  digest_note: 節点の要約値の式（正本の読み口に依らず、行の逐語の byte で決まる）。① 節点の block は、その id を持つ行から、空行でなく字下げが頭の行以下である最初の行の直前まで（判断の記録は file の全行）。② block から、入れ子の節点の block と 辺の欄の行（その行より深い続きの行も）を落とし、流れの形の行からは辺の欄の対を落とす。③ 末尾の空行を落とし、残った行を改行ごと連結した byte の sha256 の先頭 8 字が要約値。天井の印はこの要約値の表と、節点にも辺の欄にも属さない残りの byte の要約値（残差）を持つ（ADR-13 決定 (8)）
```

内訳（行末の改行を含む byte）は 2,106 → 3,607 の +1,501 で、`node` の行が +8・`node_note` の行が **−16**（新しい行 325・古い行 341）・edge_fields の 5 行が +172・edge_fields_note が +593・digest_note が +744 である。**替えるのは 2 行、足すのは 7 行で、残る 26 行は今の anchor `tests/fixtures/schema/graph-region.txt` と 1 字も変わらない。**

`crates/folio/tests/schema_docs.rs` の f95_ の 3 つの定数（生成区間の行数 28・byte 数 2106・要約値 c4385eb3…）を **35・3607・7e2515a7727d84e4df0449d54577f842778f4e244701622986f3098c51eb947b** に替える。同じ file の歯 `f95_the_closed_lists_are_the_same_as_the_index` は索引の節点の行の 2 列目（種類）と辺の行の 3 列目（型）しか読まないので、欄が 1 つ増えても字は変わらない（席が本体を逐語で読んで確かめた）。

### (g) 本便が触らないもの・次の一括の 🔴

- **design-intent の下は `graph.yaml` の生成区間だけ**。憲法・規則の表・要件書・語彙・天井の正本・入口の棚・相談窓口・判断の記録・設計ノートの正本・凍結 anchor の列（anchors/）・生成物の棚（preview/）は読むだけである。**語彙も要件書も 1 字も動かさない。**
- **`crates/folio/src/main.rs`・`check.rs`・`refs.rs`・`link.rs`・`mentions.rs`・`adr.rs`・`schema.rs`・`rules.rs`・`entrance.rs`・`intake.rs`・`ceiling.rs`・`bundle.rs`・`site.rs`・面の生成器は 1 字も触らない。** `gate.rs` は**関数 2 つの可視化を crate の中へ広げるだけ**（門の判定も断りの字も 1 つも変えない）で、天井が読む文書の file を集める口を索引の側から呼べるようにする＝同じ集め方を 2 面に増やさないため（条 P-6.3）。
- **`crates/folio/tests/check.rs`・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・`tests/fixtures/schema/graph-digest-anchor.txt`・欄の決まりの写し 17 本は 1 字も触らない。**
- 🔴 **1 点目（G4 を 2 便に割った）**: 設計ノート §8 の G4 の行と判断の記録 ADR-13 決定 (16) の ⑤ は「節点ごとの要約値の印と周の引き金」を 1 便として数えている。本便は**式と表と索引の欄まで**で、**周の引き金を判定する口（要る／要らない／まだ分からない）を運ばない**。割った理由は 2 つ。① 式と anchor だけで src が 325 行前後の見積（(i) の取り直し）で M の見積 300 に並ぶので、判定の口まで入れれば確実に超える。② §3.4 の引き金 4 つのうち、**3（面の生成器と部品目録と様式）と 4（実態の観点の実装の証拠）は設計文書の中の byte では判定できない**ので、印にもう 2 欄が要る（3 は印の faces の要約値を面を組み直して測り直す形・4 は物差しが未定）。1 と 2 だけで「周は要らない」と出すと、判定していない 2 つを異常なしとして扱うことになる（条 P-4.1 が禁じる形）。**G4 を 2 便に改めるか、G4b を便の列に足すかを持ち主が決める。** 同じ項に 3 つ添える＝① 判断の記録 ADR-13 決定 (16) は「①〜⑤ と ⑦ ⑧ は**設計文書の正本を書き換えないか**持ち主の承認欄で発効するので、門の対象外である」と書くが、**本便（⑤ そのもの）は `design-intent/graph.yaml` の生成区間を書き換える**ので、決定 (16) が挙げた理由は当たらない（本便が決定 (16) を根拠に使わず持ち主の裁定だけを引いたのはそのためである） ② 設計ノート §8 の G4 の行は門の欄を「通す」と書くが、実測は **2（印が古い）** ③ 同じ行の触る src の欄は `stamp.rs`・`graph.rs` だけで、`gate.rs` と `design-intent/graph.yaml` が抜けている。次の一括の承認要求に載せる。
- 🔴 **2 点目（設計ノート §4.3 の印の見積が違う）**: §4.3 は「8 字の要約値と id と種類と file で 1 行 40〜60 byte、表全体で約 16 KB」と書くが、本便の表は id と要約値の 2 列で **198 行・6,635 byte**（印の全体で 7,689 byte）である。種類と file を写さないのは、索引が毎回正本から組み直す導出物だからである（条 P-6.3）。**同じ 1 文の「節点は 285」も実測とずれている**（索引の実測は **198**）。**設計ノートのこの 1 文の直しを次の一括に載せる**（値も判定も動かない字の直し）。
- 🔴 **3 点目（要件書 FR20 の版上げ）**: FR20 の規範文は印の中身を「周の id・観点ごとの 3 値と反証の結果・正本の要約値・面の要約値・束の要約値・起動の記録・読んだ文書の id」と数え上げている。本便が足す **節点ごとの要約値の表と残差の要約値はその字の外**にある。判断の記録 ADR-13 決定 (8) が定めているが要件の字は追いついていない。**FR20 の版上げを次の一括の承認要求に載せる**（便 94 の 🔴 1 点目の FR14 の版上げと同じ一括にまとめる）。本便は要件の字を 1 字も動かさない。
- 🔴 **4 点目（設計ノート §3.4 の辺の欄の字と実装の一覧が違う）**: **設計ノート §3.4**（`docs/design/graph-and-incremental-ceiling.md:379`）は辺の欄を「basis・rules・relations・rows.ref・rows.req」と書き、同じ節の少し上は関係の欄を「relations・basis・rules・refs」と書くが、本便の閉じた一覧は 10 欄（relations・article・refs・basis・goals・rules・adrs・verifies・verify.ac・produced）である。差は 2 つ。① 便 90〜92 が足した 3 欄（adrs・refs・produced）と、正本の欄がもともと分かれている 4 欄（article・goals・verifies・verify.ac）が設計ノートの字に入っていない。② **rows.ref と rows.req は設計ノートの節の欄**で、設計ノートの節は今の索引の節点でない（便 94 の 🔴 2 点目）ので当たらない。**判断の記録 ADR-13 決定 (8) は欄を数え上げず「欄の分類は実装の型付きの定数に置き、生成区間へ導出する」とだけ書く**（席が決定 (8) の全文と `design-intent/` の全数を読んで、rows.ref と rows.req が判断の記録に 1 か所も無いことを確かめた）ので、**直す先は設計ノートの字であり、判断の記録の改訂は要らない**。設計ノートの版を上げる項として次の一括の承認要求に載せる。
- 🔴 **5 点目（graph.yaml が天井の読む文書の一覧に無い）**: 残差の母集団は天井の正本 ceiling.yaml の観点の reads が指した文書と索引の正本だが、**`design-intent/graph.yaml` はそのどちらにも入っていない**（便 95 の 🔴 9 点目が同じ穴を挙げている）。したがって本便が替える graph.yaml の生成区間そのものは、残差の要約値を動かさない＝引き金 2 の穴が 1 つ残る。**入口の棚 `design-intent/index.yaml` にも graph の字は 1 件も無い**ので、判断の記録 ADR-13 決定 (1) の「この正本は入口の棚と天井の正本の文書の一覧に載る」は **2 面とも未了**である。ただし棚は自分の explain で「道具自身の正本は載せない」と書いており、`graph.yaml` も道具自身の正本なので、**棚に載せるべきかは自明でない＝決定 (1) の「入口の棚に載る」のほうを直す案も並べる**。**graph.yaml を天井の文書の一覧に載せる一括の項と同じ問いなので、そこへまとめて載せる。**
- 節点ごとの要約値の欄（便 94 の 🔴 3 点目）は本便が閉じるので、次の一括では上げない。

### (h) 歯（関数名は f99_ で始める・置き場は tests/graph.rs 6 本と tests/stamp.rs 3 本）

歯は実行 file 経由で測る（`env!(CARGO_BIN_EXE_folio)` を起動する形）。器の受付は verify の旗 `--bin` の次の語を filter 語と読むので、**verify の行に `--bin` は書かない**。

**`crates/folio/tests/graph.rs`（6 本）**

1. **f99_the_independent_script_matches_the_anchor** — 独立の実装 `tests/fixtures/schema/node-digest.py` を凍結した土台 `tests/fixtures/floor_base/design-intent` に当て、標準出力が凍結 anchor `tests/fixtures/schema/node-digest-anchor.txt` と **byte 一致**（192 行・3,007 byte）し、anchor の要約値が `sha256sum` で測って **e71740609965710c1994257d9454ad8485b65b55c75cb5c30a74cfd6d411e21f** であること。python3 を起動できない環境では「まだ分からない」の 1 行を標準エラーへ出して落とさない（条 P-10.3）。**赤い歯**。
2. **f99_the_index_carries_the_digest_column** — 凍結した土台に `folio graph --print` を当て、節点の行が 5 列で 4 列目が 16 進の小文字 8 字、見出しの行が新しい字、節点の表と出力全体の要約値が凍結 anchor `graph-anchor.txt` の新しい 2 値と一致し、**辺の表の要約値が今の値 a09c3f55… のまま**であること（768 行・31,277 byte・要約の 1 行は anchor の 2 行目と byte 一致）。**赤い歯**。
3. **f99_the_digest_column_is_the_independent_value** — 同じ出力の 4 列目 189 本が、独立の実装の anchor の同じ id の値と 1 本残らず一致すること（folio の出力を folio で確かめない・条 P-10.2）。**赤い歯**。
4. **f99_an_edge_only_change_moves_no_digest** — 一時 dir へ凍結した土台を写し、辺の欄だけを 5 か所（憲法の条の relations・規則の行の refs の対〔**凍結した土台には refs の欄が 1 つも無いので新設になる**〕・要件の basis・要件の verify の中の ac・判断の記録の basis）変えて `--print` を当て、**節点の表の 4 列目 189 本が 1 字も変わらない**こと。判断の記録 ADR-13 決定 (8) の「辺を足すだけの変更は引き金に数えない」の歯である。**赤い歯**。
5. **f99_a_body_change_moves_exactly_one_digest** — 同じ写しで要件 FR1 の shall を 1 字変えると、要約値が変わるのは **FR1 の 1 本だけ**であること。**赤い歯**。
6. **f99_a_scan_that_disagrees_is_inconclusive** — 一時 dir の写しの規範文の行を索引が読めない形に崩し（頭の行の id の欄を残したまま流れの形を壊す）、`--print` が終了コード **2（まだ分からない）**で表を 1 行も出さないこと。写しの file の名と中身の要約値が 1 つも変わらないこと（repo へ書かない・条 N-1.1）。**赤い歯**。

**`crates/folio/tests/stamp.rs`（3 本）**

7. **f99_the_stamp_carries_the_node_table** — 束の凍結 fixture で周を組んで `--stamp` を当て、印の欄の並びが round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes で、要約値を落とした印が凍結 anchor `stamp-expected.yaml`（**34 行・1,342 byte**）と byte 一致すること（nodes の 23 行は (e) の逐語）。**赤い歯**。
8. **f99_the_stamp_rest_is_recomputable** — 印の rest が `sha256 ` に 16 進 64 字の形で、同じ置き場に独立の実装を当てて出た残差の要約値と一致すること。python3 を起動できない環境では「まだ分からない」の 1 行を出して落とさない（条 P-10.3）。**赤い歯**。
9. **f99_the_stamp_node_rows_are_the_index** — 印の nodes の 23 行の id と要約値が、同じ置き場に `folio graph --print` を当てた節点の行の id と 4 列目に、**同じ数・同じ順で**一致すること。**赤い歯**。

9 本とも席が期待値を独立に出した（1・2・3・7 は独立の実装の出力と `sha256sum`、4・5・6・8・9 は不変条件と終了コード）。回帰は verify の 2 行目（`tests/graph.rs` と `tests/stamp.rs` と `tests/schema_docs.rs` の全部）と、`.vessel.toml` の common-verify（workspace 全体の nextest と clippy）が見る。

### (i) 大きさ

- src は `crates/folio/src/graph.rs`（409・余地 1091・行の逐語から block を切り出す口と辺の欄を落とす口と要約値と残差を組む口と 5 列目で **+300 行前後**の見込み）と `crates/folio/src/stamp.rs`（331・余地 1169・rest と nodes の 2 欄で **+25 行**）と `crates/folio/src/gate.rs`（252・余地 1248・関数 2 つの可視化だけで **+0 行**）の 3 本。
- 歯は `crates/folio/tests/graph.rs`（456・余地 1044・**+110 行**の見込み・題の欄を読む 1 か所を 4 列目から 5 列目へ直す）と `crates/folio/tests/stamp.rs`（412・余地 1088・**+55 行**・要約値を落とす関数に rest の行を足す）と `crates/folio/tests/schema_docs.rs`（1136・余地 364・**定数 3 つの値だけ**）。
- 独立の実装と凍結 anchor は新しい file 2 本（`tests/fixtures/schema/node-digest.py` と `tests/fixtures/schema/node-digest-anchor.txt`・192 行・3,007 byte）。
- size **M**（src の増えは合わせて **325 行前後**の見積で S の 100 を大きく超える。触る src 3 本の余地 1091 / 1169 / 1248 はどれも M の見積 300 を上回るので、行が入らないことは無い）。新しい dir は作らず、縮む file も無い。外部 crate は増やさない。
- **見積の取り直しの根拠。** 起草の時点の +165 行は楽観的だった。この式を (b)(d) の字だけから書いた独立の実装（python3）は、**式の核だけで 258 行**（流れの形の対を落とす走査が 126 行）・母集団と出力を足して 390 行になる。Rust 側は母集団を `gate.rs` から借りられるぶん短くなるが、この repo の書き方（doc 注釈と結果の持ち回り）では **+250〜350 行**が現実的である。
- **M の上限に当たったときの割り方。** `graph.rs` の増えが **350 行**を超えたら、そこで止めて 2 便に割る。切る所は**印の 2 欄**（`stamp.rs`・`gate.rs`・`tests/fixtures/ceiling/findings/stamp-expected.yaml`・`crates/folio/tests/stamp.rs` の f99_ 3 本＝合わせて +80 行）で、式と索引の 5 列目と生成区間と anchor 3 本はそのまま 1 便目に残る（(j) の撤退条件が言うとおり、索引の 5 列目と印の 2 欄は別々に運べる・別々に捨てられる）。**先に割らないのは、印の anchor を 2 度書き直すことになるからである**（1 便目で `nodes` の無い印を凍結し、2 便目でまた凍結し直す）。

### (j) 本便が運ばないもの・撤退条件

- **周の引き金を判定する口**（要る／要らない／まだ分からないの 3 値）と、§3.4 の引き金 3（面の生成器と部品目録と様式）と 4（実態の観点の実装の証拠）の物差し。後続の便へ（(g) の 🔴 1 点目）。
- 注意の先の一覧と、束への差し込みと、観点の問いの文の 1 文（設計ノート §8 の G5）。
- 天井の材料の束を読む欄まで絞ること（同 G3・便 98）・門の単位の変更（同 G9）・判断どうしの改訂の欄（同 G7a）・要件書の面を組む file を割ること（同 G7b）。
- 要件書・憲法・規則の表・語彙・天井の正本・入口の棚・相談窓口・判断の記録の本文。設計ノートの正本。図の正本と部品目録と様式。台帳への記帳。印の file そのものの書き直し（次の周が書く）。
- 撤退条件: **要約値の式が正本の書き方の変更で崩れたとき**（正本の行が流れの形から block の形へ、または逆へ書き替えられ、節点の要約値が中身と無関係に全部動いたとき）は、`graph.rs` から要約値と残差を組む口と 5 列目を外して見出しの行を戻し、`stamp.rs` から rest と nodes の 2 欄を外し、anchor 4 本を便の前の値へ戻し、f99_ の歯 9 本を消す。**便 1 本で戻せる**（ほかの src を 1 字も触らないため）。索引の 5 列目と印の 2 欄は別々にも捨てられる（5 列目を外しても印は組める・印の 2 欄を外しても索引は出る）。

## 2. 範囲

- 入れる: `crates/folio/src/graph.rs` の要約値の式（block の切り出し・辺の欄の落とし・流れの形の対の落とし・sha256 の先頭 8 字）と辺の欄の閉じた一覧と残差を組む口と節点の行の 5 列目と見出しの行と索引との食い違いの断り・`crates/folio/src/stamp.rs` の印の 2 欄（rest と nodes）・`crates/folio/src/gate.rs` の関数 2 つの可視化・`design-intent/graph.yaml` の生成区間（`folio schema --write` の出力）・凍結 anchor（作り直し 3 本と新設 2 本）（`tests/fixtures/schema/graph-anchor.txt` と `graph-region.txt` の作り直し・`+tests/fixtures/schema/node-digest.py` と `+node-digest-anchor.txt` の新設・`tests/fixtures/ceiling/findings/stamp-expected.yaml` への 24 行）・`crates/folio/tests/graph.rs` の f99_ 6 本と題の欄の列の直し・`crates/folio/tests/stamp.rs` の f99_ 3 本と要約値を落とす関数・`crates/folio/tests/schema_docs.rs` の定数 3 つ。
- 入れない: 周の引き金を判定する口・引き金 3 と 4 の物差し・注意の先・束の絞り込み・門の単位・印の file そのものの書き直し・要件書と設計ノートと判断の記録の本文・語彙・`crates/folio/src/main.rs` と `check.rs` と `refs.rs` と `link.rs` と `mentions.rs` と `adr.rs` と `schema.rs` と `rules.rs` と `entrance.rs` と `intake.rs` と `ceiling.rs` と `bundle.rs` と `site.rs` と面の生成器・`crates/folio/tests/check.rs`・`tests/floor_cases.yaml`・`tests/fixtures/schema/graph-digest-anchor.txt`・id の一覧の凍結 anchor・欄の決まりの写し 17 本・新しい dir・外部 crate。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| formula | 要約値の式 | `crates/folio/src/graph.rs`（block の切り出し・辺の欄の落とし・流れの形の対の落とし・sha256 の先頭 8 字） |
| fields | 辺の欄の閉じた一覧 | `crates/folio/src/graph.rs` の `EDGE_FIELDS`（正本・生成区間へ導出する） |
| rest | 残差 | `crates/folio/src/graph.rs`（母集団の全 byte − 本文 − 辺の欄・`gate.rs` の集める口を crate の中で借りる） |
| column | 索引の 5 つ目の欄 | `crates/folio/src/graph.rs` の節点の行と見出しの行 |
| stamp | 印の 2 欄 | `crates/folio/src/stamp.rs` の rest と nodes |
| region | 生成区間 | `design-intent/graph.yaml`（35 行・3,607 byte・`folio schema --write` の出力） |
| script | 独立の実装 | `tests/fixtures/schema/node-digest.py`（python3 の標準 library だけ） |
| anchors | 凍結 anchor | node-digest-anchor.txt（新）・graph-anchor.txt・graph-region.txt・stamp-expected.yaml |
| teeth | 歯 | `crates/folio/tests/graph.rs` の f99_ 6 本と `crates/folio/tests/stamp.rs` の f99_ 3 本 |

## 4. 検査（歯）

§1 (h) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 94（行 cq）・便 95（行 cr）・便 96（行 cs）の着地が前提で、**3 本とも着地済み**（main 0806433）。便 97（行 ct・検証中）とは write-set が 1 file も重ならないので、どちらが先に着地してもよい。
<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cu"
title = "設計文書の節点ごとの要約値の式を定めて凍結し、索引と天井の印に置く。要約値は節点の本文の byte の sha256 の先頭 8 字で、本文は正本を YAML として読まず行の逐語から切り出す＝節点の block（id を持つ行から、空行でなく字下げが頭の行以下の最初の行の直前まで・判断の記録は file の全行）から、入れ子の節点の block と 辺の欄（憲法の relations、規則の表の article と refs、要件書の basis と goals と rules と adrs と verifies と verify.ac、判断の記録の basis と produced）を落とし、流れの形の行からは辺の欄の対を落とし、末尾の空行を落として連結した byte である。辺の欄の閉じた一覧は実装の型付きの定数に置き、索引の欄の決まりの正本 design-intent/graph.yaml の生成区間へ導出する（35 行・3607 byte）。folio graph --print の節点の行に 4 列目として要約値を足し（id / 種類 / file / 要約値 8 字 / 題 36 字）、索引の節点と行の逐語から切り出した節点が食い違えば表を出さずに まだ分からない で終える。folio ceiling --stamp が書く印の欄の末尾に rest（母集団の全 byte から本文と辺の欄を除いた残差の要約値）と nodes（節点ごとの要約値の表）を足す。凍結 anchor は python3 の標準 library だけの独立の実装とその出力で、folio の code を 1 行も呼ばない。周が要るかどうかを判定する口は本便に入れない"
req = ["FR20", "FR14"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/src/stamp.rs", "crates/folio/src/gate.rs", "design-intent/graph.yaml", "tests/fixtures/schema/graph-region.txt", "tests/fixtures/schema/graph-anchor.txt", "+tests/fixtures/schema/node-digest.py", "+tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/ceiling/findings/stamp-expected.yaml", "crates/folio/tests/graph.rs", "crates/folio/tests/stamp.rs", "crates/folio/tests/schema_docs.rs"]
verify = ["cargo nextest run -p folio --test graph --test stamp f99_", "cargo nextest run -p folio --test graph --test stamp --test schema_docs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f99_ の歯 9 本（独立の実装 node-digest.py の出力が凍結 anchor node-digest-anchor.txt と byte 一致し 192 行 3007 byte で sha256 が e71740609965710c1994257d9454ad8485b65b55c75cb5c30a74cfd6d411e21f、python3 が無い環境では まだ分からない の 1 行で落ちない／凍結した土台の索引の節点の行が 5 列で 4 列目が 16 進 8 字、節点の表と出力全体の要約値が新しい anchor と一致し辺の表の要約値は a09c3f55 のまま 768 行 31277 byte／索引の 4 列目 189 本が独立の実装の anchor の値と 1 本残らず一致／辺の欄だけを 5 か所変えた写しで 4 列目 189 本が 1 字も動かない／要件 FR1 の shall を 1 字変えると動く要約値は FR1 の 1 本だけ／索引と行の逐語の節点が食い違う写しで終了コード 2 で表が 1 行も出ず写しの file を 1 つも変えない／印の欄の並びが round at verdict sources faces viewpoints refutes reads rest nodes で要約値を落とした印が凍結 anchor stamp-expected.yaml と byte 一致し 34 行 1342 byte／印の rest が独立の実装の残差の要約値と一致し python3 が無ければ落ちない／印の nodes の 23 行の id と要約値が同じ置き場の索引の節点と同じ数 同じ順で一致）が全部緑、crates/folio/tests/graph.rs と tests/stamp.rs と tests/schema_docs.rs の歯が全部緑、folio schema --check が 9 行とも一致、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
