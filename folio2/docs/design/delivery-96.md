# 設計: 便 96 — 全体像の短い出力 folio graph --digest と、席が始まるときの 1 行（FR14 / FR3）

- 要件: FR14（機械が読む id の索引を出す）/ FR3（気づかせる 1 行 — セッション開始時に folio が出す 1 行の口）
- 条: P-1.1（folio は生成し検査し知らせるところまで）/ P-4.1（実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（独立した凍結 anchor）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-10.3（anchor が維持できなくなった検査は「まだ分からない」に落とす）/ N-1.1（管理下の対象を回復不能に削除する操作を拒む）/ N-3.1（規則の例外機構を足さない）
- 出所: 判断の記録 ADR-13 決定 (11)（全体像を安く速く出す口を置く。CLAUDE.md には載せない。代わりに席が始まるときの 1 行の口（folio hello）に、設計文書が在るときの 1 行を足す）・決定 (16) の 3 本目（全体像の出力と席が始まるときの 1 行）。設計ノート docs/design/graph-and-incremental-ceiling.md §6・§8 の便の列の 5 本目（G2）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cs が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は凍結 anchor の 1 本だけで先頭に + を付ける・縮む file は無い・新しい dir は作らない）。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（読むだけ）ので、天井の門の対象外である（要件書 FR20 の規範文の 1 つ目）。席は本便の write-set をそのまま渡して `folio ceiling --gate` を実測し、**0（通す・断りの字は 設計文書の正本を書き換えない便）**を確かめた（2026-09-22）。
- 前の便: 便 94（行 cq・`folio graph --print`）と便 95（行 cr・`design-intent/graph.yaml`）。**2 本とも着地済み**で、本便の base は `main a39caa6` である。§1 の余地はその base の実測なので、**取り直しは要らない**（便 95 が `graph.rs` を 296 行から 351 行にした後の値を使っている）。
- 改訂 b（2026-09-22 16:0x JST・検証役の report handoff-2026-09-22/d96-verify.md への応答）: 1 行と歯 5 と契約表の 対 を 辺 に（777 は有向の辺の本数・対 699 とは別の数・byte は 76 のまま）。🔴 3 点目の「口の名」を「旗の名」に。数値・anchor・write-set は変えない。

## 1. 目的と中身

便 94 の `folio graph --print` は索引そのものを 978 行・34,297 byte で出す。これは索引が要るときには正しい大きさだが、**席が始まるときに最初に読むものとしては大きい**。設計ノート §6 の目的は 精度（AI が何が在るかを知って始める）と 実時間（全体像を得るまでの時間）の両方で、後者はこの 34 キロバイトでは取れない。

本便は間に 1 段を置き、**3 段の階段**にする。

| 面 | 行 | byte | 何が分かるか |
| --- | --- | --- | --- |
| 席が始まるときの 1 行（`folio hello`） | 1 | 76 | 設計文書が在ること・その大きさ・次に撃つ口の名 |
| 全体像の短い出力（`folio graph --digest`） | 49 | 1,105 | 種類ごとの節点の数・型ごとの辺の数・file ごとの節点の数 |
| 索引そのもの（`folio graph --print`・便 94） | 978 | 34,297 | 節点 1 つ 1 つと辺 1 本 1 本 |

正本 4 種（憲法・規則の表・要件書・判断の記録 13 本）の合計は 486,299 byte なので、**短い出力は正本の 0.23%** である。

### (a) 実測（2026-09-22・main a39caa6・便 95 の着地後）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **graph.rs 351・余地 1149**／**hello.rs 109・余地 1391**／**main.rs 564・余地 936**。**3 本とも余地が M の見積 300 を上回る。** 歯 = tests/graph.rs 325・余地 1175／tests/hello.rs 213・余地 1287。
- 歯の関数名の接頭辞。`grep -rn 'fn f96_' crates/folio/tests` は今 **0 本**。
- 実の正本に `folio graph --print` を当てた実測は **節点 198・辺 777・型 15・端が節点でない参照 27**（978 行・34,297 byte・終了コード 0）。索引を組むのに掛かる実時間は **0.055 秒**（debug の実行 file・実の正本）。**席が始まるたびに毎回組んでも足りる速さである。**
- 凍結した土台の写し `tests/fixtures/floor_base/design-intent` の実測は **節点 189・辺 576・型 10・端が節点でない参照 26**（便 94 の凍結 anchor の数えと同じ）。
- **公開する命令の閉じた一覧は動かない。** 本便は副命令を 1 つも足さず、既にある `graph` に旗を 1 つ足すだけなので、`crates/folio/tests/check.rs` の歯 `p1_commands_closed_list` の `CLOSED` 12 本は 1 字も変わらない（席が逐語を読んで確かめた）。**`crates/folio/tests/check.rs` は write-set に入れない。**
- **`folio graph` の旗は排他の群で閉じている。** `crates/folio/src/main.rs` の副命令 `Graph` は `ArgGroup::new(mode).required(true).args([print])` を持つ。本便はこの群に `digest` を足し、**2 つのうちちょうど 1 つが要る**形にする（clap の既定の群は複数指定を許さない）。ほかの副命令の字は 1 字も変えない。
- **`folio hello` の今の形。** 判定は 4 段で、① 止める設定 `.folio-quiet` が根に在れば何も出さない ② 正本の置き場に `constitution.yaml` が在れば（＝整備済み）何も出さない ③ 出した印が在れば何も出さない ④ どれでもなければ 1 行を出して印を作る。**本便が変えるのは ② だけ**で、①③④ の字と順は 1 字も動かさない。
- **`folio check` と `folio ceiling` と面の生成器と床の判定は 1 字も変わらない。** 本便は design-intent を読むだけで書かない（席が `check.rs` の固定の 7 file の一覧を読んで確かめた）。生成区間・凍結 anchor の列（design-intent/anchors/）・土台の写しの正本 tests/floor_cases.yaml・部品目録・様式・天井の材料の束は 1 byte も動かない。

### (b) 短い出力の形（採る案と、退けた案）

**採る案**: `folio graph --digest` は **3 つの表と要約の 2 行**を標準出力へ書く。1 行はタブ区切りで、最後の行も改行で終わる。`--print` と同じく **repo へは 1 byte も書かない**（ADR-13 決定 (4)・条 N-1.1）。

実の正本に当てた出力の頭（全 49 行）は次のとおり。

```
# 種類ごとの節点（1 行 = 種類 / 数・タブ区切り・閉じた一覧の順）
条	27
規範文	67
規則行	29
目的	4
要件	20
非機能要件	3
受入基準	18
制約	9
登場人物	5
出力	3
判断の記録	13
...
```

- **1 つ目の表は 種類ごとの節点の数**（11 行・閉じた一覧 `NODE_KINDS` の順・数が 0 の種類も行を出す）。
- **2 つ目の表は 型ごとの辺の数**（17 行・閉じた一覧 `EDGE_TYPES` の順・数が 0 の型も行を出す）。列は 2 つで、**表に出た辺の数**（両端が節点）と **端が節点でない参照の数**である。便 94 は端が節点でない参照を要約の 1 行の総数でしか出さないが、短い出力は**どの型で落ちたのかを型ごとに出す**（条 P-4.2）。
- **3 つ目の表は file ごとの節点の数**（節点を 1 つ以上持つ file だけ・file の名の byte 順）。判断の記録は 1 file 1 節点なので、行の数は 正本 3 file + 判断の記録の本数 になる。
- **要約の 1 行は `--print` の最後の行と 1 byte も違わない**（`# 節点 198・辺 777・型 15・端が節点でない参照 27`）。2 つの口が同じ索引から出ていることを歯 2 が byte で見る。
- **最後にもう 1 行、次に撃つ口の名を置く**（`# 索引そのもの（節点と辺の全行）は folio graph --print`）。1 行 → 短い出力 → 索引 の階段を、出力そのものが持つ。
- **表の行の数は閉じた一覧の長さで決まる**（11 + 17）ので、設計文書が増えても短い出力の行数は 3 つ目の表の分しか伸びない。
- 索引が組めないときは、**表を 1 行も出さずに「まだ分からない」（終了コード 2）で終える**（`--print` と同じ・ADR-13 決定 (4)・条 P-4.1 / P-4.2）。

**退けた案 1（`--digest` を `--print` の別名にする）**: 同じ字を出す口を 2 つ持つ意味が無く、どちらが何かを名で見分けられなくなる（条 P-6.3 の向き）。

**退けた案 2（`--digest` を置かず、席が始まるときの 1 行が `--print` を指す）**: 最初に読むものが 34 キロバイトになり、設計ノート §6 の実時間の軸が取れない。1 行と 34 キロバイトのあいだが空く。

**退けた案 3（設計ノート §6.1 の字どおり、`--digest` を 節点と辺の 2 つの表にする）**: その 2 つの表は便 94 の `--print` が既に出している。§6.1 の表が `--print` と違うのは **節点の行の 要約値 8 字 の欄**だけで、その欄は便 94 の 🔴 3 点目で **G4（印の anchor と一緒に定める便）へ回っている**。いま作ると要約値の式を本便で決め打つことになり、凍結 anchor が folio の実装に依らない形で作れなくなる（条 P-10.2）。**設計ノート §6.1 の名と本便の名のずれは (e) の 🔴 3 点目に上げる。**

### (c) 凍結 anchor（P-10.1 / P-10.2）

凍結 anchor は `tests/fixtures/schema/graph-digest-anchor.txt` の 1 本（便 94 の `graph-anchor.txt` と同じ既存の dir に置く・新しい dir は作らない）。

**短い出力は 46 行・1,048 byte に収まるので、要約値ではなく出力そのものを凍結する**（便 94 の索引は 768 行・29,558 byte あって写しとしては大きすぎたため 3 つの要約値を凍結した）。anchor の byte の範囲は **file の全部**（見出しの行・表の行・要約の 2 行・末尾の改行 1 つを含む）である。

- anchor の行数 = **46**・byte 数 = **1,048**・要約値（sha256）= **0c3c681dd0b7e6e4bf3880198598fee469bd8056dde5733ff1d91075766e665c**
- **本便の前の main にはこの file が無いので、これを見る歯は赤い歯である。**
- **席は folio の code を 1 行も呼ばずにこの中身を組んだ。** 凍結した土台の写し `tests/fixtures/floor_base/design-intent` を、別の言語の YAML の読み口（PyYAML）だけで読み、`crates/folio/src/graph.rs` の抽出の式（憲法の条と規範文・構造の辺の両向き・relations の 4 名前空間・amended_by／規則の表の 2 節の行と article と refs／要件書の 7 節の行と 6 つの欄と verify.ac／判断の記録の記録と basis と produced と figures と amends の target の最初の区切りまで）を写した独立の実装で節点と辺を組み、上の 3 表と要約の 2 行に畳んだ。byte 数と要約値は OS の道具 `sha256sum` で独立に出した（2026-09-22）。
- **その出力の要約の 1 行は、便 94 の凍結 anchor の数え（節点 189・辺 576・型 10・端が節点でない参照 26）と 1 字も違わなかった。** 別の言語の別の実装が同じ数に着いたので、抽出の式の写しが正しいことの裏が取れている。
- 突き合わせは唯一の合格判定ではない（条 P-10.2）。anchor の中身の出所は独立の実装であり、folio の出力との一致は歯が見る事実の側である。**anchor が維持できなくなったら、その検査の結果は「まだ分からない」に落とす**（条 P-10.3）。

anchor の逐語（46 行・タブ区切り・末尾は改行 1 つ）は次のとおり。

```
# 種類ごとの節点（1 行 = 種類 / 数・タブ区切り・閉じた一覧の順）
条	27
規範文	66
規則行	26
目的	4
要件	19
非機能要件	3
受入基準	17
制約	9
登場人物	5
出力	3
判断の記録	10
# 型ごとの辺（1 行 = 型 / 表に出た数 / 端が節点でない数・タブ区切り・閉じた一覧の順）
in-article	132	0
relations.articles	33	0
relations.reqs	25	0
relations.rules	47	0
relations.sections	0	5
amended_by	0	0
article	26	0
refs	0	0
basis	232	0
goals	27	0
rules	16	0
adrs	0	0
verify.ac	19	0
verifies	19	0
figures	0	21
produced	0	0
amends	0	0
# file ごとの節点（1 行 = file / 数・タブ区切り・file の名の byte 順）
adr/ADR-1.yaml	1
adr/ADR-10.yaml	1
adr/ADR-2.yaml	1
adr/ADR-3.yaml	1
adr/ADR-4.yaml	1
adr/ADR-5.yaml	1
adr/ADR-6.yaml	1
adr/ADR-7.yaml	1
adr/ADR-8.yaml	1
adr/ADR-9.yaml	1
constitution.yaml	93
rules.yaml	26
srs.yaml	60
# 節点 189・辺 576・型 10・端が節点でない参照 26
# 索引そのもの（節点と辺の全行）は folio graph --print
```

### (d) 席が始まるときの 1 行（crates/folio/src/hello.rs）

**変えるのは判定の ②（整備済み）だけ。** いまは何も出さずに終わる所を、索引の数を持つ 1 行に替える。

- 索引が組めたとき: 標準出力へ **`folio: 設計文書 198 節点・777 辺。全体像は folio graph --digest`** の形の 1 行（実の正本での実測・76 byte）を書き、終了コード 0 で終える。数は **実行のたびに索引から取る**（設計ノート §6.2 の字 285 節点・826 辺 は古い数で、本便はその数を実装に焼き付けない）。
- 索引が組めないとき（置き場に `constitution.yaml` は在るが、ほかの正本が読めない・欄の表でない）: 標準出力へ **`folio: 設計文書はあるが索引を組めない（まだ分からない）`** の 1 行を書き、標準エラーへ `folio hello: まだ分からない: 索引を組めない: ` に理由を続けた 1 行を書き、**終了コード 2**（まだ分からない）で終える。これは **印を書けないときの今の形と同じ持ち方**（1 行は出したまま 2 で終える・条 P-4.1）である。黙って 0 を返して「設計文書は無い」と読ませない。
- **この 1 行には印を付けない＝毎回出す。** 印（`--state` の下の `greeted`）は未整備の 1 行にだけ効く今の形を変えない。理由は、席が始まるたびに中身が空の AI が来るので、プロジェクト 1 つにつき 1 回にすると **最初の席しか全体像への入口を受け取れない**からである。1 行なので毎回出しても読み手の負担にならない。
- **止める設定は先に効いたまま。** `.folio-quiet` が根に在れば、整備済みでも何も出さない（判定の順は今のまま ① が最初）。
- **repo へは何も書かない**（条 N-1.1）。整備済みの枝は印も作らないので、`--state` の下にも 1 file も増えない。
- 索引の数を取るために、`crates/folio/src/graph.rs` に **節点の数と表に出た辺の数だけを返す小さな口**を 1 つ足す（`hello.rs` は索引の組み方を持たない＝同じ組み方が 2 面に増えない・条 P-6.3）。
- **要件 FR3 の規範文との差は (e) の 🔴 1 点目に上げる。** FR3 の起動の条件は `design-intent が未整備であるあいだ` で、規範文は `プロジェクト 1 つにつき 1 回のみ` と書く。本便が足す 1 行は整備済みのときのもので毎回出るので、**要件の字の外にある**。本便は要件書を 1 字も動かさない。

### (e) 本便が触らないもの・次の一括の 🔴

- **design-intent の下は 1 file も書き換えない。** 憲法・規則の表・要件書・語彙・天井の正本・入口の棚・相談窓口・判断の記録・索引の欄の決まり（便 95 の `graph.yaml`）・設計ノートの正本・生成区間・凍結 anchor の列（anchors/）・生成物の棚（preview/）のどれも読むだけである。**語彙に `hello` の項は無く（席が全数を探して 0 件）、本便は語彙も触らない。**
- **`crates/folio/src/check.rs`・`refs.rs`・`link.rs`・`mentions.rs`・`adr.rs`・`schema.rs`・`rules.rs`・`entrance.rs`・`intake.rs`・`ceiling.rs`・`bundle.rs`・`stamp.rs`・`gate.rs`・面の生成器は 1 字も触らない。** 床の判定も天井の印も門も変わらない。
- **`crates/folio/tests/check.rs`（`p1_commands_closed_list` と `r11` の助けの字の歯）と `tests/floor_cases.yaml` と id の一覧の凍結 anchor と欄の決まりの写し 17 本も 1 字も触らない。**
- **閉じた一覧 2 本を動かさない。** 短い出力は `NODE_KINDS` 11 語と `EDGE_TYPES` 17 語を**全数その順で行に出すだけ**で、語を足しも減らしも並べ替えもしない。したがって便 95 が置いた写しの側——`design-intent/graph.yaml` の生成区間（28 行・2,106 byte）・凍結 anchor `tests/fixtures/schema/graph-region.txt`・`crates/folio/src/schema.rs` の `TARGETS` 9 対と床の木・`crates/folio/tests/schema_docs.rs` と `crates/folio/tests/schema.rs` の凍結の定数——は **1 字も動かない**ので、write-set に入れない（席が便 95 の着地後の base で 5 つとも読んで確かめた）。もし後の便が閉じた一覧に語を足すなら、そのときはこの 5 つが同じ便で連れて動く。
- 🔴 **1 点目（要件書の版上げ）**: 短い出力に当たる要件は要件書に無い（FR14 の起動の条件は `folio build が実行されたとき`・索引の中身は 文書・要件・契約の id と 1 行の題）。席が始まるときの 1 行の要件 FR3 は 未整備であるあいだ に閉じており、整備済みの 1 行と毎回出す振る舞いを覆わない。設計ノート §8.1 は新しい要件 3 本（索引・全体像・注意の先）を挙げており、その 2 本目が本便に当たる。**便 94 の 🔴 1 点目（索引の要件）と同じ一括の承認要求にまとめて載せる**（FR14 と FR3 の改訂か、新しい要件かを含めて持ち主が決める）。
- 🔴 **2 点目（設計ノート §6.2 の非機能要件の断りが違う）**: §6.2 は 1 行なので非機能要件 NFR1 の母集団（行 R-1）にも収まる と書くが、**行 R-1 の母集団は AI へ常時渡す説明文（folio2 が所有する係と skill の名前と説明）であって、folio の標準出力は入っていない**（席が行 R-1 の `population` の欄を読んで確かめた）。席が始まるときの 1 行はこの母集団の外なので、予算を 1 byte も使わない。**設計ノートのこの 1 文の直しを次の一括に載せる**（値も判定も動かない字の直し）。
- 🔴 **3 点目（口の名のずれ）**: 設計ノート §6.1 と ADR-13 決定 (16) の 3 本目は、節点と辺の 2 つの表を出す口を `--digest` と呼ぶ。実装ではその 2 つの表は便 94 の `--print` が出しており、本便の `--digest` は (b) の短い出力である。**決定の字と実装の名が違うまま残ると、次の周の 整合 の観点が止める所見を立てる見込みが高い**（便 94 の 🔴 4 点目と同じ形）。設計ノート §6.1 の名を `--print` に直し、`--digest` を短い出力として書き足す改訂を **次の一括の承認要求に載せる**。判断の記録 ADR-13 の側は 決定 (11) が旗の名（--print / --digest）を 1 つも書いていないので、改訂は要らない（席が逐語を読んで確かめた）。
- 節点ごとの要約値の欄（便 94 の 🔴 3 点目・G4）は**既に上がっている**ので、本便では重ねて上げない。

### (f) 歯（関数名は f96_ で始める・置き場は tests/graph.rs 4 本と tests/hello.rs 4 本）

歯は実行 file 経由で測る（`env!(CARGO_BIN_EXE_folio)` を起動する形）。器の受付は verify の旗 `--bin` の次の語を filter 語と読むので、**verify の行に `--bin` は書かない**。

**`crates/folio/tests/graph.rs`（短い出力・4 本）**

1. **f96_the_digest_of_the_frozen_base_matches_the_anchor** — 凍結した土台の写し `tests/fixtures/floor_base/design-intent` に `--digest` を当て、終了コード 0・標準出力が凍結 anchor `tests/fixtures/schema/graph-digest-anchor.txt` と **byte 一致**（46 行・1,048 byte）すること。**赤い歯**（本便の前の main に旗も anchor も無い）。
2. **f96_the_digest_agrees_with_the_print** — 実の `design-intent` に `--digest` と `--print` を当て、① 短い出力の要約の 1 行が索引の最後の行と byte 一致 ② 種類ごとの数の合計が要約の 1 行の節点の数と一致 ③ 型ごとの 1 列目の合計が辺の数と、2 列目の合計が端が節点でない参照の数と一致 ④ file ごとの数の合計が節点の数と一致 すること。**数そのものは固定しない**（実の正本の数は設計文書を触る便のたびに動くため・便 94 と同じ断り）。**赤い歯**。
3. **f96_the_digest_rows_are_the_closed_lists** — 実の `design-intent` の短い出力の 1 つ目の表が **11 行**で閉じた一覧 `NODE_KINDS` と同じ語・同じ順、2 つ目の表が **17 行**で閉じた一覧 `EDGE_TYPES` と同じ語・同じ順であること（数が 0 の行も出る）。3 つ目の表の file の名が file の名の byte 順に並ぶこと。**赤い歯**。
4. **f96_a_broken_source_makes_the_digest_inconclusive** — 一時 dir へ実の正本を写し、`constitution.yaml` を消して `--digest` を当て、終了コード 2（まだ分からない）で標準出力に表が 1 行も出ないこと。同じ写しに `--digest` を 2 度当てて出力が byte 一致し、写しの dir の下の file の名と中身の要約値が 1 つも変わらないこと（repo へ書かない・N-1.1）。**赤い歯**。

**`crates/folio/tests/hello.rs`（席が始まるときの 1 行・4 本）**

5. **f96_hello_with_sources_names_the_counts** — 凍結した土台の写し `tests/fixtures/floor_base/design-intent` を `--dir` に、一時 dir を `--state` に渡して `folio hello` を当て、終了コード 0・標準出力が **1 行**で、その 1 行が **`folio: 設計文書 189 節点・576 辺。全体像は folio graph --digest`** と byte 一致し、印の置き場に file が 1 つも出来ないこと。**赤い歯**（いまは整備済みで何も出さない）。
6. **f96_hello_says_the_line_every_time** — 同じ当て方を 2 度繰り返し、2 度とも同じ 1 行が出て終了コード 0 で、印の置き場に file が 1 つも出来ないこと（1 回きりの印は未整備の 1 行にだけ効く）。**赤い歯**。
7. **f96_hello_without_an_index_is_inconclusive** — `constitution.yaml` だけを置いた写し（今の `Work::prepare`）に `folio hello` を当て、標準出力が **1 行**で `索引を組めない` を含み、終了コード **2**、標準エラーが `folio hello: まだ分からない: 索引を組めない: ` で始まる 1 行を持つこと。**この歯は、今の歯 `hello_prepared_says_nothing`（整備済みなら何も出さない）を名ごと置き換えるものである。** 置き換える理由は、本便が整備済みの枝の振る舞いを変えるからで、ほかの `hello_` の歯 7 本は 1 字も触らない。**赤い歯**。
8. **f96_the_quiet_file_silences_the_line_with_sources** — 凍結した土台の写しを一時 dir へ写し、その親に `.folio-quiet` を置いて `folio hello` を当て、何も出さずに終了コード 0 であること（止める設定は整備済みの 1 行にも先に効く）。**緑の歯**（本便の前は整備済みが黙るので今も通る）＝判定の順が入れ替わる回帰を見る歯である。

8 本とも席が期待値を独立に出した（1 本目と 5 本目は (c) の独立の実装の出力、ほかは不変条件と終了コード）。回帰は verify の 2 行目（`tests/graph.rs` と `tests/hello.rs` の全部）と、`.vessel.toml` の common-verify（workspace 全体の nextest と clippy）が見る。

### (g) 大きさ

- src は `crates/folio/src/graph.rs`（351・余地 1149・短い出力を組む口と数だけ返す口で +65 行の見込み）と `crates/folio/src/hello.rs`（109・余地 1391・整備済みの枝で +35 行）と `crates/folio/src/main.rs`（564・余地 936・旗 1 つと排他の群と分岐で +12 行）の 3 本。
- 歯は `crates/folio/tests/graph.rs`（325・余地 1175・+60 行の見込み）と `crates/folio/tests/hello.rs`（213・余地 1287・+65 行の見込み）。
- 凍結 anchor は新しい file `tests/fixtures/schema/graph-digest-anchor.txt`（46 行・1,048 byte）。
- size **M**（src の増えは合わせて 112 行の見積で S の 100 を超える。触る src 3 本の余地はどれも M の見積 300 を上回る）。新しい dir は作らず、縮む file も無い。外部 crate は増やさない。

### (h) 本便が運ばないもの・撤退条件

- 節点ごとの要約値の欄（便 94 の 🔴 3 点目・設計ノート §8 の G4）と、周の引き金。
- 注意の先の一覧と、束への差し込みと、観点の問いの文の 1 文（同 G5）。
- 天井の材料の束を読む欄まで絞ること（同 G3）・門の単位の変更（同 G9）。
- 要件書・憲法・規則の表・語彙・天井の正本・入口の棚・相談窓口・判断の記録・索引の欄の決まりの本文。設計ノートの正本。図の正本と部品目録と様式。台帳への記帳。
- 席が始まるときの 1 行をセッション開始へ結ぶ仕掛け（hook）。これは各プロジェクトの設定が持ち、folio の外に在る（要件書 FR3 の注）。
- 短い出力を file へ書くこと・CLAUDE.md へ載せること（ADR-13 決定 (11)・規則の表の行 R-2 の上限 8,000 byte・条 P-14.3）。
- 撤退条件: **短い出力が索引から決定的に畳めなくなったとき**（同じ索引から違う表が出るようになったとき）は、`main.rs` から旗 `--digest` を外して排他の群を `--print` 1 つへ戻し、`graph.rs` の畳む口と数だけ返す口を消し、`hello.rs` の整備済みの枝を今の何も出さない形へ戻し、anchor と f96_ の歯 8 本を消す。**便 1 本で戻せる**（ほかの src を 1 字も触らないため）。席が始まるときの 1 行が席の役に立たないと分かったときも、`hello.rs` の枝だけを戻せば短い出力は残る＝2 つは別々に捨てられる。

## 2. 範囲

- 入れる: `crates/folio/src/graph.rs` の短い出力を組む口（3 表と要約の 2 行）と節点と辺の数だけを返す口・`crates/folio/src/main.rs` の旗 `--digest` 1 つと排他の群の 2 本目と分岐・`crates/folio/src/hello.rs` の整備済みの枝（索引が組めたら 1 行と 0・組めなければ 1 行と理由と 2）・凍結 anchor `tests/fixtures/schema/graph-digest-anchor.txt` 1 本・`crates/folio/tests/graph.rs` の f96_ 4 本・`crates/folio/tests/hello.rs` の f96_ 4 本（うち 1 本は今の歯 `hello_prepared_says_nothing` の置き換え）。
- 入れない: design-intent の下の file の書き換え（読むだけ）・要件書と設計ノートの正本の版上げ・語彙・節点ごとの要約値・注意の先・周の引き金・束の絞り込み・門の単位・`crates/folio/src/check.rs` と `refs.rs` と `link.rs` と `mentions.rs` と `adr.rs` と `schema.rs` と `rules.rs` と `entrance.rs` と `intake.rs` と `ceiling.rs` と `bundle.rs` と `stamp.rs` と `gate.rs` と面の生成器・`crates/folio/tests/check.rs`・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・欄の決まりの写し 17 本・新しい dir・外部 crate。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| digest | 短い出力 | `crates/folio/src/graph.rs` の畳む口（種類ごと 11 行・型ごと 17 行 2 列・file ごと・要約 2 行） |
| counts | 数だけ返す口 | `crates/folio/src/graph.rs`（席が始まるときの 1 行が使う・索引の組み方を 2 面に増やさない） |
| cli | 旗 | `crates/folio/src/main.rs` の `--digest`（`--print` と排他・どちらか 1 つが要る） |
| line | 席が始まるときの 1 行 | `crates/folio/src/hello.rs` の整備済みの枝（毎回出す・印は付けない・repo へ書かない） |
| anchor | 凍結 anchor | `tests/fixtures/schema/graph-digest-anchor.txt`（46 行・1,048 byte・独立の実装で組んだ） |
| teeth | 歯 | `crates/folio/tests/graph.rs` の f96_ 4 本と `crates/folio/tests/hello.rs` の f96_ 4 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 94（行 cq）と便 95（行 cr）の着地が前提で、**2 本とも着地済み**（main a39caa6）。本便の write-set は着地済みの 2 便と重ならない（`graph.rs` は共通だが、2 便とも既に main に在る）。
<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cs"
title = "設計文書の索引を 3 つの表（種類ごとの節点の数 11 行・型ごとの辺の数 17 行 2 列〔表に出た数と端が節点でない数〕・file ごとの節点の数）と要約の 2 行に畳んで標準出力へ出す旗 folio graph --digest を置き、席が始まるときの 1 行の口 folio hello に、設計文書が在るときの 1 行（設計文書 N 節点・M 対。全体像は folio graph --digest）を足す。短い出力は 49 行・1,105 byte で、索引そのもの（便 94 の --print・978 行・34,297 byte）との階段を作る。要約の 1 行は --print の最後の行と byte 一致させ、表の行は閉じた一覧 NODE_KINDS と EDGE_TYPES の全数と順で出す（数が 0 の行も出す）。席が始まるときの 1 行は毎回出し（印は未整備の 1 行にだけ効く今の形を変えない）、索引が組めなければ 1 行を出したまま まだ分からない で終える。どちらの口も repo へ 1 byte も書かず、design-intent の下は 1 file も書き換えない"
req = ["FR14", "FR3"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/src/hello.rs", "crates/folio/src/main.rs", "+tests/fixtures/schema/graph-digest-anchor.txt", "crates/folio/tests/graph.rs", "crates/folio/tests/hello.rs"]
verify = ["cargo nextest run -p folio --test graph --test hello f96_", "cargo nextest run -p folio --test graph --test hello", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f96_ の歯 8 本（凍結した土台の短い出力が凍結 anchor と byte 一致し 46 行 1,048 byte／短い出力の要約の 1 行が --print の最後の行と byte 一致し 3 表の合計が要約の 3 つの数に一致〔数は固定しない〕／1 表が閉じた一覧 11 語・2 表が 17 語と同じ語と順で 3 表が file の名の byte 順／正本を 1 つ消すと終了コード 2 で表が 1 行も出ず 2 度当てて byte 一致し写しの file を 1 つも変えない／凍結した土台に hello を当てると 1 行が 設計文書 189 節点・576 辺 の字と byte 一致し印が 1 つも出来ない／2 度当てても同じ 1 行が出て印が増えない／正本が constitution.yaml だけなら 1 行を出したまま終了コード 2 で標準エラーが 索引を組めない を言う／止める設定が在れば整備済みでも何も出さず 0）が全部緑、crates/folio/tests/graph.rs と crates/folio/tests/hello.rs の歯が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
