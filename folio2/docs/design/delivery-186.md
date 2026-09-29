# 設計: 便 186 — folio check --emit-rulings（決定の欄の裁定 id を 1 件 1 行の JSON で書き出す・判断の記録 ADR-31 の便 C）

- 要件: FR26（決定の欄の裁定 id の形を床で数え、求められたら形の種類つきで書き出す・要件書 第 1.53 版）の後半「書き出しが求められたときは、同じ関数で切り出した裁定 id を全部、形の種類を添えて 1 件 1 行の JSON（JSON Lines）で出す」と、受入基準 AC28 の後半（1 つの欄の 3 つの形 → 3 行）。本流の要件書に在る id で、字は変えない。
- 条: P-3.1（決まった答えの出る検査は床に置く）・P-4.1（実行できなかった検査を「異常なし」にしない＝終了コードは素の床と同じ）・P-5.4 と P-12.2（folio は台帳を読まない・裁定 id が台帳に在るかは器と人が見る）・P-6.3（読み手を 2 つにしない＝床と同じ歩き手と関数）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 判断の記録 ADR-31（持ち主の承認 2026-09-28 10:29 JST・対話面 R-8・逐語「全部承認する」・台帳 f2-648.267）の決定 (4)。字の正は本流の ADR-31 決定 (4)・FR26・AC28。便 A（便 181・182）と便 B（便 183・184）は着地済み。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gg` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 8 本（src 5・歯の file 3〔新 1・本文不変 2〕）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流 f300177 の組み立てに write-set 8 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 f300177**（便 184 の着地の後）。この契約の数はすべて f300177 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d186`（commit **e450c3b**・見本 5dd19ef〔親 f300177〕に改訂 a〔歯 4 に索引の床だけが落ちる写しの例〕を積んだもの）。`git diff f300177 e450c3b` が便の全体の差分（6 file・+610 −38・44,108 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout e450c3b -- <write-set の file>`）。write-set の外は変えない。
- 並行の便との重なり: 便 185（行 gf・判断の記録 ADR-32・持ち主の承認待ち・write-set 17 本）とは 1 本も重ならない＝どちらが先に着地してもよい。185 が先なら、本便の歯は folio2 の正本の数を固定していない（(c) の 3・5）ので数え直しは要らない。ほかに本便の write-set を書き換える未着地の便の契約は base の時点で無い。

## 1. 設計

### (a) いま起きていること（base f300177 の実測・参考値）

1. **切り出しはできるが、外へ出す口が無い。** 便 181 の後、床は `crates/folio/src/ruling.rs` の歩き手 `sites`（決定の欄を閉じた一覧の順に拾う）と関数 `rulings`（欄の字から裁定 id を全部切り出し、形の種類 question・notes-time・bead を付ける）で決定の欄を数える。切り出した字・形・台帳の id の部分（`Ruling` の 3 欄）を読む口は無く、`Ruling` と `Form::name` は「便 C が読む」として `#[cfg_attr(not(test), allow(dead_code))]` のまま残っている。
2. **`folio check` の旗。** `--emit-amends`（標準出力は貼れる差分だけ・違反と要約は標準エラー・終了コードは素の床と同じ）と凍結の 4 つ（`--freeze-anchor`・`--freeze-ids`・`--freeze-start`・`--freeze-adrs`）が在り、互いに同時に撃てない。`--emit-rulings` は無い（base の binary は `error: unexpected argument '--emit-rulings' found` で 2）。旗の型 `Flag`（`phase.rs`）は凍結の後始末（`freeze.rs` の `after`）が全部の場合を match する。
3. **数（参考値・書き出しの見本と、それと独立の python の数え count-186.py が byte で同じ）。** folio2 の本流 f300177 = 決定の欄 155・裁定 id 202 件（notes-time 180・bead 22・question 0・node が null の件 84）。床の土台（`tests/fixtures/floor_base/design-intent/`）= 欄 55・67 件（notes-time 46・bead 21・node が null 18）。tsuzuri の写し（改訂 a の時点の HEAD cbaa244・計画のノートへ移った後）= 欄 97・146 件（bead 100・question 46・node が null 91）で、そのうち計画のノートの判断の表が 33 欄・63 件（question 30）。起草の時点の HEAD 274fbfb では欄 63・81 件（bead 66・question 15・node が null 28）だった。どの置き場も形の無い欄は 0。
4. **base の歯（参考値）。** workspace の nextest 1037 / 1037・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 36 file。`git grep -n f186_ -- crates` は 0 件・行 id `gg` は 0 件。

### (b) 直す先 — 床と同じ歩き手と関数で、決定の欄の裁定 id を標準出力へ

1. **`crates/folio/src/main.rs`。** `folio check` に旗 `--emit-rulings` を足す（ほかの 5 つの旗と同時に撃てない＝`conflicts_with_all`）。旗の値は型 `Flag` の `EmitRulings`。違反の行と要約の行を `--emit-amends` と同じく標準エラーへ出し（標準エラーへ回す旗の条件に `EmitRulings` を足す）、判定の後に材料の書き出しの行を標準出力へ 1 行ずつ出す。終了コードは素の床の判定のまま（`verdict.exit_code()`）。「#」で始まる注（まだ分からない・機構がまだ無い条・行 R-17）は今のまま標準エラー。
2. **`crates/folio/src/phase.rs`。** `Flag` に `EmitRulings` を足す（注 1 行）。
3. **`crates/folio/src/freeze.rs`。** `after` の match で `None` と `EmitRulings` を同じ腕にして後始末の値を無し（`After::Nothing`）にする（凍結の後始末は無い）。
4. **`crates/folio/src/check.rs`。** 材料 `Materials` に `rulings: Vec<String>`（`--emit-rulings` の標準出力の行・旗の無いときは空）を足す。`check_dir` は歩き手の結果 `sites` を 1 度だけ作って床（`check_rulings`）に渡し、旗が `EmitRulings` のときだけ同じ `sites` を ruling.rs の関数 `emit` に渡す。床の判定と違反の字は変えない。
5. **`crates/folio/src/ruling.rs`。**
   - `Site` に 2 欄を足す: `path`（file の根からの欄の道・一覧の番号は 0 始まり・例 `thresholds[9].ruling`・`articles[4].amended_by[0].ruling`・`approval.ruling`・`meta.approval[2].stamp`・`sections[1].rows[0].ruling`）と `node`（欄を持つ索引の節点の id）。node は条の改訂来歴では条の id、規則の表の行では行の id、判断の記録の承認欄では判断の記録の id で、節点の無い欄（憲法の発効の承認・設計ノートの承認欄の行・5 正本の stamp・判断の表の行）では None。
   - 歩き手 `sites` は一覧の番号を表でない項を飛ばす前の番号で持つ（関数 `rows`・`approval_rows` はそれを使う）。拾う欄・順・床の違反の字（`at`）は変えない。
   - 関数 `emit`（引数は置き場の dir と歩き手の結果）を足す: 値が字の欄ごとに `rulings` で切り出した裁定 id を全部、欄の順・切り出した順に 1 件 1 行の JSON にする。欄は `ruling`（切り出した字）・`form`（`Form::name`）・`bead`（台帳の id の部分）・`node`（字か `null`）・`file`（置き場の根からの相対・`Site.file` のまま＝設計ノートは `design-note/<file 名>`・判断の記録は `adr/<id>.yaml`）・`line`（欄の鍵の行・1 始まり）・`field`（`path`）の順で、空白を挟まない。字の escape は `yaml::json_str`。値が無い・字でない欄は出さない（床の違反）。
   - 行の番号は、file の字を yaml-rust2 の event（床の読み手 `yaml.rs` と同じ parser と印）でもう 1 度読み、欄の道ごとの鍵の行を引く（受け手 `Lines`・関数 `key_lines` と `lines_of`）。同じ表の 2 度目の鍵は数えない（読み手が最初の値を残すのと同じ）。引けなければ 0（読めない file・rc が 0 でないときだけ起きる）。
   - `Ruling` と `Form::name` の、使われない字を許す印（`#[cfg_attr(not(test), allow(dead_code))]`）を外す（書き出しが読む）。単体の歯 1 本（(c) の 6）。
6. **変えないもの。** 床の判定（`check_rulings`・違反とまだ分からないの字）・決定の欄の一覧と歩き手の拾う欄と順・文法の関数 `rulings`・`--emit-amends` と凍結の旗の出力・索引（`folio graph`）・設計文書の正本（生成区間を含む）・憲法と要件書と判断の記録の字・folio2 自身の床 4 本の結果・`folio build` の出力（36 file・base と byte で同じ）。

### (c) 歯（f186_・base で 0 件）

binary 経由の歯は `crates/folio/tests/emit_rulings.rs`（新）。どれも床の土台か folio2 の正本の写しを一時 dir の `design-intent/` に、器の導出 file を写しの根の `contracts/` に作って git の 1 commit にし（土台の写しの素の床は合格 0）、字を足すか変えて撃つ。期待の字は歯の側の手書き（土台の数・行の番号・欄の道は独立の python の数えと同じ）。

1. **f186_one_field_with_three_forms_gives_three_lines（AC28 の後半）。** 土台の写しの規則の表の行 R-10 の裁定の欄を、引用符で囲んだ字「t3-hub.56:20260927T2259Z-1・f2-648 notes 2026-09-28 07:18 JST（t3-hub.1）」にすると、node が R-10 の行はちょうど次の 3 行（どれも file は rules.yaml・line は 38・field は `thresholds[9].ruling`）で、ほかの 66 行は変えない前の行と同じ。期待は歯の側に 1 行の JSON の字で丸ごと書く。
   - ruling は t3-hub.56:20260927T2259Z-1・form は question・bead は t3-hub.56。
   - ruling は f2-648 notes 2026-09-28 07:18 JST・form は notes-time・bead は f2-648。
   - ruling は t3-hub.1・form は bead・bead は t3-hub.1。
2. **f186_every_line_has_the_seven_fields_and_the_null_node。** 土台の写しに条 P-1 の改訂来歴 1 件・索引の欄の決まり graph.yaml（作成の行と、台帳の id と問いの形の 2 つを持つ承認の行）・発効した設計ノート decide.yaml（承認欄 1 行と判断の表 1 行）を足し、条 P-2 の前の版との対応（supersedes_v1）の裁定に台帳の id を書く。書き出しは 72 行（67 + 5）で、手書きの 9 行（憲法の発効の承認 null・条 P-1・行 D-8・ADR-2・設計ノートの承認欄 null・判断の表の行 null・srs の stamp null・graph の stamp の 2 行 null）を含む。どの行も 7 つの欄をこの順で持ち、node が null でないのは条・規則の表の行・判断の記録の欄だけ。supersedes_v1 の裁定は出ない（一覧の外）。
3. **f186_file_line_and_field_point_at_the_source。** 土台と folio2 の正本の写しで、どの行も file の字の line 行目に欄の鍵（`ruling: ` か `stamp: `）が在り、field を別の読み手（yaml-rust2 の高水準の読み手）で引いた値が ruling の字を含み、node は field の行の id（条・規則の表の行）か判断の記録の id に等しい。土台では ruling の字も line 行目に在る。folio2 の正本の数は固定しない。
4. **f186_the_exit_code_is_the_plain_floor_and_stdout_is_json_only。** 終了コードが素の床と同じ: 土台の写しは 0 と 0、行 R-10 の裁定の欄を `G16=A（受入 (f)）` にすると 1 と 1、`未記入` にすると 2 と 2、要件 FR1 の id を単引用符で囲む（索引の床〔`main.rs` が素の床の後に撃つ、folio graph が組めるかの検査〕だけが落ち、違反は種別 索引の節点の 1 行）と 1 と 1、folio2 の正本の写しは同じ値。標準出力はどの場合も JSON の表で欄 ruling から始まる行だけ（「#」の行・違反の行・要約の行を出さない）で、違反と要約の行は標準エラーに素の床と同じ字で出る。違反のある写しでも切り出せた行（66 行・索引の床だけが落ちる写しでは 67 行）は出る。書き出しは写しの file を 1 byte も変えない。`--emit-rulings` をほかの 5 つの旗と一緒に撃つと断られ、標準出力は空。
5. **f186_every_decision_field_is_written_out（全数・便 A と同じ母集団）。** 土台と folio2 の正本の写しで、書き出しの欄（file と field の組）の数を file ごとに数える。次に決定の欄を持つ file（憲法・規則の表・5 正本・判断の記録・設計ノート）の語頭の台帳の id の頭の字を歯の側の走査で全部大字にし、素の床の種別 裁定 id の違反を file ごとに数えると、2 つの数が file ごとに等しい（床が数える欄と書き出しの欄が同じ）。大字にした写しの書き出しは 0 行。土台の数は手書き（欄 55・67 行・notes-time 46・bead 21・node が null 18）。
6. **単体の歯 f186_key_lines_follow_block_and_flow_maps（`crates/folio/src/ruling.rs` の tests の区間）。** 段の形・流れの形（行をまたぐ表と引用符の鍵）・一覧の中の一覧・段の字（`|`）の値で、欄の道ごとの鍵の行が鍵の書かれた行を指し、同じ表の 2 度目の鍵は数えない（鍵 11）。
7. **RED の実測。** 歯の file だけ（改訂 a の見本 e450c3b の字）を base に当てると、binary の 5 本とも落ちる（旗が無く clap が断る・標準出力は空・起草の記録の red-186-bin.log・nextest の rc 100）。base の `--bin folio f186_` は 0 本（関数 `lines_of` が無い・nextest の rc 4）。

### (d) 採らなかった形

1. **field を決定の欄の閉じた一覧の項の字（例 `design-note/*.yaml meta.approval[].ruling`）にする。** 同じ file の承認欄の行どうしが同じ字になり、file と field の組が欄を 1 つに決めない（器が差分の前後を突き合わせる鍵にならない）。欄の道なら組が一意で、一覧の項は file と欄の道から決まる。
2. **判断の表の行の node を行の id（例 d1）にする。** 判断の表の行は索引の節点の種類の閉じた一覧に無く、行の id は 1 本のノートの中でしか一意でない。node を索引の節点の id に限れば、器は node で索引と結べる（null でない node は全部、索引の同じ file の節点に在る＝(e) の 4）。行の所在は file・line・field が持つ。
3. **行の番号を読み手の木（`yaml::Node`）に持たせる。** 全部の命令の読み手が変わり、write-set が `yaml.rs` と読み手を使う全部の file に広がる。書き出しの旗のときだけ同じ parser の event をもう 1 度読む形にした（値と母集団は床の木から取り、行の番号だけを引く）。
4. **違反のある置き場では何も出さない。** ADR-31 決定 (4) は「一覧が全数なのは終了コードが 0 のときだけ」と書き、出すこと自体は止めない。`--emit-amends` と同じく切り出せた行は出し、器は終了コードで全数かを見る（変異 M12 を歯 4 が落とす）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写し（改訂 a の見本 e450c3b）で、workspace の nextest **1043 / 1043**（base + f186_ の 6 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0・schema --check 一致）・`folio build --write` 36 file（base と全 file が byte で同じ）。字の期待を直した既存の歯は無い。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f186_ の 6 本を撃つ）。** 16 通りとも落ちる（生き残り 0・改訂 a の見本で撃ち直した mut-186.log）。M16 は独立の検証（d186-verify.md）の変異 V7 で、改訂 a の前の歯では生き残った（歯 4 に索引の床だけが落ちる写しの例を足して落ちるようにした）。

| 変異 | 落ちる歯（歯の番号は (c)） |
| --- | --- |
| M1 最初の id だけを出す | 1・2・4・5 |
| M2 form を取り違える | 1・2・5 |
| M3 node を常に null | 1・2・3・5 |
| M4 file を絶対 path | 1・2・5 |
| M5 終了コードを 0 に固定 | 4 |
| M6 「#」の注を標準出力に出す | 1〜5 |
| M7 違反と要約を標準出力に混ぜる | 1〜5 |
| M8 supersedes_v1 を出す | 2・4・5 |
| M9 line を 0 始まり | 1・2・3・6 |
| M10 field を一覧の項の字にする | 1・2・3・5 |
| M11 stamp の欄を出さない（母集団の欠け） | 1・2・4・5 |
| M12 違反のある置き場では何も出さない | 2・4 |
| M13 同じ表の 2 度目の鍵で上書きする | 6 |
| M14 bead に切り出した字全部を出す | 1・2 |
| M15 判断の表の行に行の id を node として出す | 2 |
| M16 旗のときは索引の床を撃たない（検証の V7） | 4 |

3. **外の置き場（tsuzuri の写し・参考値）。** 改訂 a の時点の HEAD cbaa244（計画のノートへ移った e82c3c3 の後）で、見本の binary の素の床も書き出しも 0 で 146 行。`folio schema --dir design-intent --check` も一致（生成区間は新しい・`folio schema --write` は要らない）。146 行は独立の python の数えと byte で同じ（順も同じ）。内訳は欄 97（憲法の発効の承認 1・改訂来歴 3・閾値の行 24・開発規律の行 4・判断の記録 14・設計ノートの承認欄 18・判断の表の行 33）・bead 100・question 46・node が null 91。起草の時点の HEAD 274fbfb では素の床と書き出しが 1（違反 5 = 生成区間が古い）で 81 行（bead 66・question 15・node が null 28）、`folio schema --write` の後に 0 で 81 行は byte で同じだった。
4. **索引との結び（判断の記録 ADR-32 決定 (5) の前提・便 185 の見本 impl/d185 61010f2 の binary の `folio graph --print --summary`）。** 書き出しの設計ノートの file の字は索引の file の字と同じ形（`design-note/<file 名>`）で、null でない node は全部、索引の同じ file の同じ id の節点に在る（folio2 202 行・tsuzuri cbaa244 の 146 行とも外れ 0）。tsuzuri cbaa244 で契約表を持つノート 16 本の書き出しの 24 行は全部、file の字で索引の設計ノートの行と結べる（ADR-32 撤退条件 (2) の範囲で結べない件 0）。結べる索引の行が無いのは契約表を持たないノート 2 本の 66 行（計画のノート surface-plan.yaml の承認欄 2 行と判断の表 63 行・surface.yaml の承認欄 1 行・どれも node は null）で、folio2 では索引が読まない 4 正本（天井の正本 18・入口 7・相談窓口 3・索引の欄の決まり 1）の 29 行が同じ。器は file・line・field で所在を引ける。本便は索引を変えない。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/tests/emit_rulings.rs`。`crates/folio/tests/ruling.rs` と `crates/folio/tests/freeze.rs` は本文を変えない（verify の `--test` の scope）。差分 44,108 byte（`git diff f300177 e450c3b | wc -c`・6 file・+610 −38）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 5 本（python と awk の 2 実装で一致・cap-186.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/ruling.rs` | 370 | 1130 | 515（+145） | 985 |
| `crates/folio/src/check.rs` | 1105 | 395 | 1113（+8） | 387 |
| `crates/folio/src/main.rs` | 688 | 812 | 698（+10） | 802 |
| `crates/folio/src/phase.rs` | 73 | 1427 | 75（+2） | 1425 |
| `crates/folio/src/freeze.rs` | 521 | 979 | 521（0） | 979 |

3. **size は M。** src の増分は +165 で S の見積 100 を超え、M の見積 300 の内。余地の最小（check.rs の base 395）は M の 300 を超える。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。見本の写しで 6 行とも rc 0。
   1. `cargo nextest run -p folio --test emit_rulings f186_` = (c) の 1〜5（5 本）。
   2. `cargo nextest run -p folio --bin folio f186_` = (c) の 6（1 本）。
   3. `cargo nextest run -p folio --bin folio ruling::tests` = 文法と一覧と鍵の行の単体の歯（便 181・182・186）。
   4. `cargo nextest run -p folio --test ruling` = 便 181・182 の床の歯（歩き手を変えても床の違反の字と数が同じ）。
   5. `cargo nextest run -p folio --test freeze` = `--emit-amends` と凍結の旗の歯（旗の型と後始末の match を変えても同じ）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（emit_rulings・ruling・freeze）は全部 write-set に在る。`--bin folio` の歯の在り処（ruling.rs）も write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d186 の本契約で 契約に起因する断り 0（preflight ok・`f186_` は base で 0 件・起草の記録の precheck-186.log）。改訂 a の見本 e450c3b の写しで verify の 6 行とも rc 0（verify-186-reva.log）。便 185 とは write-set が重ならない。
3. **着地の後。** 席は tsuzuri へ「`folio check --emit-rulings` が使える（7 欄・終了コードは素の床と同じ・全数は 0 のときだけ）」を返す。tsuzuri の今の写しは生成区間が新しく、取り込みに `folio schema --write` は要らない（(e) の 3）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/d186-draft.md`、script と log は同じ dir の d186-scripts。

1. 模擬: 見本 e450c3b（`git diff f300177 e450c3b` = c186-reva.patch・歯だけ = r186-teeth-reva.patch・改訂 a の前の 5dd19ef は c186.patch）。run-186.sh（組み立て・nextest・clippy・床 4 本・build）・build-186.sh（build の出力を base と byte で比べる）。
2. 独立の数え: count-186.py（ADR-31 決定 (1)(3) の字から書いた一覧と文法・PyYAML の compose の印で行を引く・`--json` で書き出しと同じ形）・tooth2-186.py（歯 2 の写しの期待の行）。
3. RED: red-186.sh。突然変異: mut-186.py（16 通り・M16 は改訂 a）。余地: cap-186.sh。
4. 外の置き場: prep-tz.sh（tsuzuri の .git だけを写して clone・remote を外す）・tz-186.sh。索引との結び: join-185.py。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 裁定 id が台帳の問いの記録に在るか・置き場の約束の形か・便の差分で新しく書かれた欄か（器の持ち分・ADR-31 決定 (4)）・索引の節点（ADR-32・便 185）・設計文書の正本と生成区間・憲法と要件書と判断の記録の字・tsuzuri の写しの書き直し（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) line は欄の鍵の行で、段の字（`|`・`>`）や行をまたぐ引用符の値では、裁定 id そのものは後の行に在る（folio2・土台・tsuzuri の今の欄はどれも鍵と同じ行・count-186.py で見た）。(2) 終了コードが 0 でないとき、一覧は全数と限らない（読めない file の欄は出ない・file 名と id の違う判断の記録は id の字の file で出て line は 0）。(3) 書き出しは形だけで、台帳に在るかは数えない（P-12.2）。
3. **撤退条件。** (1) 本便が要件書 FR26 か ADR-31 の字か憲法の条文を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `ruling.rs` の歩き手 `sites` か関数 `rulings`、または `check.rs` の `check_dir` の決定の欄の段が base（f300177）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `folio check --emit-rulings`（旗・型 `Flag` の `EmitRulings`・標準出力と標準エラーの分け方）・`ruling.rs` の `Site` の欄の道と node・`emit` と鍵の行の引き方・`check.rs` の材料と歩き手の結果の渡し方・`freeze.rs` の match の 1 行・歯の file の f186_ の 5 本と単体の 1 本。
- 入れない: 床の判定・決定の欄の一覧・文法・索引・設計文書の正本・憲法と要件書と判断の記録の字・外の置き場・台帳への記帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| flag | 旗 | `main.rs` の `--emit-rulings`・`phase.rs` の型 `Flag` の `EmitRulings`・`freeze.rs` の後始末 |
| walk | 歩き手の欄 | `ruling.rs` の `Site.path`・`Site.node`（拾う欄と順は便 181 のまま） |
| emit | 書き出し | `ruling.rs` の `emit`・`key_lines`・`lines_of`・`Lines`（7 欄の JSON Lines） |
| wire | 渡し | `check.rs` の `Materials.rulings` と `check_dir` の 1 度だけの `sites` |
| teeth | 歯 | `tests/emit_rulings.rs` の f186_ の 5 本と `ruling.rs` の単体の 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない（yaml-rust2 の event の口は床の読み手が既に使っている）。新しい dir は無い。
- 前提の着地: 便 181〜184（本流 f300177）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ書き出しの口を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gg"
title = "folio check --emit-rulings を足す（判断の記録 ADR-31 の便 C・決定 (4)・要件書 第 1.53 版の FR26 の書き出しの句と AC28 の後半）: crates/folio/src/main.rs の folio check に旗 --emit-rulings（ほかの 5 つの旗と同時に撃てない・Flag::EmitRulings）を足し、違反と要約の行を標準エラーへ、書き出しの行を標準出力へ出し、終了コードは素の床のままにする。crates/folio/src/phase.rs の Flag に EmitRulings を、crates/folio/src/freeze.rs の after の match に EmitRulings を足す（後始末は無い）。crates/folio/src/check.rs は材料に rulings を足し、check_dir で歩き手の結果を 1 度だけ作って床と書き出しに渡す。crates/folio/src/ruling.rs は Site に欄の道 path と索引の節点の id node（条・規則の表の行・判断の記録だけ・ほかは None）を足し、関数 emit で決定の欄の値から rulings で切り出した裁定 id を全部、欄 ruling・form・bead・node（null）・file・line（欄の鍵の行・yaml-rust2 の event の印で引く）・field（欄の道）の順の 1 件 1 行の JSON にする。床の判定・決定の欄の一覧・文法・設計文書の正本は変えない。歯は crates/folio/tests/emit_rulings.rs の f186_ の 5 本と ruling.rs の単体の 1 本。実装の見本は origin の枝 impl/d186 の commit e450c3b（5dd19ef に改訂 a を積んだもの・親 f300177）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 f300177"
req = ["FR26"]
section = "1"
write-set = ["crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/phase.rs", "crates/folio/src/freeze.rs", "crates/folio/src/ruling.rs", "+crates/folio/tests/emit_rulings.rs", "crates/folio/tests/ruling.rs", "crates/folio/tests/freeze.rs"]
verify = ["cargo nextest run -p folio --test emit_rulings f186_", "cargo nextest run -p folio --bin folio f186_", "cargo nextest run -p folio --bin folio ruling::tests", "cargo nextest run -p folio --test ruling", "cargo nextest run -p folio --test freeze", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/emit_rulings.rs の f186_ の 5 本（1 つの欄の 3 つの形が 3 行・7 つの欄の順と node の null・file と line と field が正本の所在を指す・終了コードが素の床と同じ〔索引の床だけが落ちる写しを含む〕で標準出力は JSON の行だけ・書き出しの欄と床の決定の欄が file ごとに同じ数）が緑、binary の単体の f186_ の 1 本（欄の道ごとの鍵の行）が緑、binary の単体の ruling::tests の全部が緑、tests/ruling.rs の歯の全部（便 181・182 の床の違反の字と数）が緑、tests/freeze.rs の歯の全部（--emit-amends と凍結の旗）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio check --dir design-intent --emit-rulings が 0 で標準出力が JSON の行だけ・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
