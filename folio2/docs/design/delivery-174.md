# 設計: 便 174 — 生成区間を外の置き場へ書くとき、folio2 の番号（条・要件・規則の表の行・判断の記録）を名指さない（判断の記録 ADR-16 決定 (2)(オ)）

- 要件: FR19（欄の決まりの生成区間を床の定数から決定的に導く＝置き場の名で外の置き場の字を選ぶ）・FR22（骨格の生成区間も同じ導出＝行 R-8・R-16 のほかの番号を書かない）。中身は判断の記録 ADR-16 決定 (2)(オ)（外の置き場で床が id で名指すのは規則の表の行 R-8 と R-16 だけ）を、生成区間の字まで広げる。
- 条: P-6.2（生成区間は folio の出力＝利用者は直せない）/ P-7.1（利用者は規則の表を folio2 の番号へ改番できない）/ P-5.2（folio2 自身の正本は規則を id で名指したまま）/ P-10.1（凍結 anchor = 歯の中の手書きの字）/ N-3.1（例外の口を足さない＝名で選ぶのは既存の列の根の表と同じ導出の形）。
- 出所: 構造の直し（判断の記録 ADR-30）の検証で、外の置き場 tsuzuri の写しに回した天井の周（t3v-round1）の整合の所見 F-4（止める・反証 支持）と F-14（直す）。規則の表・憲法・人の書く正本は変えない＝持ち主の承認は要らない形。台帳の id は席が起こす。
- 置き場: 審査の材料は行 `fu` の §1 だけ。write-set 12 本（src 5・歯の file 7〔中身の変わらない 5 本は verify の --test の scope〕）。新しい dir・新しい file・消す file は無い。設計文書の正本（design-intent/）は書き換えない。
- 門: **対象外・0（通す）**（本流 1ca1ae1 の binary・write-set 12 本・`通す（設計文書の正本を書き換えない便）`）。
- base = 本流 1ca1ae1（数はその写しの実測・参考値・行 D-13）。
- 実装の見本: origin の枝 `impl/d174`（commit 3af6ade・親は 1ca1ae1）が本便の後の中身（差分 49,012 byte・7 file）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 3af6ade -- <write-set の file>`）。write-set の外は変えない。

## 1. 設計

実装の見本は origin の枝 `impl/d174`（commit 3af6ade・親は 1ca1ae1）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 3af6ade -- <write-set の file>`）。write-set の外は変えない。

### (a) いま起きていること（参考値・1ca1ae1）

1. **生成区間は置き場を問わず folio2 の字。** `folio schema --write`（と `folio init` の骨格）が書く 9 本の生成区間（判断の記録と設計ノートの欄の決まり・天井の正本・規則の表・入口・要件書・語彙・相談窓口・索引の欄の決まり）は、列の根の表（`Floor::Pick`）の 1 行を除いて、どの置き場にも同じ字を書く。その字が folio2 の番号を **152 か所**で名指す（型付きの欄の値 10・`_note` の注の字 142。file ごとに 判断の記録 54・設計ノート 67・天井 9・規則の表 5・入口 2・索引 15・ほか 3 本 0）。
2. **型付きの欄 10 のうち、値として読まれるのは行 R-8 だけ。** 判断の記録の `enums.surface: [R-8]` と設計ノートの `doc_meta.approval.surface_enum: [R-8]` は床が承認欄の対話面の値域として読み（`adr.rs`・`note.rs`・`link.rs` の行の実在）、置き場の行 R-8 を引く。ほかの 8 は床が字面で突き合わせるだけで、どの関数も値を読まない: 設計ノートの `prose_gate_rules_row: R-16`（散文の門は `prose.rs` の定数で行 R-16 を引く）・`figures.body_classes_rules_row: R-3`・`quality_rules_row: R-14`・`tool_version_rules_row: R-15`・`retry_rules_row: R-7`・`guards.p18_4_judged_by: R-13`・`figures.spec` の字「…ADR-4 決定 (1)」・規則の表の `reverse_reference` の字「（R-4 の双方向）」。外の置き場で行 R-7・R-3 の意味を変えても、床の判定と面の字は変わらない（面は規則の表の行そのものを描くだけ・写しで実測）。
3. **外の置き場では別の意味か、行が無い。** tsuzuri の写し（`folio schema --write` 済み）で 152 か所を引くと、字が同じ 20・字が違う 98（言い換えただけの条を含む）・行が無い 34。型付きの欄では R-7（tsuzuri では 席の停止の検知条件）と R-3（同じ種類の失敗の上限）が別の意味、R-13〜R-15 は行が無い。骨格（`folio init`）では 152 か所のうち解けるのは R-8・R-16・P-1 の 8 か所だけ（P-1 は別の意味）。
4. **天井が毎周同じ 止める を出す。** 整合の所見 F-4（場所 = 設計ノートの欄の決まりの `schema.figures.retry_rules_row`・止める・反証 支持）は「修正の往復の上限を憲法は R-3、欄の決まりは R-7 とし、R-3 が 2 つの意味を負う」、F-14（直す）は天井の正本・規則の表・入口・索引の注が名指す ADR-8・FR5・R-4・P-2.3・ADR-5・P-2.4 の食い違い。利用者は生成区間を手で直せず（P-6.2）、規則の表を改番もできない（P-7.1）＝直せない燃料で、門（ADR-30 決定 (6)）はその file を書く便を止め続ける。
5. **base の歯。** nextest 992 / 992・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file。`git grep -n f174_ -- crates` 0 件・行 id `fu` 0 件。

### (b) 直す先

1. **`floor.rs`（導出と突き合わせ）。** folio2 の置き場の名 `HOME`（`folio2-constitution`＝列の根の表の folio2 の行の鍵）と、外の置き場（名が在って `HOME` でない）の規則 7 を足す。
   - 外の置き場の字 `text_for`: 文字列の値から、folio2 の番号の印を持つ片を落とす。印 = 骨格と同じ id の形（`ids_in`・条・要件・規則の表の行・判断の記録・便）のうち予約の行 R-8・R-16 でないもの・判断の記録の決定の番号「決定 (」・台帳の id の形（頭か空白か全角の字の後の英小字 1 字・数字 1 字・「-」＝正規表現の字面は数えない）。落とし方は 3 段: ① 全角の丸括弧の中の片（中に「。」が在れば文、無ければ「・」の項）のうち印を持つ片を落とし、残らなければ括弧ごと落とす（入れ子は内から）。② なお印を持つ文（深さ 0 の「。」まで）を落とす。③ 注の欄（`_note` とその下）で何も残らなければ欄ごと書かない。
   - 新しい欄の型 Home（床の木の型の枝・folio2 の置き場にだけ在る欄）: 外の置き場では書かず、在れば未知の欄。
   - `derive_for` と `floor_diff_for` が同じ規則を通る（型付きの値も同じ `text_for` で比べる）。名の無い口（床の単体の歯と名なしの突き合わせ）と folio2 の置き場は定数の字のまま＝**folio2 の生成区間は 1 byte も変わらない**。
   - 骨格の命令の `ids_in` を `init.rs` から降ろす（床は層 1・骨格は層 3＝層を上がる辺を作らない）。
2. **`floor_note.rs`。** 値が folio2 の規則の表の行を名指す 5 欄（`figures` の body_classes / quality / tool_version / retry の `_rules_row` と `guards.p18_4_judged_by`）を新しい欄の型 Home で包む。folio2 の人の書く対応表（設計ノートの欄の決まりの `mapping`）が名指す欄は folio2 の置き場に残る。
3. **`schema.rs`。** 置き場の名を 1 度だけ読み、9 本とも名つきで導く（今は判断の記録の 1 本だけ）。名が読めなければ先頭の file の手前で「まだ分からない」（字と順は今と同じ）。
4. **`note.rs`。** 設計ノートの欄の決まりの写しの突き合わせを名つき（`floor_diff_for`・名は置き場の憲法の meta.id）にする（判断の記録の突き合わせと同じ）。
5. **`init.rs`。** `ids_in` を `floor.rs` から使う（本文は動かしただけ）。
6. **変えないもの。** folio2 の 9 本の生成区間と凍結 anchor（`tests/fixtures/schema/*-region.txt`）・歯の中の要約値の定数・床の読む値（行 R-8・R-16 は外の置き場でも名指す）・人の書く正本（design-intent）・規則の表・憲法・違反の名札（台帳 f2-648.236 の件）・凍結 anchor の file の頭の注（`freeze.rs`・`seal.rs` の「ADR-2」「ADR-30」）。

### (c) 歯（f174_・base で 0 件）

1. **f174_text_for_drops_folio2_numbers_and_keeps_the_rest（`floor.rs` の単体）。** 括弧の項（`廃止（superseded_by 必須・P-7.2・承認欄を持つ）` → `廃止（superseded_by 必須・承認欄を持つ）`）・括弧の外の文（`P-8.1。床は…` → `床は…`）・予約の行を残す（`（R-8）`）・括弧ごと（`（判断の記録 ADR-13 決定 (4)・P-6.3 / P-6.4）`）・決定の番号（`決定 (3)`）・台帳の id と便（`台帳 f2-648.132`・`便 119 で入った。`）・括弧の中の文（`（違えば → A-2。測らない。…）`）・正規表現の字面は変えない（`^[a-z][a-z0-9-]*$`）・全部落ちれば None。外の置き場の判定（名なしと HOME は外でない）。**base は関数が無い＝組み立てが落ちる（RED）。**
2. **f174_abroad_derivation_names_only_reserved_rows（`schema.rs` の単体）。** 9 本の床を外の名で導くと、行 R-8・R-16 のほかの id・「決定 (」・台帳の id が 0。型付きの欄で字が変わるのは閉じた 2 欄（設計ノートの `figures.spec`・規則の表の `reverse_reference`）だけで空にならない。導いた字を同じ名で突き合わせると差 0、folio2 の名で突き合わせると判断の記録（列の根の表）・設計ノート（5 欄と spec）・規則の表（双方向の字）の 3 本だけが違う。**base は組み立てが落ちる（RED）。**
3. **f174_a_tsuzuri_shaped_place_names_no_folio2_number（`tests/place_name.rs`・binary）。** 骨格（`folio init`）の名を `tsuzuri-constitution` に替え、規則の表に別の意味の行 R-7（席の停止の検知条件）を足した置き場（R-13 は無い）。骨格の命令が書いた 9 本の生成区間と、`folio schema --write` の後の 9 本の生成区間の両方で、歯の中の手書きの id の形の語（生成器の関数は呼ばない）のうち予約の外が 0・「決定 (」が 0・予約の 2 行は名指す。`--check` は 0。設計ノートの欄の決まりに folio2 の 5 欄の行が無い。`folio check` の出力に「床の定数と違う」が無い。**base は判断の記録の生成区間が P-1・R-7・FR1… を名指して落ちる（RED・`red174-place_name.log`）。**
4. **f174_folio2_keeps_its_numbers（`tests/place_name.rs`・binary）。** repo の正本の憲法と 9 本の写しで `--check` 0、設計ノートの生成区間は 5 欄の行（`retry_rules_row: R-7` ほか）と spec の「・ADR-4 決定 (1)」を持ったまま（base でも緑・folio2 が変わらないことの見張り）。

### (d) 採らなかった形

1. **一律に書き換える（どの置き場でも注から番号を外すか「folio2 の」と断る）。** folio2 自身の 70 行（生成区間の字の 3 写し＝src・design-intent・凍結 anchor）が変わり、差分は 174 の対象の 41 行だけで約 135,000 byte（写しで実測した src と design-intent の 84,571 byte + anchor）、判断の記録の 29 行を足すと 3 便。folio2 自身の注から規則の行の id を外すと P-5.2（規則を id で参照する）とぶつかり、folio2 の印と周も動く。
2. **外の置き場に注を 1 つも写さない。** 仕組みは最も小さいが、番号を持たない説明（amends の書き方・封・凍結 anchor の規則）まで外の置き場から消える。
3. **規則の表の行に意味の印を持たせ、欄の決まりが意味で行を引く。** 規則の表の欄の追加（行 D-17 の承認・人の書く行の書き換え）が要る。値を読む関数が無い 5 欄には見合わない。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけ（src 5 本・単体の歯 2 を含む）を 1ca1ae1 に当てると既存 2 本が落ちる**（994 本中・本文は sim-src-only-174-nextest.log）。扱いは次のとおり。
   - `tests/freeze_root.rs` の f121_renamed_constitution_fails_the_floor_twice と f121_freeze_start_writes_nothing_on_any_finding: 名を folio2 の外の名に替えた写しで、設計ノートの写しが外の置き場の形でないので「床の定数と違う」6 行が増える（違反 8 / 7・期待 2 / 1）。**名を替えた写しは設計ノートの写しも外の置き場の形に揃える**（手書きの道具 `abroad_note`・5 行を落とし spec の項を落とす・列の根の表を空にする既存の `empty_table` と同じ扱い）。期待の数は変えない。
   - 落ちない形にした 1 本: `tests/modules.rs` の p106_edges_point_down は、床（層 1）から骨格（層 3）の `ids_in` を呼ぶ形と、単体の歯 2 を `floor.rs` に置く形（9 本の床の module へ上がる辺）で落ちた（sim1 の写し）＝`ids_in` を床へ降ろし（(b) の 1・5）、歯 2 を `schema.rs` に置いた。
2. **便を当てた写し。** nextest **996 / 996**（992 + 4）・clippy 0 警告・床 4 本 rc 0・`folio schema --dir design-intent --write` は 9 本とも「変わらない」・`folio build --write` 35 file（base の binary の出力と 35 file とも byte で同じ）。
3. **RED。** 歯だけ（r174-teeth.patch = tests/place_name.rs）を 1ca1ae1 に当てると `--test place_name f174_` の 1 本（f174_a）が落ちる。単体の歯 2 本は base に関数が無く組み立てが落ちる。
4. **突然変異（便の後の写しの src を 1 通りずつ変え、verify の 1〜2 を撃つ・mut-174.log）。**

| 変異 | 落ちる f174_ |
| --- | --- |
| M1 外の置き場を folio2 の置き場と同じに扱う（`abroad` が常に偽） | text_for・abroad_derivation・tsuzuri_shaped |
| M2 括弧の項を落とさない（文だけ落とす） | text_for・abroad_derivation |
| M3 設計ノートの床を名なしで突き合わせる | tsuzuri_shaped |
| M4 folio2 にだけ在る欄を外の置き場にも書く | abroad_derivation・tsuzuri_shaped |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 縮む（`-`）は `init.rs` の 1 本。ほかの src 4 本と歯の file 2 本（place_name・freeze_root）は増える。歯の file 5 本（schema・schema_docs・note・init・modules）は verify の --test の scope で中身は変わらない。差分 49,012 byte（7 file・+571 −119・`git diff 1ca1ae1 <便の後>`）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）を 1500 から引く。python と awk の 2 実装が 10 回とも一致した（lines-174.log）。

| file | base | base の余地 | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/floor.rs` | 446 | 1054 | 727（+281） | 773 |
| `crates/folio/src/floor_note.rs` | 509 | 991 | 511（+2） | 989 |
| `crates/folio/src/init.rs` | 801 | 699 | 749（−52） | 751 |
| `crates/folio/src/note.rs` | 940 | 560 | 942（+2） | 558 |
| `crates/folio/src/schema.rs` | 182 | 1318 | 240（+58） | 1260 |

3. **size は M。** src +291 行（床の導出と突き合わせの規則 7・`Home`・単体の歯 2）・binary の歯 2・既存の歯 2 本の土台の揃え。余地の最小は base の note.rs の 560（≥ 300）。
4. **verify 11 行 = done の 11 の塊。** 1 `--bin folio f174_`（(c) の 1・2）・2 `--test place_name f174_`（(c) の 3・4）・3 `--bin folio`（150 本）・4 `--test place_name`（6 本）・5 `--test freeze_root`（14 本）・6 `--test schema`（21 本・folio2 の判断の記録と設計ノートの生成区間が凍結 anchor と byte 一致）・7 `--test schema_docs`（29 本・ほかの 7 本の生成区間の anchor）・8 `--test note`（39 本）・9 `--test init`（21 本）・10 `--test modules`（2 本・層を上がる辺 0）・11 clippy 0 警告。便の後の写しで 11 行とも rc 0（verify-174.log）。`--test` の 7 本はどれも write-set に在る。

### (g) 門・受付・並行の便・外の置き場

1. **門。** 設計文書の正本を書き換えない便＝1ca1ae1 の binary で 0（通す・gate-174.log）。
2. **受付。** 並行の便と write-set が重なれば `write-set-overlap` の断り（本便の src 5 本は便 172・173 が触った file を含まない）。受付の本流が 1ca1ae1 と違えば (i) の 3。
3. **外の置き場（tsuzuri）の手順。** 本便の binary で、今の生成区間のままの置き場は設計ノートの写しが「床の定数と違う」6 件（5 欄の未知の欄と spec）と `--check` の非 0 になる（P0）。`folio schema --dir design-intent --write` → commit だけで床は合格 0 / 0・`--check` 9 本一致・9 本の生成区間が名指す番号は R-8 の 4 か所と R-16 の 2 か所だけ（P1・6 file・+63 −70 行・tsuzuri-174.log）。次の天井の周の束の骨組み（生成区間）から F-4・F-14 の燃料が消える。
4. **init の骨格。** 骨格の命令は同じ導出を呼ぶので、名が未記入の骨格も名を替えた後も、生成区間は R-8・R-16 だけを名指す（床の結果は base と同じ まだ分からない 2）。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/genids-draft.md`、差分と script は同じ dir の `genids-scripts`。1 census.py・resolve.py（152 か所の表と意味の引き当て）。2 c174.patch を 1ca1ae1 に当てて `cargo build --all-targets -p folio` → verify-174.sh・nextest の全部・床 4 本。3 r174-teeth.patch で RED。4 mut-174.py。5 lines-174.sh。6 effect-tz.sh（型付きの欄が読まれないことの実測）・tsuzuri-174.sh（P0 / P1）・gate は 1ca1ae1 の binary で write-set 12 本。7 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-174.md#fu`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** FR19 の「床の定数から決定的に導出」は、列の根の表と同じく置き場の名を入力に持つ形で保つ（同じ名なら同じ字）。FR22 の骨格は行 R-8・R-16 を足す形のまま、生成区間もその 2 行だけを名指す。
2. **言えないこと。** 外の置き場の注は、folio2 の番号を括弧の外に持つ文を丸ごと落とす（例: 設計ノートの type_note の 2 文・landing_note の頭の文・判断の記録の grill_note は欄ごと）＝説明が減る。番号でない folio2 の字（「folio2 側」「器 scribe2」「orchestrator 席」・folio2 の file 名）は残る。違反の名札（[N-4]・[P-8]・[A-2] ほか）と注入の行 R-2 は台帳 f2-648.236 の件で本便の外。凍結 anchor の file の頭の注（「凍結 anchor（ADR-2）」・封の一覧の「ADR-30 決定 (3)」）も外の置き場へ folio2 の番号を書くが、生成区間でなく、書いた anchor は書き換えない（別の便）。
3. **撤退条件。** (1) (e) の 1 の 3 本のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 着地の後の本流で `folio schema --dir design-intent --check` が一致しないか、`tests/fixtures/schema/` の凍結 anchor が変わる差分になったら止める（folio2 の生成区間は変わらない前提）。(3) 受付の本流の床の定数に、印を持つ型付きの値が (c) の 2 の閉じた 2 欄のほかに増えていたら止める。(4) 差分が 120,000 byte を超えたら止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜5・歯（f174_ の 4 本と (e) の 1 の土台の揃え）。
- 入れない: folio2 の生成区間と凍結 anchor・人の書く正本・規則の表・憲法・違反の名札（f2-648.236）・凍結 anchor の頭の注・外の置き場への書き込み・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| derive | 外の置き場の字 | floor.rs の規則 7（text_for・Home）と schema.rs の名つきの導出 |
| diff | 突き合わせ | floor_diff_for の同じ規則と note.rs の名つきの突き合わせ |
| teeth | 歯 | f174_ の 4 本・freeze_root の土台の揃え |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。前提の便は無い（本流 1ca1ae1 の上）。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の素の床が合格・tsuzuri へ (g) の 3 を渡す（`folio schema --write` → commit）・台帳 f2-648.236（名札と注入の行 R-2）との順。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fu"
title = "生成区間を外の置き場へ書くとき folio2 の番号を名指さない（判断の記録 ADR-16 決定 (2)(オ)・tsuzuri の天井の整合の所見 F-4 止める と F-14 直す）: floor.rs に folio2 の置き場の名 HOME と外の置き場の規則 7 を足し、外の置き場では文字列の値から folio2 の番号の印（予約の行 R-8・R-16 のほかの id・決定の番号・台帳の id・便）を持つ括弧の片と文を落とし（注で何も残らなければ欄ごと書かない）、新しい欄の型 Home（folio2 の置き場にだけ在る欄）を書かない。derive_for と floor_diff_for が同じ規則を通る。floor_note.rs の値が folio2 の規則の表の行を名指す 5 欄を Home で包み、schema.rs は 9 本とも置き場の名で導き、note.rs は設計ノートの写しを名つきで突き合わせ、骨格の ids_in を init.rs から床へ降ろす。folio2 自身の 9 本の生成区間と凍結 anchor は 1 byte も変えない。実装の見本は origin の枝 impl/d174 の commit 3af6ade（親 1ca1ae1）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 1ca1ae1"
req = ["FR19", "FR22"]
section = "1"
write-set = ["crates/folio/src/floor.rs", "crates/folio/src/floor_note.rs", "-crates/folio/src/init.rs", "crates/folio/src/note.rs", "crates/folio/src/schema.rs", "crates/folio/tests/place_name.rs", "crates/folio/tests/freeze_root.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/note.rs", "crates/folio/tests/init.rs", "crates/folio/tests/modules.rs"]
verify = ["cargo nextest run -p folio --bin folio f174_", "cargo nextest run -p folio --test place_name f174_", "cargo nextest run -p folio --bin folio", "cargo nextest run -p folio --test place_name", "cargo nextest run -p folio --test freeze_root", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test init", "cargo nextest run -p folio --test modules", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "binary の単体の f174_ の 2 本（外の置き場の字の落とし方と、9 本の床の外の置き場の導出が予約の行のほかの番号を持たず型付きの欄で字が変わるのは閉じた 2 欄だけで同じ名の突き合わせが差 0）が緑、tests/place_name.rs の f174_ の 2 本（tsuzuri の名の骨格に別の意味の R-7 を足した置き場の 9 本の生成区間が予約の 2 行のほかの番号を名指さず check 0 で床が設計ノートの写しを落とさない・folio2 自身の置き場は番号を持ったまま check 0）が緑、binary の単体の歯の全部と tests/place_name.rs・freeze_root.rs・schema.rs・schema_docs.rs・note.rs・init.rs・modules.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格・folio schema --dir design-intent --check が一致（9 本とも字は変わらない）・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
