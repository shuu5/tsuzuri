# 設計: 便 209 — 後継の欄の先: 設計ノートの後継が要件・非機能要件・判断の記録も指せ、床が先の実在を数え、面の後継のリンクが先の面へ飛ぶ

- 要件: FR32（設計ノートの後継の先・要件書 第 1.57 版）・FR9（設計ノートの面を 1 つの型で生成する）・FR19（設計ノートの欄の決まりの生成区間を床の定数から写す）。本流の要件書に在る id で、規範文・確かめ方・受入基準（AC35）は変えない。
- 判断の記録: ADR-35 決定 (3)(ア)〜(カ)・(6)（発効 2026-09-29 13:19 JST・持ち主の逐語「１．全部推奨で」・台帳 f2-648.275.9）。決定の字は変えない。
- 条: P-4.2（判断の記録の置き場が読めなければ まだ分からない）・P-6.3（形の読み分けは床と面が 1 つの関数・id の読み手は床のほかの口と同じ）・P-7.2（廃止は状態で表し、後継で中身の移り先を指す）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `hd` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 8 本（§1 (f) 1）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: **0（通す）**。本流の binary（main root の `target/debug/folio`・18:02 の組み立て）を枝 docs/d209 の木（本流 0914e69 の上）で撃った `folio ceiling --gate --dir design-intent --write-set <write-set の 8 本>` の答え「folio ceiling: 通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）」（起草の記録の gate-0914e69.log・載せ直す前の 17736c9 の上でも同じ答え〔gate.log〕）。見本の全体の差分 c209.patch は 0914e69 の木に `git apply --check` で当たる。
- 前提: **base = 本流 0914e69（便 207〔行 hb〕の着地）**（ADR-35 決定 (6)「209 は 207 の着地の後に数え直して運ぶ」・`crates/folio/src/note.rs`・`crates/folio/src/face_note.rs`・`crates/folio/tests/note.rs`・`crates/folio/tests/face_note.rs` が 207 と重なる）。0914e69 の木は、本便の見本の親（207 の見本 9585bb7・起草を始めた 2026-09-29 15:11 JST の `git ls-remote` の値）の木と `crates`・`design-intent`・`tests` で byte 同じ（`git diff 9585bb7 0914e69 -- crates design-intent tests` が 0 行）なので、見本の付け替えは要らず、この契約の数（9585bb7 の木とその上の本便の見本の実測・参考値・規則の表の行 D-13）は 0914e69 の上でもそのまま生きる。**受付は便 208（行 hc）の着地の後**（208 が先に着地する・重なり 0 で差分はそのまま当たる見込み・当たりは 208 の着地の後に席が確かめる）。
- 実装の見本: origin の枝 `impl/d209`（commit **36555ca**・親 9585bb7・1 commit）。`git diff 9585bb7 36555ca` が便の全体の差分（8 file・+256 −26・29,518 byte）。**作業者は、base に対する見本の差分を write-set の file に当ててよく、write-set の外は変えない。**
- 並行の便との重なり: 便 207（行 hb）とは file 4 本が重なる（着地済み・本便の base）。便 208（行 hc）とは 0（§1 (g) 3・208 が先に着地する）。

## 1. 設計

### (a) いま起きていること（base の木〔本流 0914e69 と同じ木の 207 の見本 9585bb7〕の実測・参考値）

1. **床は後継の先を設計ノートの id としか読まない。** `crates/folio/src/note.rs:647` の check_meta の中の繰り返し（前 supersedes と後継 superseded_by の 2 つの欄）は、値が置き場の設計ノートの id の集合（`note.rs:113` の note_ids）に在り自分自身でなければ通し、自分自身なら形の違反（1 つの file の違反・編集時の口も止める）、ほかはつながりの違反（`Report::link`・編集時の口は止めない・ADR-33 決定 (2)）にする。字は「{file}: meta.{欄}「{値}」の設計ノートが実在しない」。よって廃止のノートの後継に要件（例 FR19）や判断の記録（例 ADR-10）を書くと、在る id でもつながりの違反になる。
2. **面は後継のリンクを必ず設計ノートの面へ張る。** `crates/folio/src/face_note.rs:412` の status は、廃止の行を `廃止 → 後継 <a class="xref" href="note-{値}.html">{値}</a>` にする（`face_note.rs:430`・実在は確かめない）。後継が FR2 なら行き先は無い file `note-FR2.html` になる。
3. **id の読み手は在る。** 要件書の id は `note.rs:461` の requirement_ids（引数は要件書・`refs::SRS_ID_SECTIONS` の 7 つの節〔goals・requirements・nonfunctional・acceptance・constraints・actors・outputs〕の id を読む・契約表の行の req の解決先）、判断の記録の id は `crates/folio/src/link.rs:244` の adr_ids（`adr::check_adr` が読んだ記録の id・`note.rs:455` の base_known_ids も同じ関数）。面の参照 id の解き方は `face_note.rs:443` の resolve（FR・NFR と数字 → 要件書の面 `srs.html#<小字の id>`〔面の文脈の要件の id に在るときだけ〕・ADR- と数字 → 判断の記録の面 `adr-<数>.html`〔置き場の `adr/<id>.yaml` が file のときだけ〕・無ければ None）と、`face_note.rs:513` の id_link（None なら「{id}（まだ分からない）」）。
4. **欄の決まりの生成区間は後継の先を書いていない。** `crates/folio/src/floor_note.rs` の床の木 `FLOOR` の doc_meta の注は、id_note（改名は新しい id と旧 id の廃止〔status retired・superseded_by〕で表す）と status_note（retired は superseded_by 必須）だけで、後継が指せる先の種類を書かない（`design-intent/design-note/schema.yaml` の生成区間も同じ・growth-verify 1-6）。
5. **数（参考値）。** folio2 の design-note/ は見本 2 本（example.yaml・figures.yaml）で、前と後継の欄を持たない。tsuzuri の写し（本物の main ee6c10a の .git の写しから clone・hook なし）は design-note/ の設計ノート 78 本で、前と後継の欄の行 0・廃止のノート 0（起草の記録の tz-209.log）。
6. **base の数（参考値）。** workspace の nextest 1169 / 1169・clippy 0 警告（どちらも d207-draft が同じ commit 9585bb7 で撃った log `d207-scripts/logs/impl-9585bb7-nextest.log`・`-clippy.log`）・床 4 本 rc 0（check 合格 0/0・本便の起草で撃ち直した base-floor4.log）・`folio build` 40 file（base-site.log）。

### (b) 直す先

1. **形の読み分けを 1 つの関数にする（`crates/folio/src/note.rs`・`crates/folio/src/floor_note.rs`・ADR-35 決定 (3)(イ)）。**
   - `floor_note.rs` に後継の先の形の定数 3 つを置く: SUCCESSOR_REQUIREMENT（FR・NFR）・SUCCESSOR_ADR（ADR-）・SUCCESSOR_SECTIONS（requirements・nonfunctional）。
   - `note.rs` に先の種類 Successor（要件・判断の記録・設計ノートの 3 つ）と、読み分ける唯一の関数 successor（引数は値の字）を置く。頭が FR か NFR で後ろが半角の数字だけなら要件、頭が ADR- で後ろが半角の数字だけなら判断の記録、ほかは設計ノートの id（英小字で始まる形なので重ならない・受入基準の id や条の id も設計ノートの id として読み、今と同じくつながりの違反になる）。面も同じ関数を呼ぶ（P-6.3）。
2. **床が先の実在を数える（`crates/folio/src/note.rs`・ADR-35 決定 (3)(ア)(ウ)）。**
   - check_note で、前と後継の先の母集団（設計ノートの id・要件書の要件と非機能要件の節の id・判断の記録の id）を 1 つの束にして check_one から check_meta へ渡す（今の note_ids の引数を置き換える）。
   - 要件の id は今の読み手 requirement_ids を節の一覧を引数に取る形にして読む（契約表の行の req は今と同じ `refs::SRS_ID_SECTIONS`・後継の先は SUCCESSOR_SECTIONS・新しい読み手を作らない）。判断の記録の id は今の読み手 `link::adr_ids` で読む（check_note が受ける判断の記録が無い〔adr/ が読めない〕ときは無し）。
   - check_meta の繰り返し: 前（supersedes）は今のまま設計ノートの id だけで照らす。後継（superseded_by）は successor で種類を読み分け、要件なら要件の id の集合、判断の記録なら判断の記録の id の集合、設計ノートなら今と同じ集合（自分自身は数えない）に照らす。無ければ字「{file}: meta.{欄}「{値}」の{要件|判断の記録|設計ノート}が実在しない」で、自分自身なら今と同じ形の違反、ほかは今と同じつながりの違反（編集時の口は止めず、事後の床が落とす）。先の状態（発効か・廃止か）は数えない。
   - 判断の記録を指す後継で判断の記録が読めない置き場は、字「{file}: meta.superseded_by「{値}」の判断の記録を数えられない（adr/ が読めない）」の まだ分からない とする（違反にも合格にもしない・P-4.2・adr/ の不在はほかの まだ分からない の行も出る）。
   - 設計ノートでない file の path は先の種類に入れない（ADR-35 決定 (3)(エ)・path の形の字は設計ノートの id として照らされ、今と同じつながりの違反）。
3. **面の後継のリンク（`crates/folio/src/face_note.rs`・ADR-35 決定 (3)(オ)）。** status に参照 id の文脈を渡し（引数に Env を足す・呼ぶ所は 1 か所）、廃止の行の後継を successor で読み分ける。設計ノートなら今のまま `note-{値}.html` へ（実在を確かめない）。要件と判断の記録なら今の id_link（resolve で要件書の面の要件の場所か判断の記録の面へ解き、解けなければリンクを張らず「{値}（まだ分からない）」）。面は導出できる（0）のまま。
4. **生成区間の注（`crates/folio/src/floor_note.rs`・`design-intent/design-note/schema.yaml`・ADR-35 決定 (3)(カ)）。** 床の木の doc_meta の status_note の後に注 supersede_note を 1 行足す（字は次の項）。生成区間は `folio schema --dir design-intent --write` の導出で書く（手で書かない）。注は床の突き合わせの外（`_note` で終わる欄）なので、`folio check` の数は変わらない。外の置き場へは注の字から folio2 の番号の括弧の項（判断の記録の番号と決定の番号）が落ちて写る（`floor.rs` の text_for・便 174 の決まり）。
   - 字: 「後継（superseded_by）の先は、同じ置き場の設計ノートの id のほか、要件の id と判断の記録の id を指せる（判断の記録 ADR-35 決定 (3)）。先の種類は字の形で読み分け、頭が FR か NFR で後ろが半角の数字なら要件書の要件の節（requirements）か非機能要件の節（nonfunctional）の id、頭が ADR- で後ろが半角の数字なら判断の記録（adr/）の id、ほかは設計ノートの id とする（設計ノートの id は英小字で始まるので重ならない）。床は先の実在を数え、無ければつながりの違反、自分自身を指せば形の違反とし、先の状態は数えない。前（supersedes）は設計ノートの id だけを指す。設計ノートでない file の path は入れない。面は廃止の行の後継のリンクを先の種類の面（要件書の面の要件の場所・判断の記録の面・設計ノートの面）へ張り、要件と判断の記録の先が解けなければリンクを張らず「まだ分からない」を添える」
5. **凍結 anchor と数の定数。** `tests/fixtures/schema/note-region.txt` は、前の anchor の status_note の表の直後に同じ字の 1 行を手で挿入する（導出の出力を写さない）。`crates/folio/tests/schema.rs` の生成区間の数の定数 3 つ（行数 165 → 166・byte 数 20,312 → 21,426・sha256 789ff974… → b3c3f8b7…）を直す。tests/schema.rs は base で器の式 700 行ちょうど（`tests/schema_docs.rs` の f89 の上限）なので行を足さず、由来の注は既存の注の行の末尾に寄せる（§1 (e) 1）。床の土台 `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の注は今も本流と揃っていない（床は注を突き合わせない）ので直さない。
6. **変えないもの。** 要件書・憲法・判断の記録・欄の決まりの型付きの欄（注の外）・状態の値域・前（supersedes）の意味・設計ノートの面の凍結の面（expected-note.html・見本は下書き）・判断の記録の面・索引（索引は前と後継の欄を読まない）・導出・`folio build` の出力（folio2）・外部 crate。

### (c) 歯（f209_・base で 0 件・9 本）

絞り込みの語 `f209_` は base で 0 件（`git grep -n f209_ -- crates` が 0 行）。`tests/note.rs` の歯は、便 207 が置いた床の土台の写し（growth_base・要件 FR1〜FR19・非機能要件 NFR1〜NFR3・判断の記録 ADR-1〜ADR-10）に廃止のノートを手書きで足す（growth_note の廃止の後継の字を置き換える）。無い先には folio2 の本流に在って土台に無い id（FR32・ADR-35）を使い、土台の置き場を読むことを確かめる（folio2 の本流の値に依らない）。

1. `crates/folio/src/note.rs` の単体 `f209_successor_reads_the_kind_from_the_shape`: FR1・FR32・NFR3・NFR10 は要件、ADR-1・ADR-35 は判断の記録、example・fr1・FR・NFR・ADR-・ADR35・AC35・CON1・P-7・R-23・FR3a・全角の数字・adr-35・xFR1 は設計ノートの id。
2. `crates/folio/tests/note.rs` の `f209_successor_naming_an_existing_requirement_nonfunctional_or_adr_passes`（AC35）: 後継を FR19・NFR3・ADR-10・ADR-1・example にした廃止のノートで、それぞれ合格（違反 0）。先が廃止のノートでも合格（先の状態は数えない）。
3. `crates/folio/tests/note.rs` の `f209_missing_requirement_or_adr_successor_is_one_link_violation`（AC35・決定 (3)(ウ)）: 後継を FR32・NFR4・ADR-35・AC1 にすると、それぞれ違反ちょうど 1「[note] design-note/gone.yaml: meta.superseded_by「FR32」の要件が実在しない」（NFR4 も要件・ADR-35 は判断の記録・AC1 は設計ノート）。同じ中身を編集時の口に渡すと通し（0・止める行なし）、つながりの行「# つながり（編集は止めない・事後の床が数える）: …」に同じ字が在る。
4. `crates/folio/tests/note.rs` の `f209_requirement_successor_reads_only_the_requirement_sections`（決定 (3)(ウ)）: 目的の節に要件の形の id（FR99）の行を足した写しで、後継を FR99 にすると要件の不在の違反ちょうど 1（要件の先は要件と非機能要件の節だけ）。
5. `crates/folio/tests/note.rs` の `f209_successor_pointing_at_itself_stays_a_shape_violation`（AC35）: 後継が自分自身の id は違反ちょうど 1「…meta.superseded_by「gone」の設計ノートが実在しない」で、編集時の口は同じ字で止める（1）。
6. `crates/folio/tests/note.rs` の `f209_supersedes_still_names_design_notes_only`（AC35・決定 (3)(ア)）: 前（supersedes）に在る要件 FR19・在る判断の記録 ADR-10 を書くと、それぞれ設計ノートの不在の違反ちょうど 1。同じ中身を編集時の口に渡すと通し（0）、つながりの行「# つながり（編集は止めない・事後の床が数える）: …」に同じ字が在る（前の欄の不在もつながり・独立の検証の変異 W1 を落とす）。
7. `crates/folio/tests/note.rs` の `f209_adr_successor_without_the_adr_place_is_unknown`（決定 (3)(ウ)・P-4.2）: adr/ を消した写しで後継 ADR-10 は、まだ分からない の行「design-note/gone.yaml: meta.superseded_by「ADR-10」の判断の記録を数えられない（adr/ が読めない）」が在り、後継の違反は無い（終了は 0 でない）。
8. `crates/folio/tests/face_note.rs` の `f209_retired_successor_links_to_the_srs_and_adr_faces`（AC35）: 面の fixture の見本を廃止にし、後継 FR2 → `href="srs.html#fr2"`・NFR1 → `srs.html#nfr1`・ADR-2（写しの adr/ に fixture の ADR-2.yaml を置く）→ `adr-2.html`・full2 → `note-full2.html`（今のまま）の `廃止 → 後継 <a class="xref" href="…">…</a>`。
9. `crates/folio/tests/face_note.rs` の `f209_unresolved_successor_has_no_link_and_is_marked_unknown`（AC35・決定 (3)(オ)）: 要件書に無い FR9・写しの adr/ に無い ADR-9・adr/ の無い写しの ADR-2 は「廃止 → 後継 {値}（まだ分からない）」で、行き先の file 名（`srs.html#fr9`・`adr-9.html`・`adr-2.html`・`note-{値}.html`）を持たない。面は導出できる（0）。

- **既存の歯の直し。** `crates/folio/tests/schema.rs` の設計ノートの生成区間の数の定数 3 つと由来の注の行の末尾（§1 (b) 5・行数は増やさない）。ほかの既存の歯の本文は変えない。
- **RED（base 9585bb7 に歯の file 3 本と凍結 anchor 1 本だけを当てた写し・起草の記録の red-*.log）。** `tests/note.rs` は 4 本落ちる（f209_ の 6 本のうち、在る先で合格・無い先の字・要件の節だけ・判断の記録が読めない の 4 本・base は後継を設計ノートの id としか読まない）。残る 2 本（自分自身は形の違反・前は設計ノートの id だけ）は今の振る舞いを保つ歯で、base でも緑。`tests/face_note.rs` は f209_ の 2 本とも落ちる（base は後継のリンクを必ず `note-{値}.html` にする）。`tests/schema.rs` は 4 本落ちる（設計ノートの生成区間が新しい凍結 anchor と数の定数に合わない）。note.rs の単体の歯の区間は 1 本落ちる（note_floor_derives_the_frozen_anchor_byte_for_byte・base の床の木に注が無い）。単体の f209_ は base の関数 successor を呼ぶので当てない。

### (d) 採らなかった形

1. **要件の先を要件書の id を持つ節の全部（`refs::SRS_ID_SECTIONS` の 7 つ）で照らす。** 決定 (3)(ウ) は要件の節と非機能要件の節に限る。形の読み分けだけでは、目的の節などに要件の形の id が在る置き場で割れる（§1 (c) 4 の歯が落とす）。読み手は同じ requirement_ids に節の一覧を渡して共有する。
2. **面の判断の記録の先を床と同じ判断の記録の id の集合で解く。** 面の生成器は床の読んだ記録を持たず、今の参照 id の解き方（`adr/<id>.yaml` が file か）で解くのが決定 (3)(オ) の字。面に 2 つ目の解き方を作らない。
3. **判断の記録が読めない置き場の後継 ADR-n をつながりの違反にする。** 在るかどうかを数えていないのに「実在しない」と言うことになる（P-4.1・P-4.2）。まだ分からない にする（席の裁定 2026-09-29）。前例は割れている: 同じ `note.rs` の base_known_ids（ほかの参照 id の母集団）は、adr/ が読めない写しで判断の記録の id を足さないので、契約表の行の ref・図の refs・散文の参照 id の ADR-n を「実在しない」のつながりの違反にする（独立の検証の実測で base も見本も 4 件・全体の終了はどちらも 2）。本便は後継だけを まだ分からない にし、base_known_ids は変えない（割れを揃えるかは席が別に控える）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本 36555ca で workspace の nextest 1178 / 1178（base の 1169 に f209_ の 9 本）・clippy 0 警告・床 4 本 rc 0（check 合格 0/0）・`folio build` 40 file で base と sha が全部同じ（起草の記録の impl-36555ca-*.log）。§1 (c) の tests/schema.rs の定数のほかに落ちる既存の歯は無い。途中の見本 599f9b1 では `crates/folio/tests/schema_docs.rs` の f89_schema_teeth_are_split_and_under_the_cap が 1 本落ちた（tests/schema.rs の器の式〔各行 ceil(字数 / 120) の和〕の上限 700 に対し、base が 700 ちょうどで、由来の注を 1 行足して 701・本文は impl-599f9b1-nextest.log）。由来の注は新しい行にせず、既存の注の行（便 130 の行・81 字）の末尾に「便 209: 注 supersede_note の 1 行。」を寄せて 700 に戻した（36555ca）。
2. **突然変異（見本 36555ca の src だけを 1 通りずつ変え、単体〔f209_ と note::tests〕と tests/ の 3 本〔note・face_note・schema〕を撃った・起草の記録の mut.log）。** 変異 16 通りとも、どれかの歯が落ちた（生き残り 0）。M16 は独立の検証が見つけた生き残り W1 で、(c) 6 の歯に編集時の口の 3 行を足して落とすようにした（検証役の fix-w1.py と同じ直し・起草の記録の mut.log）。

| 変異 | 変えたもの | 落ちた歯（本） |
| --- | --- | ---: |
| M1 | 形を読み分けない（後継を全部 設計ノートの id として読む＝base の形） | 7（単体 1・note 4・face_note 2） |
| M2 | 要件の先を要件書の id を持つ節の全部（7 つ）で照らす | 1（note 1・要件の節だけの歯） |
| M3 | 前（supersedes）も形で読み分ける | 1（note 1） |
| M4 | 判断の記録が読めない置き場の後継 ADR-n をつながりの違反にする | 1（note 1） |
| M5 | 判断の記録が読めない置き場の後継 ADR-n を黙って通す | 1（note 1） |
| M6 | 要件の先を設計ノートの id の集合で照らす | 1（note 1） |
| M7 | 自分自身を指す後継もつながり（編集時の口が止めない）にする | 1（note 1） |
| M8 | 面が要件と判断の記録の後継も設計ノートの面へリンクする（base の形） | 2（face_note 2） |
| M9 | 面が解けない先にもリンクを張る（まだ分からない を添えない） | 1（face_note 1） |
| M10 | 非機能要件の形（NFR）を要件として読まない | 4（単体 1・note 2・face_note 1） |
| M11 | 判断の記録の形に「-」の無い ADR も入れる | 1（単体 1） |
| M12 | 頭の後ろが数字だけかを問わない | 1（単体 1） |
| M13 | 生成区間の注の字を変える（床の木と凍結 anchor が割れる） | 5（単体 1・schema 4） |
| M14 | 要件の先の節から非機能要件の節を外す | 1（note 1） |
| M15 | 面が判断の記録の後継だけ設計ノートの面へリンクする | 2（face_note 2） |
| M16 | 前の欄の不在を形の違反にする（編集時の口が止める・検証役の W1） | 1（note 1・(c) 6） |
3. **外の置き場（tsuzuri の写し・本物の main ee6c10a・起草の記録の tz-209.log・見本の組み立ては src が 36555ca と同じ 599f9b1）。** 行の無い今の木では、base（9585bb7）と見本の床の出力は標準出力も標準エラーも byte で同じ（まだ分からない 3〔便 207 の数の上限の行が無い〕・違反 0）で、`folio build` は 101 file・sha の差 0。値の行 3 本（今の実測 78 本・32 行・20 行）と条 P-7 の relations.rules を足した木でも、base と見本の床は byte で同じ（合格 0/0）・build 101 file・sha の差 0。見本の `schema --dir design-intent --write` は設計ノートの欄の決まりに注 supersede_note の 1 行を足し（外の置き場の字は括弧の「判断の記録 ADR-35 決定 (3)」が落ちる）、その後の `schema --check` は 0・床は同じ・build の sha の差 0。`schema --check` は見本では design-note/schema.yaml の生成区間の違い（17,049 byte ≠ 18,124 byte）で 1（base は便 207 の rules.yaml の違いで 1）。よって今の tsuzuri の後継の欄は 0 行で、**床の数は変わらない**。
   - 独立の検証（d209-verify.md の N-6）が今の HEAD c0acf55（設計ノート 80 本・build 104 file）で撃ち直し、後継の欄 0 行・廃止 0・床の出力は base と見本で byte 同じ・schema --write で注 1 行・その後 schema --check 0 の結論は同じだった。
   - 試し（起草の記録の tz-209-retire.log）: 値の行を足した木で発効のノート 1 本（surface-wave12b）を廃止にし、後継を FR1（在る要件）・ADR-10（在る判断の記録）にすると、base は違反 1「…meta.superseded_by「FR1」の設計ノートが実在しない」、見本は合格。後継を FR999 にすると見本は違反 1「…の要件が実在しない」。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・縮む file・消す file・新しい dir は無い。`crates/folio/src/note.rs` は単体の歯 f209_ を持つので verify の `--bin folio` の scope。内訳は src 3（note.rs・face_note.rs・floor_note.rs）・歯の file 3（tests/note.rs・tests/face_note.rs・tests/schema.rs）・設計文書 1（design-intent/design-note/schema.yaml の生成区間）・凍結 anchor 1（tests/fixtures/schema/note-region.txt）。`tests/schema.rs` は verify の `--test schema` の scope。
2. **余地（CapHeadroom）。** write-set の src の 3 本（base 9585bb7 の木と見本の木・python と awk の 2 実装で一致・起草の記録の cap.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/note.rs` | 1082 | 418 | 1141（+59・単体の歯 1 本を含む） | 359 |
| `crates/folio/src/face_note.rs` | 1038 | 462 | 1040（+2） | 460 |
| `crates/folio/src/floor_note.rs` | 617 | 883 | 627（+10） | 873 |

   歯の file（余地の数えの外・参考・同じ 2 実装で一致・起草の記録の cap-tests-36555ca.log）: `crates/folio/tests/note.rs` 1207 → 1321（1500 からの残り 293 → 179）・`crates/folio/tests/face_note.rs` 1272 → 1325（228 → 175）・`crates/folio/tests/schema.rs` 700 → 700（器の式の上限 700 ちょうど・§1 (b) 5）。

3. **size は S。** file ごとの増分の最大は note.rs（単体の歯 1 本を含む）で S の見積 100 の内、余地の最小（note.rs の本便の後）は S の 100 を超える。
4. **verify は 8 行**で、done の 8 つの塊と 1 対 1 に揃える。見本の木で 8 行とも rc 0（1・6・2・53・37・21・6 本と clippy 0 警告・起草の記録の verify-*.log）。
   1. `cargo nextest run -p folio --bin folio f209_` = (c) 1（1 本）。
   2. `cargo nextest run -p folio --test note f209_` = (c) 2〜7（6 本）。
   3. `cargo nextest run -p folio --test face_note f209_` = (c) 8〜9（2 本）。
   4. `cargo nextest run -p folio --test note` = 設計ノートの床の歯の全部（folio2 の本流の写しの床は合格のまま・便 207 の数の上限の歯を含む）。
   5. `cargo nextest run -p folio --test face_note` = 面の生成器の歯の全部（凍結の面の byte 一致・今の後継のリンクの歯）。
   6. `cargo nextest run -p folio --test schema` = 設計ノートの生成区間が新しい凍結 anchor と byte で一致し、数の定数が合い、書き直しが冪等。
   7. `cargo nextest run -p folio --bin folio note::tests` = 床の木の導出と凍結 anchor の byte 一致（note_floor_derives_the_frozen_anchor_byte_for_byte）ほか note.rs の単体の歯。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   床の土台を使うほかの歯（proposed・plan ほか）は common-verify の workspace の nextest が撃つ。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり。write-set に `design-intent/design-note/schema.yaml`（生成区間）が在るので門の対象（規則の表の行 D-12）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d209 の本契約で 契約に起因する断り 0・preflight ok（起草の記録の precheck-*.log・歯の絞り込みの語 f209_ と note::tests は base で 0 件）。
3. **重なりの表**（origin の枝の見本の差分の file と本便の write-set・`git diff --name-only <merge-base> <枝>`・2026-09-29 15:2x）。

| 便・行 | 見本 | 本便の write-set と重なる file | 扱い |
| --- | --- | --- | --- |
| 207・hb | impl/d207 9585bb7（親 17736c9） | `crates/folio/src/note.rs`・`crates/folio/src/face_note.rs`・`crates/folio/tests/note.rs`・`crates/folio/tests/face_note.rs` | 着地済み（本流 0914e69・木は 9585bb7 と同じ）＝本便の base。本便の歯は 207 の歯の口（growth_base・growth_note・put_note・propose）を使う |
| 208・hc | impl/d208 3f57332（親 207 の前の先端 f796219） | 0（208 自身の差分は `crates/folio/src/graph.rs`・`crates/folio/tests/graph_summary.rs`・`crates/folio/tests/graph_notes.rs` の 3 本） | 208 が先に着地し、本便の受付はその後。208 の見本の差分（`git diff f796219 3f57332`）は本便の見本 36555ca の木に `git apply --check` で当たる（衝突 0）。当たりは 208 の着地の後に席が確かめる |

   - ほかの impl/* の枝で本流の祖先でなく本便の write-set と file が重なるもの（便 172〜199 の 16 本と f2-648.254-run1）: 便 187・190・192・193・195・196・199 の枝の差分は本流 17736c9 に在る（本流の木に `git apply -R --check` が当たる）。便 172〜185 と f2-648.254-run1 は後の便が同じ file を書き換えたので逆当てでは測れないが、どれも着地済みの便の元の枝と見ている（台帳で席が確かめる）。走っているレーンで本便と重なる未着地の便は 207 だけ。
4. **着地の後（外の置き場）。** tsuzuri は folio の組み立てを上げるとき `folio schema --write` で設計ノートの欄の決まりの生成区間を書き直す（注 1 行・ADR-35 決定 (7)）。今の後継の欄は 0 行なので床の数は変わらない（§1 (e) 3）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/d209-draft.md`、script と log は同じ dir の `d209-scripts/`。
1. `run.sh <木> <印>`: 組み立て・workspace の nextest（--test-threads 2）・clippy・床 4 本・`folio build --write` の file 数と sha（撃つ前にほかのレーンの nextest が止むのを `waitidle.sh` で待つ・/ の空きが 20 GB を切ったら止める）。
2. `red.sh <base の木> <見本の rev>`: 歯の file 3 本・凍結 anchor 1 本だけを base に当てて RED。
3. `mut.py <見本の clone>`: 変異 16 通り（M16 は chain3.sh で 1 通りだけ撃った）。
4. `cap.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk）。
5. `verify.sh <木> <印>`: 契約の verify の 8 行。
6. `prep-tz.sh`・`tz-209.sh <base の folio> <見本の folio>`: tsuzuri の写し（hook なし）で base と見本の床・schema --check・build を撃つ（写しの中だけで書き、最後に戻す）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書・憲法・判断の記録・欄の決まりの型付きの欄・状態の値域・前の欄の意味・外の置き場の file・台帳への記帳（席）・数の上限（便 207）・索引の状態の欄（便 208）・外部 crate・新しい dir。
2. **言えないこと。** 床が数えるのは先の id の実在だけで、中身を移したかの意味は数えない（ADR-35 の検証の要点 (iii)・ADR-3 決定 (3) の境界）。先の状態（要件の保留・判断の記録の廃止）は数えない。面の判断の記録の先は `adr/<id>.yaml` が file かで解き、床は読めた記録の id で照らす（読めない記録の file が在ると面はリンクを張り床は違反にしうる・どちらも今の解き方のまま）。要件の先も母集団が違う: 床は要件と非機能要件の節だけ、面の解き方は 5 つの節（要件・非機能要件・受入基準・制約・目的）を読むので、目的の節に FR99 を置いた写しで、後継 FR99 は床が「要件が実在しない」の違反、面は `srs.html#fr99` へリンクを張る（決定 (3)(オ) の「面の生成器の今の参照 id の解き方」の字どおり）。
3. **撤退条件。** (1) 要件書・判断の記録・憲法・欄の決まりの型付きの欄を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に §1 (c) の直しのほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、`folio build` の file 数が変われば、止めて席へ返す。(4) 208 の着地の木で本便の見本が当たらなければ、席が付け替えを頼む（数を撃ち直す）。

## 2. 範囲

- 入れる: 後継の先の形の定数と生成区間の注（`floor_note.rs`）・形の読み分けの関数 successor と先の母集団・床の照らし（`note.rs`）・面の後継のリンク（`face_note.rs`）・生成区間（`design-intent/design-note/schema.yaml`）・凍結 anchor・生成区間の数の定数・歯。
- 入れない: 要件書・憲法・判断の記録・欄の決まりの型付きの欄・値域・前の欄の意味・索引・導出・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| shape | 形の読み分け | `note.rs` の Successor と successor・`floor_note.rs` の SUCCESSOR_REQUIREMENT・SUCCESSOR_ADR・SUCCESSOR_SECTIONS |
| floor | 床の照らし | `note.rs` の check_meta（先の母集団の束・requirement_ids に節の一覧・`link::adr_ids`） |
| face | 面のリンク | `face_note.rs` の status（設計ノートは今のまま・要件と判断の記録は id_link） |
| region | 生成区間 | `FLOOR` の supersede_note・`schema --write`・凍結 anchor・数の定数 |
| teeth | 歯 | 単体 1 本・`tests/note.rs` 6 本・`tests/face_note.rs` 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 本流 0914e69（ADR-35・要件書 第 1.57 版・便 207〔行 hb〕の着地）。本便は便 208（行 hc）の着地の後に受け付ける。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) 4 の 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "hd"
title = "後継の欄の先（判断の記録 ADR-35 決定 (3)(ア)〜(カ)・(6)・要件 FR32 と AC35・台帳 f2-648.275.9 の 3 便の 1 つ）: crates/folio/src/floor_note.rs に後継の先の形の定数 3 つ（頭 FR と NFR・頭 ADR-・要件の節 requirements と nonfunctional）と床の木の doc_meta の注 supersede_note（後継の先の 3 種・床の数え・面のリンク）を足す。crates/folio/src/note.rs に先の種類と、後継の値を形で読み分ける唯一の関数 successor（頭が FR か NFR で後ろが数字なら要件・頭が ADR- で後ろが数字なら判断の記録・ほかは設計ノートの id）を置き、check_note が前と後継の先の母集団（設計ノートの id・今の読み手 requirement_ids に節の一覧を渡して読んだ要件と非機能要件の id・今の読み手 link::adr_ids の判断の記録の id）を束ねて check_meta へ渡す。後継は種類ごとの集合に照らし、無ければ今と同じつながりの違反（字は 要件 か 判断の記録 か 設計ノート が実在しない）、自分自身は今と同じ形の違反、判断の記録が読めない置き場の判断の記録の先は まだ分からない、先の状態は数えない。前（supersedes）は設計ノートの id だけのまま。crates/folio/src/face_note.rs の状態の行は、廃止の後継を同じ successor で読み分け、要件と判断の記録は今の id_link（resolve）で要件書の面と判断の記録の面へ張り、解けなければリンクを張らず まだ分からない を添える（設計ノートの先は今のまま）。design-intent/design-note/schema.yaml の生成区間は folio schema --write で書き、凍結 anchor note-region.txt は同じ字の 1 行を手で挿入し、tests/schema.rs の生成区間の数の定数 3 つを直す。歯は f209_ の 9 本（単体 1・tests/note.rs 6・tests/face_note.rs 2）。実装の見本は origin の枝 impl/d209 の commit 36555ca（親 9585bb7 = 便 207 の見本の先端・本流 0914e69 と crates design-intent tests の木が同じ）で、作業者は base に対する見本の差分を write-set の file に当て、write-set の外は変えない。base = 本流 0914e69（便 207 の着地）・受付は便 208 の着地の後"
req = ["FR32", "FR9", "FR19"]
section = "1"
write-set = ["crates/folio/src/note.rs", "crates/folio/src/face_note.rs", "crates/folio/src/floor_note.rs", "crates/folio/tests/note.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/schema.rs", "design-intent/design-note/schema.yaml", "tests/fixtures/schema/note-region.txt"]
verify = ["cargo nextest run -p folio --bin folio f209_", "cargo nextest run -p folio --test note f209_", "cargo nextest run -p folio --test face_note f209_", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --bin folio note::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "binary の単体の f209_ の 1 本（後継の値の形の読み分け: FR と NFR と数字は要件・ADR- と数字は判断の記録・ほかは設計ノートの id）が緑、tests/note.rs の f209_ の 6 本（床の土台の写しで後継を在る要件・非機能要件・判断の記録にすると違反 0・無い要件と無い判断の記録と受入基準の id はつながりの違反ちょうど 1 で編集時の口は通す・要件の先は要件と非機能要件の節だけ・自分自身は形の違反 1 で口も止める・前に要件と判断の記録の id を書くと設計ノートの不在の違反 1 で編集時の口は通す・判断の記録が読めない写しは まだ分からない）が緑、tests/face_note.rs の f209_ の 2 本（廃止の行の後継のリンクが要件書の面の要件の場所と判断の記録の面へ・解けない先はリンクなしで まだ分からない）が緑、tests/note.rs の歯の全部（folio2 の本流の写しの床は合格のまま）が緑、tests/face_note.rs の歯の全部（凍結の面の byte 一致・設計ノートの後継のリンクは今のまま）が緑、tests/schema.rs の歯の全部（設計ノートの生成区間が新しい凍結 anchor と byte で一致し数の定数が合う）が緑、binary の単体の note::tests の全部（床の木の導出と凍結 anchor の byte 一致）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
