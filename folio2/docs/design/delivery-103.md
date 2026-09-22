# 設計: 便 103 — 設計ノートの欄の決まりの索引の節を、着地した索引の欄の決まりを指す形に改め、事後の guard の一覧に散文の言及の歯を足す（FR19 / FR14）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から決定的に導出する）/ FR14（機械が読む id の索引を出す・第 1.28 版）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.2（文書からは規則を id で参照する）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-18.3（止める仕掛けは極性と段を型で持ちその一覧を出す）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）
- 出所: 天井の 25 周目（main 278ba59）の 2 本の所見。文書どうしの整合の F-1（重さ 止める・置き場 design-note の schema.index.index_note）＝設計ノートの欄の決まりの生成区間が、索引の中身を 文書・要件・契約の id と 1 行の題 と書き、口を 未実装 と書くが、要件書 FR14 の第 1.28 版はその字を置き換えた前の字として名指し、口は 2026-09-22 に着地したと書く。判断の記録 ADR-14 決定 (1) は節点を id を持つ行に閉じ、欄は id と種類と所属 file と欄の要約値と 1 行の題である。実態との整合の F-3（重さ 直す・置き場 design-note の schema.guards.post）＝便 93 で着地した規則の表の行 R-17 の床の歯（実装 crates/folio/src/mentions.rs・対象の file の閉じた一覧に design-note/ が在る）が、事後の guard の一覧 4 つに無い。所見の全文は 2026-09-22 の 25 周目の束の coherence と reality の findings.yaml。どちらも直す先が生成区間なので、実装の型付きの定数を直す便が要る。雛形は便 101（docs/design/delivery-101.md・同じ形で adr/schema.yaml の生成区間を直した便）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 da が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規の file は無い・縮む file も無い・消す file も無い）。
- 門: 本便は design-intent の下の正本（`design-note/schema.yaml`）を書き換えるので天井の門の対象である。席が実測した `folio ceiling --gate --write-set`（本便の 8 種）は **2（まだ分からない・印が古い）** を返す（2026-09-22・main 278ba59）。持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・逐語「一通り完成を目指して速度を上げたいので、あまりにも無駄に似たような審査を繰り返しまくっているならやめてどんどん進めて」・台帳 f2-648 notes）を規則の表の開発規律行 D-12 が言う持ち主の裁定として受け、設計文書を触る便は印が古くても門を経ずに出す（便 101 と同じ扱い）。
- 並行する便: 便 101（行 cx・main 168e7a6 に着地済み＝本便の base に含まれる・重なりは解けた）とは本便の起草時に **3 本**重なっていた＝`crates/folio/tests/schema.rs`・`tests/fixtures/schema/node-digest-anchor.txt`・`crates/folio/tests/graph.rs`。便 102（起草中・天井の束と印の側）とは `tests/fixtures/schema/node-digest-anchor.txt` と `crates/folio/tests/graph.rs` が重なる。**器が順に運び、後に着地する側が anchor を組み直す**（組み直し方は (h) のとおり独立の script で、先に着地した側の値から写さない）。便 98（行 cy・`bundle.rs` の側）とは 1 本も重ならない。

- 改訂 b（2026-09-22 22:1x JST・検証役の report handoff-2026-09-22/d103-verify.md への応答）: tests/schema.rs の着地後の値を 619（余地 81）に、便 101 は着地済みの旨に。数値と逐語と write-set は変えない。
## 1. 目的と中身

設計ノートの欄の決まり（`design-intent/design-note/schema.yaml`）の生成区間は、**索引について 2 つの古い字**を持つ。① 索引の中身を 文書・要件・契約の id と 1 行の題 と書き、その口を 未実装 と書く。② 事後の guard の一覧が 4 つのままで、便 93 で着地した規則の表の行 R-17 の歯を持たない。①は要件書 FR14 の第 1.28 版（口の名・起動の条件・索引の中身を着地した実装に合わせた版）と、判断の記録 ADR-14 の決定 (1) と両立しない。この生成区間の正本は実装の型付きの定数 `crates/folio/src/note.rs` の `FLOOR` なので、直すには便が要る。本便はその 1 本で、床の定数の字を今の事実に改め、`folio schema --write` で生成区間を導出し直し、凍結 anchor と写しと凍結の定数を揃える。

### (a) 実測（2026-09-22・main 278ba59・席は使い捨ての写しの木で測り、repo の src と tests と design-intent は 1 字も触っていない）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **`note.rs` 1354・余地 146**。本便が触る src はこの 1 本だけで、**余地 146 は S の見積（100）を上回るが M の見積（300）には足りない**。本便の実測の増分は **+14**（器の式・生の行では +9）で、着地後の `note.rs` は 1368・余地 132 になる。
- 歯の置き場の余地。`tests/schema.rs` は 624 だが、**便 89 の歯 `f89_schema_teeth_are_split_and_under_the_cap` が器の式で 700 を上限に凍結している**ので余地は 76 しか無い。`tests/schema_docs.rs` は 1137 で同じ歯の上限 1200 に対し余地 63。本便の歯 4 本と口 2 つは器の式で 112 行なので **どちらにも入らない**。置き場は設計ノートの歯の file `tests/note.rs`（536・余地 964）とする。席は当てた木で `tests/schema.rs` に置いた場合を実測し、732 になって上限の歯が落ちることを確かめた。
- 生成区間の今の値。**135 行 15305 byte**・要約値（sha256）836e07fadc3e32b897e975cab8454aacb4ca02507e992d2979b332c4d96a019f。凍結 anchor は `tests/fixtures/schema/note-region.txt` で、実の生成区間と byte 一致する。
- **欄の決まりの写しは 1 本だけ。** 席は `find tests -name schema.yaml -path '*design-note*'` で全数を列挙し、**`tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の 1 本**だけであることを確かめた（判断の記録の側が 17 本だったのと違う）。念のため字面でも測り、事後の guard の一覧の行（post の行）を持つ file は repo 全体で **3 本**（実の正本・この写し・凍結 anchor）だけだった。
- **写しの注は直さない。** 床の突き合わせ `floor_diff` は `strip_notes` を通した木を見るので、名前が `_note` で終わる欄は突き合わせの外である（`crates/folio/src/schema.rs`）。実測でも、この写しの 4 つの注は便 57 より前の字のまま残っており、床は落ちていない。本便も**構造の欄だけ**を写しに反映し、注は触らない（便 101 と同じ扱い）。
- **索引の節と事後の guard の一覧を読む枝は 1 本も無い。** 席は `note.rs` の中で `index` と `guards` の字が `FLOOR` の中にしか無いことを確かめた。本便は**床の判定の枝を 1 本も足さず、1 本も変えない**＝直すのは `FLOOR` の字と file の頭の注だけである。
- 索引の欄の決まりの正本は `design-intent/graph.yaml`（便 95 で起草・生成区間の正本は `crates/folio/src/graph.rs`）で、節点の欄 node（id・kind・file・digest・title）・節点の種類 node_kinds（11）・辺の欄 edge（from・to・type）・辺の型 edge_types（17）を持つ。
- 歯の関数名の接頭辞。`grep -rn 'fn f103_' crates/folio` は今 0 本（使われている最大は f101_・便 102 が f102_ を使う見込み）。
- workspace の nextest は今 **760 本**が全部緑。席は当てた木で本便の全部を入れて **764 本が全部緑**、`cargo clippy --workspace --all-targets -- -D warnings` が 0 警告であることを実測した。

### (b) 直す先と、新しい字

床の定数 `crates/folio/src/note.rs` の `FLOOR` の **index の節**と **guards の節**、および同じ file の頭の注の 1 行を直す。

**① index の節。** 写しの一覧 2 つ（entries・entry_fields）を落とし、索引の欄の決まりの正本を指す欄を 4 つ置く。指す形は同じ `FLOOR` の figures の節が既に使っている形（type_enum_ref が 部品目録の file と欄 を指す）と同じで、**値は 置き場 と 欄の名 の 2 語**である。同じ一覧を 2 つの面が持つ形を避ける（P-6.3）ため、種類も欄も**写しを持たず置き場だけを指す**。

**② guards の節。** 事後の検査の一覧（post）の末尾に prose-mentions を足して 5 つにし、注にその歯の出所を書く。名は同じ一覧の中で散文を見るもう 1 つの検査（prose-gate）と揃えた。

**③ file の頭の注。** 索引が未実装だと書いている行を、着地した事実に直す。

### (c) 床の側（`crates/folio/src/note.rs` だけを触る）

1. `FLOOR` の index の節の葉を差し替える。落とすのは entries（3 語の閉じた一覧）と entry_fields（2 語の閉じた一覧）、置くのは node_fields_ref・node_kinds_ref・edge_fields_ref・edge_types_ref の 4 つで、値はどれも 1 行の字である。並びは索引の欄の決まりの並び（節点の欄 → 節点の種類 → 辺の欄 → 辺の型）に揃える。注 index_note は (d) の逐語に置き換える。
2. `FLOOR` の guards の節の post の一覧に prose-mentions を足す（末尾・5 つ目）。注 guards_note は (d) の逐語に置き換える。
3. file の頭の注（`//!` の 6 行目）を (d) の逐語に置き換える。
4. **床の判定の枝は 1 本も足さず、1 本も変えない。** (a) のとおりこの 2 つの節を読む枝は無く、突き合わせは `floor_diff` が `FLOOR` の木の字面で行う。新しい module・新しい旗・例外の口（無効化の旗・今回だけの口）は持たない（N-3.1）。
5. 閉じた一覧が 2 つ減るが、**減った先はどちらも別の正本（索引の欄の決まり）が既に閉じて持っている**ので、閉じた一覧の総数は減らない（P-2.4 の求める「閉じた一覧で持つ」は満たしたまま）。判断の記録は新しく起こさない（ADR-14 決定 (1) と決定 (2) の射程の中である）。

### (d) 生成区間と凍結 anchor

`folio schema --write` が `design-intent/design-note/schema.yaml` の生成区間を **135 行 15305 byte から 137 行 16424 byte** に書き直す。ほかの 8 file の生成区間は 1 byte も動かない（席が `folio schema --check` を当て、9 file とも 一致 で終了コード 0 になることを実測した）。`schema.rs` の TARGETS は既に `design-note/schema.yaml`（`note::FLOOR`）を持つので **`schema.rs` も `main.rs` も 1 行も触らない**。要件 FR19 の対象も増えない。

変わるのは次の **3 か所**だけである。

- index の節の 3 行（`  index:` の次）を、この 5 行に置き換える。

```
    node_fields_ref: design-intent/graph.yaml node
    node_kinds_ref: design-intent/graph.yaml node_kinds
    edge_fields_ref: design-intent/graph.yaml edge
    edge_types_ref: design-intent/graph.yaml edge_types
    index_note: 要件書 FR14。機械が読む id の索引は、設計文書の正本から毎回組み直す導出物として口 folio graph --print が出す（2026-09-22 着地・実装 crates/folio/src/graph.rs・台帳 f2-648.132）。索引は節点（設計文書の中で id を持つ行）と辺（両端の節点の id と型）を持ち、節点と辺の欄も、節点の種類と辺の型の閉じた一覧も、索引の欄の決まり design-intent/graph.yaml が正本として持つ＝この節はその置き場を指すだけで写しを持たない（P-6.3）。判断の記録 ADR-14 決定 (1) のとおり節点は id を持つ行に閉じるので、設計ノートの節と契約表の行は節点にならない。索引の中身そのものは版管理に置かず、中身を席へ届ける経路は器の役割の注入が持つ（要件書 CON9）
```

- 事後の検査の一覧の 1 行を、この 1 行に置き換える。

```
    post: [yaml-form, derived-diff-zero, own-id-space, prose-gate, prose-mentions]
```

- 注 guards_note の 1 行の末尾（`…「まだ無い検査」として寄せる）` の直後）に、この字を継ぎ足す。

```
。prose-mentions は規則の表の行 R-17 の床の歯（2026-09-22 着地・実装 crates/folio/src/mentions.rs・台帳 f2-648.131）で、対象の file の閉じた一覧に design-note/ が在る＝設計ノートの散文の欄に現れた id が、その行の型付きの欄にも相手の行の型付きの欄にも無ければ事後に数える
```

file の頭の注（生成区間の外・`note.rs` の 6 行目）は次の 1 行に置き換える。

```
//! 散文の門（FR12・rules 行 R-16）は便 24 で入り、索引（FR14）は 2026-09-22 に着地した（folio graph --print・graph.rs）。導出物（FR11）は未実装（欄の決まりの注が明記する）。
```

凍結 anchor `tests/fixtures/schema/note-region.txt` は、生成器にも検査側にも依らずに組む（P-10.2）。組み方は、いまの anchor（135 行）の上の 3 か所を上の逐語で差し替えることで、OS の道具だけで足りる。**席は 2 通りで独立に組んで突き合わせた**＝① 床の定数を直した binary の `folio schema --write` が書いた file から生成区間を切り出したもの、② いまの anchor に上の 3 か所を差し込んだもの。2 つは **1 byte も違わず**、行数・byte 数・要約値も同じだった（2026-09-22）。

- 新しい anchor の行数 = **137**・byte 数 = **16424**・要約値（sha256）= **bcc821c9ccf458e06cc2591896901a5e3c11358762117e9e5eb59df631e99a8f**
- **本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**

凍結の定数は `crates/folio/tests/schema.rs` の 3 つを直す。NOTE_REGION_LINES を 135 から 137・NOTE_REGION_BYTES を 15305 から 16424・NOTE_REGION_SHA256 を上の要約値へ。同じ file の歯 `schema_design_note_region_matches_the_frozen_anchor_and_says_unimplemented` は **注 4 つ**（folio_check_note・derived_note・index_note・guards_note）が 未実装である を含むことを見ているが、本便で index_note がその字を失うので、**見る注を 3 つ**（folio_check_note・derived_note・guards_note）に直す。残る 3 つが 未実装である を持つことは実測で確かめた（導出物の差分 0 の口が入るまでの字である）。**`tests/schema_docs.rs` は 1 字も触らない。**

### (e) 欄の決まりの写し 1 本

`folio check` は `design-note/schema.yaml` の schema の節を床の定数と突き合わせる（違反「床の定数と違う」）ので、欄の集合を変えると写しの側も直さないと歯が落ちる。直すのは (a) のとおり **`tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の 1 本**だけで、直すのは**構造の 2 か所**である。

1. index の節の 2 行（entries と entry_fields）を、(d) の 4 つの指す欄の行に置き換える。
2. 事後の検査の一覧の 1 行を、(d) の 5 つの一覧の行に置き換える。

この写しの注（index_note・guards_note ほか）は**直さない**＝(a) のとおり注は突き合わせの外で、この写しは便 57 より前の注の字を凍結の土台として持ち続けている。席は当てた木でこの 2 か所だけを直し、workspace の nextest が全部緑になることを実測した。

### (f) 出さないもの — 面と要件書と判断の記録

- **要件書・規則の表・憲法・語彙・判断の記録は 1 字も触らない。** 本便が合わせる先（要件 FR14 の第 1.28 版・判断の記録 ADR-14 の決定 (1)・規則の表の行 R-17）はどれも発効済みで、新しい要件も新しい行も要らない。行の追加も変更も起きないので裁定 id は要らない（P-17.1）。
- **索引の欄の決まり `design-intent/graph.yaml` も 1 字も触らない。** 本便はその置き場を指すだけで、指す先の字は動かさない。
- **面（`face_*.rs`）にも入口の棚にも出さない。** 設計ノートの欄の決まりは人が読むページを持たない。
- **見本 `design-intent/design-note/example.yaml` の索引の行は本便で直さない。** 25 周目の実態との整合の F-2（重さ 直す・置き場 example の 口の表 の索引の行）が同じ古さを指しているが、この file は生成区間ではなく手書きの見本なので、便ではなく設計文書の側の PR（次の一括）で直す。(i) に置く。

### (g) 歯（関数名は f103_ で始める・置き場は `crates/folio/tests/note.rs`）

置き場の理由は (a) のとおり（`tests/schema.rs` と `tests/schema_docs.rs` はどちらも便 89 の上限の歯に余地が無い）。この file は `repo_root` を既に持つので、生成区間を切り出す小さな口（`schema_region`）と、生成区間から字下げを落とした 1 行を引く口（`note_schema_line`）を 2 つだけ足す。新しい歯の file も新しい dir も作らない。

1. `f103_the_index_section_points_at_the_index_schema` — 実の生成区間の index の節が、4 つの指す欄（node_fields_ref・node_kinds_ref・edge_fields_ref・edge_types_ref）を (d) の逐語のとおり持ち、写しの一覧の行（entries と entry_fields）が 1 本も残っていないこと。本便の前の main には指す欄が 1 つも無いので **赤い歯**。
2. `f103_the_index_refs_resolve_in_the_index_schema` — 生成区間の中で 索引の欄の決まりを指す 値を持つ欄を全部拾い、その欄の名が指す欄の形（末尾が _ref）であり、指す先の欄が `design-intent/graph.yaml` の生成区間に実在すること。拾えた数がちょうど 4 であること。同じ理由で **赤い歯**。
3. `f103_the_index_note_names_the_landed_port` — 注 index_note が 着地した口の名 と 判断の記録 ADR-14 と 索引の欄の決まりの置き場 と 制約 CON9 を名指し、未実装 の字を持たないこと。同じ理由で **赤い歯**。
4. `f103_the_post_guards_carry_the_mentions_tooth` — 事後の検査の一覧が (d) の逐語のとおり 5 つで、注 guards_note が prose-mentions と 規則の表の行 R-17 と 歯の実装の置き場 と 対象の file の一覧に在る設計ノートの dir を名指すこと。同じ理由で **赤い歯**。

**4 件とも席が当てた木で実測した**＝本便の全部を当てた木で 4 件とも緑、実の生成区間だけを main の字に戻した木で 4 件とも赤（2026-09-22）。

回帰は verify の 2 行目で見る。`tests/note.rs`（本便の前の本数 ＋ f103_ の 4 本）と `tests/schema.rs`（本数は変わらず、凍結の 3 つの値と見る注の数だけが変わる）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない（`note.rs` の中の単体の歯 `note_floor_derives_the_frozen_anchor_byte_for_byte` と、便 89 の上限の歯 `f89_schema_teeth_are_split_and_under_the_cap` は `.vessel.toml` の common-verify の workspace の nextest が見る）。席は当てた木で **workspace の nextest 764 本が全部緑**であることを実測した（本便の前は 760 本）。

### (h) 大きさ

src は `crates/folio/src/note.rs` の 1 本だけ（1354・余地 **146**・**+14 の実測**・着地後は 1368・余地 132）。歯は `crates/folio/tests/note.rs`（536・余地 964・**+112 の実測**・着地後は 648）と `crates/folio/tests/schema.rs`（624・**−5 の実測**・着地後は 619〔rustfmt が 3 項の配列を 1 行に畳む〕・便 89 の上限 700 に対し余地 81）。正本は `design-intent/design-note/schema.yaml` の生成区間が 135 行から 137 行になる。凍結 anchor `tests/fixtures/schema/note-region.txt` は 135 行から 137 行になり、欄の決まりの写し 1 本は 2 か所が変わる。size **S**（触る src は `note.rs` 1 本で、余地 146 は S の見積 100 を上回る。**M の見積 300 には足りないので本便は S に収める**）。新しい file も新しい dir も無く、縮む file も消す file も無い。外部 crate は増やさない。

**便 99 の凍結 anchor の巻き添え。** 本便が凍結の土台 `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の写しを直すと、便 99 の凍結 anchor `tests/fixtures/schema/node-digest-anchor.txt` の**残差の 2 行**（191〜192 行 = 残差の要約値と byte の内訳）が変わる。この写しは節点を持たない file なので、**節点の要約値の 189 行（1〜189 行）は 1 byte も動かず**、`graph-anchor.txt` も動かない。席は実測し、変わるのは残差の要約値と、残差の byte（141004 → 141162）と、合計の byte（389176 → 389334）だけで、**anchor file の行数 192 と byte 数 3007 は変わらない**ことを確かめた（差分は +158 byte = 写しの 2 か所の増分）。直し方: 写しを直した後に独立の script `tests/fixtures/schema/node-digest.py` を土台に当てて anchor を組み直し（folio の出力から写さない）、`sha256sum` で測ったその file の要約値を `crates/folio/tests/graph.rs` の定数 `F99_ANCHOR_SHA256` に写す。**新しい要約値 = ed795897c3fb9d413c19e94269029f540f81981ab47b2441075230b33e980b45**（席の実測）。残差以外の行が 1 byte でも変わったら本便の写しの直しが間違っている。回帰は verify の 3 行目（歯 `f99_the_independent_script_matches_the_anchor`）で見る。**便 101 と便 102 もこの 2 file を触る**ので、後に着地する側は同じ独立の script で組み直し、先に着地した側の値から写さない。

### (i) 本便が運ばないもの・撤退条件

- 見本 `design-intent/design-note/example.yaml` の索引の行の古い字（25 周目の実態との整合の F-2）。生成区間ではなく手書きの見本なので、設計文書の側の PR（次の一括）で直す。
- 索引の欄の決まり `design-intent/graph.yaml` の側の字。本便は指すだけで、指す先は動かさない。
- 規則の表の行 R-17 の注が言う「機械の読みの 5 つの閉じた一覧の写しを設計文書の置き場へ導出する」便。その行の注が後続だと明記しており、置き場も欄の決まりも別に決める。
- 25 周目のほかの所見（文書どうしの整合の F-2・F-3・F-4・F-5・F-6 と、実態との整合の F-4）。どれも要件書・語彙・判断の記録の側の字で、生成区間ではない。
- 憲法の条文・規則の表・語彙・要件書・判断の記録・入口の棚・ほかの 8 file の生成区間・`face_*.rs`・`link.rs`・`refs.rs`・`check.rs`・`rules.rs`・`schema.rs`・`main.rs`・`graph.rs`・`mentions.rs`・`tests/floor_cases.yaml`・`tests/schema_docs.rs`・id の一覧の凍結 anchor・CI の yml。便 99 の anchor の残差の 2 行と `tests/graph.rs` の定数 1 つは (h) のとおり直す（運ばないものではない）。
- 撤退条件: 索引の欄の決まりを**指す**形（写しを持たない形）が、設計ノートの欄の決まりを読む人に届かなくなったとき＝指す先の欄の名が変わって指す欄が宙に浮く事故が 2 度起きたとき（数えるのは歯 2 の落ちた回数で、記帳は台帳 f2-648）は、指す形をやめて索引の節そのものを落とし、索引の欄の決まりだけを正本とする便を 1 本で出す。写しを持つ形へ戻すことはしない（同じ一覧を 2 つの面が持つ形に戻るため・P-6.3）。

## 2. 範囲

- 入れる: 床の定数の index の節の差し替え（写しの一覧 2 つ → 指す欄 4 つ）・注 index_note の置き換え・事後の検査の一覧に 1 語・注 guards_note の継ぎ足し・file の頭の注 1 行・生成区間 135 行 → 137 行・凍結 anchor 1 本・欄の決まりの写し 1 本の 2 か所・`tests/schema.rs` の凍結の定数 3 つと見る注の数・`tests/note.rs` の口 2 つと f103_ の歯 4 本・便 99 の anchor の残差の 2 行と `tests/graph.rs` の定数 1 つ。
- 入れない: 見本 example.yaml・索引の欄の決まり graph.yaml・要件書と規則の表と憲法と語彙と判断の記録・面と入口の棚・床の判定の枝・`schema.rs` と `main.rs` と `mentions.rs` と `graph.rs`・`tests/schema_docs.rs`・`tests/floor_cases.yaml`・新しい file と新しい dir。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| refs | 指す欄 | 索引の欄の決まりを指す 4 つの欄（節点の欄・節点の種類・辺の欄・辺の型） |
| floor | 床の定数 | `crates/folio/src/note.rs` の `FLOOR` の index の節と guards の節・file の頭の注 |
| region | 生成区間 | `design-note/schema.yaml` の生成区間 137 行（`folio schema --write` が書く） |
| anchor | 凍結 anchor | `tests/fixtures/schema/note-region.txt`（137 行・16424 byte） |
| copy | 欄の決まりの写し | `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の 2 か所 |
| pins | 凍結の定数 | `tests/schema.rs` の 3 つの値と見る注の数・`tests/graph.rs` の便 99 の要約値 |
| rest | 便 99 の残差 | `tests/fixtures/schema/node-digest-anchor.txt` の 191〜192 行 |
| teeth | 歯 | `crates/folio/tests/note.rs` の口 2 つと f103_ 4 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 98（行 cy）とは書き換える file が 1 本も重ならない。便 101（行 cx）と便 102（起草中）とは §1 の冒頭のとおり重なり、器が順に運ぶ。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "da"
title = "設計ノートの欄の決まりの生成区間の索引の節を、着地した索引の欄の決まり design-intent/graph.yaml を指す形へ改める。写しの閉じた一覧 2 つ（索引に載る種類・1 つあたりの欄）を落とし、節点の欄・節点の種類・辺の欄・辺の型の 4 つを 置き場 と 欄の名 の 2 語で指す欄に置き換え、注は着地した口 folio graph --print と判断の記録 ADR-14 決定 (1) と制約 CON9 を名指して 未実装 の字を落とす。あわせて事後の guard の一覧に、便 93 で着地した規則の表の行 R-17 の床の歯 prose-mentions を 5 つ目として足し、注にその出所（実装 crates/folio/src/mentions.rs・対象の file の閉じた一覧に design-note/ が在ること）を書く。生成区間の正本は実装の型付きの定数 note.rs の FLOOR で、design-note/schema.yaml の生成区間へ folio schema --write で導出する。床の判定の枝は 1 本も足さず 1 本も変えない（この 2 つの節を読む枝は無く、突き合わせは床の木の字面で行う）。凍結 anchor と欄の決まりの写し 1 本と凍結の定数を同時に揃え、凍結の土台を直す巻き添えで動く便 99 の anchor の残差 2 行と歯の定数 1 つを独立の script で組み直す"
req = ["FR19", "FR14"]
section = "1"
write-set = ["crates/folio/src/note.rs", "design-intent/design-note/schema.yaml", "tests/fixtures/schema/note-region.txt", "tests/fixtures/floor_base/design-intent/design-note/schema.yaml", "crates/folio/tests/note.rs", "crates/folio/tests/schema.rs", "tests/fixtures/schema/node-digest-anchor.txt", "crates/folio/tests/graph.rs"]
verify = ["cargo nextest run -p folio --test note f103_", "cargo nextest run -p folio --test note --test schema", "cargo nextest run -p folio --test graph f99_the_independent_script_matches_the_anchor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f103_ の歯 4 本（索引の節が 4 つの指す欄を逐語どおり持ち写しの一覧の行が 1 本も残っていない／指す欄が 4 つ拾えてその指す先が索引の欄の決まりの生成区間に実在する／注 index_note が着地した口と判断の記録 ADR-14 と索引の欄の決まりの置き場と制約 CON9 を名指し 未実装 の字を持たない／事後の検査の一覧が 5 つで注 guards_note が prose-mentions と R-17 と歯の実装の置き場と設計ノートの dir を名指す）が全部緑、tests/note.rs と tests/schema.rs の既存の歯が全部緑（生成区間 137 行 16424 byte と凍結 anchor が byte 一致し、凍結の要約値が sha256sum の測り直しと一致し、残る注 3 つが 未実装である を持つ）、folio schema --check が 9 file とも一致、tests/graph.rs の f99_the_independent_script_matches_the_anchor が独立の script で組み直した anchor（残差の 2 行だけが変わり節点の 189 行は不変）で緑、便 89 の上限の歯が tests/schema.rs と tests/schema_docs.rs の両方で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
