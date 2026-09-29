# 設計: 便 172 — 判断の記録の封と --freeze-adrs を足し、天井の束に判断の記録の面を写さない（便 170 を割った前半・判断の記録 ADR-30 決定 (3)(4)）

- 要件: FR5（床は 3 値で返し、測れないものを合格にしない＝封の一覧が無ければ まだ分からない）・FR19（判断の記録の欄の決まりの生成区間を床の定数から導く＝封の欄を足す）・FR17（天井の材料の束＝判断の記録の面を写さない）・FR24（始まりの凍結の前提＝判断の記録の封の欠けを数えない）。中身は判断の記録 ADR-30 決定 (3)（床の封）と決定 (4)（束に面を写さない）。決定 (2)（改訂の欄を削る）は後半の便 173（行 ft）。
- 条: P-6.2（封の一覧は folio の出力）/ N-1.1（在る行を書き換えない）/ P-10.1（凍結 anchor = 歯の fixture）/ P-10.3（封の一覧が無ければ まだ分からない）/ P-5.1（要約値は置き場ごとの data）/ P-7.2（退役は status で）/ N-3.1（旗は封を足すだけ）。
- 出所: 持ち主の承認 2026-09-27 12:50 JST（対話面 R-8・逐語「承認する」・構造の直し A）。便 170 の契約 `docs/design/delivery-170.md`（行 fq・記録として残す）。
- 置き場: 審査の材料は行 `fs` の §1 だけ。write-set 54 本（src 8〔新規 1〕・歯の file 11〔新規 1〕・設計文書 2〔生成区間 1・新規の封の一覧 1〕・fixture 33〔新規 2〕）。新しい dir・消す file は無い。
- 門: **本流 f27c859 の binary で 0（通す）**（write-set 54 本・`通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）`）。
- base = 本流 f27c859（数はその写しの実測・参考値・行 D-13）。

## 1. 設計

### (a) いま起きていること（参考値・f27c859）

1. **便 170 は審査の上限で止まった。** 便 170（行 fq・台帳 f2-648.254）の run `f2-648.254-20260927T073348Z` は実装まで済み、verify 17 行は全部 rc 0 だった。しかし門の審査（lens・別の AI が差分を読む段）が「diff 196520 byte が cap 150000 を超えた」で Gated INCONCLUSIVE（まだ分からない）。cap は器の規則の行 gate.token_cap（持ち主の裁定）で変えない。**実装は正しい見込みなので、同じ実装を 2 便に割り、各便の差分を cap の下に収める。** 本便はその前半（封と束・差分 108,793 byte・54 file）、後半の便 173（行 ft）は改訂の欄の削除（104,073 byte・30 file）。目安は各便 120,000 byte 以下。
2. **実装の見本。** origin の枝 `impl/d172`（commit 57648da・base f27c859）が本便の後の中身。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 57648da -- <write-set の file>`）。write-set の外は変えない。見本は便 170 の実装（`impl/f2-648.254-run1` = 6099cf4）の字をそのまま使い、封と束の分だけを取り出したもの（取り出し方は (b) の 8）。後半の見本は `impl/d173`（8e4d0c8・57648da の上）。
3. **封は無い。** 発効した判断の記録 28 本（ADR-1〜27・30・全部 accepted・revises の行 0）の本文の変化を床は見ない。凍結 anchor（`design-intent/anchors/`）は憲法の列・id の一覧・入口だけ。
4. **束は判断の記録の面を丸ごと写す。** `ceiling_src.rs` の FACE_NAMES が adr に面の名の形 `adr-<数>.html` を持ち、4 観点の束に面 28 枚ずつが入る。
5. **版管理の履歴に封の一覧が既に在る。** 便 170 の run の枝（手元の `scribe2/f2-648.254-…`）と `origin/impl/f2-648.254-run1` と本便・後半の見本の枝が `design-intent/anchors/adr-seals.yaml` を持つ。gitcheck（床の版管理の照合）は全ての参照の履歴を読むので、それらの参照を持つ写し（本流の根と worktree）では、**今の本流の素の床が `[anchor] anchors/adr-seals.yaml は版管理の履歴に在ったが作業ツリーに無い` で不合格**（違反 1）になり、床を撃つ歯 5 本（tests/check.rs の check_canonical_design_intent_passes と f77・parts・place_name の f154・site）も落ちる（f27c859 の写しで実測）。参照が本流だけの写し（CI の浅い写し・`clone --single-branch`）では合格・nextest 1000 / 1000。本便の着地（封の一覧が本流に入る）で解ける。**本便の数はこの違反が出る前提で取った**（base の床・歯の数は参照が本流だけの写しで、便の後の数は封の一覧を持つ枝 4 本〔run の枝・impl/f2-648.254-run1・impl/d172・impl/d173〕を参照に持つ写しで取った・CI は浅い写しで落ちない）。run の worktree を模した写し（その参照を持ち本流 f27c859 から作る）で見本の file を取ると、commit の前は `anchors/adr-seals.yaml が版管理に追跡されていない` で違反 1、**commit の後は素の床が合格・workspace の nextest 1008 / 1008**＝作業者は commit してから床と歯を撃つ。
6. **base の歯。** 参照が本流だけの写しで nextest 1000 / 1000・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file。`git grep -n f170_ -- crates` 0 件・行 id `fs` 0 件。

### (b) 直す先

1. **新しい `seal.rs`（封）。** 置き場 = anchor.dir（`anchors/`）の下の `adr-seals.yaml`。中身 = 頭の注・`kind: adr-seals`・`digest_algo: sha256-json-1`・`outside: [status, superseded_by]`・`rows`（`{id, sum}` を足した順）・`digest`。要約値 = 記録の木から outside の欄を除いた表の正規化（凍結 anchor の digest と同じ yaml::canonical）の sha256。
   - check_seals（`check.rs` が ids の検査の後に呼ぶ）: 発効した記録ごとに、行と違えば違反 `[adr] <id>: 発効した判断の記録の本文が封（anchors/adr-seals.yaml）の行と違う（…判断を変えるなら新しい判断の記録を立てる）`、行が無ければ違反 `[adr] <id>: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）`。行が在るのに発効した記録が無い・同じ id の行が 2 つ・digest が中身と合わない（行を照らさない）・symlink も違反、形が違えば まだ分からない。**file が無く発効した記録が在れば まだ分からない 1 行**（P-10.3）。行と file の欠けは凍結の旗（`--freeze-anchor`・`--freeze-ids`・`--freeze-start`・`--freeze-adrs`）の前提の検査に数えない（FR24）。
   - freeze（`--freeze-adrs`）: 本文が行と違う記録が 1 つでも在れば断る（rc 1・`anchors/adr-seals.yaml の在る行（ADR-n）は書き換えない（封は足すだけ・N-1.1…）`）。全検査が 0 違反で まだ分からない も無いときだけ、在る行を字のまま残し欠けた行を id の数の順で末尾に足す。足す行が無ければ `足す封の行は無い` で書かない。
2. **口。** `check.rs`（Materials に seals）・`phase.rs`（Flag::FreezeAdrs）・`freeze.rs`（after の腕 1 つ）・`main.rs`（`mod seal`・旗 `--freeze-adrs`〔ほかの凍結の旗と `--emit-amends` と排他〕）。
3. **`floor_adr.rs`（封の分だけ）。** 封の定数 SEAL_FILE（値 adr-seals.yaml）・SEAL_KIND（値 adr-seals）・SEAL_OUTSIDE（status と superseded_by）と、床の木の `seal: {file, kind, outside}` と `seal_note`（limits_note の前）、頭の注の読み手に seal.rs。**改訂の欄 REVISES・REVISE_KIND・REVISES_ENTRY と床の木の revise_kind・revises_entry・revises_note は残す（便 173）。**
4. **`adr.rs`。** 単体の歯 floor_notes_are_outside_the_diff の注の数を 24 → 25（seal_note が足され revises_note はまだ在る）。検査の本体は変えない。
5. **`ceiling_src.rs`。** FACE_NAMES の adr を `None`（束に面を写さない）・表の注と単体の歯。面そのもの（folio build・folio face の出力）と束の sources/adr/ の写しは変えない。
6. **生成物（手で書かない）。** `design-intent/adr/schema.yaml` の生成区間 = `folio schema --dir design-intent --write` の出力（140 行・26772 byte・sha256 bfec83ea…＝凍結 anchor adr-region.txt と byte 一致）。`design-intent/anchors/adr-seals.yaml` = `folio check --dir design-intent --freeze-adrs` の出力（28 行・digest d860d689…）。**参照が本流だけの写しで命令を撃った出力と見本の file は byte 一致。** (a) の 5 の参照を持つ写しでは gitcheck の違反 1 が立つので `--freeze-adrs` は `凍結しない` で断る＝そこでは見本の file を取り、素の床が合格し `--freeze-adrs` が `足す封の行は無い（…行 28 はどれも発効した記録と同じ…）` を返すことで命令の出力と同じだと確かめる。受付の本流の判断の記録が f27c859 と違えば、参照が本流だけの写しで命令を撃ち直す（(i) の 3）。
7. **変えないもの。** 判断の記録の本文・改訂の欄 revises の欄の決まりと床と面（便 173）・amends / supersedes の床・既存の凍結の旗の前提（封の欠けを数えないことだけ足す）・天井の TRIGGER_ADR_FIELDS と ceiling.yaml・`folio init` の骨格。
8. **見本の取り出し方。** 丸ごと便 170 の実装の字 = seal.rs・check.rs・freeze.rs・phase.rs・main.rs・ceiling_src.rs と歯の tests/freeze.rs・freeze_root.rs・note.rs・floor_cases.rs・modules.rs・bundle.rs・findings.rs と fixture（封の一覧 2・seal-ADR-11.yaml・束の凍結 anchor 2・所見 10）。分けた file = floor_adr.rs（封の塊だけ）・adr.rs（注の数）・tests/seal.rs（revises の歯 1 本を便 173 へ）・tests/adr.rs（封の直し + 便 101 の歯の封の直し）・欄の決まりの写し 17 本（seal の 1 行だけ）・adr-region.txt（2 行の挿入）・生成物と定数（命令と独立の Python の出力）。script は (h)。

### (c) 歯（f170_・base で 0 件）

土台は `tests/fixtures/floor_base/design-intent/` の写し（git init と 1 commit）。封の一覧の fixture `tests/fixtures/floor_base/design-intent/anchors/adr-seals.yaml`（10 行）と、新しい発効した記録の最小の手書き `tests/fixtures/adr/seal-ADR-11.yaml`（その要約値は歯が json の字と `sha256sum` で測った値を持つ）。名は便 170 の f170_ のまま（便 170 の §1 (c) の 2〜10 と同じ歯・(c) の 1 と 9 は便 173 の f173_）。

1. **f170_one_char_in_an_effective_record_is_caught（`tests/seal.rs`）。** ADR-3 の決定に「。」を 1 字 → rc 1・違反は封と違う の 1 行だけ。**base は合格＝RED。**
2. **f170_retiring_changes_only_status_and_superseded_by（同）。** 新しい ADR-11（supersedes: ADR-2）を置き ADR-2 を retired + superseded_by: ADR-11 → 違反は ADR-11 の行が無い の 1 行だけ → `--freeze-adrs` で前の 10 行が字のまま・11 行目が歯の値 → 合格。**base は旗が無い＝RED。**
3. **f170_a_missing_row_is_caught_and_the_freeze_appends_it（同）。** ADR-11 だけ足す → 行が無い の 1 行 → `--freeze-adrs` は rc 0・`封を足した: ` → 合格 → 2 度目は `足す封の行は無い` で 1 byte も書かない。**base は合格＝RED。**
4. **f170_the_freeze_does_not_rewrite_a_row（同）。** ADR-3 を変えて `--freeze-adrs` → rc 1・`在る行（ADR-3）は書き換えない`・封の一覧は byte で不変。別の写しで ADR-11 に未知の欄 `foo: 1` → `--freeze-adrs` は `凍結しない`・byte で不変。**base は旗が無い＝RED。**
5. **f170_a_sealed_record_stays_effective_and_the_list_is_not_hand_edited（同）。** ADR-4 を proposed に戻す → 行が在るのに発効した記録が無い の 1 行。要約値の 1 桁を手で変える → digest が中身と合わない の 1 行。**base は合格＝RED。**
6. **f170_folio2_itself_is_sealed（同）。** 本流の封の行の id の集合 = 発効した記録の file の集合・写しの素の床は合格・`--freeze-adrs` は `足す封の行は無い`。**base は file が無い＝RED。**
7. **f170_the_body_sum_skips_only_status_and_superseded_by（`seal.rs` の単体）。** SEAL_OUTSIDE が 2 欄・status と superseded_by だけ違う記録は同じ要約値・題の 1 字で違う・歯の持つ値と一致。**base は src に無い（0 件で rc 4）。**
8. **f170_the_bundle_copies_no_adr_face（`tests/bundle.rs`）。** 凍結の土台 `tests/fixtures/ceiling/bundle/`（配信先に adr-1.html・adr-2.html）から 4 観点の束を組むと faces/ に `adr-` の面が無く、`faces/srs.html` と `sources/adr/ADR-1.yaml` は在る。**base は面を写す＝RED。**

### (d) 採らなかった形

1. **割り方を逆にする（前半 = 改訂の欄の削除・後半 = 封）。** 試しの写しで撃った: 前半だけの本流は (a) の 5 の参照を持つ写しで素の床が不合格のまま（封の一覧が履歴に在って作業ツリーに無い）で、作業者の写しで床 4 本を緑にできない。外の置き場の手順（行を消してから封を足す）はこの順でも守れる（(g) の 3）。便 101 の歯 7 本を本便で直して便 173 で消す手間は、この断りより軽い。
2. **欄の決まりの写し 17 本と生成区間を 1 便にまとめる（封の欄の 1 行も便 173 へ）。** 便 173 が 124 KB を超え目安に入らない。封の定数だけ先に入れて生成区間に写さない形は、床の定数と生成区間が食い違う（FR19・P-5.6）。
3. **便 170 の封の形そのもの（置き場・命令・file が無いときの まだ分からない）の採らなかった形**は便 170 の契約 §1 (d) の 1〜6 のとおり（変えていない）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけ（src と design-intent の 10 file）を f27c859 に当てると既存 93 本が落ちる**（1001 本中・本文は d172-scripts の sim0-s0-172-nextest.log）。扱いは次の 3 つだけ（便 170 の (e) の 1 の封と束の分）。
   - fixture の直しで緑になる: 欄の決まりの写し 17 本（seal の 1 行）・floor_base の封の一覧・adr-region.txt・node-digest-anchor.txt・束の凍結 anchor（独立の script `bundle-anchor.py` の FACES から adr を外した出力）・所見の fixture 10 本。
   - 歯の本文を直す: 発効した記録を写しの上で変える歯は封の違反 1 行を確かめて外す（`tests/adr.rs` の図と f92_ の 9 本と **便 101 の 7 本〔写しの ADR-13 に植えた行・ADR-13 の封の違反 1 行を外す道具 beyond_the_adr13_seal・便 173 で区間ごと消える〕**・`tests/note.rs` の f161・`tests/freeze_root.rs` の f158 と f121〔`--freeze-adrs` と commit の段を足す〕）。`tests/freeze.rs` の合成の改訂 2 本は上書きの ADR-5 を新しい ADR-11 にして `--freeze-adrs`。`tests/floor_cases.rs` の runner は folio を撃つ前ごとに写しの封の一覧を今の発効した記録で組み直す（floor_cases.yaml は不変）。定数 = `tests/schema.rs` の生成区間の行数・byte・sha256・`tests/graph.rs` の F99_ANCHOR_SHA256・`tests/modules.rs` の層の表に seal・`adr.rs` の単体の注の数。`tests/bundle.rs` の 4 本と `tests/findings.rs` の 3 本は束の凍結 anchor と要約値の定数。
   - 消す歯は無い（便 101 の 7 本も本便では残す）。
2. **便を当てた写し。** nextest **1008 / 1008**（1000 + 8）・clippy 0 警告・床 4 本 rc 0・build 35 file（本流と byte で同じ）。参照に (a) の 5 の枝を持つ写しでも同じ（封の一覧が作業ツリーに在る）。
3. **RED。** 歯だけ（r172-teeth.patch = tests/seal.rs・tests/bundle.rs・seal-ADR-11.yaml・floor_base の封の一覧）を f27c859 に当てると `--test seal f170_` 6 本・`--test bundle f170_` 1 本が落ち、`--bin folio f170_` は 0 件で rc 4（本文は red-red172-*.log）。
4. **突然変異（便の後の写しの src を 1 通りずつ変え、verify の 1〜3 を撃つ）。** 9 通りとも f170_ が落ちる（mut.log）。

| 変異 | 落ちる f170_ |
| --- | --- |
| M1 要約値が status と superseded_by も含む | 7 本（単体を含む） |
| M2 本文が行と違っても封を足す旗が断らない | does_not_rewrite |
| M3 file が在るのに行の欠けを違反にしない | missing_row・retiring |
| M5 封の一覧の digest を照らさない | stays_effective |
| M6 足す行を在る行の前に書く | missing_row・retiring |
| M7 行が在るのに発効を戻した記録を落とさない | stays_effective |
| M9 凍結の旗でも行の欠けを数える | missing_row・retiring |
| M10 束に判断の記録の面を写す（前の形） | copies_no_adr_face |
| M11 封を足す旗がほかの違反・まだ分からない を見ない | does_not_rewrite |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新規（`+`）5 本（seal.rs・tests/seal.rs・本流の封の一覧・floor_base の封の一覧・seal-ADR-11.yaml）・縮む（`-`・行数）1 本（bundle-anchor.py 292 → 291）。ほかは増えるか同じ。差分 108,793 byte（`git diff f27c859 57648da | wc -c`）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）を 1500 から引く。python と awk の 2 実装が 8 本とも一致した（lines.log）。

| file | base | base の余地 | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/adr.rs` | 1049 | 451 | 1049（0） | 451 |
| `crates/folio/src/ceiling_src.rs` | 660 | 840 | 662（+2） | 838 |
| `crates/folio/src/check.rs` | 1045 | 455 | 1053（+8） | 447 |
| `crates/folio/src/floor_adr.rs` | 520 | 980 | 542（+22） | 958 |
| `crates/folio/src/freeze.rs` | 514 | 986 | 521（+7） | 979 |
| `crates/folio/src/main.rs` | 674 | 826 | 682（+8） | 818 |
| `crates/folio/src/phase.rs` | 71 | 1429 | 73（+2） | 1427 |
| `crates/folio/src/seal.rs`（新規） | 0 | 1500 | 316 | 1184 |

3. **size は M。** 新しい src 316 行と歯 7 本・既存の歯の直し。余地の最小は便の後の check.rs の 447（≥ 300）。
4. **verify 15 行 = done の 15 の塊。** 1 `--test seal f170_`（(c) の 1〜6・6 本）・2 `--test bundle f170_`（(c) の 8）・3 `--bin folio f170_`（(c) の 7）・4 `--bin folio`（150 本・注の数の単体を含む）・5 `--test adr`（20 本）・6 `--test bundle`（20 本）・7 `--test findings`（39 本）・8 `--test floor_cases`（12 本）・9 `--test freeze`（5 本）・10 `--test freeze_root`（14 本）・11 `--test graph`（16 本）・12 `--test modules`（2 本）・13 `--test note`（39 本）・14 `--test schema`（21 本）・15 clippy 0 警告。便の後の写しで 15 行とも rc 0（verify.log）。base では 1 が組み立ての断り（rc 101・tests/seal.rs が無い）、2 と 3 が 0 件で rc 4。`--test` の 11 本（重なりを除く）はどれも write-set に在り、f170_ の単体の歯を持つ `seal.rs` も write-set に在る。

### (g) 門・受付・並行の便・外の置き場

1. **門。** 本流 f27c859 の binary で write-set 54 本は 0（通す）。受付の時点で撃ち直す。
2. **受付と作業の写し。** live な run（便 170・f2-648.254）と write-set が重なる＝`write-set-overlap` の断りは run を止めるまで出る（席が持ち主に停止を頼む）。本流の根の worktree は (a) の 5 の参照を共有するので、作業者は封の一覧を見本 57648da から取る（(b) の 6）。着地の後は本流の根でも素の床が合格する。
3. **後半の便 173 との関係。** 便 173 は本便の着地の後の本流の上で受け付ける（base = 57648da と同じ中身）。本便は改訂の欄を残すので、便 101 の歯 7 本は本便で封の直しを受け、便 173 で区間ごと消える。
4. **外の置き場（tsuzuri）の手順。** 本便の binary で封を足す前に、発効した記録 ADR-7・8・11 の revises の塊（計 15 行）を消す: ① revises の塊を消す → ② `folio schema --dir design-intent --write` → ③ `folio check --dir design-intent --freeze-adrs` → ④ commit → 素の床は合格。便 173 の binary では ⑤ `folio schema --write` → ⑥ commit で合格（tsuzuri 56352b2 の clone で実測・tsuzuri.log の P1）。**revises を残したまま本便の binary で封を足すと、便 173 の binary で revises が未知の欄になり、消すと封と違う 3 本が残って抜けられない**（同 P2 の実測）。迷うなら便 173 の着地の後にまとめて上げる。
5. **init の骨格。** 骨格の ADR-1 は proposed で封は要らない（便 170 の (g) の 4 と同じ）。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d172-draft.md`、差分と script は同じ dir の d172-scripts。1 mk-s172.py（本流の写しに便 170 の実装から封と束を取り出す）→ fin-regen.sh 172（生成区間と定数と node-digest-anchor.txt）→ commit → suite-172.sh（build・nextest・clippy・床 4 本）。2 red-172-173.sh（r172-teeth.patch で RED）。3 mut-172-173.py（突然変異・元へ戻す）。4 lines-172.sh（python と awk の余地）。5 sim0-172-173.sh（実装だけで落ちる歯）。6 verify-172-173.sh・gate-172-173.sh・build-count.sh・tsuzuri-172-173.sh・ws.sh（write-set の印）。7 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-172.md#fs`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** 封の一覧が在るときは行ごとに違反、file が無いときは まだ分からない 1 行（FR5）。封の欠けは凍結の旗の前提に数えない（FR24・AC21・要件書 第 1.50 版の字）。FR17 の束は判断の記録の面を除く（第 1.50 版の字）。FR16 の面は本便では変えない（便 173）。
2. **言えないこと。** 封は版管理の上の生成物で暗号の署名ではない。本便の後・便 173 の前の本流は、改訂の欄を欄の決まりに残したまま封を持つ（本流の記録は行 0 なので床は合格）。この間に外の置き場が revises を残して封を足すと (g) の 4 の落とし穴に入る。
3. **撤退条件。** (1) (e) の 1 の 93 本のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 受付の時点の本流の判断の記録の数・状態・本文が f27c859（28 本・全部 accepted）と違えば、封の一覧は見本を取らず参照が本流だけの写しで命令を撃ち直し、行の数を数え直す。(3) 差分が 120,000 byte を超えたら止める。(4) 着地の後の本流で `folio build` の file 数が着地の直前と違うか、面が 1 つでも byte で変われば止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜6・歯（f170_ の 8 本と (e) の直し）・fixture 33 本。
- 入れない: 改訂の欄 revises の削除（便 173）・判断の記録の本文・天井の TRIGGER_* と ceiling.yaml・門と印・init の骨格・外の置き場への書き込み・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| seal | 封 | seal.rs の check_seals・freeze・封の一覧の形・口の配線 |
| floor | 欄の決まり | floor_adr.rs の封の定数と床の木・生成区間 |
| bundle | 束 | ceiling_src.rs の FACE_NAMES の adr |
| teeth | 歯 | f170_ の 8 本・封の一覧と seal-ADR-11.yaml の fixture・束の凍結 anchor・既存の歯の封の直し |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。前提は無い（本流 f27c859 の上）。後続は便 173（行 ft）。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の根の素の床が合格（封 28 行・(a) の 5 の違反が消える）・便 173 の受付。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fs"
title = "便 170（行 fq）を審査の上限（diff 196520 byte が cap 150000 を超えた）で 2 便に割った前半・構造の直し A（持ち主の承認 2026-09-27 12:50 JST・判断の記録 ADR-30 決定 (3)(4)）: 新しい seal.rs が封の一覧 anchors/adr-seals.yaml（行 = id と status・superseded_by を除く本文の要約値・digest）を読み、発効した記録の本文が行と違う・行が無い・行が在るのに発効した記録が無い・digest が合わないを違反に、file が無ければ まだ分からない にする。folio check --freeze-adrs は全検査が 0 違反のときだけ欠けた行を末尾に足し、違う本文が在れば断る。封の欠けは凍結の旗の前提に数えない（FR24）。floor_adr.rs に封の定数と床の木の seal・seal_note を足す（改訂の欄 revises は便 173 まで残す）。ceiling_src.rs の FACE_NAMES の adr を None にし束に判断の記録の面を写さない。生成区間と本流の封の一覧（28 行）は folio の命令の出力。実装の見本は origin の枝 impl/d172 の commit 57648da で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 f27c859"
req = ["FR5", "FR19", "FR17", "FR24"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "crates/folio/src/ceiling_src.rs", "crates/folio/src/check.rs", "crates/folio/src/floor_adr.rs", "crates/folio/src/freeze.rs", "crates/folio/src/main.rs", "crates/folio/src/phase.rs", "+crates/folio/src/seal.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/bundle.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/floor_cases.rs", "crates/folio/tests/freeze.rs", "crates/folio/tests/freeze_root.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "crates/folio/tests/note.rs", "crates/folio/tests/schema.rs", "+crates/folio/tests/seal.rs", "design-intent/adr/schema.yaml", "+design-intent/anchors/adr-seals.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "+tests/fixtures/adr/seal-ADR-11.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "-tests/fixtures/ceiling/bundle-anchor.py", "tests/fixtures/ceiling/bundle-anchor.txt", "tests/fixtures/ceiling/findings/fabricated-evidence.yaml", "tests/fixtures/ceiling/findings/fail-no-findings.yaml", "tests/fixtures/ceiling/findings/missing-field.yaml", "tests/fixtures/ceiling/findings/pass-coherence.yaml", "tests/fixtures/ceiling/findings/pass-fidelity.yaml", "tests/fixtures/ceiling/findings/pass-readability.yaml", "tests/fixtures/ceiling/findings/pass-reality.yaml", "tests/fixtures/ceiling/findings/stop-refuted.yaml", "tests/fixtures/ceiling/findings/stop-unrefuted.yaml", "tests/fixtures/ceiling/findings/stop-upheld.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "+tests/fixtures/floor_base/design-intent/anchors/adr-seals.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test seal f170_", "cargo nextest run -p folio --test bundle f170_", "cargo nextest run -p folio --bin folio f170_", "cargo nextest run -p folio --bin folio", "cargo nextest run -p folio --test adr", "cargo nextest run -p folio --test bundle", "cargo nextest run -p folio --test findings", "cargo nextest run -p folio --test floor_cases", "cargo nextest run -p folio --test freeze", "cargo nextest run -p folio --test freeze_root", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test modules", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "tests/seal.rs の f170_ の 6 本（発効した記録の 1 字は封と違う / 退役の status と superseded_by だけは封と同じ・後継の行を足すと合格 / 行が無ければ落ち --freeze-adrs が前の行を字のまま末尾に歯の値の行を足し 2 度目は書かない / 違う本文が在れば --freeze-adrs は rc 1 で封は byte で不変、ほかの違反が在っても書かない / 行の在る記録を proposed に戻すと落ち手で直した封は digest で落ちる / folio2 自身の封の行 = 発効した記録で床が合格）が緑、tests/bundle.rs の f170_（配信先に判断の記録の面が在っても 4 観点の束の faces/ に adr- の面が無く、srs.html と sources/adr/ の写しは在る）が緑、seal.rs の単体の f170_（要約値は status と superseded_by だけを除き歯の値と一致）が緑、binary の単体の歯の全部が緑、tests/adr.rs・bundle.rs・findings.rs・floor_cases.rs・freeze.rs・freeze_root.rs・graph.rs・modules.rs・note.rs・schema.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（封の行 = 発効した判断の記録の全部）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数と面の byte は着地の直前の main と同じである"
<!-- contracts:end -->
