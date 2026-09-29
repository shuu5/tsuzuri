# 設計: 便 208 — 索引の --summary の 1 行の末尾に状態の欄 status を置く（判断の記録 ADR-35 決定 (2)）

- 要件: FR31（索引の点ごとの 1 行に、文書の状態の欄を足す・要件書 第 1.57 版・2026-09-29 発効）と FR14（索引の --summary）。規範文・確かめ方・受入基準 AC34 は変えない。
- 条: P-6.3（状態は正本の字を写すだけで、訳さず、語の置き場を 2 つにしない）・P-4.2（状態の欄が無いか字でなければ null と出し、黙って埋めない）・P-10.1（期待の字と便 208 の前の出力の要約値は歯の側の手書き）。
- 出所: 台帳 f2-648.275.9（外の利用者 tsuzuri の増殖の案 v0.2 の B-4）。決定の正本は本流 17736c9 の `design-intent/adr/ADR-35.yaml` 決定 (2)(7) と `design-intent/srs.yaml` の FR31・AC34（持ち主の承認 2026-09-29 13:19 JST・承認の PR #391）。起草の記録は `~/.local/share/folio2/handoff-2026-09-28/d208-draft.md`。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `hc` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 3 本（src 1・歯の file 2）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 3 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 0914e69（便 207〔行 hb〕の着地）**（席の裁定・着地の順は 207 → 208）。0914e69 は便 207 の見本 9585bb7 と crates・design-intent・tests で byte で同じ（git diff 0）。この契約の数はすべて便 207 の見本 `impl/d207` 9585bb7（親 本流 17736c9）の写しの実測（参考値・規則の表の行 D-13・base の workspace の nextest は独立の検証役が 9585bb7 で測った値・起草役は前の見本 72d11f1 でも同じ数）。便 207 の着地の木が見本 9585bb7 と、本便の write-set に重なる file（`crates/folio/tests/graph_notes.rs`）で同じなら数え直しは要らない（§1 (g) 4）。
- 実装の見本: origin の枝 `impl/d208`（commit **375cbfa**・親 = 便 207 の見本 9585bb7）。`git diff 9585bb7 375cbfa` が便の全体の差分（3 file・+190 −21・28,161 byte）。**作業者は、受付の時点の本流（便 207 の着地の木）にこの差分を当てる**（write-set の file を見本の file の中身へ置き換えない＝便 207 が同じ `tests/graph_notes.rs` に書いた字を消さない）。write-set の外は変えない。
- 並行の便との重なり: §1 (g) の表（便 203・204 は本流 a5b27df に着地済みで 0・便 207 は `tests/graph_notes.rs` が重なり、本便はその着地の後）。

## 1. 設計

### (a) いま起きていること（便 207 の見本 9585bb7 の木の実測・参考値）

1. **--summary の 1 行は 7 欄。** `crates/folio/src/graph.rs` の `Index::jsonl` は節点ごとに id・kind・file・line・title・plain・eng の 7 欄を空白なしで出す（便 180）。状態の欄は無く、外の利用者の地図は廃止した設計ノートや判断の記録の点を畳むのに、正本の YAML を自分で読むしかない（読み手が 2 つ・ADR-35 前提 (4)）。
2. **索引はもう状態の在る木を読んでいる。** 判断の記録は `adr()` が記録の file 1 本を丸ごと読み（欄 status を持つ）、設計ノートは `notes()` が `note::load_notes` の木から meta の id を取る（meta.status を持つ）。状態の字を出すのに、正本を読む口を足す要は無い。
3. **状態の字の分布（--summary の行の数と状態の欄の字・見本の binary で数えた・起草の記録の logs）。**

| 置き場 | 行 | 判断の記録 | 設計ノートの行 | ほかの節点 |
| --- | ---: | --- | --- | ---: |
| 床の土台 `tests/fixtures/floor_base/design-intent`（便 207 の後・行 R-23〜R-25 が在る） | 194 | accepted 10 | example 1 | 183 |
| folio2 の `design-intent`（本流 17736c9） | 262 | accepted 33 | example 1 | 228 |
| tsuzuri の写し（本物の main 8c656f4 の clone・hook なし） | 452 | accepted 17・proposed 1 | 行を持つノート 59 本は全部 effective（207 行）・行を持たない draft のノートが 1 本在る（節点は無い） | 227 |
| 骨格 `folio init`（外の置き場の最小の形） | 7 | proposed 1 | （ノート無し） | 6 |

4. **--print と --digest。** --print は節点の表（id・種類・file・要約値・題）と辺の表と要約の 1 行、--digest は種類・型・file の 3 表と要約の 2 行で、どちらも --summary の欄を持たない（凍結 anchor `tests/fixtures/schema/graph-anchor.txt`・`graph-digest-anchor.txt` が床の土台の出力を持つ）。
5. **base の歯（参考値）。** workspace の nextest 1169 / 1169（便 207 の見本 9585bb7 の木・独立の検証役の測り）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 40 file。`git grep -n f208_ -- crates` は 0 件・行 id `hc` は 0 件（本流 17736c9 と 9585bb7 とも）。

### (b) 直す先

1. **`crates/folio/src/graph.rs` の `Index`。** 表 `states`（節点の id → 所属 file の文書の状態の字）を足し、覚える口 `state(id, 欄)` を置く。欄の値が字（`Node::as_str` が Some）のときだけ覚え、欠け・空の値・一覧・表は覚えない（= null）。
2. **`adr()`。** 記録の節点を組んだ後に `state(id, 記録の status)`。
3. **`notes()`。** ノートごとに meta の status を 1 度取り、そのノートの契約表の行の節点ごとに `state(行の id, その status)`。行の欄の status は読まない。
4. **`Index::jsonl`。** eng の後に `,"status":` と覚えた字（JSON の字の書き方は plain・eng と同じ `json_str`）か null を置く。欄の順は id・kind・file・line・title・plain・eng・status。
5. **ほかの節点は null。** 条・規範文・規則行・目的・要件・非機能要件・受入基準・制約・登場人物・出力は覚えない。規則の表の行の status（仮・凍結・未定）と要件書・憲法の見出しの欄の status は写さない（ADR-35 決定 (2)(ウ)）。
6. **変えないもの。** ほかの 7 欄の字と順（status を外した行は base と byte で同じ）・--print・--digest・`folio hello`・節点の種類（12）と辺の型（19）・索引の欄の決まりの生成区間（`graph.yaml`・床の木 `FLOOR`）・床の判定・`folio build` の出力・`main.rs` の旗の説明の字（便 203 は本流に着地済み。`folio graph --help` の --summary の説明「id の行の番号・平易文・技術の要約を添えた」は欄を列挙しきらない形のままとし、欄の中身は要件 FR14・FR31 が持つ。説明に状態の欄を足すなら別の小さな直し）。

### (c) 歯（f208_・base で 0 件・4 本と既存の歯の期待の字 16 行）

1. **`tests/graph_summary.rs` f208_the_frozen_base_lines_end_with_the_state_of_their_file。** 床の土台の --summary の各行で、status の欄がちょうど最後に 1 つ在り、判断の記録は `"accepted"`・設計ノートの行は `"example"`・ほかは null（数 10・1・183）。status の欄を外した出力は **便 208 の前の組み立ての出力と byte で同じ**: 凍結 anchor = 定数 `BEFORE_208`（194 行・115,579 byte・sha256 `78eab543…cac35`・起草役が便 207 の見本の binary で撃って sha256sum で測った手書き・便 207 の起草役の測りと一致・P-10.1）。
2. **`tests/graph_summary.rs` f208_a_changed_state_moves_only_its_status。** 床の土台の写しで ADR-3 を proposed・ADR-4 を値域の外の字（`status: 'Accepted "x"'`・YAML の一重引用・大字と引用符を持つ）・ADR-5 を retired・ADR-7 の status の行を消す・ADR-8 を `[accepted]`（字でない）、設計ノート example を retired（後継 figures と承認欄つき・頭の注 2 行を落として行の番号を動かさない）に替えると、動く行はこの 6 つだけで、各行は status の欄の前が 1 字も動かず、status が proposed・`"Accepted \"x\""`（訳さず・畳まず・JSON の逃がしだけ・検証役の N-1）・retired・null・null・retired に替わる。--digest は土台と byte で同じ。
3. **`tests/graph_notes.rs` f208_a_note_line_carries_the_note_state_word。** 床の土台の写しに外の利用者の形のノート（塊の形の行・状態 draft）・退役のノート（retired）・発効のノート（effective）・状態の欄の無いノート・状態が一覧のノートを足すと、設計ノートの行の status は example・draft・retired・effective の字そのままで、欠けと一覧は null。
4. **`tests/graph_notes.rs` f208_an_outside_place_has_the_same_shape。** 外の置き場の最小の写し（`folio init` の骨格・git なし）に外の利用者の形のノートを足すと、各行の欄は 8 つがこの順に在り、字を持つのは ADR-1（proposed）と wave#p・q・r（draft）だけで、ほかの 6 節点は null（10 行）。
5. **既存の歯の期待の字（status の欄を足すだけ・判定の対象は変えない）。** `tests/graph_summary.rs` の 12 行（f180_the_frozen_base_lines_are_the_hand_written_ones の 7 行・f180_text_fields_are_escaped_into_one_json_line の 4 行と受入基準 AC12 の末尾の 1 つ）に `,"status":null`、`tests/graph_notes.rs` の 4 行（f185_the_frozen_base_row_is_a_node_with_its_req_edge の example#a に `"example"`・f185_block_rows_keep_the_meta_id_the_section_title_and_depends の wave#p・q・r に `"draft"`）。ADR-9 の行の頭と中の字を見る assert と、--print・--digest の凍結 anchor の歯（f180_the_summary_leaves_the_print_and_the_digest_on_their_anchors）は変えない。
6. **RED の実測。** 歯の file 2 本だけ（見本の tests の字）を base に当てると tests/graph_summary.rs と tests/graph_notes.rs の 16 本のうち 8 本が落ちる（f208_ の 4 本と、期待の字に status を足した既存の 4 本〔f180_ 2 本・f185_ 2 本〕・nextest の rc 100・落ちた歯の本文は red-208.log）。落ちない 8 本は期待の字を直していない歯。単体の歯は無い（`--bin folio f208_` は 0 本・nextest の rc 4）（起草の記録の red-208.log）。

### (d) 採らなかった形

1. **文書の見出しの欄の状態を全部の節点に写す（ADR-35 の選択肢 d）。** 保留の要件 FR13 や廃止の行 R-22 に「発効」が付き、欄の意味が「その点の一生」と「文書の版」に割れる。
2. **設計ノートの行だけに状態を出す（選択肢 e）。** 判断の記録の退役を外の利用者が記録の YAML から別に読むことになる（読み手が 2 つ）。
3. **状態の字を日本語に訳す・畳む。** 語の置き場が 2 つになる（P-6.3）。字は正本のまま（変異 M10・M12 が落ちる）。
4. **--print の表に状態の列を足す。** 表の凍結 anchor と外の利用者の読み手が動く。ADR-35 決定 (2)(エ) は --print を変えない。
5. **便 208 の前の出力を凍結 anchor の file（`tests/fixtures/schema/graph-anchor.txt`）に置く。** 床の土台を書き換える便が 2 つの file を直すことになる。定数は歯の file の中に置き、土台が変わったら前の組み立ての binary で測り直す（(g) 4）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写しで workspace の nextest 1173 / 1173（base 1169 + f208_ の 4 本）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 40 file が base と全 file で byte で同じ。期待の字を直した既存の歯は (c) 5 の 16 行だけ。
2. **突然変異（見本の写しの `graph.rs` だけを 1 通りずつ変え、tests/graph_summary.rs と tests/graph_notes.rs を撃つ）。** 17 通りとも落ちる（生き残り 0・M15〜M17 は独立の検証役の V5〜V7 で、歯 (c) 2 の ADR-4 の値域の外の字が落とす＝検証役の N-1・M1〜M14 は本流 001c865 と a5b27df の上でも撃って同じ本数で落ちた＝起草の記録の chain2・chain3）（起草の記録の mut-208-M*.log）。

| 変異 | 落ちる歯（tests/graph_summary.rs と tests/graph_notes.rs の 16 本のうち） |
| --- | --- |
| M1 status の欄を出さない | 8 本（f180 escape・f180 土台の手書き・f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M2 状態を覚えない（全部 null） | 6 本（f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M3 判断の記録の状態を読まない | 3 本（f208 替えた写し・f208 外の置き場・f208 土台） |
| M4 設計ノートの状態を読まない | 6 本（f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M5 設計ノートの行の状態を行の欄から読む | 6 本（f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M6 status を eng の前に置く | 8 本（f180 escape・f180 土台の手書き・f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M7 字でない状態を空の字にする | 2 本（f208 替えた写し・f208 ノートの字） |
| M8 要件書の行に要件書の meta.status を写す | 4 本（f180 escape・f180 土台の手書き・f208 外の置き場・f208 土台） |
| M9 欄の名を state にする | 8 本（f180 escape・f180 土台の手書き・f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M10 廃止の字を訳す | 2 本（f208 替えた写し・f208 ノートの字） |
| M11 規則の表の行の status を写す | 4 本（f180 escape・f180 土台の手書き・f208 外の置き場・f208 土台） |
| M12 状態の字を畳んで切る（題と同じ fold） | 6 本（f185 塊の形・f185 土台の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M13 --summary の口でも表を出す（status を足した表） | 11 本（f180 表と 1 行ずつ・f180 escape・f180 土台の手書き・f180 anchor・f185 塊の形・f185 土台の行・f185 外の行・f208 替えた写し・f208 ノートの字・f208 外の置き場・f208 土台） |
| M14 見本の状態の設計ノートを null にする | 4 本（f185 土台の行・f208 替えた写し・f208 ノートの字・f208 土台） |
| M15 状態の字を小字にする（検証役の V5） | 1 本（f208 替えた写し） |
| M16 値域の外の字を null にする（検証役の V6） | 1 本（f208 替えた写し） |
| M17 状態の字を JSON の逃がしなしで書く（検証役の V7） | 1 本（f208 替えた写し） |

3. **外の置き場。** tsuzuri の写し（本物の main 8c656f4 の `.git` を cp -a して clone・remote を外した・hooksPath 0・sample でない hook 0）で base と見本の binary を撃つと、--print と --digest は byte で同じ、--summary は 452 行とも status の欄を外すと byte で同じ（tz-208.log）。状態の欄の値は (a) 3 の表のとおりで、契約表の行を持つ設計ノートは全部 effective（行を持たない draft のノートが 1 本在り、節点にならない）、retired の設計ノートと退役の判断の記録は今は 0。骨格 `folio init` の写しでも同じ形（(c) 4）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・縮む file・消す file は無い。単体の歯は無い（歯は binary を撃つ 2 本の file だけ）。
2. **余地（CapHeadroom）。** write-set の src は 1 本（python と awk の 2 実装で一致・起草の記録の cap-208.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/graph.rs` | 1166 | 334 | 1182（+16） | 318 |

3. **size は S。** src の増分は +16 で S の見積 100 の内。余地 334 は S の 100 を超える。
4. **verify は 5 行**で、done の 5 つの塊と 1 対 1 に揃える。見本の 375cbfa で 5 行とも rc 0（2・2・7・9 本と clippy 0 警告）（verify-208.log）。
   1. `cargo nextest run -p folio --test graph_summary f208_` = (c) 1・2（2 本）。
   2. `cargo nextest run -p folio --test graph_notes f208_` = (c) 3・4（2 本）。
   3. `cargo nextest run -p folio --test graph_summary` = (c) 5 の 12 行と、--print・--digest の凍結 anchor の歯（7 本）。
   4. `cargo nextest run -p folio --test graph_notes` = (c) 5 の 4 行と、便 185 の設計ノートの行の歯（9 本）。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり対象外・0（通す）。本流の binary（`target/debug/folio`・06:30 の組み立て）に write-set の 3 本を渡した答え（契約の枝の木で撃った）「folio ceiling: 通す（設計文書の正本を書き換えない便）」（gate-208.log）。
2. **受付。** 受付の先撃ち（precheck）は枝 docs/d208 の本契約で 契約に起因する断り 0（preflight ok・write-set 3 本・歯の接頭辞 f208_ は base で 0 本・precheck-208.log）。
3. **ほかのレーンとの重なり（起草の時点の origin の impl/* の枝・読むだけ）。**

| 便・枝 | 書き換える file | 本便の write-set との重なり |
| --- | --- | --- |
| 便 203・204（本流 a5b27df に着地済み） | 203 は src 9 本と歯 7 本、204 は src 3 本・歯 2 本・fixture と生成区間 18 本 | 0（graph.rs・graph_summary.rs・graph_notes.rs を触らない。本便の差分は a5b27df の上でも同じ数で緑だった＝起草の記録の chain3） |
| 便 207・impl/d207 9585bb7（親 17736c9・本便の前に着地） | src 5 本・歯 12 本・床の土台の rules.yaml と constitution.yaml・索引の凍結 anchor 3 本ほか | **`crates/folio/tests/graph_notes.rs` が重なる**（207 は f185_ の要約の 1 行の数 4 か所・本便は期待の字 4 行と f208_ 2 本と頭の注＝別の塊・9585bb7 の上に本便の差分は衝突なく当たる）。床の土台の変更で本便の定数 `BEFORE_208` と (c) 1 の数が決まる |

   graph.rs・tests/graph_summary.rs・tests/graph_notes.rs を書き換えるほかの impl/* の枝は、本流に squash で着地済み（175・177・180・185・195）か取り下げ済み（201）だけ。

4. **数え直し。** 席の裁定（2026-09-29）で着地の順は便 207 → 便 208（便 207 は本流 0914e69 に着地済みで、見本の親 9585bb7 と crates・design-intent・tests で byte で同じ＝今は数え直しは要らない）。便 207 が土台の規則の表に行 R-23〜R-25 を足すので、本便の定数は便 207 の後の土台の値にした: `BEFORE_208` = (194, 115_579, `78eab543…`)・(c) 1 の数 = [10, 1, 183]（便 207 の前の土台〔本流 17736c9〕では (191, 114_730, `53a7d113…`) と [10, 1, 180]・起草の記録の chain3）。本便は便 207 の着地の後に受ける。受付の時点の本流（便 207 の着地の木）が、見本の親 9585bb7 と、`crates/folio/src/graph.rs`・`crates/folio/tests/graph_summary.rs`・`crates/folio/tests/graph_notes.rs`・床の土台 `tests/fixtures/floor_base/design-intent` で同じなら、数え直しは要らない（差分をそのまま当てる）。どれかが違えば、差分を受付の時点の本流に当て、`BEFORE_208` を**受付の時点の本流の（便 208 の前の）binary** で測り直し、(c) 1 の数・verify・tsuzuri の写しの答えを数え直してから運ぶ。
5. **着地の後（外の置き場への知らせ・ADR-35 決定 (7)）。** 生成区間も床の判定も変わらないので、外の置き場の file の書き直しは要らない。新しい binary では --summary の各行の最後に `"status":` が付く（判断の記録と設計ノートの行は正本の字・ほかは null）。--summary の 1 行を閉じた欄の集合で読む側と、その字を比べる側は、folio の組み立てを上げる同じ commit で直す。--print と --digest は変わらない。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `~/.local/share/folio2/handoff-2026-09-28/d208-draft.md`、script と log は同じ dir の `d208-scripts/`。
1. `run-208.sh <木> <印>`: 組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と sha。
2. `red-208.sh <base の木>`: 歯の file 2 本（r208-teeth.patch）だけを base に当てて RED。
3. `mut-208.py <見本の clone> [M..]`: 変異 17 通り。
4. `cap-208.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk）。
5. `verify-208.sh <木> <印>`: 契約の verify の 5 行。
6. `prep-tz.sh`・`tz-208.sh <base の folio> <見本の folio>`: tsuzuri の写しを作り、graph の 3 つの口を base と見本で比べる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** --print・--digest・`folio hello`・節点の種類と辺の型・索引の欄の決まりの生成区間・`main.rs` の旗の説明の字（(b) 6）・床の判定・要件書・判断の記録・規則の表・外の置き場の file・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** status の値は正本の字を写すだけで、状態の欄が欄の決まりの値域に入っているかは床（判断の記録と設計ノートの欄の決まりの検査）が数える。索引は状態で節点を落とさない（ADR-32 決定 (1)）。畳むのは読み手。
3. **撤退条件。** (1) ADR-35 か要件書 第 1.57 版の字が承認で変わり、status の値の決まり・欄の置き場所・変えないものが本契約と食い違えば、止めて席へ返す。(2) 本便の後に (c) 5 の 16 行のほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に --print か --digest の出力、または status の欄を外した --summary の出力が 1 byte でも変われば、止めて席へ返す。(4) 便 207 が着地する前に受け付けることになったら、止めて席へ返す（定数 `BEFORE_208` と (c) 1 の数は便 207 の後の土台の値）。

## 2. 範囲

- 入れる: `crates/folio/src/graph.rs` の状態の表と覚える口・判断の記録と設計ノートの状態の読み・--summary の 8 欄目・歯 `tests/graph_summary.rs` と `tests/graph_notes.rs` の f208_ 4 本と既存の期待の字 16 行。
- 入れない: --print・--digest・`main.rs`・床・生成区間・要件書・判断の記録・規則の表・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| states | 状態の表 | `Index` が節点の id ごとに所属 file の文書の状態の字を持つ |
| read | 状態の読み | `adr()` が記録の status、`notes()` がノートの meta.status を覚える（字だけ） |
| line | 8 欄目 | `Index::jsonl` が eng の後に status を置く（無ければ null） |
| teeth | 歯 | f208_ 4 本と既存の期待の字 16 行 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 判断の記録 ADR-35 と要件書 第 1.57 版の発効（持ち主の承認）。
- 本便の着地の後に席が見ること: 台帳の便の項を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) 5 の知らせを出す（ADR-35 決定 (7)・便 207・209 の知らせと束ねてよい）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "hc"
title = "索引の --summary の 1 行の末尾に状態の欄 status を置く（判断の記録 ADR-35 決定 (2)・要件 FR31・台帳 f2-648.275.9）: crates/folio/src/graph.rs の Index に節点の id ごとの状態の字の表を足し、adr() は判断の記録の status、notes() は設計ノートの meta.status の字（字のときだけ）を覚え、jsonl は eng の後に status の欄（覚えた字か null）を置く。ほかの節点（条・規範文・規則行・要件・受入基準など）は null で、規則の表の行の status と文書の見出しの欄の status は写さない。--print と --digest と status を外した --summary の各行は変えない。歯は tests/graph_summary.rs の f208_ 2 本（便 207 の後の床の土台の状態の字と、status を外した出力が便 208 の前の組み立ての出力の sha256 と同じ・状態を替えた写しで動くのはその行の status だけ）と tests/graph_notes.rs の f208_ 2 本（設計ノートの状態の 4 つの字と欠けと一覧は null・骨格 folio init の外の置き場でも同じ形）と、既存の期待の字 16 行に status の欄を足すこと。便 207（行 hb）の着地の後に受け、base は便 207 の着地の木。実装の見本は origin の枝 impl/d208 の commit 375cbfa（親 = 便 207 の見本 9585bb7）で、作業者は受付の時点の本流に git diff 9585bb7 375cbfa の差分を当て、write-set の file を見本の file の中身へ置き換えず（便 207 が tests/graph_notes.rs に書いた字を残す）、write-set の外は変えない"
req = ["FR31", "FR14"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/tests/graph_summary.rs", "crates/folio/tests/graph_notes.rs"]
verify = ["cargo nextest run -p folio --test graph_summary f208_", "cargo nextest run -p folio --test graph_notes f208_", "cargo nextest run -p folio --test graph_summary", "cargo nextest run -p folio --test graph_notes", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/graph_summary.rs の f208_ の 2 本（床の土台の各行の最後が status で判断の記録は accepted・設計ノートの行は example・ほかは null、status を外した出力が便 208 の前の組み立ての出力の行の数と byte の数と sha256 に一致、状態を替えた写しで動くのはその行の status だけ）が緑、tests/graph_notes.rs の f208_ の 2 本（設計ノートの行の status が draft・effective・retired・example の字のままで欠けと一覧は null、骨格 folio init に外の利用者の形のノートを足した写しで 8 欄がこの順に在り字を持つのは判断の記録と設計ノートの行だけ）が緑、tests/graph_summary.rs の歯の全部（--print と --digest の凍結 anchor を含む）が緑、tests/graph_notes.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
