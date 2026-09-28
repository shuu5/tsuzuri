# 設計: 便 185 — 索引に設計ノートの契約表の行を節点として足す（判断の記録 ADR-32）

- 要件: FR14（索引の口・要件書 第 1.54 版で設計ノートの契約表の節の行を「文書 id#行 id」の節点にし、1 行の題を行が指す節の題にする）。本流の要件書に在る id で、字は変えない。受入は AC30。
- 条: P-2.4（節点の種類と辺の型の閉じた一覧に足す・判断の記録 ADR-32）・P-5.6（閉じた一覧の写しは索引の欄の決まりの生成区間へ導出）・P-6.3（索引は正本から毎回組む導出物）・P-4.1 / P-4.2（読めない設計ノート・行の逐語で切れない行は まだ分からない）・P-10.1 / P-10.2（期待の字は歯の手書き・要約値は独立の script）。
- 出所: 判断の記録 ADR-32（下書き・持ち主の承認待ち・台帳 f2-648.270）の決定 (1)〜(8)。出どころは外の利用者 tsuzuri の設計席の問い（tsuzuri-f7・地図の設計ノートの帯が空・tsuzuri の行 c-dn-rows の前提）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gf` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 17 本（src 2・歯の file 6〔新 1〕・設計文書の正本 3・凍結 anchor と独立の script 6）。縮む file・消す file・新しい dir は無い。
- 門: 対象。write-set に設計文書の正本（`design-intent/graph.yaml` と `design-intent/design-note/schema.yaml` の生成区間・`design-intent/vocabulary.yaml`）が在る。本流 8fb31c9 の組み立てで、下書きの枝 docs/adr32 の上に write-set 17 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 本流 8fb31c9（便 183 着地）+ ADR-32 と要件書 第 1.54 版の発効（持ち主の承認）**。受付は発効の後。この契約の数はすべて、本流 8fb31c9 を取り込んだ下書きの枝 docs/adr32（write-set の 17 本は本流 8fb31c9 と同じ字）の写しの実測（参考値・規則の表の行 D-13）。受付の時点の本流の write-set の file が 8fb31c9 と違えば (i) の撤退条件で数え直す。
- 実装の見本: origin の枝 `impl/d185`（commit **61010f2**・改訂 a〔検証役の N2・N3・N4〕を積み、本流 8fb31c9 と下書きの枝 docs/adr32 を取り込んだもの・本流との衝突 3 file は見本の字を採った）。便の全体の差分は `git diff 8fb31c9 61010f2 -- <write-set の 17 本>`（+745 −78・98,378 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 61010f2 -- <write-set の file>`）。ただし 2 本の生成区間は `folio schema --dir design-intent --write` の出力（見本と byte で同じ）。write-set の外は変えない。
- 並行の便との重なり: 便 183（行 gd）は write-set が 7 本重なっていた（`floor_note.rs`・`design-note/schema.yaml`・`tests/schema.rs`・`tests/schema_docs.rs`・`tests/graph.rs`・`note-region.txt`・`node-digest-anchor.txt`）が、本流 8fb31c9 に着地した（着地した字は見本が積んだ 366f96f と byte で同じ）。便 184（行 ge・`face_note.rs` と `tests/face_note.rs` だけ）とは重ならない。

## 1. 設計

### (a) いま起きていること（base の実測・参考値）

1. **索引は設計ノートを読まない。** 節点の種類は 11（`graph.rs` の NODE_KINDS）・辺の型は 17（EDGE_TYPES）・辺の欄の一覧は 4 つの file（EDGE_FIELDS）。設計ノートの契約表の行（契約 id = `<文書 id>#<行 id>`・欄の決まりの注 id_note）は索引に出ず、行 → 要件・行 → 先に済ませる行 の指し合いを器も席も索引から引けない。
2. **注が「契約表の行は節点にならない」と書く。** 設計ノートの欄の決まりの注 index_note（`floor_note.rs`）の最後の文。
3. **外の利用者の写し（tsuzuri）。** 索引は 240 節点・539 辺。契約表の行は、写し 1c26a6f で 126 本（req 264・depends 83）・71b4043 で 128 本（req 267・depends 83）・ad6a6fc で 130 本（req 270・depends 84）。
4. **base の歯（参考値）。** workspace の nextest 1036 / 1036・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 37 file（判断の記録 ADR-32 の面が 1 つ増える）。`git grep -n f185_ -- crates` は 0 件・行 id `gf` は 0 件。

### (b) 直す先

1. **`crates/folio/src/graph.rs` の閉じた一覧。** NODE_KINDS に `設計ノートの行`（12）・EDGE_TYPES に `req` と `depends`（19）・EDGE_FIELDS に `design-note: [req, depends]`（5）。床の定数 FLOOR の注 4 つ（節点の欄の id と title・辺の欄の一覧・要約値の式 ① に設計ノートの行の塊）を直す。
2. **`graph.rs` の組み立て（関数 notes）。** 設計ノートの置き場 `design-note/` を床と導出と同じ読み手 `note::load_notes` で読み（置き場が無ければ 0 本・dir でない・読めない file が在れば Err＝索引を組まない）、型 contract-table の節の rows の行ごとに節点を 1 つ: id は meta の id と行 id を「#」でつないだ字・file は `design-note/<file 名>`・題は行の section が指す節の title・技術の要約は行の title の全文・平易文は無し。辺は行の req（型 req）と depends（型 depends・先は同じノートの `<文書 id>#<行 id>`）。索引の file の一覧に読めた設計ノートを足す（--digest の file の行）。**同じ id の節点を 2 度組もうとしたら覚えておき（Index::node）、口（--print・--summary・--digest・folio hello の数）は関数 whole で まだ分からない にする**（黙って 1 つに数えない・種類を問わない・床の check_index は今のまま行の逐語の側で違反に数える・P-4.1）。
3. **`graph.rs` の行の逐語の読み（Scan::note）。** meta の中の字下げ 2 の `id: ` と、sections の字下げ 2 の頭の行の塊のうち字下げ 4 に `type: contract-table` を持つ節の、字下げ 4 の `rows:` の塊の中の字下げ 6 の `- ` の行ごとに塊を切る。行 id は流れの形なら表の一番上の段の欄 id（先頭でなくてよい）、塊の形なら頭の行か字下げ 8 の `id: ` の行。塊から辺の欄（req・depends）の行を落として要約値を測り、行の番号は行 id の行。組み立てと読みの id の集合が割れれば今のとおり まだ分からない。
4. **`graph.rs` の索引の節点の床（check_index）。** 設計ノートの行は「#」の前後を別に引用符を外して比べる（引用符つきの行 id を 2 度数えない）。種類 索引の節点 の違反は設計ノートの file にも掛かる。
5. **`crates/folio/src/floor_note.rs`。** 注 index_note の最後の文を「設計ノートの節は節点にならず、契約表の節の行は文書 id と行 id を「#」でつないだ id の節点になる（判断の記録 ADR-32）」に。注 semantic_check_note の終わりに「ただし 1 本のノートの中の契約表の行 id の重なりは、索引の id の一意として床が種類 索引の節点 の違反に数える（ADR-3 決定 (3) の folio2 が所有する文書の id 空間の解決・判断の記録 ADR-32）」を足す。
6. **生成区間と語彙。** `design-intent/graph.yaml`（生成区間 38 行・3,909 byte・+7 −4）と `design-intent/design-note/schema.yaml`（注 2 行）を `folio schema --dir design-intent --write` で。`design-intent/vocabulary.yaml` の語「節点」の定義に「設計ノートの契約表の行」と id の形と ADR-32 を足す（規範の欄の外）。
7. **凍結 anchor と独立の script。** `tests/fixtures/schema/node-digest.py`（独立の要約値の script に設計ノートの行の塊の切り方を足す・folio の実装を読まない）・`node-digest-anchor.txt`（その出力・土台の example#a の 1 行が増える）・`graph-anchor.txt`（土台の索引 191 節点・579 辺・773 行・31,543 byte）・`graph-digest-anchor.txt`（1,119 byte・50 行）・`graph-region.txt`（38 行・3,909 byte）・`note-region.txt`（164 行・19,979 byte）。どれも前の anchor の字の置き換えで組んだ（起草の記録の anchors-185.py・導出の命令の出力を写していない）。
8. **既存の歯の字の期待（中身は変えない）。** `tests/graph.rs`（閉じた一覧の写し 12・19・土台の数・anchor の byte と行・要約値の列・辺の表の頭 8 字）・`tests/graph_summary.rs`（節点の表の題の鍵を id の「#」の後で引く）・`tests/hello.rs`（土台の 1 行 191 節点・579 辺）・`tests/schema.rs`（設計ノートの生成区間の定数・歯 f89 の 700 行に収めるため注 2 行を 1 行に）・`tests/schema_docs.rs`（graph.yaml の生成区間の定数・閉じた一覧の数 12・19・字のずれの当て先）。
9. **変えないもの。** 設計ノートの外の節点・辺・--summary の行の字・--digest の外の行（種類と型の行と file の行と要約の行だけが増える）・--summary の欄 7 つ・床の結果（--digest と folio hello が変わるのは同じ id の節点を 2 度組んだ索引だけで、そのとき床は前から違反に数える）・裁定 id の書き出し（便 C）・憲法と要件書と判断の記録の字・folio2 自身の床の結果。`folio build` は語彙の 1 語の定義の字だけが変わる（2 面の用語集の 1 行）。

### (c) 歯（f185_・base で 0 件）

`crates/folio/tests/graph_notes.rs`（新）は凍結した土台（`tests/fixtures/floor_base/design-intent`）か、その写しの一時 dir に最小の手書きの設計ノート wave-file.yaml（meta の id は wave・契約表の行 p と q は塊の形・r は流れの形で id が先頭でない・q は id が 2 行目・depends に p と行の無い zz・部品表の行 part・計画の行の索引〔row-index〕の行 s）か、退役の設計ノート old.yaml（状態 retired・契約表の行 z）を足して命令を撃つ。要約値は手で書いた塊を命令 sha256sum で測る。

1. **f185_the_frozen_base_row_is_a_node_with_its_req_edge**（AC30 の 1）: 土台の契約表の行 a が節点の行 `example#a・設計ノートの行・design-note/example.yaml・4914ab68・目的` と辺 `example#a → FR15（req）` で出て、--summary の行（line 55・plain は null・eng は行の題）と --digest の種類・型・file の行が手書きの字のとおり。
2. **f185_block_rows_keep_the_meta_id_the_section_title_and_depends**（AC30 の 2）: id は file 名でなく meta の id・題は節の題（36 字で切る）・eng は行の題の全文・line は行 id の行（23・30・36）・depends の辺は wave#q → wave#p・行の無い zz は辺に出さず端が節点でない参照に 1 足す・部品表の行と計画の行の索引の行は節点にならない（要約の行 194 節点・583 辺・型 12・端が節点でない参照 27）。
3. **f185_an_edge_only_change_moves_no_digest_and_a_title_moves_one**（AC30 の 3）: req と depends だけを変えても節点の行は動かず、行の題を変えるとその行の要約値だけが動く。
4. **f185_the_lines_outside_the_notes_do_not_move**（AC30 の 4）: 設計ノートの置き場を消した写しと比べ、設計ノートの外の節点・辺・--summary の行が byte で同じ。--digest で片方だけに在る行は手書きの 6 行と 4 行だけ（行の数の差は file の行 2 つ）。
5. **f185_an_unscannable_or_unreadable_note_is_inconclusive**（AC30 の 5）: 引用符つきの行 id・読めない設計ノート・dir でない置き場で --print は表を出さず 2（まだ分からない）。床は引用符つきの行 id を種類 索引の節点 の違反ちょうど 1 行に数える。
6. **f185_a_row_id_twice_in_one_note_is_inconclusive**（AC30 の 6）: 1 本のノートの 2 つの契約表に同じ行 id が在れば --print・--summary・--digest・folio hello がどれも表を出さず 2・床は 索引の節点 の違反に数える。
7. **f185_a_retired_note_row_is_still_a_node**（ADR-32 決定 (1)・状態で絞らない）: 退役の設計ノートの契約表の行が節点の行 `old#z・設計ノートの行・design-note/old.yaml・<手書きの塊の要約値>・行 z — 退役した便の見出し` と辺 `old#z → FR1（req）` で出る。
8. **RED の実測。** base に歯の file（crates/folio/tests/ の 6 本と tests/fixtures/schema/ の 6 本）だけを当てると f185_ の 7 本とも落ち（設計ノートの行が索引に無く、読めないノートでも 0 で終わる）、workspace で 25 本が落ちる（f185_ 7・字の期待を直した既存 18〔単体の note::tests の 1 本を含む〕・起草の記録の red-185-*.log）。

### (d) 採らなかった形

1. **種類の名を「契約表の行」にする。** 持ち主の 2 つの道具（folio と器）で同じものに 2 つの名が付く。種類の名は置き場の名（設計ノート）にそろえる。
2. **裁定 id を節点の欄にする。** --summary の欄が 8 つになり、便 C の書き出しと同じ字を 2 か所に持つ（P-6.3）。書き出しの行と file で結べば足りる（ADR-32 決定 (5)）。
3. **設計ノートの節やほかの表の行も節点にする。** ADR-14 決定 (1)（節点は id を持つ行）の外で、今の使い手（tsuzuri の地図の帯）も要らない。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写しで workspace の nextest **1043 / 1043**（1036 + f185_ の 7 本）・clippy 0 警告・床 4 本 rc 0（合格 違反 0・まだ分からない 0）・`folio build --write` 37 file（base と比べて constitution.html と srs.html の用語集の「節点」の 1 行だけが違う）。(c) の 8 の既存の 18 本は、本便の src と字の期待が揃って通る。
2. **突然変異（見本の graph.rs だけを 1 通りずつ変え f185_ と索引の既存の歯を撃つ・mut-185.log）。** 18 通りとも落ち、どれも f185_ が 1 本以上落ちる（生き残り 0）。V1・V4 は検証役が生き残りを見つけた変異で、改訂 a の歯 2（計画の行の索引の節）と歯 7 で落ちる。

| 変異 | 落ちる f185_ の歯 |
| --- | --- |
| M1 設計ノートの行の種類を別の種類にする | 1・2・4・7 |
| M2 題を節の題でなく行の題にする | 1・2・7 |
| M3 req の辺を張らない | 1・2・4・7 |
| M4 depends の辺の先に文書 id を付けない | 2・4 |
| M5 契約表でない節の行も節点にする | 1〜7 |
| M6 読めない設計ノートを黙って飛ばす | 5 |
| M7 dir でない置き場を黙って飛ばす | 5 |
| M8 索引の file の一覧に設計ノートを入れない | 1〜7 |
| M9 行の逐語の読みで辺の欄を落とさない | 1・2・3・7 |
| M10 行の逐語の読みで 2 行目の id を見ない | 2・3・4 |
| M11 流れの形で先頭でない id を見ない | 2・3・4 |
| M12 行番号を 1 つずらす | 1・2 |
| M13 床が引用符つきの行 id を 2 度数える | 5 |
| M14 meta の id を見ずに file 名を文書 id にする | 2・3・4・6 |
| V1 計画の行の索引の節の行も節点にする（組み立てと行の逐語の両方） | 2・4 |
| V4 退役のノートの行を節点にしない | 7 |
| N3a folio hello の数が 2 度組んだ節点を黙って 1 つに数える | 6 |
| N3b graph の口が 2 度組んだ節点を黙って 1 つに数える | 6 |

3. **独立の確かめ（repo の外の index-notes.py・PyYAML で行を読み、要約値は独立の script の関数で測る）。** 土台・folio2 自身・tsuzuri の写しの 3 つで、設計ノートの行の節点・辺・--summary の行が見本の出力と byte で一致し、設計ノートの外の行が base の binary の出力と byte で同じ。
4. **外の置き場（tsuzuri の写し・参考値）。** 索引は 240 節点・539 辺 → 71b4043 で **368 節点・889 辺**（設計ノートの行 128・req 267・depends 83）・今の ad6a6fc で **370 節点・893 辺**（行 130・req 270・depends 84）・どちらも端が節点でない参照 0（1c26a6f の時点は 366・886）。本流 8fb31c9 の binary の床も見本の binary の素の床も違反 5（欄の決まりの生成区間の写しが古い＝便 183 の分）→ `folio schema --dir design-intent --write`（adr・design-note・rules・graph の 4 本）の後に合格 0/0・`folio build --write` は 38 file（71b4043）・39 file（ad6a6fc）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/tests/graph_notes.rs`。差分 98,378 byte（本流 8fb31c9 からの write-set の差・17 file）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 2 本（python と awk の 2 実装で一致・cap-185.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/graph.rs` | 868 | 632 | 1036（+168） | 464 |
| `crates/folio/src/floor_note.rs` | 576 | 924 | 578（+2） | 922 |

3. **size は M。** file ごとの増分の最大は graph.rs の 168（S の 100 を超え M の 300 の内）で、余地の最小 632 は M の 300 を超える。歯の file の `tests/schema.rs` は歯 f89 の上限 700 ちょうど（base も 700）。
4. **verify は 7 行**で、done の 7 の塊と 1 対 1 に揃える。便の後の写しで 7 行とも rc 0。
   1. `cargo nextest run -p folio --test graph_notes f185_` = (c) の 1〜7。
   2. `cargo nextest run -p folio --test graph` = 土台の索引の凍結 anchor と閉じた一覧の写し。
   3. `cargo nextest run -p folio --test graph_summary` = 節点ごとの要約の行。
   4. `cargo nextest run -p folio --test hello` = 土台の 1 行の数。
   5. `cargo nextest run -p folio --test schema` = 設計ノートの生成区間の凍結 anchor。
   6. `cargo nextest run -p folio --test schema_docs` = 索引の生成区間の凍結 anchor と閉じた一覧の数。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（graph_notes・graph・graph_summary・hello・schema・schema_docs）は全部 write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は下書きの枝 docs/adr32 で契約に起因する断り 0（起草の記録の precheck-185.log）。着地の後、席は tsuzuri へ「本便の binary を取り込むときは `folio schema --write` を 1 回（graph.yaml と design-note/schema.yaml）」を返す。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/adr32-draft.md`、script と log は同じ dir の adr32-scripts。

1. 模擬: 見本 61010f2（本流 8fb31c9 からの write-set の差）。凍結 anchor と既存の歯の定数は anchors-185.py、fixture の字は fixtures-185.py で組み直せる。床 4 本と build は floor4.sh。
2. RED: red-185.sh。突然変異: mut-185.py（18 通り）。余地: cap-185.sh（lines.py と lines.awk）。独立の確かめ: index-notes.py。
3. 外の置き場: prep-tz.sh（.git だけを写す）→ tz-185.sh <写し> <base の binary> <見本の binary> <見本の clone>。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 裁定 id の書き出し（便 C）・設計ノートの面・計画の 3 つの型の行（便 183 と 184）・tsuzuri の写しの書き直し（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) 契約表の行の題の欄と section の欄の字の良し悪しは床が見ない（形だけ）。(2) 行の無い depends は索引が数えるだけで、違反は設計ノートの床の持ち分。(3) 状態が見本・下書き・退役の設計ノートの行も節点になる（ADR-32 決定 (1)）。
3. **撤退条件。** (1) 本便が要件書 FR14 の字か憲法の条文か判断の記録の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `graph.rs`・`floor_note.rs` が base と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) の 8 に挙げた外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が (e) の 1 の 1 行のほかに 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: (b) の 1〜8。
- 入れない: 裁定 id の書き出し・面・憲法と要件書と判断の記録の字・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| lists | 閉じた一覧 | `graph.rs` の NODE_KINDS・EDGE_TYPES・EDGE_FIELDS と FLOOR の注 |
| build | 組み立て | `graph.rs` の notes と source_files |
| scan | 行の逐語の読み | `graph.rs` の Scan::note・flow_value・meta_id |
| floor | 索引の節点の床 | `graph.rs` の check_index の行 id の比べ |
| teeth | 歯 | f185_ の 7 本と既存の歯の字の期待・凍結 anchor |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提: 判断の記録 ADR-32 と要件書 第 1.54 版の発効・便 183（行 gd）の着地（本流 8fb31c9・済み）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ `folio schema --write` の 1 回を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gf"
title = "索引に設計ノートの契約表の行を節点として足す（判断の記録 ADR-32 決定 (1)〜(8)・要件書 第 1.54 版の FR14・AC30）: crates/folio/src/graph.rs の閉じた一覧に節点の種類 設計ノートの行（12）と辺の型 req・depends（19）と辺の欄 design-note（5）を足し、床の定数の注 4 つを直す。組み立ては設計ノートの置き場を床と導出と同じ読み手で読み、型 contract-table の節の rows の行ごとに id = meta の id と行 id を「#」でつないだ字・file = design-note/<file 名>・題 = 行の section が指す節の題・技術の要約 = 行の題の全文の節点と、req と depends（先は同じノートの行）の辺を作る（読めない file・dir でない置き場は まだ分からない）。行の逐語の読みは rows の中の字下げ 6 の行ごとに塊を切り、行 id の欄の順を問わず、辺の欄を落として要約値と行の番号を測る。索引の節点の床は設計ノートの行 id の引用符を「#」の前後で外して比べる。同じ id の節点を 2 度組んだ索引は、どの口（--print・--summary・--digest・folio hello）も まだ分からない にする。crates/folio/src/floor_note.rs の注 index_note の最後の文を直し、注 semantic_check_note に 1 句足す。2 本の生成区間は folio schema --dir design-intent --write で書き、語彙の「節点」の定義と凍結 anchor 5 本と独立の script node-digest.py と既存の歯の字の期待を直す。歯は crates/folio/tests/graph_notes.rs の f185_ の 7 本。実装の見本は origin の枝 impl/d185 の commit 61010f2 で、作業者は write-set の file をその中身にしてよく（2 本の生成区間は folio schema --write の出力と同じ）、write-set の外は変えない。base = 本流 8fb31c9（便 183 着地）+ ADR-32 と要件書 第 1.54 版の発効"
req = ["FR14"]
section = "1"
write-set = ["crates/folio/src/floor_note.rs", "crates/folio/src/graph.rs", "crates/folio/tests/graph.rs", "+crates/folio/tests/graph_notes.rs", "crates/folio/tests/graph_summary.rs", "crates/folio/tests/hello.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/schema_docs.rs", "design-intent/design-note/schema.yaml", "design-intent/graph.yaml", "design-intent/vocabulary.yaml", "tests/fixtures/schema/graph-anchor.txt", "tests/fixtures/schema/graph-digest-anchor.txt", "tests/fixtures/schema/graph-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/schema/node-digest.py", "tests/fixtures/schema/note-region.txt"]
verify = ["cargo nextest run -p folio --test graph_notes f185_", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test graph_summary", "cargo nextest run -p folio --test hello", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test schema_docs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/graph_notes.rs の f185_ の 7 本（土台の契約表の行 a が節点・req の辺・--summary の行・--digest の行で手書きの字のとおりに出る・塊の形と流れの形の行で id は meta の id と行 id・題は節の題・eng は行の題の全文・line は行 id の行・depends は同じノートの行への辺で行の無い先は数えるだけ・契約表でない表の行は節点にならない・辺の欄だけの変更は要約値を動かさず行の題の変更はその行だけを動かす・設計ノートの外の行は置き場を消した写しと byte で同じ・引用符つきの行 id と読めないノートと dir でない置き場は まだ分からない で床は違反 1・1 本のノートの行 id の重なりは --print・--summary・--digest・folio hello のどれも まだ分からない で床は違反・計画の行の索引の行は節点にならず退役のノートの行は節点になる）が緑、tests/graph.rs の歯の全部（土台の索引 191 節点・579 辺の凍結 anchor と閉じた一覧 12・19 の写しを含む）が緑、tests/graph_summary.rs の歯の全部が緑、tests/hello.rs の歯の全部（土台の 1 行を含む）が緑、tests/schema.rs の歯の全部（設計ノートの生成区間 164 行・19,979 byte を含む）が緑、tests/schema_docs.rs の歯の全部（索引の生成区間 38 行・3,909 byte を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じで中身の違いは用語集の「節点」の定義の 1 行だけである"
<!-- contracts:end -->
