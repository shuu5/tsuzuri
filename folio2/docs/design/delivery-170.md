# 設計: 便 170 — 判断の記録の改訂の欄 revises を欄の決まりから削り、発効した判断の記録の本文を床の封で凍らせ、天井の束に判断の記録の面を写さない（封 A・判断の記録 ADR-30 決定 (2)(3)(4)）

- 要件: FR5（床は 3 値で返し、測れないものを合格にしない＝封の一覧が無ければ まだ分からない）・FR19（判断の記録の欄の決まりの file の生成区間を床の定数から導く＝revises を削り封の欄を足す）・FR16（判断の記録の面は正本の欄から組む＝消えた欄の札と行を描かない）・FR17（天井の材料の束＝判断の記録の面を写さない）・FR24（始まりの凍結の前提＝判断の記録の封の欠けを数えない・(i) の 1）。中身は判断の記録 ADR-30 決定 (2)（改訂の欄を削る）・決定 (3)（床の封）・決定 (4)（天井は判断の記録の題と状態だけを読む＝束に面を写さない・席の追記 2026-09-27）の字のとおり。
- 条: P-6.2（封の一覧は folio の出力）/ N-1.1（在る行を書き換えない）/ P-10.1（凍結 anchor = 歯の fixture）/ P-10.3（封の一覧が無ければ まだ分からない）/ P-5.1（要約値は置き場ごとの data）/ P-7.2（退役は status で）/ N-3.1（旗は封を足すだけ）。
- 出所: 持ち主の承認 2026-09-27 12:50 JST（対話面 R-8・逐語「承認する」・構造の直し A）。診断 loop-diag.md の表 3。
- 置き場: 審査の材料は行 `fq` の §1 だけ。write-set 58 本（src 9〔新規 1〕・歯の file 12〔新規 1〕・設計文書 2〔生成区間 1・新規の封の一覧 1〕・fixture 35〔新規 2〕）。新しい dir・消す file は無い。
- 門: **本流 03b9323 の binary で 0（通す）**（write-set 58 本・`通す（印の周 2026-09-27-round51（判定 合格）に…印の後の変更は審査していない）`）。便 169 の前（base 62111b1 と 783cd4a の binary）は 2（`まだ分からない（印が古い（引き金の要約値が違う））`）だった。**便 169 の着地の後は 0 の見込み**（783cd4a に便 169 の差分〔起草役の c169.patch・15:19 の版〕と本便を当てた binary で `通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）`）。
- 前提: **設計の PR（docs/struct30・ADR-30 と本流の revises の行の削除）と便 169（docs/d169・門 C）の着地の後に受け付ける**。受付の先撃ちの ADR-30 の name-unresolved は設計の PR の着地の前だけのもの。
- **base = 設計の PR の枝 783cd4a（数はその写しの実測・参考値・行 D-13）**。作業ツリーは origin/main 62111b1 で、src は 783cd4a と同じ（設計の PR は src を書かない）。受付の時点の main が違えば §1 (h) で数え直す。
- 改訂 b（2026-09-27・独立の検証の blocking B-1 / B-2 と字の直し）: **本流 03b9323（設計の PR #359 と便 169 の着地の後）の上で撃ち直した**＝本便の差分がそのまま当たり、nextest **992 / 992**・clippy 0 警告・床 4 本 rc 0・封 28 行（digest d860d689…・独立の Python と一致）・門 0（通す）・precheck 0。B-1 は (i) の 1 と req の FR24、B-2 は (c) の 5 の 1 場合と変異 M11。

## 1. 設計

### (a) いま起きていること（参考値・783cd4a）

1. **改訂の欄は欄の決まりに在る。** `crates/folio/src/floor_adr.rs` の REVISES（optional の欄）・REVISE_KIND・REVISES_ENTRY・注 revises_note。`adr.rs` の check_revises が形・target の実在・同じ決定の二重を見る。生成区間 `design-intent/adr/schema.yaml` と凍結 anchor `tests/fixtures/schema/adr-region.txt`（138 行）が同じ字を持つ。
2. **面が改訂の欄を描く。** `face_adr.rs` は表紙の札「判断の記録の改訂 n 件」・章 05 の行・受けた改訂の逆向きの行と札（便 148）・機械の面の `<dt>revises</dt>` を出す。
3. **本流の記録は行を持たない。** 設計の PR の後、判断の記録 28 本（ADR-1〜27・30）は全部 accepted・revises の行 0。設計の PR の歯は `tests/adr.rs` の便 101 の 7 本で写しの ADR-13 に行を植えて欄の床を縛り、`tests/face_adr.rs` の実の正本の 3 本は行 0 を見る。
4. **発効した記録の本文の変化を床は見ない。** 凍結 anchor（`design-intent/anchors/`）は憲法の列・id の一覧（ids-v1.24）・入口だけ。gitcheck が anchors/ の yaml の消えと未追跡を落とす。
5. **外の置き場 tsuzuri（cc8e514）。** ADR-7・8・11（どれも accepted）に revises の塊 3 つ（計 15 行）・発効 10 本・床は合格。
6. **束は判断の記録の面を丸ごと写す。** `ceiling_src.rs` の FACE_NAMES が adr に面の名の形 `adr-<数>.html` を持ち、copy_faces が配信先の面を観点ごとの faces/ へ byte のまま写す。folio2 自身の束（参考値）= 4 観点 296 file・10,195,835 byte のうち、判断の記録の面が観点ごとに 28 枚・1,315,761 byte（約半分・決定と改訂の欄の字が全部載る）。reads の adr を題と状態にしても面から読める。FACE_NAMES の写しは生成区間に無い（ceiling.yaml の schema は bundle.contents の 5 語だけ）。
7. **base の歯。** nextest 1003 / 1003・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file。`git grep -n f170_ -- crates` 0 件・行 id `fq` 0 件。

### (b) 直す先

1. **`floor_adr.rs`。** REVISES（欄の一覧から）・REVISE_KIND・REVISES_ENTRY・床の木の revise_kind・revises_entry・revises_note を削る。封の定数 SEAL_FILE（値 adr-seals.yaml）・SEAL_KIND（値 adr-seals）・SEAL_OUTSIDE（値 status と superseded_by の 2 語）と、床の木の `seal: {file, kind, outside}` と `seal_note`（limits_note の前）を足す。revises を持つ記録は欄の閉じた一覧の外＝`[adr] ADR-n: 未知の欄（N-3）: revises` で落ちる。
2. **`adr.rs`。** check_revises と呼び出しを削る。
3. **新しい `seal.rs`（封）。**
   - 置き場 = anchor.dir（`anchors/`）の下の `adr-seals.yaml`。中身 = 頭の注 1 行（置き場の名・手で直さない）・`kind: adr-seals`・`digest_algo: sha256-json-1`・`outside: [status, superseded_by]`・`rows`（`{id, sum}` を足した順）・`digest`（ids の anchor と同じ anchor::digest_of）。
   - 要約値 = 記録の木から outside の欄を除いた表の yaml::canonical の sha256（凍結 anchor の digest と同じ正規化）。
   - check_seals（`check.rs` が ids の検査の後に呼ぶ）: 発効した記録（EFFECTIVE_STATUS）ごとに、行と違えば違反 `[adr] <id>: 発効した判断の記録の本文が封（anchors/adr-seals.yaml）の行と違う（…判断を変えるなら新しい判断の記録を立てる）`、行が無ければ違反 `[adr] <id>: 発効しているのに封の行が無い（folio check --freeze-adrs で封を足し、commit する）`。行が在るのに発効した記録が無い・同じ id の行が 2 つ・digest が中身と合わない（行を照らさない）・symlink も違反、形が違えば まだ分からない。**封の一覧の file が無く発効した記録が在れば まだ分からない 1 行**（行の欠けを 1 本ずつ違反にしない・P-10.3）。行と file の欠けは凍結の旗（`--freeze-anchor`・`--freeze-ids`・`--freeze-start`・`--freeze-adrs`）の前提の検査に数えない（素の床と `--emit-amends` は数える）。
   - freeze（`--freeze-adrs`）: 本文が行と違う記録が 1 つでも在れば断る（終了コード 1・`anchors/adr-seals.yaml の在る行（ADR-n）は書き換えない（封は足すだけ・N-1.1…）`・書かない）。全検査が 0 違反で まだ分からない も無いときだけ、在る行を字のまま残し、欠けた行を id の数の順で末尾に足して書く（`封を足した: …（足した行 …・在る行 n は変えない）・版管理に commit する`）。足す行が無ければ書かない。
4. **口。** `check.rs`（Materials に seals）・`phase.rs`（Flag::FreezeAdrs）・`freeze.rs`（after の腕 1 つ）・`main.rs`（`mod seal`・旗 `--freeze-adrs`〔ほかの凍結の旗と `--emit-amends` と排他〕）。
5. **`face_adr.rs`。** 判断の記録の改訂の札・h3 と行・受けた改訂の逆向きの行と札（RevisedBy と revised_by）・機械の面の revises の数・表 REVISE と名札の定数・単体の歯 2 本（f137 / f148）を削る。条文の改訂（amends）の札と行は残し、章 05 の空の断りは「条文の改訂なし」の 1 段落。
6. **`ceiling_src.rs`（席の追記）。** FACE_NAMES の adr を `None`（束に面を写さない）にし、表の注と単体の歯（adr を面の無い側へ）を直す。判断の記録の面そのもの（folio build・folio face の出力）と、束の sources/adr/ の正本の写しは変えない。生成区間は変わらない。folio2 自身の束は 184 file・4,930,715 byte（−51.6%・観点ごとに −1,316,280 byte＝面 28 枚と生成区間の縮み）。
7. **生成物（手で書かない）。** `design-intent/adr/schema.yaml` の生成区間 = `folio schema --dir design-intent --write` の出力（131 行・25174 byte・sha256 f7dfcd85…＝凍結 anchor adr-region.txt と byte 一致）。`design-intent/anchors/adr-seals.yaml` = `folio check --dir design-intent --freeze-adrs` の出力（本流 03b9323 で 28 行 ADR-1〜27・30・digest d860d689…・独立の Python seal-170.py と 28 行の要約値と digest が一致）。**封の一覧の中身は、受付の本流の本文で作業者が命令を撃って書いたものにする（起草の差分の封の一覧は写さない＝783cd4a の差分では ADR-30 の行が今の本文と違う）。** 書いた後に commit する（gitcheck が未追跡の anchor を落とす）。
8. **変えないもの。** 判断の記録の本文・amends / amended_by / supersedes / superseded_by の床・憲法と ids と入口の anchor と既存の凍結の旗の前提（封の欠けを数えないことだけ足す）・天井の TRIGGER_ADR_FIELDS（revises の字を含む・生成区間 ceiling.yaml の trigger も＝便 169 の (g) 3 の撤去に任せる）・`folio init` の骨格（骨格の ADR-1 は proposed＝封は要らない）。

### (c) 歯（f170_・base で 0 件）

土台は `tests/fixtures/floor_base/design-intent/` の写し（git init と 1 commit）。封の一覧の fixture `tests/fixtures/floor_base/design-intent/anchors/adr-seals.yaml`（10 行）は命令の出力を独立の Python で照らした凍結 anchor、新しい発効した記録は最小の手書き `tests/fixtures/adr/seal-ADR-11.yaml`（14 行）で、その要約値も Python で測った字を歯が持つ。

1. **f170_a_record_with_revises_is_an_unknown_field（`tests/seal.rs`）。** 写しの ADR-2 に revises の 1 行 → rc 1・違反 2 行 `[adr] ADR-2: 未知の欄（N-3）: revises` と封と違う。**base は 0 ＝RED。**
2. **f170_one_char_in_an_effective_record_is_caught（同）。** ADR-3 の決定に「。」を 1 字 → rc 1・違反は封と違う の 1 行だけ。**base は 0 ＝RED。**
3. **f170_retiring_changes_only_status_and_superseded_by（同）。** 新しい ADR-11（supersedes: ADR-2）を置き、ADR-2 を retired + superseded_by: ADR-11 → 違反は ADR-11 の行が無い の 1 行だけ（ADR-2 は封と同じ）→ `--freeze-adrs` で前の 10 行が字のまま・11 行目が Python の値 → 素の床は合格。**base は旗が無い＝RED。**
4. **f170_a_missing_row_is_caught_and_the_freeze_appends_it（同）。** ADR-11 だけ足す → 行が無い の 1 行 → `--freeze-adrs` は rc 0・`封を足した: `・前の 10 行が字のまま・末尾が `(ADR-11, <Python の値>)` → 合格 → 2 度目は `足す封の行は無い` で 1 byte も書かない。**base は 0 ＝RED。**
5. **f170_the_freeze_does_not_rewrite_a_row（同）。** ADR-3 を変えて `--freeze-adrs` → rc 1・`anchors/adr-seals.yaml の在る行（ADR-3）は書き換えない`・封の一覧は byte で不変。改訂 b（検証役の歯の案 tooth-candidate-v4.patch）: 別の写しで新しい ADR-11 に未知の欄 `foo: 1` を置き `--freeze-adrs` → rc 1・`凍結しない`・封の一覧は byte で不変（足す行が在っても、ほかの違反が在れば書かない＝全検査が 0 違反で まだ分からない も無いときだけ書く）。**base は旗が無い＝RED。**
6. **f170_a_sealed_record_stays_effective_and_the_list_is_not_hand_edited（同）。** ADR-4 を proposed に戻す → 行が在るのに発効した記録が無い の 1 行。封の一覧の要約値の 1 桁を手で変える → digest が中身と合わない の 1 行。**base は 0 ＝RED。**
7. **f170_folio2_itself_is_sealed（同）。** 本流の `design-intent/` の封の行の id の集合 = 発効した記録の file の集合・写しの素の床は合格・`--freeze-adrs` は `足す封の行は無い`。**base は file が無い＝RED。**
8. **f170_the_body_sum_skips_only_status_and_superseded_by（`seal.rs` の単体）。** SEAL_OUTSIDE が 2 欄・status と superseded_by だけ違う記録は同じ要約値・題の 1 字で違う・要約値が Python の値 aca2869f… と一致。**base は src に無い（0 件で終了コード 4）。**
9. **f170_the_adr_face_draws_no_revision_of_records（`tests/face_adr.rs`）。** 凍結 fixture の写しと、正本に revises の 1 行を足した写しの面の両方で、札は「条文の改訂 0 件」の 1 つ・断りは「条文の改訂なし」の 1 段落・「ほかの判断の記録による改訂」「`<dt>revises</dt>`」「を狭める」が無い・機械の面は `<dt>amends</dt><dd>0</dd>`。**base は札「判断の記録の改訂」と revises の数を出す＝RED。**
10. **f170_the_bundle_copies_no_adr_face（`tests/bundle.rs`）。** 凍結の土台 `tests/fixtures/ceiling/bundle/`（配信先に adr-1.html・adr-2.html が在る）から 4 観点の束を組むと、どの faces/ にも `faces/adr-` が無く、`faces/srs.html` と `sources/adr/ADR-1.yaml` は在る。**base は faces/adr-1.html と adr-2.html を写す＝RED。**

### (d) 採らなかった形

1. **封を記録ごとの欄に持つ。** 本文と同じ手で直せるので守らない（P-6.3）。
2. **要約値を床の定数の表に焼く（ROOT_DIGESTS の形）。** 置き場ごとに folio2 の便が要る（台帳 f2-648.233 と同じ穴）。
3. **版ごとの file（憲法の列の形）。** 足すだけなら 1 file の末尾への追記で足り、消えは gitcheck が落とす。
4. **新しい命令（folio seal）。** 既存の凍結の旗と同じ「全検査が 0 違反のときだけ書く」前提なので check の旗にした。
5. **file が無いのを違反にする。** 外の置き場が folio を上げた時点で行の数だけ違反が並ぶ。ids と憲法の anchor が無いときと同じ まだ分からない に揃えた（(i) の 1）。
6. **束の面を読む欄に合わせて切る（判断の記録の面から題と状態のほかの章を落とす）。** 面の HTML を切る口が要り、面の字を切った写しは面と byte で違う。面を写さず、題と状態は sources/adr/ の絞った写しで渡す。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけ（src と design-intent）を 783cd4a に当てると既存 105 本が落ちる**（1002 本中・本文は sim0-nextest.log・うち束の追記の分 36 本）。fixture の直しまで当てると 45 本（sim0f-nextest.log）。扱いは次の 3 つだけ。
   - fixture の直しで緑になる 60 本（歯の本文は不変）: 欄の決まりの写し 17 本（`tests/fixtures/*/adr/schema.yaml`・floor_base を含む）・floor_base の封の一覧・adr-region.txt・node-digest-anchor.txt・面の凍結 fixture 2 本（expected-adr.html・expected-site-adr-2.html）・束の凍結 anchor（独立の script `tests/fixtures/ceiling/bundle-anchor.py` の FACES から adr を外し、その出力で bundle-anchor.txt を書き直す）・所見の fixture 10 本（起動の記録の bundle の要約値を新しい anchor の字に）。
   - 消す 14 本（欄と面が無くなる）: `tests/adr.rs` の便 101 の区間の f101_ 7 本と道具（REVISES_ROW・REVISES_HEAD・assert_adr13_violation・Work::plant_revises・Work::mutate_file）、`tests/face_adr.rs` の f137_ 4 本（revises_rows・a_record_without_revisions・unknown_when_a_revise_kind・real_sources_draw_every_revises_row）と便 148 の区間（f148_ 3 本が落ち、行 0 を見る 2 本も区間ごと消す）。
   - 歯の本文を直す 31 本: 発効した記録を写しの上で変える歯は封の違反 1 行を確かめて外す（`tests/adr.rs` の図と f92_ の 9 本・`tests/note.rs` の f161・`tests/freeze_root.rs` の f158）。f121 の始まりの凍結は `--freeze-adrs` と commit の段を足す。`tests/freeze.rs` の合成の改訂 2 本は上書きの ADR-5 を新しい ADR-11 にして `--freeze-adrs` を撃つ。`tests/floor_cases.rs` の runner は folio を撃つ前ごとに写しの封の一覧を今の発効した記録で組み直す（case は土台の発効した記録を書き換えて改訂の経路を作る＝この fixture は封を測らない・floor_cases.yaml は不変）。`tests/face_adr.rs` の f137_amends と census と札の道具は条文の改訂だけに。定数 3 か所（`tests/schema.rs` の生成区間の行数・byte・sha256 で 6 本・`tests/graph.rs` の F99_ANCHOR_SHA256・`tests/modules.rs` の層の表に seal）。`tests/bundle.rs` の 4 本は凍結 anchor の期待（file の一覧から faces/adr- の 2 行・byte・要約値を新しい anchor の字に）と実の正本の歯（adr の面 0 枚）、`tests/findings.rs` の 3 本は起動の記録の要約値の定数 2 か所と反証の束の要約値（歯の定数 5 つの連結の sha256 を独立の Python で測り直した値・連結の byte 2,338 は不変）。
2. **便を当てた写し。** nextest **995 / 995**（1003 − 18 + 10）・clippy 0 警告・床 4 本 rc 0・build 35 file（base と同じ数・判断の記録の面 28 枚が違う・面の revises の札と `<dt>revises</dt>` は 0）。本流 03b9323 + 本便（改訂 b の歯を含む・封は命令で書き直し）= nextest **992 / 992**（1000 − 18 + 10）。783cd4a + 便 169 の差分（c169.patch・15:19 の版）+ 本便 = nextest **992 / 992**・clippy 0・床 4 本 rc 0（main.rs は別の塊で当たる）。
3. **RED。** 歯だけ（r170-teeth.patch）を 783cd4a に当てると `--test seal f170_` 7 本・`--test face_adr f170_` 1 本・`--test bundle f170_` 1 本が落ち（落ち方は (c)）、`--bin folio f170_` は 0 件で終了コード 4。
4. **突然変異（src を 1 通りずつ変え、verify の 1〜4 を撃つ）。** 11 通りとも f170_ が落ちる（M11 は改訂 b の歯の 1 場合で落ちる・検証役の V4 は改訂 a までの歯を全部生き残った）。

| 変異 | 落ちる f170_ |
| --- | --- |
| M1 要約値が status と superseded_by も含む | 8 本（単体を含む） |
| M2 本文が行と違っても封を足す旗が断らない | does_not_rewrite |
| M3 file が在るのに行の欠けを違反にしない | missing_row・retiring |
| M4 欄の閉じた一覧に revises を残す | 6 本 |
| M5 封の一覧の digest を照らさない | stays_effective |
| M6 足す行を在る行の前に書く | missing_row・retiring |
| M7 行が在るのに発効を戻した記録を落とさない | stays_effective |
| M8 面の機械の欄に revises の数を戻す | draws_no_revision |
| M9 凍結の旗でも行の欠けを数える | missing_row・retiring |
| M10 束に判断の記録の面を写す（前の形） | copies_no_adr_face |
| M11 封を足す旗がほかの違反・まだ分からない を見ない（検証役の V4） | does_not_rewrite |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新規（`+`）5 本・縮む（`-`・行数）27 本（src 3・`tests/adr.rs` 508 → 373・`tests/face_adr.rs` 1792 → 1416・生成区間・欄の決まりの写し 17 本・面の fixture 2 本・adr-region.txt・bundle-anchor.py 292 → 291）。ほかは増えるか同じ。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）。python と awk の 2 実装が 9 本とも一致した。

| file | base | base の余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/adr.rs` | 1049 | 451 | 993（−56） | 507 |
| `crates/folio/src/ceiling_src.rs` | 660 | 840 | 660（0） | 840 |
| `crates/folio/src/check.rs` | 1045 | 455 | 1053（+8） | 447 |
| `crates/folio/src/face_adr.rs` | 1100 | 400 | 955（−145） | 545 |
| `crates/folio/src/floor_adr.rs` | 520 | 980 | 494（−26） | 1006 |
| `crates/folio/src/freeze.rs` | 514 | 986 | 518（+4） | 982 |
| `crates/folio/src/main.rs` | 674 | 826 | 682（+8） | 818 |
| `crates/folio/src/phase.rs` | 71 | 1429 | 73（+2） | 1427 |
| `crates/folio/src/seal.rs`（新規） | 0 | 1500 | 317 | 1183 |

3. **size は M。** 新しい src 317 行と歯 381 行・既存の歯 31 本の直し。束の追記は src 1 行と注 3 行。余地の最小は base の face_adr.rs の 400（≥ 300）。便 169 は main.rs を 674 のまま（0）にする。新しい字は rustfmt に合わせた（repo の既存の差は触らない）。
4. **verify 14 行 = done の 14 の塊。** 1 `--test seal f170_`（(c) の 1〜7）・2 `--test face_adr f170_`（(c) の 9）・3 `--test bundle f170_`（(c) の 10）・4 `--bin folio f170_`（(c) の 8）・5 `--test adr`（13 本）・6 `--test face_adr`（40 本）・7 `--test bundle`（20 本）・8 `--test findings`（39 本）・9 `--test floor_cases`・10 `--test freeze`・11 `--test freeze_root`（14 本）・12 `--test note`（39 本）・13 `--test schema`（21 本）・14 clippy 0 警告。base では 1〜4 が 0 件で終了コード 4、5〜13 は緑（adr 20・face_adr 48・bundle 19 本）。`--test` の 10 本はどれも write-set に在り、f170_ の単体の歯を持つ `seal.rs` も write-set に在る。

### (g) 門・受付・並行の便・外の置き場

1. **門。** 冒頭のとおり 今は 2・便 169 の後は 0 の見込み。受付の時点で撃ち直す。
2. **並行の便。** 設計の PR（783cd4a）は `tests/adr.rs`・`tests/face_adr.rs`・`tests/check.rs` と判断の記録の revises の行を書く＝本便の `tests/adr.rs`・`tests/face_adr.rs` と重なるので、本便の数は設計の PR の形の上で測った（先に着地）。便 169 とは **`main.rs` が重なる**（便 169 は gate::run の 1 行・本便は mod と旗と freeze::after の引数＝別の塊・c169.patch の後に本便の差分が当たることを実測）。`tests/findings.rs` は便 169 の verify の scope（本文は不変）で、本便は要約値の定数 3 か所を書く。便 169 の f169_ の歯が使う所見の fixture（stop-upheld.yaml ほか）の要約値の行も本便が書くが、重ねた写しで全部緑（(e) の 2）。仕様の順（設計の PR → 169 → 170）で逐次に受け付け、受付の時点の main で数え直す。便 164（docs/d164）は差分が delivery-164.md だけで、seal・--freeze-adrs・封の名はぶつからない。
3. **外の置き場（tsuzuri）の手順（行を消してから封を足す）。** 本便の binary を入れたら: ① 発効した記録 ADR-7・8・11 の revises の塊（計 15 行）を消す（凍結の前の最後の書き換え・決定 (2) と同じ）→ ② `folio schema --dir design-intent --write`（生成区間から revises を削り seal を足す）→ ③ 素の床は まだ分からない（封の一覧が無い）→ ④ `folio check --dir design-intent --freeze-adrs`（10 行を書く）→ ⑤ commit → ⑥ 素の床は合格。tsuzuri の clone で撃って実測（①の前の本便の binary は 不合格・違反 7〔欄の決まりの写し 4・未知の欄 revises 3〕・まだ分からない 1）。
4. **init の骨格。** base と本便の binary で `folio init` の骨格の素の床は同じ字（まだ分からない・違反 0・まだ分からない 2＝憲法の列と id の一覧が無い）。骨格の ADR-1 は proposed で封は要らない。持ち主が発効させた後は `--freeze-start` → commit → `--freeze-adrs` → commit で合格（f121 の歯）。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d170-draft.md`、差分と script は同じ dir の d170-scripts。1 setup-sim-s30-170.sh（base の写し）と suite-170.sh（build・nextest・clippy・床 4 本）。2 sim-170.sh（本便の全部を当て、生成物は folio の命令で書く）→ commit → suite-170.sh・seal-170.py（独立の Python の封）。3 red-170.sh（r170-teeth.patch で RED）。4 mut-170.py（突然変異・元へ戻す）。5 lines-170.sh（python と awk の余地）。6 sim0-170.sh（実装だけで落ちる歯）。7 gate-170.sh・init-build-170.sh・tsuzuri-170.sh・bundle-size-170.sh（束の大きさ）。束の追記は sim-170.sh の中の apply-170-bundle.py。8 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-170.md#fq`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** 決定 (3) は「発効した判断の記録で封の行が無いものも床が落とす」。本便は封の一覧が在るときは行ごとに違反、file そのものが無いときは まだ分からない 1 行にした（FR5 の「測れないものを合格にしない」・ids と憲法の anchor が無いときと同じ形）。どちらも合格にはならない。違反にそろえるかは席が決める（変えるなら (b) の 3 の 1 行と f121 の歯 1 本）。FR16 の「改訂の一覧」は条文の改訂（amends）として残る。FR17 の束の面は、要件書 第 1.50 版（枝 docs/srs150）で判断の記録の面を除く字になる。**席の裁定（改訂 b・B-1）: 判断の記録の封の欠け（行の欠けと封の一覧の file の無さ・ADR-30 決定 (3)）は、凍結の旗（`--freeze-start`・`--freeze-anchor`・`--freeze-ids`）の前提の検査に数えない。** 要件 FR24（始まりの凍結は 2 つの基準の不在を除く全検査が違反 0 で まだ分からない も無いときだけ書く）と受入基準 AC21（ほかの検査に まだ分からない が 1 つ在る写しでは非 0）の今の字と食い違うので名指す。除かないと、封の無い置き場で `--freeze-start` が断り、条の改訂の流れでは `--freeze-adrs` を先に撃てず（`[A-2] 憲法の版と最新 anchor の版が違う` で断る）、旗どうしが断り合う（検証役の変異 V12 と order.log）。FR24 の規範文と AC21 の判定の文は、席が要件書 第 1.50 版で同じ句（2 つの基準の不在と判断の記録の封の欠けを除く）に合わせる。
2. **言えないこと。** 封は版管理の上の生成物で、暗号の署名ではない。手で行を消して digest を計算し直す改ざんは床が見ない（版管理の差分に残り、file ごと消すのは gitcheck が落とす）。本文を変えたのに誰も `--freeze-adrs` を撃たない間は、違反が残り続ける（断るのが正しい向き）。天井の TRIGGER_ADR_FIELDS と生成区間 ceiling.yaml の trigger には revises の字が残る（読む欄が無いだけ・撤去は便 169 の後続）。
3. **撤退条件。** (1) (e) の 1 の 105 本のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 受付の時点の main の判断の記録の数・状態が 03b9323（28 本・全部 accepted）と違えば、封の行の数を (h) で数え直す（封の中身は命令の出力なので字では縛らない）。(3) 設計の PR か便 169 が未着地なら受け付けない。(4) 着地の後の main で `folio build` の file 数が着地の直前と違うか、判断の記録の面のほかの面が byte で変われば止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜7・歯（f170_ の 10 本と (e) の直し）・fixture 35 本。
- 入れない: 判断の記録の本文・天井の TRIGGER_* と ceiling.yaml・門と印・init の骨格・外の置き場への書き込み・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| floor | 欄の決まり | floor_adr.rs の封の定数と revises の削除・生成区間 |
| seal | 封 | seal.rs の check_seals・freeze・封の一覧の形 |
| face | 面 | face_adr.rs の revises の札と行の削除 |
| bundle | 束 | ceiling_src.rs の FACE_NAMES の adr |
| teeth | 歯 | f170_ の 10 本・fixture の封の一覧と seal-ADR-11.yaml・束の凍結 anchor |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。前提は設計の PR と便 169 の着地。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の素の床が合格（封 28 行）・tsuzuri へ (g) の 3 を渡す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fq"
title = "構造の直し A（持ち主の承認 2026-09-27 12:50 JST・判断の記録 ADR-30 決定 (2)(3)(4)）: floor_adr.rs から改訂の欄 revises の定数と注を削り、adr.rs の check_revises と face_adr.rs の revises の札と行を削る（revises を持つ記録は未知の欄で落ちる）。新しい seal.rs が封の一覧 anchors/adr-seals.yaml（行 = id と status・superseded_by を除く本文の要約値・digest）を読み、発効した記録の本文が行と違う・行が無い・行が在るのに発効した記録が無い・digest が合わないを違反に、file が無ければ まだ分からない にする。folio check --freeze-adrs は全検査が 0 違反のときだけ欠けた行を末尾に足し、違う本文が在れば断る。封の欠けは凍結の旗の前提に数えない（席の裁定・FR24）。要約値は source に焼かない。生成区間と本流の封の一覧（28 行）は folio の命令の出力。ceiling_src.rs の FACE_NAMES の adr を None にし、天井の束に判断の記録の面を写さない（面そのものは変えない・束の凍結 anchor は独立の script の出力）。設計の PR（docs/struct30）と便 169 の着地の後に受け付ける。base = 本流 03b9323・受付の時点の main で数え直す"
req = ["FR5", "FR19", "FR16", "FR17", "FR24"]
section = "1"
write-set = ["-crates/folio/src/adr.rs", "crates/folio/src/ceiling_src.rs", "crates/folio/src/check.rs", "-crates/folio/src/face_adr.rs", "-crates/folio/src/floor_adr.rs", "crates/folio/src/freeze.rs", "crates/folio/src/main.rs", "crates/folio/src/phase.rs", "+crates/folio/src/seal.rs", "-crates/folio/tests/adr.rs", "crates/folio/tests/bundle.rs", "-crates/folio/tests/face_adr.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/floor_cases.rs", "crates/folio/tests/freeze.rs", "crates/folio/tests/freeze_root.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "crates/folio/tests/note.rs", "crates/folio/tests/schema.rs", "+crates/folio/tests/seal.rs", "-design-intent/adr/schema.yaml", "+design-intent/anchors/adr-seals.yaml", "-tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "-tests/fixtures/adr/schema-drift/adr/schema.yaml", "+tests/fixtures/adr/seal-ADR-11.yaml", "-tests/fixtures/adr/two-adopted/adr/schema.yaml", "-tests/fixtures/anchor/no-anchor/adr/schema.yaml", "-tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "-tests/fixtures/ceiling/bundle-anchor.py", "tests/fixtures/ceiling/bundle-anchor.txt", "tests/fixtures/ceiling/findings/fabricated-evidence.yaml", "tests/fixtures/ceiling/findings/fail-no-findings.yaml", "tests/fixtures/ceiling/findings/missing-field.yaml", "tests/fixtures/ceiling/findings/pass-coherence.yaml", "tests/fixtures/ceiling/findings/pass-fidelity.yaml", "tests/fixtures/ceiling/findings/pass-readability.yaml", "tests/fixtures/ceiling/findings/pass-reality.yaml", "tests/fixtures/ceiling/findings/stop-refuted.yaml", "tests/fixtures/ceiling/findings/stop-unrefuted.yaml", "tests/fixtures/ceiling/findings/stop-upheld.yaml", "-tests/fixtures/check/dup-key/adr/schema.yaml", "-tests/fixtures/check/empty-field/adr/schema.yaml", "-tests/fixtures/check/unknown-section/adr/schema.yaml", "-tests/fixtures/face/expected-adr.html", "-tests/fixtures/face/expected-site-adr-2.html", "-tests/fixtures/floor_base/design-intent/adr/schema.yaml", "+tests/fixtures/floor_base/design-intent/anchors/adr-seals.yaml", "-tests/fixtures/link/adr-id-missing/adr/schema.yaml", "-tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "-tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "-tests/fixtures/refs/bad-counts/adr/schema.yaml", "-tests/fixtures/refs/dangling-id/adr/schema.yaml", "-tests/fixtures/refs/orphan-rule/adr/schema.yaml", "-tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "-tests/fixtures/vocab/exemptions/adr/schema.yaml", "-tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test seal f170_", "cargo nextest run -p folio --test face_adr f170_", "cargo nextest run -p folio --test bundle f170_", "cargo nextest run -p folio --bin folio f170_", "cargo nextest run -p folio --test adr", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test bundle", "cargo nextest run -p folio --test findings", "cargo nextest run -p folio --test floor_cases", "cargo nextest run -p folio --test freeze", "cargo nextest run -p folio --test freeze_root", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "tests/seal.rs の f170_ の 7 本（revises は未知の欄 / 発効した記録の 1 字は封と違う / 退役の status と superseded_by だけは封と同じ・後継の行を足すと合格 / 行が無ければ落ち --freeze-adrs が前の行を字のまま末尾に Python の値の行を足し 2 度目は書かない / 違う本文が在れば --freeze-adrs は rc 1 で封は byte で不変、ほかの違反が在っても書かない / 行の在る記録を proposed に戻すと落ち手で直した封は digest で落ちる / folio2 自身の封の行 = 発効した記録で床が合格）が緑、tests/face_adr.rs の f170_（面に判断の記録の改訂の札・行・逆向きの行・機械の面の revises が無い）が緑、tests/bundle.rs の f170_（配信先に判断の記録の面が在っても 4 観点の束の faces/ に adr- の面が無く、srs.html と sources/adr/ の写しは在る）が緑、seal.rs の単体の f170_（要約値は status と superseded_by だけを除き Python の値と一致）が緑、tests/adr.rs・face_adr.rs・bundle.rs・findings.rs・floor_cases.rs・freeze.rs・freeze_root.rs・note.rs・schema.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（封の行 = 発効した判断の記録の全部）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
