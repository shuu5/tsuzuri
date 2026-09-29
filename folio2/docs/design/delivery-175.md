# 設計: 便 175 — 周の引き金の仕掛けを外し、生成区間から周の引き金と門の偽の字を落とす（判断の記録 ADR-30 決定 (5)(6)）

- 要件: FR20（印の中身と門の式）・FR19（欄の決まりの生成区間は床の定数から決定的に導く）・FR22（骨格の生成区間も同じ導出）。中身は ADR-30 決定 (5)「印の古さ・引き金の欄の変更は周を起こさない」と決定 (6)「門は印の古さで止めない」の後に、実装と生成区間に死んだまま残った周の引き金の仕掛けを外すこと。便 169 の契約 §1 (g) の 3 で予告した後続の便の前半（後半 = 便 177・行 fx = 印の節点の表）。
- 条: P-6.2（生成区間は folio の出力＝利用者は直せない）/ P-6.3（同じ内容の 2 面は一方を導出）/ P-10.1（凍結 anchor は生成側から独立＝前の anchor を行と字で直した写しを sha256sum で測る）/ P-4.1（偽の字を「異常なし」として配らない）。
- 出所: 構造の直し（ADR-30・持ち主の承認 2026-09-27 12:50 JST）の後の起草役の全数の読み（記録 `~/.local/share/folio2/handoff-2026-09-27/d175-draft.md`）。要件書・規則の表・憲法・人の書く正本は変えない＝持ち主の承認は要らない形。台帳の id は席が起こす。
- 置き場: 審査の材料は行 `fv` の §1 だけ。write-set 35 本（src 5・歯の file 5・生成区間 2・fixture と凍結 anchor 23）。新しい dir・新しい file・消す file は無い。
- 門: **0（通す）**（本流 efe3e7b の binary・`通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）`）。
- base = 本流 efe3e7b（数はその写しの実測・参考値・行 D-13）。
- 実装の見本: origin の枝 `impl/d175`（commit 501c39a・efe3e7b の上の 2 commit〔d047870 と歯の注の行数の直し〕）が本便の後の中身（差分 111,806 byte・35 file・+92 −985）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 501c39a -- <write-set の file>`）。write-set の外は変えない。

## 1. 設計

実装の見本は origin の枝 `impl/d175`（commit 501c39a・base は efe3e7b）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 501c39a -- <write-set の file>`）。write-set の外は変えない。

### (a) いま起きていること（参考値・efe3e7b）

1. **門はもう引き金を測らないのに、生成区間はそう言う。** 天井の正本 `design-intent/ceiling.yaml` の生成区間（`folio schema --write` が床の定数から書く・手で直さない）が、周の引き金の一覧 `schema.trigger`（見出しと 15 行の計 16 行）と注 `schema.trigger_note`「周の引き金の閉じた一覧（規範の欄）…この写しを決まった順に並べた要約値が引き金の要約値で、印と門が同じ関数で測る。何にするかの裁定の正本は判断の記録 ADR-18 決定 (1)（ADR-20・ADR-26 が改訂）」を持つ。便 169 の後の門（`gate.rs` の run）は印の round・verdict・viewpoints・refutes だけを読み、引き金の要約値を測らない（ADR-30 決定 (6)）。周も引き金では起きない（決定 (5)）＝一覧も注も今の振る舞いと違う主張。一覧の adr の欄 `revises` は、便 172・173 で欄の決まりから削った欄（決定 (2)）。
2. **索引の欄の決まりの注も同じ。** `design-intent/graph.yaml` の生成区間の注 `schema.edge_fields_note` が「周の引き金になるかは天井の正本の引き金の一覧が決め、この欄のうち受入基準の verifies・要件の verify.ac・規則の表の行の article は規範の欄として引き金に入る（判断の記録 ADR-18 決定 (1)・ADR-20）。ほかの辺の欄に id を足すだけの変更は周の引き金にならない（ADR-13 決定 (8)）。」の 2 文を持つ。
3. **外の置き場でも同じ字。** tsuzuri の写し（`folio schema --write` 済み・便 174 の後の binary）では、番号の片を落とした同じ 3 つの主張が出る（天井の正本 2 行・索引の欄の決まり 1 行）。骨格（`folio init`）も同じ導出。利用者は直せない（P-6.2）＝天井の燃料（便 174 で直した整合 F-4 と同じ族）。
4. **仕掛けは死んだまま残る。** `ceiling.rs` の TRIGGER_* 15 定数と型 TriggerRows（床の木の trigger の葉）・`gate.rs` の `trigger_digest` と下請け（sections・pick・adr_records）・印の欄 `trigger`（`stamp.rs` が書く）。読み手の実測（grep）: 印の trigger を読む関数は 0（門は便 169 から読まない・面の名札 `stamp::marks` は sources と観点の行だけ）。TRIGGER_* の読み手は `trigger_digest` と床の木と単体の歯 2 本だけ。
5. **ほかの 7 本の生成区間。** 判断の記録と設計ノートの欄の決まり（計 272 行）は別の読み役（sonnet）が 1 文ずつ実装と照らして偽 0（記録の census-adr-note.md）。規則の表・入口・要件書・語彙・相談窓口は ADR-30 に触れる字を持たない（全文を読んだ）。
6. **base の歯。** nextest 1000 / 1000・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file（要約 6a4c51d7c0fb66c7）。`git grep -n f175_ -- crates` 0 件・行 id `fv` 0 件。

### (b) 直す先

1. **`ceiling.rs`。** TRIGGER_* と TriggerRows と、床の木 FLOOR の trigger・trigger_note の 2 欄を外す（生成区間は 17 行減って 27 行・3,176 byte）。使わなくなる `floor_adr` の 2 定数の use を外す。単体の歯 f126_・f129_ の 2 本（引き金の一覧の実在）を外し、注の数の歯を 9 → 8 に。
2. **`gate.rs`。** `trigger_digest` と sections・pick・adr_records と TRIGGER_* の use を外し、頭の注を今の式に（門の式は 1 字も変えない）。
3. **`stamp.rs`。** 印は trigger を書かない（欄の並び = round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes）。rest と nodes は便 177。
4. **`graph.rs`。** 床の木の注 edge_fields_note から周の引き金の 2 文を落とす（(a) の 2・graph.yaml の生成区間は 3,692 byte に）。digest_note の印の 1 文と `stamp_table` は便 177。
5. **`floor_adr.rs`。** 注の「天井の周の引き金の憲法の一覧も同じ配列を指す」を落とす（定数は変えない）。
6. **生成区間。** `design-intent/ceiling.yaml`・`design-intent/graph.yaml` を `folio schema --dir design-intent --write` の出力に（ほかの 7 本は「変わらない」）。
7. **fixture と凍結 anchor（席の裁定 2026-09-27: 案 A・tests/fixtures の歯の anchor は前例どおり測り直す・design-intent/anchors/ の下は動かさない）。** 天井の正本の写しの fixture 20 本（`tests/fixtures/*/…/ceiling.yaml`）から trigger の 16 行と trigger_note の 1 行（計 17 行の塊）を落とす。凍結 anchor 3 本は、folio の code を呼ばない独立の実装（python）で組み、folio の出力と byte で一致させる（歯 `tests/schema_docs.rs`・`tests/graph.rs` が突き合わせる）。
   - ① `tests/fixtures/schema/ceiling-region.txt`: 手順 = 前の anchor の行の列から「`  trigger:` の行〜`  trigger_note: ` で始まる行」の塊を落とす（apply-tests-175.py の 1）。変わる範囲 = 行 28〜44 の 17 行（schema.trigger と schema.trigger_note）だけ・4,619 → 3,176 byte・sha256 1cc1401c…。**便 102 の後・便 126 の前の anchor（commit 37ee092）と byte で同じ**。定数 CEILING_REGION_*（44 → 27 行）。
   - ② `tests/fixtures/schema/graph-region.txt`: 手順 = 前の anchor の edge_fields_note の行から (a) の 2 の 2 文（376 byte）を字の一致で 1 か所だけ落とす。変わる範囲 = 行 34 の 1 行の中のその字だけ・4,068 → 3,692 byte。定数 F95_GRAPH_*（行数 35 のまま）。
   - ③ `tests/fixtures/schema/node-digest-anchor.txt`: 手順 = 独立の script `tests/fixtures/schema/node-digest.py` を、塊を落とした後の土台 `tests/fixtures/floor_base/design-intent` に当てた出力（anchor-175.sh）。変わる範囲 = 末尾の 2 行（行 191 の残差の要約値・行 192 の byte の内訳）だけ・節点 189 と本文と辺の欄の byte は同じ・残差と合計がともに 1,443 byte 減＝土台の天井の正本が減った byte と同じ・192 行 3,007 byte のまま。定数 F99_ANCHOR_SHA256。
   - **外す字の分だけ動くことの確かめ（手順）**: `anchor-diff-175-177.py`（git の中身を python で比べる・folio を呼ばない）が、① は塊のほかの行が同じ、② は 1 行の中の 2 文のほかが同じ、③ は末尾 2 行のほかが同じで内訳の差が土台の減りと同じ、fixture 20 本はどれも同じ 17 行の塊だけ、を assert で確かめて通った（anchor-diff.log）。
8. **変えないもの。** 門の式と理由の字・印のほかの欄・面の名札・`folio check` の判定・人の書く正本（要件書・天井の正本の人の書く節・語彙・規則の表・憲法）・本流の印 `design-intent/preview/ceiling-stamp.yaml`（周の生成物・次の --stamp まで trigger を持ったまま＝門も名札も読まない）・門の印の fixture stamp-pass / fail / unknown.yaml（trigger の行を仮の値のまま持つ＝前の形の印も門が同じ答えで読む見張り）。

### (c) 歯（f175_・base で 0 件）

1. **f175_the_ceiling_floor_has_no_round_trigger（`ceiling.rs` の単体）。** 床の木の最上位の欄が閉じた 16 欄（top_level〜refute_note）と並びまで等しい。**base は trigger・trigger_note が在って落ちる（RED）。**
2. **f175_no_region_claims_the_round_trigger（`tests/place_name.rs`・binary）。** folio2 自身の置き場・骨格の命令が書いた置き場・tsuzuri の名で `schema --write` した置き場の 9 本の生成区間に、手書きの字「引き金」「trigger」「印と門が同じ関数で測る」「revises」が無い（生成器の関数は呼ばない・P-10.1）。**base は 3 つとも天井の正本と索引の欄の決まりで落ちる（RED）。**
3. **f175_the_stamp_has_no_trigger（`tests/stamp.rs`・binary）。** `--stamp` の書いた印の最上位の欄に trigger が無く、sources の直後が faces。**base は落ちる（RED）。**
4. RED の本文は `red175.log`（歯だけ = r175-teeth.patch と単体の歯を efe3e7b に当てた写し・3 本とも落ちる）。

### (d) 採らなかった形

1. **1 便で印の節点の表（rest・nodes・stamp_table）まで外す。** 差分 124,156 byte が本便の上限 120,000 byte を超えた（見本 local/d175-whole で実測）＝便 177 へ割った。
2. **生成区間の注の字だけ直し、仕掛けは残す。** 注を「印は書くが門は読まない」に直しても凍結 anchor は同じ本数動き、読み手 0 の仕掛けと偽の一覧（revises）が残る。
3. **凍結 anchor を変えずに歯の側で行を読み飛ばす。** anchor が今の生成区間と違う字を持ち続け、anchor の意味（生成区間の byte 一致）が崩れる（P-10.1）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **外す歯 4 本**（引き金を縛る）: `ceiling.rs` の f126_trigger_lists_name_real_documents_and_sections・f129_trigger_constitution_scope_is_the_anchor_scope_minimum、`tests/schema_docs.rs` の f129_the_ceiling_region_lists_the_constitution_scope、`tests/stamp.rs` の f126_the_stamp_carries_the_trigger_after_sources。**字を合わせる歯**: `tests/schema_docs.rs` の f129_the_graph_region_notes_follow_adr20_and_adr14（落とした 2 文を期待から外す）・`tests/stamp.rs` の f99_the_stamp_carries_the_node_table（期待の欄の並びから trigger を外す）・without_digests（trigger の行を読み飛ばさない）・`tests/gate.rs`（引き金の要約値の独立の実装 trigger_hex と正規化の道具 J を外す・印の trigger の行は仮の値のまま）・`ceiling.rs` の注の数（9 → 8）。凍結 anchor の定数 3 組は (b) の 7。
2. **便を当てた写し。** nextest **999 / 999**（1000 − 4 + 3）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file（要約 6a4c51d7c0fb66c7 = base と同じ＝面は変わらない）。
3. **突然変異（便の後の写しの src を 1 通りずつ変え、f175_ を撃つ・mut.log）。**

| 変異 | 落ちる f175_ |
| --- | --- |
| M1 天井の床の木に trigger_note を戻す | ceiling_floor・no_region |
| M2 印が trigger の行を書く | stamp_has_no_trigger |
| M3 索引の注に周の引き金の 1 文を残す | no_region |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 縮む（`-`）は src 5 本・歯の file 3 本（gate・schema_docs・stamp）・生成区間 2 本・fixture 20 本・anchor 2 本（ceiling-region・graph-region）。増えるのは歯の file 2 本（place_name・graph）と anchor 1 本（node-digest-anchor・byte 数は同じで字が変わる）。差分 111,806 byte（35 file・+92 −985・`git diff efe3e7b 501c39a`）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）を 1500 から引く。python と awk が一致（lines.log）。

| file | base | base の余地 | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/ceiling.rs` | 736 | 764 | 576 | 924 |
| `crates/folio/src/floor_adr.rs` | 494 | 1006 | 493 | 1007 |
| `crates/folio/src/gate.rs` | 599 | 901 | 466 | 1034 |
| `crates/folio/src/graph.rs` | 837 | 663 | 836 | 664 |
| `crates/folio/src/stamp.rs` | 513 | 987 | 511 | 989 |

3. **size は M。** src は外すだけ（−297 正規化行）だが、歯の file 5 本と fixture 20 本と凍結 anchor 3 本の測り直しを伴う。余地の最小は base の graph.rs の 663（≥ 300）。
4. **verify 10 行 = done の 10 の塊。** 1 `--bin folio f175_`（(c) の 1）・2 `--test place_name f175_`（(c) の 2）・3 `--test stamp f175_`（(c) の 3）・4 `--bin folio`（151 本）・5 `--test gate`（15 本・門の答えは便 169 / 176 のまま）・6 `--test stamp`（18 本）・7 `--test schema_docs`（28 本・生成区間と凍結 anchor の byte 一致）・8 `--test graph`（16 本・残差の anchor）・9 `--test place_name`（7 本）・10 clippy 0 警告。便の後の写しで 10 行とも rc 0（verify175.log）。`--test` の 5 本はどれも write-set に在る。

### (g) 門・受付・並行の便・外の置き場

1. **門。** write-set の設計文書の正本は ceiling.yaml と graph.yaml の 2 本（生成区間だけ）。efe3e7b の binary で 0（通す・gate.log・印の 51 周目に支持の 止める は無い）。
2. **受付。** 便 177（行 fx）は本便の後の本流の上で受け付ける（graph.rs・stamp.rs・tests/stamp.rs・tests/place_name.rs・tests/schema_docs.rs・graph.yaml・graph-region.txt が重なる）。受付の本流が efe3e7b と違えば (i) の 3。
3. **外の置き場（tsuzuri）。** 本便の binary で `folio schema --dir design-intent --write` → commit だけで、天井の正本と索引の欄の決まりの生成区間から周の引き金の字が消える（t3v の写し tz2 の写しで実測・「引き金」「trigger」の行 4 → 0・`--check` 0・tsuzuri.log）。
4. **本流の印。** 次の `--stamp`（席の周）で trigger の無い印になる。それまでの印の trigger は読み手が無い。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d175-draft.md`、差分と script は同じ dir の `d175-scripts`。1 apply-175.sh（apply-src-175.py → apply-tests-175.py → anchor-175.sh → schema --write）を efe3e7b の写しに当てると 501c39a の中身。1b anchor-diff-175-177.py（凍結 anchor が外す字の分だけ動くこと）。2 run-175.sh（組み立て・nextest の全部・clippy・床 4 本・build）。3 red-175-177.sh（RED）。4 mut-175-177.py（M1〜M3）。5 lines-175.sh。6 verify-175-177.sh <写し> 175。7 門は efe3e7b の binary。8 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-175.md#fv`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** FR20 の規範文・平易文・受入基準 AC18 は引き金も印の trigger も名指さない＝字の直しは要らない。FR20 の注の「印は引き金の要約値と節点の表〔trigger・rest・nodes〕を後続の便まで書き続けるが、門は読まず、後続の便で消す」は来歴の字として残る（一括の候補）。語彙の round-trigger の項（人の書く正本）は「印が古くなる変化」の定義を持つ＝本便では変えず一括の候補に挙げる。
2. **言えないこと。** 生成区間の外の字（設計ノート docs/design/ceiling-gate.md・判断の記録の本文〔凍結〕・語彙）は本便の外。印の節点の表は便 177。
3. **撤退条件。** (1) (e) の 1 のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 着地の後の本流で `folio schema --dir design-intent --check` が一致しないか、凍結 anchor 3 本のほかの anchor が動く差分になったら止める。(3) 差分が 120,000 byte を超えたら止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜7・歯（f175_ の 3 本と (e) の 1 の外しと字合わせ）。
- 入れない: 印の rest・nodes と stamp_table（便 177）・人の書く正本・規則の表・憲法・本流の印・門の印の fixture・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| floor | 床の木 | ceiling.rs の TRIGGER_* と trigger・trigger_note・graph.rs の注 |
| stamp | 印と門 | gate.rs の trigger_digest・stamp.rs の trigger |
| anchor | 凍結 anchor | ceiling-region・graph-region・node-digest-anchor と fixture 20 本 |
| teeth | 歯 | f175_ の 3 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。前提の便は無い（本流 efe3e7b の上）。後続 = 便 177（行 fx）。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の素の床が合格・tsuzuri へ (g) の 3 を渡す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fv"
title = "周の引き金の仕掛けを外し、生成区間から周の引き金と門の偽の字を落とす（判断の記録 ADR-30 決定 (5)(6)・便 169 の後続の前半）: ceiling.rs の TRIGGER_* と TriggerRows と床の木の trigger・trigger_note を外し（天井の正本の生成区間が 17 行減る）、gate.rs の trigger_digest と下請けを外し、stamp.rs の印は trigger を書かず、graph.rs の床の木の注 edge_fields_note から周の引き金の 2 文を落とし、floor_adr.rs の注を合わせる。design-intent/ceiling.yaml と graph.yaml の生成区間を folio schema --write の出力にし、天井の正本の写しの fixture 20 本から同じ 17 行を落とし、凍結 anchor 3 本（ceiling-region.txt・graph-region.txt・node-digest-anchor.txt）を前の anchor を直した写しと独立の script で測り直す。門の式・印のほかの欄・人の書く正本は変えない。実装の見本は origin の枝 impl/d175 の commit 501c39a（base efe3e7b）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 efe3e7b"
req = ["FR19", "FR20", "FR22"]
section = "1"
write-set = ["-crates/folio/src/ceiling.rs", "-crates/folio/src/floor_adr.rs", "-crates/folio/src/gate.rs", "-crates/folio/src/graph.rs", "-crates/folio/src/stamp.rs", "-crates/folio/tests/gate.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/place_name.rs", "-crates/folio/tests/schema_docs.rs", "-crates/folio/tests/stamp.rs", "-design-intent/ceiling.yaml", "-design-intent/graph.yaml", "-tests/fixtures/adr/effective-no-approval/ceiling.yaml", "-tests/fixtures/adr/schema-drift/ceiling.yaml", "-tests/fixtures/adr/two-adopted/ceiling.yaml", "-tests/fixtures/anchor/no-anchor/ceiling.yaml", "-tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "-tests/fixtures/ceiling/bundle/source/ceiling.yaml", "-tests/fixtures/check/dup-key/ceiling.yaml", "-tests/fixtures/check/empty-field/ceiling.yaml", "-tests/fixtures/check/missing-file/ceiling.yaml", "-tests/fixtures/check/unknown-section/ceiling.yaml", "-tests/fixtures/face/ceiling.yaml", "-tests/fixtures/floor_base/design-intent/ceiling.yaml", "-tests/fixtures/link/adr-id-missing/ceiling.yaml", "-tests/fixtures/link/amended-by-orphan/ceiling.yaml", "-tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "-tests/fixtures/refs/bad-counts/ceiling.yaml", "-tests/fixtures/refs/dangling-id/ceiling.yaml", "-tests/fixtures/refs/orphan-rule/ceiling.yaml", "-tests/fixtures/schema/ceiling-region.txt", "-tests/fixtures/schema/graph-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "-tests/fixtures/vocab/exemptions/ceiling.yaml", "-tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio --bin folio f175_", "cargo nextest run -p folio --test place_name f175_", "cargo nextest run -p folio --test stamp f175_", "cargo nextest run -p folio --bin folio", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test place_name", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "binary の単体の f175_ の 1 本（天井の床の木の最上位の欄が閉じた 16 欄で trigger と trigger_note を持たない）が緑、tests/place_name.rs の f175_ の 1 本（folio2・骨格・tsuzuri の名の置き場の 9 本の生成区間に 引き金・trigger・印と門が同じ関数で測る・revises の字が無い）が緑、tests/stamp.rs の f175_ の 1 本（印の最上位の欄に trigger が無く sources の直後が faces）が緑、binary の単体の歯の全部と tests/gate.rs（門の答えは便 169 と便 176 のまま）・stamp.rs・schema_docs.rs（生成区間と凍結 anchor の byte 一致）・graph.rs（残差の凍結 anchor）・place_name.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
