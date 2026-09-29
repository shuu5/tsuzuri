# 設計: 便 207 — 数える口: 生きた設計ノートの本数・1 本の契約表の行の数・計画だけの行の合計を、置き場の規則の表の上限の行と比べる

- 要件: FR30（設計ノートの増殖を数える・要件書 第 1.57 版）・FR19（規則の表の生成区間へ床の定数から導出して書く）・FR9（設計ノートの形の床・章の上限の違反の字を揃える）。本流の要件書に在る id で、規範文・確かめ方・受入基準（AC33）は変えない。
- 判断の記録: ADR-35 決定 (1)(ア)〜(ク)・(5)・(6)（発効 2026-09-29 13:19 JST・持ち主の逐語「１．全部推奨で」・台帳 f2-648.275.9）。決定の字は変えない。
- 条: P-4.2（行が無い・2 本・形が違うは まだ分からない）・P-5.1（上限の値は置き場の規則の表の型付きの行）・P-6.3（値を読む関数 1 つ・数える関数 1 つ・面と床が同じ字）・P-7.2（下げる道は廃止を状態で表すこと）・P-15.2（編集時の口の止める行は素の床にも同じ字で在る）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `hb` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 23 本（§1 (f) 1）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: **0（通す）**。本流の binary（main root の `target/debug/folio`・13:48 の組み立て）を枝 docs/d207 の木で撃った `folio ceiling --gate --dir design-intent --write-set <write-set の 23 本>` の答え「folio ceiling: 通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）」（起草の記録の gate.log）。
- 前提: **base = 本流 17736c9**（承認の PR #391 の着地・ADR-35 accepted・要件書 第 1.57 版・行 R-23〜R-25・便 203 と 204 を含む）。17736c9 の木は、起草で撃った枝 docs/growth の 80a413d の木と同じ（tree b89606a・`git rev-parse` で確かめた）。この契約の数は 80a413d の木と、その上の見本の写しの実測（参考値・規則の表の行 D-13）で、同じ木なので 17736c9 の上でもそのまま生きる。
- 実装の見本: origin の枝 `impl/d207`（commit **9585bb7**・親 17736c9・1 commit）。`git diff 17736c9 9585bb7` が便の全体の差分（23 file・+514 −87・103,695 byte）。**作業者は、base に対する見本の差分を write-set の file に当ててよく、write-set の外は変えない。**
- 並行の便との重なり: 便 208（行 hc）とは file 1 本（`crates/folio/tests/graph_notes.rs`）と、208 の歯の土台の数の定数 2 つが重なる（§1 (g) 3）。**席の裁定（2026-09-29）で本便が先に着地し、208 は本便の着地の木の上で 2 つの定数を直す。**便 209（行 hd・note.rs）は本便の着地の後に起草する。

## 1. 設計

### (a) いま起きていること（base の木〔17736c9 と 80a413d は同じ木〕の実測・参考値）

1. **上限の行は在るが、道具は読まない。** 規則の表に閾値の行 R-23（生きたノートの本数・1 本 以下）・R-24（1 本の契約表の行の数・1 行 以下）・R-25（計画だけの行の数の合計・1 行 以下）が在る（ADR-35 決定 (5)）。行は欄 key を持たず、注の末尾は「道具がこの行を欄 key で引くのは便 207 からで、便 207 が欄 key を行に足してこの 1 文を消す」。欄 key の閉じた一覧は 3 つ（`crates/folio/src/rules.rs:122` の `KEYS` = note-chapters・plan-note・in-loop-min）で、行に `key: live-notes` を足すと素の床は種別 schema の違反「行 R-n の key「live-notes」が閉じた一覧 [...] に無い」を出す（tsuzuri の写しで 3 行とも違反・`folio build` は 1 file も書かない・起草の記録の tz-207.log）。
2. **数える関数が無い。** 床（`note.rs:70` の `check_note`）が設計ノートに掛ける上限は章の数だけ（`note.rs:108` で `rules::chapter_cap`〔`rules.rs:167`〕を読み、`note.rs:141` の `over_cap` で比べる）。生きたノートの本数・1 本の契約表の行の数・計画だけの行の数は数えない。
3. **章の上限の違反の字は今の数を持つ。** `over_cap` の字は「{file}: 章が {chapters} 本ある＝章が多すぎる（上限 {cap}）」。編集時の口（`folio check --proposed`）は書く前と後の違反の字を完全一致で比べる（`proposed.rs` の judge・ADR-33 決定 (2)）ので、章が上限を 2 以上超えたノートの章を 1 つ減らす編集も、字が変わって新しい違反として止まる（growth-verify の B-1・7 章 → 6 章・上限 1 章で 止める 1 の実測）。面の生成器（`face_note.rs:238`〜`243`）は同じ `over_cap` の字を断りの字にする。
4. **数（参考値）。** folio2 の design-note/ は見本 2 本（example.yaml・figures.yaml・状態 example）だけ＝生きたノート 0 本・契約表の行 0・計画だけの行 0。tsuzuri の写し（本物の main 93389e3 の .git の写しから clone・hook なし）は生きたノート 70 本・1 本の契約表の行の最大 32（surface-board.yaml）・計画だけの行の合計 26（起草の記録の growth-scripts/count-notes.py の出力・tz-207.log）。
5. **base の数（参考値）。** workspace の nextest 1159 / 1159・clippy 0 警告・床 4 本 rc 0（check 合格 0/0）・`folio build` 40 file（起草の記録の base-*.log）。

### (b) 直す先

1. **値を読む関数を 1 つにする（`crates/folio/src/rules.rs`・ADR-35 決定 (1)(ア)(ウ)(カ)）。**
   - 欄 key の値 3 つの定数 `LIVE_NOTES`（live-notes）・`NOTE_ROWS`（note-rows）・`PLAN_ROWS`（plan-rows）を足し、`KEYS` を 6 つにする（3 → 6・順は note-chapters・plan-note・in-loop-min・live-notes・note-rows・plan-rows）。
   - 数の上限の表 `CAPS`（欄 key・単位・数えるものの名）を置く: note-chapters = 章・章／live-notes = 本・生きたノート／note-rows = 行・契約表の行／plan-rows = 行・計画だけの行。
   - `chapter_cap(rules)` を 新しい関数 cap（引数は規則の表と欄 key） に置き換える（章の上限と同じ読み手を 4 つの値で共有する）。値の形は「<正の整数> <単位> 以下」（頭が 0 でない半角の数字・半角の空白 1 つ・単位・半角の空白 1 つ・以下）。行が無い・2 本以上・形が違う（単位の取り違えを含む）は Err。字は今の `chapter_cap` と同じ（「欄 key が {key} の閾値の行が無い」「… が N 本ある」「行 {id} の value「…」が「<正の整数> {単位} 以下」の形でない」）。`CAPS` に無い key は Err「欄 key の {key} は数の上限でない」。
   - 違反の字の関数 `over_cap(at, key, count, cap)` を rules.rs に置く（`note.rs` から移す）。数が上限を超えるときだけ、違反の字「{at}: {数えるもの}が多すぎる（{key} の上限 {cap} {単位} 以下）」と、今の数の字「{at}: {数えるもの}の今の数 {count}（{key} の上限 {cap} {単位} 以下）」の組を返す。違反の字は名指す先・欄 key・上限の値だけを持ち、今の数を持たない（ADR-35 決定 (1)(オ)(ク)）。名指す先は欄 key を採る（行の id は置き場ごとに意味が違う・決定 (1)(オ) の「上限の行の id か欄 key の値」の後者）。
   - 1 本の契約表の行の数の違反で名指す先 {at} は、そのノートの file の字（`design-note/<id>.yaml`）とする。ADR-35 決定 (1)(オ) の「そのノートの id」を、床のほかの違反と同じ file の字で表す（file の名とノートの id は 1 対 1・独立の検証 N-A）。
   - 床の木 `FLOOR` の `key_note` の末尾に 2 文を足す（3 つの key の値の形・行が無いときの扱い〔まだ分からない〕・違反の字は欄 key と上限の値と名指す先だけで今の数は床の標準エラーの「今の数」の行）。`enums.key` は `KEYS` から導出されるので 6 つになる。
2. **数える関数を 1 つ置く（`crates/folio/src/note.rs`・ADR-35 決定 (1)(イ)(エ)(カ)）。**
   - 生きたノートの状態の定数 `LIVE`（draft・effective）と、数え `Growth`（本数・1 本ごとの契約表の行の数・計画だけの行の合計）と、数える唯一の関数 growth（引数は読めたノート） を足す。読むのは `load_notes` が読めたノートだけ（床と導出と索引と同じ読み手・下の dir と symlink と読めない file は今のまま まだ分からない）。状態がちょうど draft か effective のノートだけを数える。1 本の行の数は `sections` の項のうち `type` が contract-table の節の `rows` の項の数の和、計画だけの行は全部の生きたノートの `type` が row-plan の節の `rows` の項の数の合計。`rows` が一覧でない節は 0 行。
   - 床の関数 新しい関数 check_growth（引数は読めたノート・規則の表・所見） を足し、`check_note` が読めたノートが 1 本以上在るとき（見本だけでも）章の数えの後に呼ぶ。3 つの key の行を rules.rs の cap で読み、読めない key は「rules.yaml: 設計ノートの数の上限 {key} が読めない: {理由}」の まだ分からない、読めた key は数を rules.rs の over_cap で比べ、超えたら種別 note の違反と今の数の行を積む。名指す先は、本数と計画だけの行は置き場（design-note/）、1 本の行の数はそのノートの file。
   - 章の上限は rules.rs の cap（欄 key は NOTE_CHAPTERS） で読み、rules.rs の over_cap（名指す先は file・欄 key は NOTE_CHAPTERS） の字で積む（今の数の行も積む）。`chapters` は変えない。
3. **今の数の行（`crates/folio/src/verdict.rs`・`crates/folio/src/main.rs`・ADR-35 決定 (1)(オ)）。**
   - `Report` に今の数の行 `counts` と積む口 `count` を足す（判定に数えない・違反の数にも入らない）。
   - 素の床 `folio check` は、まだ分からない の行の後に `# 今の数: {字}` を標準エラーに出す（ほかの # の行と同じ置き場）。
   - 編集時の口 `folio check --proposed` の標準出力には今の数の行を出さない（`proposed.rs` と口の出し方は変えない）。要件 FR28 の規範文は口の標準出力の行を「止める行・# まだ分からない の行・要約の 1 行・つながりの行」と列挙しているからで、ADR-35 決定 (1)(オ) の「今の数は # の行に出す」は素の床の標準エラーで満たす（席の裁定 2026-09-29）。止めるかどうかの比べ（違反の字の完全一致）も変えない。
4. **面の断りの字（`crates/folio/src/face_note.rs`・ADR-35 決定 (1)(ク)）。** 章の上限を rules.rs の cap（欄 key は NOTE_CHAPTERS） で読み、超えたら断りの字を床と同じ違反の字にし、今の数を次の行 `# 今の数: {字}` にする（面は書かない・読めない行は今と同じ字で まだ分からない）。面の生成器はほかの 3 つの上限で止めない（決定 (1)(カ)）。
5. **設計文書（`design-intent/rules.yaml`・ADR-35 決定 (5)(6)）。** 行 R-23 に `key: live-notes`、R-24 に `key: note-rows`、R-25 に `key: plan-rows` を足し（`stage: post` の後）、3 行の注の末尾の 1 文「道具がこの行を欄 key で引くのは便 207 からで、便 207 が欄 key を行に足してこの 1 文を消す」を消す（前の文の句点は残す）。ruling・ruled_at・値・種別・段・状態・母集団・refs は変えない（行 R-19 と便 179 の前例・行の追加と値は承認済み）。生成区間（`enums.key` と `key_note`）は `folio schema --dir design-intent --write` の導出で書く（手で書かない）。
6. **fixture と凍結 anchor。**
   - 床の土台 `tests/fixtures/floor_base/design-intent/rules.yaml` に閾値の行 R-23〜R-25（欄 key live-notes・note-rows・plan-rows・値 **99 本 以下・99 行 以下・99 行 以下**・条 P-7・裁定の字は行 R-19 の土台の行と同じ形）を行 R-19 の次に足し、`constitution.yaml` の条 P-7 の関係の欄を `{reqs: [NFR3], rules: [R-23, R-24, R-25]}` にする（行 R-4 の双方向）。値は歯の写しの数を超える 99 にし、folio2 の 1 を写さない（土台の上の歯が設計ノートを足しても上限に掛からない）。
   - 生成区間の凍結 anchor `tests/fixtures/schema/rules-region.txt` は、前の anchor の `enums.key` の行に 3 つを足し、`key_note` の行の末尾に同じ 2 文を手で足す（導出の出力を写さない）。床の土台の索引の凍結 anchor 3 本（`graph-anchor.txt`・`graph-digest-anchor.txt`・`node-digest-anchor.txt`）は、土台に節点 3（R-23〜R-25）と辺 6（P-7 → R-23〜R-25 の relations.rules・R-23〜R-25 → P-7 の article）が入った分だけ直す。`graph-digest-anchor.txt` は前の anchor の 5 行（規則行 27 → 30・relations.rules 48 → 51・article 27 → 30・rules.yaml 27 → 30・要約の行）を手で直し、見本の `folio graph --digest` と byte で一致することを確かめた。`node-digest-anchor.txt` は独立の実装（`tests/fixtures/schema/node-digest.py`・src を使わない）の出力で、3 節点の要約値（72ce01be・a70476f7・0beb5c45）は見本の `folio graph --print` の 4 列目と一致する。`graph-anchor.txt` は base と見本の `folio graph --print` の差が節点 3 行・辺 6 行・要約の 1 行だけであることを diff で確かめてから、見本の出力の節点の表・辺の表・出力全体を sha256 で測った（782 行・32,047 byte）。
   - 土台の数を字で持つ歯の定数を直す（便 179 と同じ種類）: `tests/graph.rs`（要約の行・出力の行数 773 → 782 と byte 数 31,543 → 32,047・節点の数 191 → 194・独立の実装の anchor の行数 194 → 197 と byte 数 3,040 → 3,082 と要約値・辺の表の要約値の頭 9e2a3cb5 → 71c66375）・`tests/graph_notes.rs`（要約の行 4 か所・節点 +3・辺 +6）・`tests/hello.rs`（挨拶の 1 行 191 節点・579 辺 → 194 節点・585 辺）・`tests/polarity.rs`（閾値の行 17 → 20・仕掛け 46 → 49 の 3 か所）・`tests/ruling.rs`（裁定 id の違反の数 55 → 58・rules.yaml の 27 → 30）・`tests/emit_rulings.rs`（書き出しの行の数 67 → 70 ほか 6 か所・行 D-8 の行番号 55 → 58）・`tests/floor_cases.yaml`（case mentions-rule-row-counts-the-frozen-base が足す行 R-17 の置き場を thresholds[17] → thresholds[20]〔末尾〕に・前の添字は本便の行 R-23 を上書きする）。
7. **変えないもの。** 要件書・憲法・判断の記録・値域・行 R-23〜R-25 の値と裁定・行 R-19 の値と字・`chapters` の数え方・面の章の帯・編集時の口の止める比べ・`folio build` の出力（folio2）・導出の命令・索引・外部 crate。

### (c) 歯（f207_・base で 0 件・10 本）

絞り込みの語 `f207_` は base で 0 件（`git grep -n f207_ -- crates` が 0 行）。

1. `crates/folio/src/rules.rs` の単体 `f207_cap_reads_the_three_growth_keys_with_their_units`: 3 つの key の行を行の id に依らず読み（52 本・40 行・1 行）、単位の取り違え・「以上」・0・全角・桁区切りは Err、無い・2 本は Err、数の上限でない key（in-loop-min）は Err。
2. `crates/folio/src/rules.rs` の単体 `f207_over_cap_words_carry_the_key_and_the_cap_but_not_the_count`: ちょうど上限は None。章の上限の違反の字は「design-note/x.yaml: 章が多すぎる（note-chapters の上限 12 章 以下）」で、13 章と 14 章で同じ字。3 つの key の字は数 2・3・99 で同じ字で、今の数は 2 つ目の字だけが持つ。
3. `crates/folio/src/note.rs` の単体 `f207_growth_counts_only_live_notes`: draft と effective だけを数え（retired・example・大文字の Draft・一覧の状態は数えない）、1 本の行の数は契約表の節の行の和（一覧でない rows は 0）、計画だけの行は合計。
4. `crates/folio/tests/note.rs` の `f207_live_notes_over_the_cap_is_one_violation_without_the_count`（AC33）: 床の土台の写し（値を 1 に下げた・git の 1 commit）で見本 2 本だけなら合格、下書き 1 本（ちょうど上限）も合格、下書きと発効の 2 本で違反ちょうど 1「[note] design-note/: 生きたノートが多すぎる（live-notes の上限 1 本 以下）」、標準エラーの今の数の行は「design-note/: 生きたノートの今の数 2（…）」。
5. `crates/folio/tests/note.rs` の `f207_retired_and_example_notes_are_not_counted`（AC33）: 2 本目を廃止（承認欄と後継つき）にすると合格、見本にしても合格。
6. `crates/folio/tests/note.rs` の `f207_note_rows_are_counted_per_live_note`（AC33）: 1 行ずつの生きたノート 2 本は合格（合計でなく 1 本ごと）、1 本を 2 行にすると違反 1 でそのノートを名指し、廃止にすると合格。
7. `crates/folio/tests/note.rs` の `f207_missing_twice_or_misshaped_cap_rows_are_unknown`（AC33・決定 (1)(エ)）: 3 つの key のそれぞれで、欄 key を外すと まだ分からない（見本だけの置き場でも読む）、読めた設計ノートが 0 本なら読まない、行を 2 本にすると まだ分からない（2 本ある）、値を形の違う字（単位の取り違え・「1 行以上」）にすると まだ分からない。
8. `crates/folio/tests/note.rs` の `f207_proposed_does_not_stop_retiring_or_adding_over_the_cap_but_stops_a_note_crossing_it`（AC33・決定 (1)(オ)）: 生きたノート 3 本・上限 1 の写しで、1 本を廃止にする中身も 1 本足す中身も口は通す（0）。上限の内のノートが契約表の行の上限を跨ぐ中身は、そのノートを名指す行で止め（1）、口の標準出力に今の数の行は無く（要件 FR28 の 4 種の行だけ）、同じ中身を書いた置き場の素の床にも同じ行が在る（P-15.2）。
9. `crates/folio/tests/note.rs` の `f207_chapter_cap_does_not_stop_reducing_over_the_cap_but_stops_crossing`（決定 (1)(ク)）: 章の上限 2 の写しで 4 章のノートの違反の字に 4 が無く今の数の行に 4 が在る。4 章 → 3 章の中身は通し、2 章のノートが 3 章になる中身はそのノートを名指して止める。
10. `crates/folio/tests/plan.rs` の `f207_plan_rows_are_summed_over_the_place`（AC33・決定 (1)(イ)）: 計画のノートの計画だけの行 2 で上限 1 は違反 1（置き場を名指す・今の数 2）、上限 2 は合格、見本に足した計画だけの行は数えない、もう 1 本の下書きに 1 行足すと合計 3 で上限 2 を超える。

- **既存の歯の期待の字の直し（3 か所）。** `crates/folio/tests/note.rs` の `f179_floor_counts_chapters_with_the_same_cap_and_words_as_the_face` の期待の字、`crates/folio/tests/face_note.rs` の `f179_face_reads_the_cap_from_the_keyed_row_whatever_its_id` の断りの字（新しい字・字に 14 が無い・次の行の今の数）、`crates/folio/src/note.rs` の単体 `f179_chapters_add_one_figure_chapter_and_over_cap_names_the_counts` は章の数えだけの `f179_chapters_add_one_figure_chapter` に（over_cap の字は 2 の歯へ移る）。`crates/folio/src/rules.rs` の `f179_` の 2 本は `chapter_cap` を cap（欄 key は NOTE_CHAPTERS） に、`KEYS` の期待を 6 つに。`crates/folio/tests/schema_docs.rs` の規則の表の生成区間の凍結 anchor の定数 3 つ（行数・byte 数・要約値）。
- **RED（base 80a413d に歯の file と fixture と anchor だけを当てた写し・起草の記録の red-*.log）。** `tests/note.rs` は 7 本落ちる（f207_ の 6 本すべてと、期待の字を直した f179_ の 1 本・base は土台の値の行の欄 key を閉じた一覧の外として種別 schema の違反にし、章の上限の字は今の数を持つ）。`tests/plan.rs` は 8 本落ちる（f207_ の 1 本と、同じ土台を使う f183_ の 7 本・土台の値の行の欄 key が base で種別 schema の違反になるため）。`tests/face_note.rs` は 1 本（f179_ の断りの字）、`tests/schema_docs.rs` は 4 本（規則の表の生成区間が新しい凍結 anchor と違う）落ちる。単体の f207_ の 3 本は新しい関数を呼ぶので base では組めない（0 本）。見本では全部緑。

### (d) 採らなかった形

1. **違反の字に行の id を入れる。** 決定 (1)(オ) は「上限の行の id か欄 key の値」を許すが、行の id は置き場ごとに違い（folio2 の R-23 は tsuzuri では別の番号）、行を付け替える編集で字が変わる。欄 key は置き場を越えて同じで、道具が行を引く名でもある。
2. **今の数を違反の字の後ろに括弧で残す・数を丸める。** 字が前後で変わる限り B-1 は残る。今の数は判定に入らない別の行にする（決定 (1)(オ)）。
3. **編集時の口の標準出力にも今の数の行を出す。** 要件 FR28 の規範文が列挙する口の標準出力の行（止める行・# まだ分からない の行・要約の 1 行・つながりの行）に無い行を足すことになる（席の裁定 2026-09-29・起草の最初の見本 f796219 はこの形だった）。
4. **面の生成器もこの 3 つの上限で止める。** 決定 (1)(カ) が採らない（量の上限は面の形に関わらない）。面が後で数を出すときは `growth` と rules.rs の cap を呼ぶ。
5. **`chapter_cap` を残して 3 つの読み手を足す。** 読み手が 4 つになり、値の形の判定が割れる（P-6.3）。`cap` 1 つに寄せ、字は章の上限の今の字を保つ。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本で workspace の nextest を撃つと、§1 (c) に書いた期待の字の直しのほかに、床の土台の数を字で持つ歯 20 本が落ちた（graph 6・graph_notes 2・graph_summary 1〔凍結 anchor で直る〕・hello 2・polarity 3・ruling 1・emit_rulings 4・floor_cases 1〔case mentions-rule-row-counts-the-frozen-base〕・起草の記録の s2-nextest.log）。どれも土台に行 3 本（節点 3・辺 6・裁定の欄 3・閾値の行 3）が入った分の数で、§1 (b) 6 の直しで全部緑になる。folio2 の本流の写しを使う歯（tests/note.rs の Work::new ほか）は、本流の値 1 と見本 2 本だけの design-intent で落ちない（本流の生きたノートは 0 本）。見本の workspace の nextest は 1169 / 1169（base 1159 + f207_ の 10 本）・clippy 0 警告・床 4 本 rc 0（check 合格 0/0）・`folio build` は 40 file のままで、base と中身が違うのは constitution.html の 1 file だけ（行 R-23〜R-25 の欄 key と注の末尾の 1 文・起草の記録の impl-9585bb7-*.log）。
2. **突然変異（見本の src だけを 1 通りずつ変え、単体と tests/ の 4 本〔note・plan・face_note・schema_docs〕を撃った・見本 9585bb7 で撃った・起草の記録の mut.log と mut-recount-9585bb7.log）。** 変異 17 通りとも、どれかの歯が落ちた（生き残り 0・起草の記録の mut.log と mut-recount.log〔落ちた歯の名の数え直し〕）。

| 変異 | 変えたもの | 落ちた歯（本） |
| --- | --- | ---: |
| M1 | 床が数の上限を数えない（`check_growth` を呼ばない） | 6（note 5・plan 1） |
| M2 | 見本も生きたノートに数える | 38（単体 1・note 36・plan 1） |
| M3 | 発効を数えない（下書きだけ） | 3（単体 1・note 1・plan 1） |
| M4 | 廃止も生きたノートに数える | 3（単体 1・note 2） |
| M5 | 1 本の行の数に節の型を問わない | 11（単体 1・note 9・plan 1） |
| M6 | 計画だけの行を合計でなく 1 本の最大で数える | 2（単体 1・plan 1） |
| M7 | 上限ちょうども違反にする（境の取り違え） | 14（単体 1・face_note 1・note 11・plan 1） |
| M8 | 違反の字に今の数を入れる（前の章の上限の字の形） | 9（単体 1・face_note 1・note 6・plan 1） |
| M9 | 行が読めない数えを黙って飛ばす | 1（note 1） |
| M10 | 本数の行の単位を「行」にする（単位の取り違え） | 38（単体 2・note 35・plan 1） |
| M11 | `KEYS` を 5 つにする | 単体の歯が組めない（`KEYS` の期待が 6 つ） |
| M12 | 生きたノートが 0 本なら行を読まない | 1（note 1） |
| M13 | 面の断りが今の数の行を出さない | 1（face_note 1） |
| M14 | 床が今の数の行を出さない | 4（note 3・plan 1） |
| M15 | 生成区間の注の字を変える | 16（単体 1・schema_docs 15） |
| M16 | 1 本の行の数を全部の生きたノートの合計で比べる | 4（note 4） |
| M17 | 欄 key の床が 3 つを閉じた一覧の外として数える | 49（note 41・plan 8） |
2a. **編集時の口の答え（見本 9585bb7 の binary・床の土台の写しで生きたノート 3 本・上限 1・章の上限 2・起草の記録の proposed-check-9585bb7.log）。** 超えたまま減らす（1 本を廃止・3 → 2）= 通す 0、超えたまま足す（3 → 4）= 通す 0、上限の内のノートが契約表の行の上限を跨ぐ = 止める 1（止める行はそのノートの file を名指す 1 行）、章の上限を超えたまま減らす（4 章 → 3 章）= 通す 0、章の上限を跨ぐ（2 章 → 3 章）= 止める 1。5 つとも口の標準出力は止める行と要約の 1 行だけで、今の数の行は無い（今の数は素の床の標準エラーに出る）。
3. **外の置き場（tsuzuri の写し・本物の main 93389e3・起草の記録の tz-207-9585bb7.log・見本 9585bb7 で撃ち直して答えは前と同じ）。**
   - 行の無い写し（今の本物の木）: base の床は合格、見本の床は まだ分からない 3（「rules.yaml: 設計ノートの数の上限 live-notes が読めない: 欄 key が live-notes の閾値の行が無い」ほか 2）。`folio build` は床が まだ分からない でも書く（93 file・base と sha が同じ）。`schema --check` は生成区間の違い（4355 byte ≠ 5225 byte）で 1。
   - 値の行 3 本（R-29 live-notes 70 本 以下・R-30 note-rows 32 行 以下・R-31 plan-rows 26 行 以下・条 P-7）と条 P-7 の relations.rules に 3 つを足した写し: base の床は種別 schema の違反 3（欄 key が閉じた一覧に無い）で build は 1 file も書かない。見本の床は合格、`schema --write` の後の `schema --check` も 0、build は 93 file（base と sha が違うのは constitution.html と index.html の 2 file・値の行と関係の欄が増えた分）。
   - live-notes の値を実測より 1 小さい 69 にすると、見本の床は違反 1「[note] design-note/: 生きたノートが多すぎる（live-notes の上限 69 本 以下）」と今の数の行「design-note/: 生きたノートの今の数 70（…）」。
   - よって外の置き場は、folio の組み立てを上げる同じ commit で値の行 3 本（入れる時点の実測以上）・条の relations.rules・`folio schema --write` の生成区間を入れる（ADR-35 決定 (7) の知らせのとおり）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・縮む file・消す file・新しい dir は無い。`crates/folio/src/rules.rs` と `crates/folio/src/note.rs` は単体の歯 f207_ を持つので verify の `--bin folio` の scope。内訳は src 5（rules.rs・note.rs・face_note.rs・verdict.rs・main.rs）・歯の file 4（tests/note.rs・plan.rs・face_note.rs・schema_docs.rs）・土台の数を持つ歯の file 6（tests/graph.rs・graph_notes.rs・hello.rs・polarity.rs・ruling.rs・emit_rulings.rs）・設計文書 1（design-intent/rules.yaml）・床の土台 2・凍結 anchor 4・床の case 1（tests/floor_cases.yaml）。verify の `--test note`・`--test plan`・`--test face_note`・`--test schema_docs` の scope の file は全部 write-set に在る。
2. **余地（CapHeadroom）。** write-set の src の 5 本（base の木と見本 9585bb7・python と awk の 2 実装で一致・起草の記録の cap-9585bb7.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/rules.rs` | 644 | 856 | 722（+78・単体の歯 2 本を含む） | 778 |
| `crates/folio/src/note.rs` | 1005 | 495 | 1082（+77・単体の歯 1 本を含む） | 418 |
| `crates/folio/src/face_note.rs` | 1037 | 463 | 1038（+1） | 462 |
| `crates/folio/src/verdict.rs` | 113 | 1387 | 120（+7） | 1380 |
| `crates/folio/src/main.rs` | 782 | 718 | 786（+4） | 714 |

3. **size は M。** file ごとの増分の最大は rules.rs の +78（単体の歯 2 本を含む）で S の見積 100 に近く、歯と fixture の手直しが多い（便 179 と同じ種類）。余地の最小（note.rs の本便の後 418）は M の 300 を超える。
4. **verify は 8 行**で、done の 8 つの塊と 1 対 1 に揃える。見本 9585bb7 で 8 行とも rc 0（3・6・1・47・35・28・11 本と clippy 0 警告・起草の記録の verify-impl-9585bb7.log）。
   1. `cargo nextest run -p folio --bin folio f207_` = (c) 1〜3（3 本）。
   2. `cargo nextest run -p folio --test note f207_` = (c) 4〜9（6 本）。
   3. `cargo nextest run -p folio --test plan f207_` = (c) 10（1 本）。
   4. `cargo nextest run -p folio --test note` = 設計ノートの床の歯の全部（章の上限の f179_ の期待の字の直しを含む・folio2 の本流の写しの床は合格のまま）。
   5. `cargo nextest run -p folio --test face_note` = 面の生成器の歯の全部（章の上限の断りの字と今の数の行・凍結の面の byte 一致）。
   6. `cargo nextest run -p folio --test schema_docs` = 規則の表の生成区間が新しい凍結 anchor と byte で一致し、書き直しが冪等。
   7. `cargo nextest run -p folio --bin folio rules::tests` = 床の木の導出と凍結 anchor の byte 一致・欄 key の歯（f179_ の直しを含む）。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   床の土台を使うほかの歯（proposed・graph・hello・ruling ほか）は common-verify の workspace の nextest が撃つ。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり。write-set に `design-intent/rules.yaml`（行 R-23〜R-25 の欄 key と注の末尾の 1 文・生成区間）が在るので門の対象（規則の表の行 D-12）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d207 の本契約で 契約に起因する断り 0・preflight ok（起草の記録の precheck-final.log・前の 2 回は base に無い新しい名の字面を 10 か所断られ、地の文に直した）。
3. **便 208（行 hc・起草中・契約は枝 docs/d208）とは file が 1 本と、数の意味が 1 つ重なる。** 208 の write-set は `crates/folio/src/graph.rs`・`crates/folio/tests/graph_summary.rs`・`crates/folio/tests/graph_notes.rs` の 3 本。 (1) `crates/folio/tests/graph_notes.rs` は本便も書き換える（土台の要約の行 4 か所）。208 の見本の差分（起草の記録の d208-scripts/c208.patch）を base の木に当てた写しと本便の見本は `git merge-tree` で衝突 0（違う行）。(2) 208 の歯 `f208_the_frozen_base_lines_end_with_the_state_of_their_file`（`crates/folio/tests/graph_summary.rs`）は土台の `folio graph --print --summary` の行を数え、定数 `BEFORE_208 = (191, 114_730, 53a7d113…)` と `[10, 1, 180]` を持つ。本便の後の土台では (194, 115_579, 78eab543a65fa0cc38c0e4b5f465705a8d0b411edb4ba8e5e0273050cdc0ac35) と [10, 1, 183] になる（見本の binary で測った）。**器は write-set の重なる便を並べて運ばないので、208 と本便は順に運ぶ。** 208 が先なら本便は受付の前に `graph_summary.rs` の 2 つの定数を直す行を write-set に足して数え直す。本便が先なら 208 が同じ 2 つを直す。**席の裁定（2026-09-29）: 本便が先。** 208 は本便の着地の木の上で、`BEFORE_208` を (194, 115_579, 78eab543…) に、`[10, 1, 180]` を `[10, 1, 183]` に直す（208 の見本は本便の見本 9585bb7 の上に作り直す）。本便の write-set は 208 の file を足さない。208 の見本の差分を当てた写しに本便の見本を merge した木で、`--test graph_summary` と `--test graph_notes` を撃つと、落ちるのは 208 の歯 `f208_the_frozen_base_lines_end_with_the_state_of_their_file` の 1 本だけ（[10, 1, 183] ≠ [10, 1, 180]）で、その 2 つの定数を上の値に直すと 16 / 16 緑（起草の記録の merged-208.log）。
4. **ほかのレーン。** origin の impl/* と docs/* の枝のうち本流 a5b27df に入っていないものは、便 196（impl/d196 8c8f9ed・着地済みの 196 の載せ替え）と取り下げた便 201（impl/d201）で、impl/d196 と本便の見本の `git merge-tree` は衝突 0。便 203・204 は 80a413d に入っている（本便の base）。
5. **便 209（行 hd・後継の欄の先）は本便の後に起草する**（`crates/folio/src/note.rs` と `crates/folio/tests/note.rs` が重なる・ADR-35 決定 (6)）。
6. **着地の後（外の置き場）。** tsuzuri は folio の組み立てを上げる同じ commit で、値の行 3 本（入れる時点の実測以上・今の写しで 70 本・32 行・26 行）と条の relations.rules に 3 行と `folio schema --write` の生成区間を入れる（(e) 3）。行が無いまま上げると床は まだ分からない 3、行だけ先に入れると今の folio の床が種別 schema の違反 3 で build も止まる。席は tsuzuri へこの 1 行を返す（ADR-35 決定 (7)）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/d207-draft.md`、script と log は同じ dir の `d207-scripts/`。
1. `run.sh <木> <印>`: 組み立て・workspace の nextest（--test-threads 4）・clippy・床 4 本・`folio build --write` の file 数と sha（撃つ前にほかのレーンの nextest が止むのを `waitidle.sh` で待つ・/ の空きが 20 GB を切ったら止める）。
2. `red.sh <base の木> <見本の rev>`: 歯の file 4 本・床の土台の 2 本・凍結 anchor 1 本だけを base に当てて RED。
3. `mut.py <見本の clone>`: 変異 17 通り（M17 は後から足して 1 通りだけ撃った）。
4. `cap.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk）。
5. `verify.sh <木> <印>`: 契約の verify の 8 行。
6. `prep-tz.sh`・`tz-207.sh <base の folio> <見本の folio>`: tsuzuri の写し（hook なし）で、行の無い木・値の行 3 本を足した木・schema --write の後・値を 1 小さくした木を base と見本で撃つ（写しの中だけで書き、最後に戻す）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書・憲法・判断の記録・値域・行の値と裁定・外の置き場の file・台帳への記帳（席）・索引の状態の欄（便 208）・後継の欄の先（便 209）・面が数を出すこと・外部 crate・新しい dir。
2. **言えないこと。** 生きたノートを見本に付け替えると数えから外れる（ADR-35 決定 (4)(カ)・知っている穴）。超えた置き場でさらに足す編集（本数・計画だけの行の合計・既に超えたノートの行）は書く前には止まらない（書く前にも同じ字の違反が在る・事後の床が落とす）。編集時の口の標準出力には今の数の行が無いので、止められた編集の今の数は素の床（`folio check` の標準エラー）で見る。
3. **撤退条件。** (1) 要件書・判断の記録・憲法・値域・行の値を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に §1 (c) の直しと §1 (b) 6 の fixture と anchor のほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、`folio build` の file 数が変われば、止めて席へ返す。(4) tsuzuri の写しで値の行 3 本を足した床が合格にならなければ、止めて席へ返す。

## 2. 範囲

- 入れる: 数の上限の欄 key 3 つと値を読む関数 `cap`・違反の字の関数 `over_cap`・生成区間の注（`rules.rs`）・数える関数 `growth` と床 `check_growth`・章の上限の字（`note.rs`）・今の数の行（`verdict.rs`・`main.rs`〔素の床の標準エラー〕）・面の断りの字（`face_note.rs`）・行 R-23〜R-25 の欄 key と注・生成区間（`design-intent/rules.yaml`）・床の土台の値の行と条の関係・凍結 anchor・歯。
- 入れない: 要件書・憲法・判断の記録・値域・行の値と裁定・索引・導出・面の上限・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| keys | 欄 key | `rules.rs` の `LIVE_NOTES`・`NOTE_ROWS`・`PLAN_ROWS`・`KEYS`（6 つ）・`CAPS` |
| cap | 値を読む口 | `rules.rs` の 新しい関数 cap（引数は規則の表と欄 key）（章の上限と 3 つの上限で 1 つ） |
| words | 違反の字 | `rules.rs` の `over_cap`（名指す先・欄 key・上限の値だけ・今の数は 2 つ目の字） |
| count | 数える口 | `note.rs` の `growth`（生きたノートの本数・1 本の行の数・計画だけの行の合計） |
| floor | 床 | `note.rs` の `check_growth` と章の上限の字 |
| now | 今の数の行 | `verdict.rs` の `counts`・`main.rs` の素の床の標準エラーの `# 今の数: `（編集時の口には出さない） |
| region | 生成区間 | `FLOOR` の `key_note` と `enums.key`・`schema --write`・凍結 anchor |
| teeth | 歯 | 単体 3 本・`tests/note.rs` 6 本・`tests/plan.rs` 1 本・期待の字の直し・床の土台の値の行 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 本流 17736c9（ADR-35・要件書 第 1.57 版・行 R-23〜R-25 の承認の PR #391・便 203・204 を含む）。着地済み。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) 6 の 1 行を返す。便 209 の起草を始める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "hb"
title = "数える口（判断の記録 ADR-35 決定 (1)(ア)〜(ク)・(5)・(6)・要件 FR30 と AC33・台帳 f2-648.275.9 の 3 便の 1 つ）: crates/folio/src/rules.rs の欄 key の閉じた一覧 KEYS に live-notes・note-rows・plan-rows を足して 6 つにし、章の上限の読み手 chapter_cap を 4 つの key で共有する cap（値の形は 正の整数・空白・単位〔章・本・行〕・空白・以下 だけ・行が無い 2 本 形が違うは Err）に置き換え、違反の字の関数 over_cap（名指す先・欄 key・上限の値だけを持ち今の数を持たない字と、今の数の字の組）を置き、床の木の key_note に 2 文を足す。crates/folio/src/note.rs に数える唯一の関数 growth（状態がちょうど draft か effective のノートの本数・1 本ごとの契約表の節の行の数・計画だけの行の節の行の合計）と床 check_growth（読めた設計ノートが 1 本でも在れば 3 行を読み、読めなければ まだ分からない、超えたら種別 note の違反）を足し、章の上限も同じ over_cap の字にする。今の数は Report の counts に積み（verdict.rs）、folio check が標準エラーに # 今の数: の行で出す（main.rs・編集時の口 folio check --proposed の標準出力には出さない＝要件 FR28 の行の列挙のまま・止める比べは変えない）。面の生成器 face_note.rs は章の上限の断りの字を床と同じ字にし今の数を次の行にする。design-intent/rules.yaml は行 R-23〜R-25 に欄 key を足して注の末尾の便 207 までの 1 文を消し（ruling ほかは変えない）、生成区間は folio schema --write で書く。凍結 anchor rules-region.txt は前の anchor に同じ字を手で足し、床の土台に値 99 の 3 行と条 P-7 の関係の欄を足し、土台の索引の凍結 anchor 3 本と土台の数を字で持つ歯 6 本と床の case 1 つを直す。歯は f207_ の 10 本（単体 3・tests/note.rs 6・tests/plan.rs 1）と f179_ ほかの期待の字の直し。実装の見本は origin の枝 impl/d207 の commit 9585bb7（親 17736c9）で、作業者は base に対する見本の差分を write-set の file に当て、write-set の外は変えない。base = 本流 17736c9（ADR-35 と要件書 第 1.57 版の承認の着地・PR #391）"
req = ["FR30", "FR19", "FR9"]
section = "1"
write-set = ["crates/folio/src/rules.rs", "crates/folio/src/note.rs", "crates/folio/src/face_note.rs", "crates/folio/src/verdict.rs", "crates/folio/src/main.rs", "crates/folio/tests/note.rs", "crates/folio/tests/plan.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/graph_notes.rs", "crates/folio/tests/hello.rs", "crates/folio/tests/polarity.rs", "crates/folio/tests/ruling.rs", "crates/folio/tests/emit_rulings.rs", "design-intent/rules.yaml", "tests/fixtures/floor_base/design-intent/rules.yaml", "tests/fixtures/floor_base/design-intent/constitution.yaml", "tests/fixtures/schema/rules-region.txt", "tests/fixtures/schema/graph-anchor.txt", "tests/fixtures/schema/graph-digest-anchor.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/floor_cases.yaml"]
verify = ["cargo nextest run -p folio --bin folio f207_", "cargo nextest run -p folio --test note f207_", "cargo nextest run -p folio --test plan f207_", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --bin folio rules::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "binary の単体の f207_ の 3 本（cap が 3 つの key を行の id に依らず単位つきで読み形違い 無い 2 本は Err・over_cap の字は数が変わっても同じで今の数は 2 つ目の字だけ・growth は draft と effective だけを数え 1 本の行は契約表の節の和で計画だけの行は合計）が緑、tests/note.rs の f207_ の 6 本（床の土台の写しで値 1 のとき生きたノート 2 本で違反 1 と今の数の行・廃止と見本は数えない・1 本ごとの契約表の行・3 つの key の行が無い 2 本 形違いは まだ分からない で 0 本なら読まない・上限を 2 超えた写しで廃止と追加の中身を口が通し跨ぐ中身は止め口の標準出力に今の数の行は無い・章の上限も同じ）が緑、tests/plan.rs の f207_ の 1 本（計画だけの行は置き場の合計で数え見本は数えない）が緑、tests/note.rs の歯の全部（章の上限の f179_ の新しい字・folio2 の本流の写しの床は合格のまま）が緑、tests/face_note.rs の歯の全部（章の上限の断りの字に今の数が無く次の # の行に在る・凍結の面の byte 一致）が緑、tests/schema_docs.rs の歯の全部（規則の表の生成区間が新しい凍結 anchor と byte で一致）が緑、binary の単体の rules::tests の全部（床の木の導出と凍結 anchor の byte 一致・欄 key の 6 つ）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
