# 設計: 便 176 — 止める の場所の頭を、設計ノートと判断の記録の置き場で meta.id からも 1 file に解く（印の場所が dir 全体へ広がる穴）

- 要件: FR20（天井の印と門・要件書 第 1.50 版）。FR20 の規範文は、印に記す 止める の場所の file を「判断の記録と設計ノートは場所の頭の id が指す 1 file・頭の id が file に解けないときはその文書の置き場の全体」と書く。文書の id は各 file の meta.id（meta の欄の id）なので、頭を meta.id でも解くのは今の字の中の直しで、要件の字は変えない。契約表の行の req は FR20 の 1 つ（本流に在る id）。
- 条: P-4.1 / P-4.2（読めないものを「異常なし」として狭めない＝解けなければ広い側）/ P-10.1（凍結 anchor＝歯の中の手書きの最小の字）/ N-3.1（例外の口を足さない＝解けないときの広い側を data や旗で狭められない）。
- 出所: tsuzuri（外の置き場）の写しで天井を 1 周した検証（2026-09-27 18:4x・記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/t3v-report.md`）。整合 F-4（反証 支持）の印の場所が `file: design-note/`（dir 全体）になり、門が design-note/ の全 file を書く便を止めた。依頼は席から起草役へ（2026-09-27）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fw` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 2 本（src 1・歯の file 1）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象外。本便は設計文書の正本（`design-intent/` の下）を書き換えない。write-set 2 本を本流の binary（本流の作業ツリーの `target/debug/folio`・1ca1ae1 の組み立て）で `folio ceiling --gate` に渡すと **0（通す・`通す（設計文書の正本を書き換えない便）`）**。
- 前提: **base = main 1ca1ae1。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 実装の見本: origin の枝 `impl/d176`（commit 2510cee・親は e06b8d3〔実装と歯 3 本〕・その親は 1ca1ae1）が本便の後の中身。2510cee は改訂 a（独立の検証役の B-1）の単体の歯 1 本を積んだもの。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 2510cee -- <write-set の file>`）。write-set の外は変えない。
- 並行の便との重なり: 起草の時点で便 174（行 fu・生成区間の id）が別の起草役の手で起草中（契約は未 push）で、その模擬が書き換えるのは床の定数の file（floor_note.rs ほか）と生成区間で、本便の 2 本と重ならない見込み。行 fv（便 175）は便 174 の割り増しに空けてある。受付の時点で precheck が重なりを見る。

## 1. 設計

### (a) いま起きていること（base 1ca1ae1 の実測・参考値）

1. **印の場所の解き方（便 169）。** `crates/folio/src/stamp.rs` の関数 stop_file は、止める の所見の場所 `{doc, at}` の doc を天井の正本の documents で file に解き、file 形（末尾が / でない）はその file、dir 形（`adr/`・`design-note/`）は at の頭（最初の `.` の前）の `<dir><頭>.yaml` が file として在ればそれ、無いか頭が空・`.` 始まり・区切りを含むか symlink なら dir そのもの（広い側）を返す。頭を照らすのは file 名だけで、meta.id は見ない。
2. **tsuzuri の周で dir 全体へ広がった（base の binary で写しに撃ち直した）。** tsuzuri の設計ノートの欄の決まりは file 名 `design-note/schema.yaml`・meta.id `design-note-schema`（流れの形 `meta: {id: design-note-schema, …}`）。整合 F-4 の場所は `{doc: design-note, at: design-note-schema.schema.figures.retry_rules_row}` で、頭 `design-note-schema` の `design-note/design-note-schema.yaml` は無い。t3v の写し（design-intent と周 t3v-round1 の束）を起草役の scratch に写し、base と本便の後の binary でそれぞれ `folio ceiling --stamp` → `--gate` を 1 本ずつ撃った結果は次のとおり（印は F-4 の行のほか byte で同じ・base 10001 byte → 本便の後 10012 byte）。

| write-set（1 本ずつ） | base の門 | 本便の後の門 |
| --- | --- | --- |
| design-intent/design-note/schema.yaml | 1（F-4・場所 design-note/） | 1（F-4・場所 design-note/schema.yaml） |
| design-intent/design-note/surface-base.yaml | 1（F-4・場所 design-note/） | **0（通す）** |
| design-intent/design-note/bakeoff-surface.yaml | 1（F-4・場所 design-note/） | **0（通す）** |
| design-intent/design-note/surface.yaml | 1（F-4 が先に当たる） | 1（実態 F-3・その file の支持の 止める） |
| design-intent/design-note/surface-board.yaml | 1（整合 F-3） | 1（整合 F-3） |
| design-intent/srs.yaml・constitution.yaml | 1・1 | 1・1（変わらない） |
| vocabulary.yaml・adr/ADR-8.yaml・crates/tsuzuri-surface/src/view.rs | 0・0・0 | 0・0・0（変わらない） |

3. **folio2 の本流も同じ形の file を持つ。** `design-intent/design-note/schema.yaml` の meta.id は `folio2-design-note-schema`、`design-intent/adr/schema.yaml` の meta.id は `folio2-adr-schema` で、どちらも file 名の stem（拡張子を除いた名）と違う。設計ノートの本体は床（`note.rs`）が meta.id = file 名の stem を課し、判断の記録の本体は最上位の id = file 名を床（`adr.rs`）が課すので、頭の字が file 名と違いうるのは欄の決まりの schema.yaml だけである。本流の印（周 2026-09-27-round51）は `refutes: []` で、本便で変わらない。
4. **過去の周の場所は 1 件も変わらない（独立の 2 通りの解き）。** folio の code を呼ばない python の script（heads-176.py・base の規則と本便の規則の写し）で、folio2 の 52 周の所見 file 209 本の dir 形の場所 312 件（頭 35 種・`ADR-n`・`schema`・`example`・`figures`・`sections`・`meta`・`plain` ほか）を本流の design-intent に当てると、2 通りの解きの違いは **0 件**。t3v の写しの周では 16 件のうち 1 件（F-4・`design-note/` → `design-note/schema.yaml`）だけが違う。
5. **base の歯（参考値）。** workspace の nextest 992 / 992・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file。`crates/folio/tests/stamp.rs` 16 本・binary の単体の歯 148 本。`git grep -n f176_ -- crates` は 0 件・行 id `fw` は 0 件。

### (b) 直す先 — stop_file に meta.id の解きを 1 段足す

1. **`stamp.rs` の stop_file。** 順は次のとおり。(i) file 形はその file（今どおり）。(ii) 頭が空・`.` 始まり・区切り（`/`・`\`・NUL）を含むか、`<dir><頭>.yaml` が symlink なら dir（今どおり）。(iii) `<dir><頭>.yaml` が file として在ればそれ（今どおり）。(iv) 無ければ新しい関数 meta_file に問い、解けた file か、解けなければ dir（広い側）。doc が文書の一覧に無ければ Err（今どおり・印を組まない）。
2. **`stamp.rs` の meta_file（新しい関数）。** 置き場の dir の直下の `.yaml`（下の dir〔retired/ など〕と symlink は見ない・置き場の dir が symlink なら解かない）を全部読み、最上位の meta.id が字として頭と同じ file を数える。**ちょうど 1 つ**ならその file（`<dir><名>` の形）、0 か 2 つ以上なら解かない。読めない `.yaml` が 1 つでも在れば（読めない・UTF-8 でない・parse できない・重複キー）数えずに解かない＝広い側（読み違いで狭めない・P-4.1）。数えるだけなので dir の並びに依らない（決定的）。
3. **頭の注。** stamp.rs の file の頭の注に便 176 の 1 項（2 行）を足し、stop_file の注を (1) の順に直す。
4. **変えないもの。** 門（`gate.rs`・印の file を読んで write-set と照らすだけ・(a) の 2 の表を本便の binary の門がそのまま出した）・所見の数え（`findings.rs`）・印の欄の並びと行の形・file 形の文書の解き・名の規則が先（`<dir><頭>.yaml` が在れば meta.id を見ない）・要件書と判断の記録と設計ノートの字・`folio init` の骨格・生成区間。folio2 自身の床の結果と `folio build` の出力（35 file）は base と byte で同じ（起草役の実測）。

### (c) 歯（f176_・base で 0 件）

1. **f176_a_stop_on_the_schema_meta_id_is_its_file（`crates/folio/tests/stamp.rs`・binary 経由）。** 既存の周の道具 Round::passing（凍結 fixture の束 `tests/fixtures/ceiling/bundle/` を一時 dir へ写して組む・設計ノートは full.yaml〔meta.id full〕の 1 本）の写しの design-note/ に、tsuzuri の形の最小の字 2 つを足す: `schema.yaml` = `meta: {id: design-note-schema, version: 1}`、`retired/schema-v0.yaml` = 同じ meta.id（下の dir は数えない）。観点 fidelity の所見を既存の fixture stop-upheld.yaml（支持の 止める 1 件）の場所だけ `{doc: design-note, at: design-note-schema.schema.figures.retry_rules_row}`（tsuzuri の F-4 の場所）に替えて `--stamp` → 印の F-4 の行の file が `design-note/schema.yaml`。門を 1 本ずつ: `src/design-note/full.yaml` → 0・`src/design-note/schema.yaml` → 1・`src/design-note/`（dir の項目）→ 1・`src/srs.yaml` → 0。**base は場所が design-note/ で、full.yaml が 1 ＝RED。**
2. **f176_the_meta_id_narrows_to_exactly_one_readable_file（同）。** 同じ組み方で 4 通り。対照 = schema.yaml と `.yaml` でない notes.txt（parse できない字）→ schema.yaml・門 full.yaml 0 / schema.yaml 1。頭がどの meta.id にも当たらない（schema.yaml を置かない）・2 file が同じ meta.id（schema.yaml と twin.yaml）・読めない `.yaml` が在る（schema.yaml と parse できない broken.yaml）の 3 通りは場所が design-note/ のまま・門 full.yaml 1 / schema.yaml 1。**base は対照が design-note/ で落ちる＝RED。**
3. **f176_the_meta_id_skips_links_and_bails_on_unreadable_files（`crates/folio/src/stamp.rs` の既存の tests の区間・単体の歯）。** 一時 dir の design-note/ に full.yaml（meta.id full）と schema.yaml（meta.id design-note-schema）を置き、stop_file を直に呼ぶ: 頭 design-note-schema → schema.yaml・頭 full → full.yaml（名の規則）・頭 nothing → dir。schema.yaml を指す symlink link.yaml を足しても schema.yaml（symlink は数えない）・UTF-8 でない latin1.yaml を足すと dir・それを外して重複キーの dup.yaml を足すと dir。symlink と重複キーは binary 経由では組めない（symlink は正本の要約値の口が先に断る）ので単体で縛る。**base は 1 つ目が dir ＝RED。**
   3b. **f176_the_meta_id_matches_exactly_whatever_the_order（同じ tests の区間・単体の歯・改訂 a で独立の検証役の提案を範囲を狭めずに採った）。** 一時 dir の design-note/ に schema.yaml（meta.id design-note-schema）と、字の近い id の file（longer.yaml = design-note-schema-v2・design.yaml = design-note・upper.yaml = DESIGN-NOTE-SCHEMA）、最上位の id だけの top.yaml（id design-note-schema・meta なし）、meta.id schema の alias.yaml、meta.id a/b の slash.yaml、置き場の根に meta.id FR2 の root.yaml を置き、stop_file を直に呼ぶ（文書は design-note/・srs.yaml・linked/ の 3 つ）。期待は順に: 頭 design-note-schema → schema.yaml（字のまま同じ 1 つだけ・前方一致・大小文字の違い・最上位の id は当たらない）・頭 schema → schema.yaml（名の規則が先＝alias.yaml の meta.id schema を見ない）・頭 a/b → dir（区切りを含む頭は meta.id を見ない）・file 形の文書 srs の頭 FR2 → srs.yaml（file 形は meta.id を見ない）・名の順で最後に並ぶ読めない zz-broken.yaml を足すと dir（読めない file は並びのどこに在っても広い側）・置き場の dir が symlink（linked/ → design-note/）なら linked/・`<頭>.yaml` が symlink（via.yaml → schema.yaml）で別の target.yaml が meta.id via を持っても dir。**base は 1 つ目が dir ＝RED。**
4. **今の `<頭>.yaml` の形の歯は変えずに通る。** 便 169 の f169_the_stamp_writes_the_stop_places_and_the_gate_reads_them（`ADR-1.decision` → `adr/ADR-1.yaml`・`sections.x` → `design-note/`）ほか tests/stamp.rs の既存 16 本は本文も fixture も変えない（verify の 3）。
5. **RED の実測。** 歯だけの差分（r176-teeth.patch = tests/stamp.rs と stamp.rs の tests の区間・9,280 byte）を base に当てると f176_ の 4 本とも落ちる（本文は red-a-176.log）。

### (d) 採らなかった形

1. **頭の字の形から file を推す（接尾辞 `-schema` を schema.yaml に当てる等）。** tsuzuri は `design-note-schema`、folio2 は `folio2-design-note-schema` で、字の形の決まりは無い。推しは読み違いで狭める口になる。meta.id は文書の id の正本なので、照らすのはそれだけにする。
2. **2 つ以上当たったら最初の 1 つ（名の順）を取る。** 並びに依って答えが変わり、狭めを読み違える（突然変異 M2 で縛る）。
3. **文書の id → file の表を天井の正本か索引に持たせる。** 同じ内容を 2 面に人が書くことになる（P-6.3・P-6.4）。meta.id から毎回導けば 1 面で足りる。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest **996 / 996**（992 + f176_ の 4 本）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file（base と全 file の sha256 の要約が同じ）。tests/stamp.rs 18 / 18・binary の単体の歯 150 本。
2. **突然変異（本便を当てた写しの meta_file / stop_file だけを 1 通りずつ変え、f176_ の 4 本を撃つ）。** 起草役の 6 通り（M1〜M6）と独立の検証役の 14 通り（V1〜V14）の 20 通りとも f176_ のどれかが落ちる（見本 2510cee に起草役が撃ち直した実測・mut20.log）。落ちる歯の略: 印 = a_stop_on_the_schema_meta_id_is_its_file・狭め = narrows_to_exactly_one_readable_file・単体 1 = skips_links_and_bails_on_unreadable_files・単体 2 = matches_exactly_whatever_the_order。

| 変異 | 落ちる f176_ |
| --- | --- |
| M1 meta.id で解かない（base と同じ） | 4 本とも |
| M2 2 つ以上当たっても最初の 1 つを取る | 狭め |
| M3 読めない .yaml を飛ばして数え続ける | 狭め・単体 1・単体 2 |
| M4 重複キーを見ない | 単体 1 |
| M5 symlink も数える | 単体 1 |
| M6 .yaml でない file も読む | 狭め |
| V1 meta.id が頭で始まれば当てる（前方一致） | 単体 2 |
| V2 頭が meta.id で始まれば当てる（前方一致） | 単体 2 |
| V3 下の dir の .yaml も数える（1 段） | 印 |
| V4 file 形の文書でも親の dir で meta.id を見る | 単体 2 |
| V5 名の規則より meta.id を先にする | 単体 2 |
| V6 `<頭>.yaml` が symlink でも meta.id で解く | 単体 2 |
| V7 置き場の dir が symlink でも解く | 単体 2 |
| V8 大文字小文字を無視して比べる | 単体 2 |
| V9 最上位の id も meta.id の代わりに見る | 単体 2 |
| V10 UTF-8 でない file だけ飛ばす | 単体 1 |
| V11 parse できない file だけ飛ばす | 狭め・単体 2 |
| V12 読めない file が当たりの後に並べば飛ばす（並びに依る） | 単体 2 |
| V13 頭が空・`.` 始まり・区切りでも meta.id で解く | 単体 2 |
| V14 2 つ以上当たったら名の順で最後を取る | 狭め |

3. **外の置き場（tsuzuri）。** 印は生成物なので、着地の後の binary で `folio ceiling --stamp` を撃ち直せば F-4 の場所が design-note/schema.yaml に狭まり、schema.yaml 以外の設計ノートを書く便は門を通る（(a) の 2 の表の右の列・t3v の写しで実測）。F-4 そのもの（生成区間の folio2 の id）は便 174 の領分で、本便は場所の広がりだけを直す。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 2 本とも印なし（`crates/folio/src/stamp.rs` と `crates/folio/tests/stamp.rs`・どちらも増える）。差分 13,348 byte（`git diff 1ca1ae1 2510cee | wc -c`・2 file・+185 −5）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/stamp.rs` の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した（lines.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/stamp.rs` | 394 | 1106 | 513（+119） | 987 |

   歯の file は src の外なので余地を測らない（参考値 913 行 → 974 行）。
3. **size は S。** 余地 1106 は S の見積 100 を超える。
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1 に揃える。便の後の写しで 4 行とも rc 0（verify-a.log・2 は 2 本）。base では 1 と 2 が 0 件で rc 4、3 と 4 は rc 0。
   1. `cargo nextest run -p folio --test stamp f176_` = (c) の 1 と 2（2 本）。
   2. `cargo nextest run -p folio --bin folio f176_` = (c) の 3 と 3b（2 本）。
   3. `cargo nextest run -p folio --test stamp` = 印の歯の全部（便 169 の `<頭>.yaml` の形の歯を含む・参考値 18 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は stamp の 1 本で write-set に在る。`--bin folio` の絞り込みの語 f176_ を関数名に持つ src は `crates/folio/src/stamp.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0。
2. **受付。** 受付の先撃ち（precheck）は、本契約を commit した作業ツリーで契約に起因する断り 0（起草の記録）。共通の検証は同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d176-draft.md`、差分と script は同じ dir の d176-scripts（repo には入れない）。

1. 模擬: 差分 c176.patch を base に当て、run-176.sh（組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と要約）・verify-176.sh（verify の 4 行）。
2. RED: r176-teeth.patch を base に当てて `cargo nextest run -p folio f176_`（4 本とも落ちる）。
3. 突然変異: mut-176-all.py（独立の検証役の mut-v176.py の写し・(e) の 2 の 20 通り・`MUTLABEL=mut20`）。
4. 外の置き場: tsuzuri-176.sh（t3v の写しを scratch に写し、base と本便の binary で --stamp と門 10 本）。
5. 過去の周: heads-176.py（本流の design-intent と 52 周の所見 file・t3v の写し）。
6. 余地: lines-176.sh（lines-176.py と lines-176.awk）。門: gate-176.sh。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 門（gate.rs）・所見の数え・印の欄の形・要件書と判断の記録と設計ノートの字・生成区間（便 174）・外の置き場の印の撃ち直し（その置き場の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 頭の字が file 名でも meta.id でもない場所（欄の名 `sections`・`plain`・`meta` など）は今どおり dir 全体。meta.id が 2 file で重なる置き場（設計ノートなら床の違反）と、読めない `.yaml` を持つ置き場も dir 全体。判断の記録の本体の最上位の id は見ない（file 名 = id を床が課すので名で解ける）。名の規則が先なので、頭の id が名（`<dir><頭>.yaml`）と別の file の meta.id とで 2 file を指す置き場では、名の file へ狭める（欄の決まりの schema.yaml の meta.id が別の file の stem と同じときだけ起きる・今の folio2 と tsuzuri には無い）。印は `--stamp` の時点の置き場で解くので、後で file の名や meta.id を変えると印の場所は古いまま（今と同じ）。
3. **撤退条件。** (1) 本便が要件書 FR20 の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の stop_file が base（1ca1ae1）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果か `folio build` の出力（35 file）が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/stamp.rs` の stop_file の 1 段と meta_file・頭の注・単体の歯 2 本。`crates/folio/tests/stamp.rs` の頭の注の 1 行・道具 schema_stop と定数 2 つ・歯 2 本。
- 入れない: 門・所見の数え・印の欄の形・設計文書・生成区間・fixture の file・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| place | 場所の file | `stamp.rs` の stop_file（名の規則が先・次に meta_file・解けなければ dir） |
| meta | meta.id の解き | `stamp.rs` の meta_file（直下の読める .yaml のちょうど 1 つ） |
| teeth | 歯 | tests/stamp.rs の f176_ 2 本（印と門を binary で）・stamp.rs の単体の f176_ 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 便 169（印の refutes の行と stop_file・本流に在る）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ (e) の 3（印の撃ち直し）を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fw"
title = "印の 止める の場所の穴の直し（tsuzuri の写しの天井の周の整合 F-4 で、場所の頭が設計ノートの欄の決まりの meta.id design-note-schema で file 名 schema.yaml と違い、印の場所が design-note/ 全体に広がって門が設計ノートの全 file を書く便を止めた）: crates/folio/src/stamp.rs の stop_file は、dir 形の文書で at の頭の <dir><頭>.yaml が無いとき、新しい関数 meta_file で置き場の直下の .yaml（下の dir と symlink を除く）の最上位の meta.id が頭と同じ file がちょうど 1 つならその file に解き、0 か 2 つ以上か読めない .yaml（UTF-8 でない・parse できない・重複キー）が在れば今どおり dir 全体（広い側）。名の規則が先・file 形の文書・門・所見の数え・印の欄の形・要件書と設計文書は変えない（FR20 の頭の id が指す 1 file の字の中の直し）。歯は f176_ の 4 本（tests/stamp.rs の 2 本 = tsuzuri の形の最小の字で印と門を binary で・stamp.rs の単体 2 本 = symlink と読めない file、meta.id は字のまま同じ 1 つだけ・読めない file は並びに依らず広い側・名の規則が先・file 形と symlink の置き場と symlink の頭の file と区切りの頭では meta.id を見ない）。実装の見本は origin の枝 impl/d176 の commit 2510cee（親 e06b8d3・根 1ca1ae1）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = main 1ca1ae1"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/stamp.rs", "crates/folio/tests/stamp.rs"]
verify = ["cargo nextest run -p folio --test stamp f176_", "cargo nextest run -p folio --bin folio f176_", "cargo nextest run -p folio --test stamp", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/stamp.rs の f176_ の 2 本（周の写しの design-note/ に meta.id design-note-schema の schema.yaml を置くと、場所 design-note-schema.schema.figures.retry_rules_row の支持の 止める の印の file が design-note/schema.yaml で、門が full.yaml を書く便を 0・schema.yaml と design-note/ を書く便を 1 で返し、下の dir の同じ meta.id は数えない。頭がどの meta.id にも当たらない・2 file が同じ meta.id・読めない .yaml が在るときは design-note/ のままで門が 1、.yaml でない file は読まない）が緑、binary の単体の f176_ の 2 本（stop_file が symlink を数えず、UTF-8 でない file と重複キーの file で dir に落ちる。meta.id は字のまま同じ 1 つだけに当て〔前方一致・大小文字の違い・最上位の id は当たらない〕、読めない file は名の順のどこに並んでも dir、名の規則が先〔頭 schema は alias の meta.id schema でなく schema.yaml〕、file 形の文書・置き場の dir が symlink・<頭>.yaml が symlink・頭が区切りを含むときは meta.id を見ない）が緑、tests/stamp.rs の歯の全部（便 169 の頭の file 名の形の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
