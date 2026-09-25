# 設計: 便 135 — 要件書と憲法の面の判断の記録の番号と、要件書の範囲の節の番号を行き先へのリンクにする

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は面の字を 1 字も変えず、字の上に行き先の印（リンク）を足すだけで、FR4 の規範文も面の部品・色・並びも変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-2.1（人が読むページは 1 つの生成器から出す＝2 つの面の生成器が同じ 1 つの口を呼ぶ）/ P-6.1（面は正本から逐語で生成する＝包むだけで字は変えない）/ P-6.3（同じ内容を 2 つの面が持たない＝行き先の規則は図の根拠のリンクの口と共有し、id の形は床の走査の口と共有する）/ P-4.2（判定できないものを黙って飛ばさない＝面の無い番号は字のまま残し、黙ってリンクの先を作らない。番号が正本に無いことを表に出すのは床〔`crates/folio/src/link.rs` の判断の記録の id の参照の検査〕で、面ではない）/ P-10.1（独立した凍結 anchor＝期待の字は歯の中の手で写した字で持つ）。
- 出所: 台帳 f2-648.182（控え・面の生成器の便）。天井の読みやすさの所見が 3 周続けて同じ所を指した。31 周目 F-1（要件書と憲法の面に出る判断の記録の番号 ADR-n が、判断の記録の面 adr-N.html への導線になっていない）・32 周目 F-2（要件書の範囲の章の M3 の札が、指し先の番号だけで行き先の印が無い）・33 周目 F-3（要件書の範囲の節の番号 ADR-16・ADR-21・AC23〜AC25・FR22〜FR25 がリンクにならない・持ち主の home の下の `.local/share/folio2/ceiling/2026-09-25-round33/2026-09-25-round33/readability/findings.yaml`）。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eh` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 12 本。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 12 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。天井の印の面の要約値（欄 faces）は、着地の後に束を組み直すと変わる（要件書と憲法の面の 2 枚の byte が変わる）。門と周の引き金は正本の要約値（欄 sources と trigger）だけを見て、面の要約値を突き合わせないので（`crates/folio/src/stamp.rs` の頭の注）、面の変化は周の引き金にならない。
- 前の便: 前提の着地は無い。**base = main 06cd100。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。改訂 b の時点の main 66fd1e1（便 137 の着地）に本便の差分 c135.patch はそのまま当たり、workspace の nextest は 880 / 880（便 137 の歯 6 本を含む）だった（参考値・n137.log）。
- 改訂 b（2026-09-25・独立の検証 d135-verify.md〔条件付き支持・blocking 0・数値は全部一致〕と席の依頼）: 逐語の歯の包み外しの式 unlink_adr を、中の字が判断の記録の番号で行き先がその番号の面であることを確かめる形に締め、憲法の面の歯では包みを外す置き場を読み口 real_html から逐語の 4 本の 1 行ずつへ狭めた（(e) の 1）。突然変異の表に変異 g（番号でない字を adr- の行き先で包む）と逐語の歯と workspace の列を足した（(e) の 3）。§0 の並行の便を今の事実に直し、条 P-4.2 の行に「表に出すのは床」、(i) の 2 に file 名の番号もリンクになる帰結、初出の言い換え 2 つ（字の節点・位置の口）を足した。src の実装・write-set・verify・done・size・余地は変えていない（歯の file の行数だけが変わった）。
- 並行の便との重なり（改訂 b の時点）: 便 137（行 ej）は main 66fd1e1 に着地済みで、書き換えたのは `crates/folio/src/face_adr.rs`・`crates/folio/tests/face_adr.rs`・凍結の面 expected-adr.html と expected-site-adr-2.html の 4 本である。便 137 が書き換えた file は本便の write-set と 1 本も重ならない（起草の時点で重なって見えた badge.rs・site.rs は両便とも本文を変えない）。ただし badge の「名札の無い 7 枚」の歯は便 137 の凍結の面も読むので、受付の時点の main（66fd1e1 以降）で数え直す。便 136（行 ei・枝 docs/d136・7477f64）は `crates/folio/tests/site.rs` に歯を足し、本便は同じ file を本文不変で verify に名指すので、2 便は逐次で運び、後の便は先の便の着地の後の main で数え直す（撤退条件 (4)）。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。

## 1. 設計

### (a) いま起きていること（実測・base main 06cd100・数は参考値）

1. **面の本文の番号はリンクにならない。** 要件書の面と憲法の面は、正本の散文の欄（規範文・平易文・注・来歴・裁定・語彙の定義など）を escape した字のまま出す。判断の記録の番号 ADR-n を字で持つ所は、`folio build` の出力で要件書の面に 224 か所（番号 20 種）、憲法の面に 111 か所（番号 19 種）在り、どれもリンクの外にある（起草役が HTML の読み口で全部の字の節点〔タグとタグの間の字のかたまり〕を数えた・タグの属性の中と `<svg>` の中は 0 か所）。
2. **リンクの口は型付きの欄にしか無い。** 要件書の面の図の章の根拠（欄 refs）は `crates/folio/src/face_srs.rs` の `Ctx::ref_link` が id の形ごとに行き先を解く（要件 = 同じ面の anchor `#fr1`・条と rules 行 = `constitution.html#<条>`・判断の記録 = 正本 `adr/ADR-n.yaml` が在れば `adr-n.html`・無ければ「（まだ分からない）」）。機能要件の小窓の確かめ方・根拠と対応表の番号も、型付きの欄から同じ面か憲法の面の anchor へのリンクを作る。散文の中の番号を拾ってリンクにする口は、どの面の生成器にも無い。
3. **範囲の節は字を並べるだけ。** 章 02 の段の範囲の塊（`face_srs.rs` の `scope_chapter`）は、節 scope・scope_m1・scope_m3 の build と not_build を「 ／ 」で繋いだ字と、注（note）の字をそのまま出す。実の要件書の範囲の節には、要件 FR9・FR11・FR16〜FR18・FR22〜FR25、受入基準 AC23〜AC25、制約 CON1・CON8、条 P-3.2・P-4.2・P-6.3・P-6.4、rules 行 R-4、判断の記録 ADR-3・ADR-4・ADR-8・ADR-16・ADR-21・ADR-22 の番号が在る。
4. **番号を拾う口は床に在る。** 床は散文の中の参照 id を正規表現なしの文字の走査で拾う。条・rules 行・要件の id は `crates/folio/src/refs.rs` の `scan_ids`（位置の口は私的な `id_end`）、判断の記録の番号は `crates/folio/src/link.rs` の `scan_adr_ids`（形 = 前が英数字でも「-」でもない ADR- に 1〜9 で始まる数字列が続き、後ろが英数字でない）。
5. **面の出る番号の規則。** `folio build` は正本 `adr/ADR-n.yaml` の数だけ判断の記録の面 `adr-n.html` を出す（`crates/folio/src/site.rs` の build_all と `crates/folio/src/face_index_read.rs` の records・file 名は id を ASCII 小文字にした字 + `.html`）。状態（accepted・proposed など）で面を出し分けない。実の置き場の判断の記録は ADR-1〜ADR-23 の 23 本で、状態は全部 accepted。
6. **base の歯（参考値）。** workspace の nextest 868 / 868・clippy 0 警告・床 4 本 rc 0・`folio build` 30 file。`git grep -n 'f135_' -- crates` は 0 件。

### (b) 直す先 — 1 つの口で字の上に行き先の印を足す

1. **口（`crates/folio/src/face.rs` に 2 つ足す）。**
   - adr_face（置き場の dir と番号を受ける）: 正本 `<dir>/adr/<id>.yaml` が file として在れば `adr-<数>.html`（id を ASCII 小文字にした字 + `.html`）、無ければ None。`folio build` が面を出す番号の規則（(a) の 5）と同じで、図の根拠のリンクの判定（(a) の 2）とも同じ式である。
   - link_ids（組み立てた HTML・旗 all・行き先の関数 href を受ける）: 組み立て済みの HTML の字の部分（タグの外）だけを走査し、番号を拾って `href` が行き先を返した番号だけを `<a class="xref" href="<行き先>"><番号></a>` で包む（字は 1 字も変えない）。タグの中（属性を含む）・`<a>` の中（入れ子のリンクを作らない）・`<head>`・`<script>`・`<svg>` の中は触らない。番号は床と同じ口で拾う＝判断の記録は link.rs の adr_end、`all` が真なら条・rules 行・要件の id も refs.rs の id_end。class は既存の xref（部品目録の中・新しい部品も class も足さない）。
2. **床の走査の口を位置の口（番号がどこで終わるかを返す関数）として開く（字の規則は変えない）。** `crates/folio/src/refs.rs` の `id_end` を `pub(crate)` にする（本体は 1 字も変えない）。`crates/folio/src/link.rs` の `scan_adr_ids` の 1 か所の判定を関数 adr_end（`pub(crate)`・文字の並びと位置を受けて番号の終わりの位置を返す）に切り出し、`scan_adr_ids` はそれを呼ぶ形にする（拾う番号は変わらない・既存の単体の歯 link の scan_adr_ids の字の例がそのまま緑）。
3. **要件書の面（`face_srs.rs`）。**
   - 面全体: `derive` の最後に、組み立てた HTML を link_ids（旗 all は偽・行き先は adr_face）に通す＝本文の判断の記録の番号のうち、正本が在る番号だけが判断の記録の面へのリンクになる。
   - 範囲の節: `scope_chapter` の build・not_build・注の字を link_ids（旗 all は真・行き先は下の target）に通す。行き先は図の根拠と同じ規則で、`Ctx::ref_link` の中の形ごとの行き先の判定を Ctx の関数 target（置き場の dir と id を受けて行き先を返す）に切り出して共有する（ref_link の出す字は 1 byte も変えない）。行き先の無い番号（要件に無い FR99・正本の無い ADR-99 など）は字のまま。
   - 範囲の節の外の要件・条・rules 行の番号はリンクにしない（今の所見が指す範囲に留める・(d) の 2）。
4. **憲法の面（`face_constitution.rs`）。** `derive` の最後に、組み立てた HTML を link_ids（旗 all は偽・行き先は adr_face）に通す（要件書の面の面全体と同じ 1 行）。憲法の面の条・rules 行の番号は今までどおり（型付きの欄の既存のリンクだけ）。
5. **決定の段（既に在るか → 1 行で書けるか → 最小の実装）。** 散文の番号をリンクにする口は既に無い（(a) の 2）。番号を拾う口（床の走査）と行き先の規則（図の根拠の ref_link）は既に在るので作り直さずに共有する。1 行では書けないので、足す実装は字の部分を切り出して包む 1 つの口（`link_ids`）だけにする。
6. **契約で決めること（起草役の判断）。**
   - 面を生成しない番号: 正本 `adr/ADR-n.yaml` が無い番号はリンクにせず字のまま残す。「（まだ分からない）」のような字も足さない（散文の逐語を変えない・P-6.1）。番号が正本に無いこと自体は床（`link.rs` の判断の記録の id の参照の検査）が違反として数える。状態が proposed の判断の記録も面は出るので（(a) の 5）リンクにする。
   - 同じ面の中の番号: 範囲の節の要件（FR・NFR・AC・CON・GOAL）は同じ面の anchor（`#fr22` の形）へ、条と rules 行は憲法の面の anchor へ、判断の記録は adr-n.html へ。
   - 上の帯（site-bar）に判断の記録の入口は置かない（範囲を広げない・入口の面の棚が判断の記録の入口のまま）。
   - 面の見た目の直しの数え（判断の記録 ADR-5 の撤退条件 ②）: **当たらない。** 撤退条件 ② が数えるのは「見た目だけを直す便」で、本便は部品・色・余白・並び・名札の class を変えず（既存の class xref のリンクを足すだけ）、面の字も変えない。足すのは番号から行き先へ辿れる導線（面の働き）である。見た目を部品・色・余白・並び・名札の class と読むのは、便 132 の契約（撤退条件 ② に当たらないとした読み）と同じである。持ち主の walk（面の承認）は求めない。
7. **変えないもの。** 面の字（逐語）・部品と class・章と節の並び・判断の記録の面・入口の面・設計ノートの面・図の章の根拠のリンクの字・床の拾う番号と判定・設計文書・`folio build` の出す file の数（30 のまま）。

### (c) 歯（関数名 `f135_…`・base では 6 本とも RED）

1. **f135_link_ids_wraps_only_the_text_outside_links（単体の歯・`crates/folio/src/face.rs` の既存の tests の区間 face_tests）。** 手書きの HTML 片（`<head>` の中の title・属性 title・字の ADR-1・床の形に当たらない ADR-12a と xADR-1 と ADR-0・`<a>` の中・`<svg>` の中・行き先を返さない ADR-3）を link_ids（旗 all は偽）に通し、字の ADR-1 だけが `<a class="xref" href="adr-1.html">ADR-1</a>` になり、ほかは 1 字も変わらないことを出力の全字の等しさで数える。`all` が真のとき条（枝番付き P-6.3）・要件 FR1・rules 行 R-4・判断の記録 ADR-2 の 4 つが包まれることも全字で数える。**base では関数 link_ids が無く組み立てが落ちる＝RED。**
2. **f135_scope_m3_numbers_link_to_their_pages（`crates/folio/tests/face_srs.rs`・binary 経由・実の置き場の写し）。** 章 02 の 3 つ目の段の範囲の塊（M3）の「M3 で作る」の枠に、`adr-16.html`・`adr-21.html`・`#ac23`〜`#ac25`・`#fr22`〜`#fr25`・`#fr11`・`constitution.html#p-6`（字 P-6.3）の 11 本のリンクが、手で写した全字で在ることを数える（33 周目 F-3 と 32 周目 F-2 の場所）。**base では 1 本も無い＝RED。**
3. **f135_every_adr_mention_on_the_real_srs_is_a_link（同・実の置き場の写し）。** 面の本文（`<body>` から後・`<svg>` の中を除く）の判断の記録の番号の出現の全部が、直前が `<a class="xref" href="adr-<数>.html">` で直後が `</a>` の形（歯の側の式）で、行き先の正本 `design-intent/adr/<番号>.yaml` が在り、1 つ以上在り、`<a` の入れ子の深さが 1 であることを数える（31 周目 F-1）。**base ではリンクでない番号が 224 か所＝RED。**
4. **f135_numbers_without_a_page_and_ids_outside_the_scope_stay_plain（同・面の fixture の写し）。** 面の fixture（`tests/fixtures/face/` の正本 5 file）に判断の記録 1 本（fixture の adr/ADR-2.yaml の写し）を添え、要件書に手書きの scope_m3（build に「判断の記録 ADR-2 と ADR-99・要件 FR1 と FR99・条 P-1」）と not_frozen（「ADR-2 と FR2 を読む。」）を書いて面を書く。M3 の塊に `<a class="xref" href="adr-2.html">ADR-2</a> と ADR-99・`、`要件 <a class="xref" href="#fr1">FR1</a> と FR99・`、`条 <a class="xref" href="constitution.html#p-1">P-1</a>` が在り、面のどこにも `adr-99.html` と `href="#fr99"` が無く、範囲の節の外の凍結しないものの枠が `<p><a class="xref" href="adr-2.html">ADR-2</a> と FR2 を読む。</p>`（FR2 は字のまま）であることを数える。**base ではリンクが 1 本も無い＝RED。**
5. **f135_every_adr_mention_on_the_real_constitution_is_a_link（`crates/folio/tests/face_constitution.rs`・実の置き場の写し）。** 3 と同じ数えを憲法の面に当て、改訂の例の判断の記録の欄が `<dt>判断の記録</dt><dd><a class="xref" href="adr-11.html">ADR-11</a></dd>` であることも数える。**base ではリンクでない番号が 111 か所＝RED。**
6. **f135_number_without_a_page_stays_plain_on_the_constitution（同・実の置き場の写しに変異）。** 写しの rules.yaml の最初の裁定の欄（行 R-1）の頭に「ADR-99・ADR-11・」を足して面を書き、行 R-1 の裁定の枡が `ADR-99・<a class="xref" href="adr-11.html">ADR-11</a>・f2-648 notes` で始まり、面のどこにも `adr-99.html` が無いことを数える。**base では ADR-11 がリンクでない＝RED。**

歯 2〜6 の fixture は写しの一時 dir の中で書き換え、版管理の下の正本と面は書き換えない。

### (d) 採らなかった形

1. **散文の欄ごとに生成器の呼び出しの口で包む。** 包む所が 2 つの面で数十か所に散り、見出しや既存のリンクの字（図の要件の札・根拠の札）の中へ包むと入れ子のリンクになる。組み立てた HTML の字の部分を 1 か所で走査すれば、欄を足しても漏れず、`<a>` の中を避ける規則も 1 か所で済む。
2. **面の全体で要件・条・rules 行の番号もリンクにする。** 要件書の面の散文には要件と条の番号が数百か所在り、所見が求めた範囲（判断の記録の番号と範囲の節）を大きく超えて面が変わる。範囲の節だけを全部の形で包み、面全体は判断の記録の番号に留める（`all` の旗 1 つで分ける）。要件書の面の全体へ広げるなら、次の所見を待って別の便で決める。

### (e) 既存の歯のうち落ちるもの・直すもの・突然変異

1. **src だけを当てた写しで落ちる既存の歯は 12 本（起草役の実測・`c135-src.patch`）。** workspace の nextest は 869 本のうち 857 本が緑（単体の歯 1 本が増える）。落ちる 12 本は次の 2 種で、どちらも本便が面の byte を変えることの直の帰結である。
   - **凍結 fixture の 1 行（6 本）。** 面の fixture の要件書の scope_m1 の注「M1 の着手は CON1 の順序に従う。」の CON1 が範囲の節のリンク `<a class="xref" href="#con1">CON1</a>` になる。凍結の面 `tests/fixtures/face/expected-srs.html` の 91 行目の注の字の 1 か所だけを手で書き換える（ほかの行・ほかの凍結の面は 1 byte も変えない・fixture の憲法と要件書と語彙に判断の記録の番号は 0 か所）。この凍結の面を読む歯 = face_srs の f118_handwritten_scope_m3_adds_one_escaped_block と f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required・face_srs_body の face_srs_write_matches_the_frozen_fixture・face_srs_figure の face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face・badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures・site の site_write_matches_the_frozen_fixture。この 6 本の歯の本文は変えない。
   - **逐語を比べる歯（6 本）。** 実の置き場の面の字を正本の逐語と比べる歯が、字の途中に入ったリンクの包みで一致しなくなる。歯の側に包みを外す式 `unlink_adr`（生成側の式を写さない手書きの式）を置き、比べる前に外す。式は `<a class="xref" href="adr-` で始まるリンクごとに、中の字が判断の記録の番号（ADR- に数字列）であることと、行き先がその番号の面（`adr-<数>.html`）であることを assert してから、開きと閉じを外して字だけを残す（改訂 b・番号でない字を adr- の行き先で包む誤りを逐語の歯でも落とす・(e) の 3 の変異 g）。包みを外すのは逐語を比べる 6 本だけで、1 本につき 1 行: `crates/folio/tests/face_constitution.rs` の f79_rules_ruling_shows_only_the_latest・f79_rules_ruling_folds_the_previous_ones・f79_rules_ruling_without_previous_is_unchanged・f80_amendment_previous_text_is_verbatim の面を読む 1 行（読み口 real_html は変えない＝この file のほかの 11 本はリンクの在る面を読む）、`crates/folio/tests/face_srs.rs` の f81_approval_history_is_folded の比べる 1 行、`crates/folio/tests/face_srs_body.rs` の face_srs_census_on_the_real_sources_counts_and_verbatims の読んだ直後の 1 行。比べる字そのもの（正本の逐語）と数えは変えない。
2. **本便の全体を当てた写し（`c135.patch`）。** workspace の nextest 874 / 874（base 868 + 歯 6）・clippy 0 警告・`cargo build --all-targets -p folio` の警告 0・床 4 本 rc 0・`folio build` 30 file（床 合格 0/0）。base の出力と比べて変わる file は要件書の面と憲法の面の 2 枚だけ（下の表）。判断の記録の面 23 枚・入口の面・設計ノートの面・様式 2 本は 1 byte も変わらない。

| file | base（byte） | 本便の後（byte） | 足したリンク |
| --- | ---: | ---: | --- |
| srs.html | 368249 | 377733 | 判断の記録 224（番号 20 種）・範囲の節の要件と制約 16・条と rules 行 6 |
| constitution.html | 245612 | 249910 | 判断の記録 111（番号 19 種） |

   面の本文の判断の記録の番号でリンクの外に残るものは、2 枚とも 0 か所（歯 3 と歯 5 の数え）。`<a` の入れ子の深さは 2 枚とも 1。`folio parts --check`（実の要件書の面）は合格のまま（face_srs_body の face_srs_on_the_real_sources_passes_parts_check が緑）。要件書の面の生成器の大きさの歯（face_srs の f100_face_srs_is_split_and_under_the_cap・上限 1100）は 1039 で緑。
3. **突然変異（起草役の実測・本便を当てた写しの src だけを 1 通りずつ変える）。** 表の「本便の歯」の列は f135_ の 6 本だけで数えた落ちる歯の番号（(c) の番号）で、「逐語の歯」の列は (e) の 1 の逐語を比べる 6 本のうち落ちる本数である。workspace 全体で撃つと、ほかの既存の歯も落ちる変異がある（最後の列・参考値）。

| 変異 | 本便の歯（f135_ の 6 本） | 逐語の歯 6 本 | workspace で落ちる既存の歯（f135_ を除く） |
| --- | --- | --- | --- |
| a: 正本の在る無しを見ずに判断の記録の番号を全部リンクにする | 4・6 | 0 | 0 |
| b: 範囲の節も判断の記録の番号だけにする（all を偽に） | 2・4 | 0 | 6（凍結の面を読む 6 本） |
| c: 要件書の面の全体で要件の番号もリンクにする | 4 | 2（f81・census） | 10（凍結の面を読む 6 本・f81・census・用語集の 2 本） |
| d: `<a>` の中も包む（入れ子のリンク） | 1・3・4 | 1（census） | 1（census） |
| e: 憲法の面は包まない | 5・6 | 0 | 2（用語集の 2 本） |
| f: `<head>` と `<svg>` の中も包む | 1 | 0 | 0 |
| g: 憲法の面で条の番号（判断の記録の番号でない字）を adr- の行き先で包む | なし | 4（f79 の 3 本・f80 の前の文） | 12（逐語の 4 本・凍結の面の face・badge・site・census の face・f80 の欠番の 2 本・用語集の 2 本） |

   変異 a で歯 3 と歯 5 が緑なのは、実の置き場の判断の記録の番号が全部正本を持つからで、正本の無い番号は歯 4 と歯 6 が数える。変異 f を落とすのは単体の歯だけで、実の置き場の図の本体（`<svg>`）と head に判断の記録の番号は 0 か所である。変異 g（改訂 b で足した）は f135_ の 6 本がどれも落とさない形で、逐語の歯の包み外しの式が中の字と行き先を確かめるので、憲法の面の逐語の歯 4 本が落ちる。改訂 a の式（中の字と行き先を見ずに外す）のままの写しに同じ変異を当てると、逐語の歯 6 本は全部緑だった（mut-g-a.log）。用語集の 2 本（要件書の面の用語集の行が憲法の面と byte 一致する歯）が変異 c・e・g で落ちるのは、2 つの面が同じ 1 つの口で同じ形に包むことを縛っている（P-2.1）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 12 本とも印なし。書き換える 9 本 = `crates/folio/src/face.rs`・`crates/folio/src/face_srs.rs`・`crates/folio/src/face_constitution.rs`・`crates/folio/src/link.rs`・`crates/folio/src/refs.rs`・`crates/folio/tests/face_srs.rs`・`crates/folio/tests/face_constitution.rs`・`crates/folio/tests/face_srs_body.rs`・`tests/fixtures/face/expected-srs.html`。本文を変えない 3 本 = `crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`（verify の `--test` で名指すので write-set に入れる）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 5 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、全部一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/face.rs` | 1005 | 495 | 1103（+98・単体の歯 22 を含む） |
| `crates/folio/src/face_srs.rs` | 1031 | 469 | 1039（+8） |
| `crates/folio/src/face_constitution.rs` | 1039 | 461 | 1041（+2） |
| `crates/folio/src/link.rs` | 660 | 840 | 669（+9） |
| `crates/folio/src/refs.rs` | 431 | 1069 | 431（±0） |

   5 本とも余地は M の見積 300 を超える。src の増分の和は模擬で +117 で、S の見積 100 を超えるので M にする。歯の file は src の外なので余地を測らない（参考値 face_srs 566 → 782・face_constitution 649 → 808・face_srs_body 735 → 768）。
3. **size は M。**
4. **verify は 10 行**で、done の 10 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f135_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_srs f135_` = (c) の 2〜4（3 本）。
   3. `cargo nextest run -p folio --test face_constitution f135_` = (c) の 5〜6（2 本）。
   4. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（凍結 fixture を読む f118 の 2 本と逐語の f81 を含む・参考値 16 本）。
   5. `cargo nextest run -p folio --test face_constitution` = 憲法の面の歯の全部（逐語の f79 の 3 本と f80 を含む・参考値 17 本）。
   6. `cargo nextest run -p folio --test face_srs_body` = 要件書の面の本体の歯の全部（凍結 fixture との byte 一致と逐語の census を含む・参考値 18 本）。
   7. `cargo nextest run -p folio --test face_srs_figure` = 図の章の歯の全部（凍結の面を読む歯を含む・参考値 10 本）。
   8. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い 7 枚の凍結 fixture との一致（参考値 1 本）。
   9. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 配信先の凍結 fixture との一致（参考値 1 本）。
   10. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は 6 本（face_srs・face_constitution・face_srs_body・face_srs_figure・badge・site）で、全部 write-set に在る。`--bin folio` の絞り込みの語 f135_ を関数名に持つ src は `crates/folio/src/face.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。共通の検証を同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d135-draft.md`、模擬の差分と script は同じ dir の d135-scripts（repo には入れない）。写しは base を `cp -r` し、`.git` の file を消して git の init と 1 commit をした dir。

1. 模擬: 差分 c135.patch を base の写しに当て、run-135.sh（組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と base の出力との差・verify の 1〜9 の本数）。
2. RED: 歯の 2 file だけの差分 r135-teeth.patch を base の写しに当てて red-135.sh（f135_ の 5 本とも落ちる・2 file のほかの歯は緑）。単体の歯は base に関数 link_ids が無く組み立てが落ちる（red.log）。
3. src だけ: c135-src.patch を base の写しに当てた workspace の nextest で、落ちる既存の歯 12 本（srconly.log）。
4. 突然変異: mut-135.sh（(e) の 3 の 7 通り・src を 1 通りずつ変えて f135_ の 6 本と逐語の歯 6 本を撃ち、環境変数 WS=1 なら workspace 全体も撃ち、元に戻して時刻を新しくする）。改訂 a の式との比べは、c135-a.patch を当てた写しに M_ONLY=g で撃つ（mut-g-a.log）。
5. 余地: lines-135.py と lines-135.awk（同じ式の 2 実装）を repo の根で write-set の src に当てる（lines.log）。
6. 番号の居場所: adr-contexts.py（HTML の読み口で字の節点ごとに判断の記録の番号の数と、リンクの中か・属性の中かを数える）を base と本便の後の `folio build` の出力の 2 枚に当てる。
7. 字の不変: unlink-eq.py（本便の後の 2 枚から、判断の記録の面へのリンクと範囲の節のリンクの包みを外し、base の 2 枚と比べる）。起草役の模擬では 2 枚とも byte 一致（unlink-eq.log）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 判断の記録の面・設計ノートの面・入口の面の散文の番号のリンク（所見が指したのは要件書と憲法の 2 面）。要件書の面の範囲の節の外の要件・条・rules 行の番号のリンク（(d) の 2）。上の帯の判断の記録の入口。判断の記録どうしの番号と題の一覧（33 周目 F-6・別の控え）。設計文書の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** リンクの行き先は正本の file の在る無しで決め、判断の記録の面が生成できるかは見ない（`folio build` は 1 枚でも生成できなければ全部を出さないので、配信先に在る面と食い違うことはない）。`folio face --face srs` で要件書の面を 1 枚だけ書いたときは、行き先の判断の記録の面が同じ dir に無くてもリンクになる（`folio build` の配信先で揃う）。散文の番号が 1 字の全角・半角の違いなどで床の形に当たらなければ、リンクにならない（床が番号として数えないものは面も番号として扱わない）。逆に、床の形に当たる字は、判断の記録そのものを指していなくてもリンクになる。例: 要件書の受入基準の固定の材料の字「判断の記録 2 本 = ADR-1.yaml と ADR-2.yaml」は面の fixture（`tests/fixtures/face/adr/`）の file 名だが、ADR-1・ADR-2 が実の判断の記録の面 adr-1.html・adr-2.html へのリンクになる（床の scan_adr_ids も同じ字を番号として数える＝床と同じ口を使う設計の直の帰結）。
3. **撤退条件。** (1) 本便の後に、(e) の 1 に名指した 12 本と本便の歯 6 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に folio2 自身の床の結果か、`folio build` の出力のうち要件書の面と憲法の面のほかの 28 file のどれかが 1 byte でも変わったら、止めて席へ返す。(3) 要件書の面と憲法の面で、本便が足したリンクの包みを外した面が base の面と 1 byte でも違ったら、止めて席へ返す（起草役の模擬では 2 枚とも一致・(h) の 7）。(4) 受付の時点の main で write-set の file が base から書き換わっていたら、余地と歯の効きを測り直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face.rs` の口 2 つ（adr_face・link_ids）と単体の歯 1 本。`crates/folio/src/face_srs.rs` の面全体の包みと範囲の節の包みと行き先の判定の切り出し（Ctx::target）。`crates/folio/src/face_constitution.rs` の面全体の包み。`crates/folio/src/refs.rs` の id_end の可視性。`crates/folio/src/link.rs` の adr_end の切り出し。歯 5 本（face_srs 3・face_constitution 2）。逐語の歯 6 本の比べる前の包み外し（3 か所）。凍結の面 expected-srs.html の 1 か所。
- 入れない: 面の字・部品・class・並び・判断の記録の面・入口の面・設計ノートの面・上の帯・床の判定・設計文書・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| page | 判断の記録の面の file の規則 | `crates/folio/src/face.rs` の adr_face（正本の在る番号だけ adr-n.html） |
| wrap | 字の部分の番号を包む口 | `crates/folio/src/face.rs` の link_ids（タグ・`<a>`・head・script・svg の外だけ） |
| scan | 番号の位置の口 | `crates/folio/src/link.rs` の adr_end と `crates/folio/src/refs.rs` の id_end（床と共有） |
| target | 範囲の節の行き先 | `crates/folio/src/face_srs.rs` の Ctx::target（図の根拠の ref_link と共有） |
| teeth | 面の歯 | `crates/folio/tests/face_srs.rs` と `crates/folio/tests/face_constitution.rs` の f135_ |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無い。
- 本便の着地の後に席が見ること: 台帳 f2-648.182 を閉じる。次の天井の周で、読みやすさの観点の同じ場所の所見（範囲の節の番号・判断の記録の番号の導線）が立たないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eh"
title = "天井の読みやすさの所見 31 周目 F-1・32 周目 F-2・33 周目 F-3（台帳 f2-648.182）の面の生成器の便: 要件書の面と憲法の面の本文に字で出る判断の記録の番号 ADR-n を、正本 adr/ADR-n.yaml が在る番号だけ判断の記録の面 adr-n.html へのリンクにし（正本の無い番号は字のまま）、要件書の範囲の節（scope・scope_m1・scope_m3）の build・not_build・注の中の要件・条・rules 行・判断の記録の番号を、図の根拠と同じ行き先（同じ面の anchor・憲法の面・adr-n.html）へのリンクにする。口は crates/folio/src/face.rs の adr_face と link_ids（組み立てた HTML の字の部分だけを走査し、タグ・a・head・script・svg の中は触らず、既存の class xref で包むだけで字は変えない）1 組で、番号は床の走査の口（link.rs の adr_end・refs.rs の id_end）で拾い、範囲の節の行き先は face_srs.rs の Ctx::ref_link から切り出した Ctx::target を共有する。範囲の節の外の要件・条・rules 行の番号・上の帯・判断の記録と入口と設計ノートの面は変えない。凍結の面 expected-srs.html は注の CON1 の 1 か所だけが変わり、逐語を比べる既存の歯 6 本は、歯の側でリンクの中の字が判断の記録の番号で行き先がその面であることを確かめてから包みを外して比べる。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/link.rs", "crates/folio/src/refs.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face_srs_body.rs", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face_srs_figure.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --bin folio f135_", "cargo nextest run -p folio --test face_srs f135_", "cargo nextest run -p folio --test face_constitution f135_", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --test face_srs_body", "cargo nextest run -p folio --test face_srs_figure", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "単体の歯 f135_（link_ids が字の部分の番号だけを包み、head・属性・a・svg の中と床の形に当たらない字と行き先の無い番号を 1 字も変えない）が緑、face_srs の f135_ の 3 本（実の要件書の M3 の範囲の塊の 11 本のリンク・実の要件書の本文の判断の記録の番号が全部 adr-n.html へのリンクで入れ子が無い・fixture で正本の無い ADR-99 と要件に無い FR99 が字のままで範囲の節の外の FR2 がリンクでない）が緑、face_constitution の f135_ の 2 本（実の憲法の面の判断の記録の番号が全部リンクで改訂の例の ADR-11 がリンク・裁定の枡の正本の無い ADR-99 が字のまま）が緑、face_srs の歯の全部が緑、face_constitution の歯の全部が緑、face_srs_body の歯の全部が緑、face_srs_figure の歯の全部が緑、badge の名札の無い 7 枚の凍結 fixture との一致が緑、site の配信先の凍結 fixture との一致が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check が一致・folio derive --dir design-intent --out ../contracts --check が一致を返し、folio build の出力は 30 file で、base と違う file は srs.html と constitution.html の 2 枚だけ"
<!-- contracts:end -->
