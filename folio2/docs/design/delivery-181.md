# 設計: 便 181 — 決定の欄の裁定 id の形を床で数える（判断の記録 ADR-31 の便 A の前半・決定 (1)(3)）

- 要件: FR26（決定の欄の裁定 id の形を床で数え、求められたら形の種類つきで書き出す・要件書 第 1.53 版）の前の 2 文（床の数え方）と、FR5（検査結果を必ず返す＝骨格の欄の未記入は「まだ分からない」）。どちらも本流の要件書に在る id で、字は変えない。FR26 の最後の文（書き出し `folio check --emit-rulings`）は便 C で、本便は運ばない。受入基準は AC28 の前半（決定の欄から台帳の id を消すと違反 1・骨格の印にすると「まだ分からない」1）。
- 条: P-12.1 / P-12.2（承認は逐語と日付を添えて台帳に記帳する＝決定の欄は台帳の番号を持つ）・P-17.1 / P-17.3（規則の表の行は裁定 id を持つ・初版の行の裁定は発効の承認）・P-3.1（機械で決定的に数えられる形は床に置く）・P-4.1 / P-4.2（骨格のまま書いていない欄は「まだ分からない」）・N-3.1（選ぶ行〔検査を外す旗〕を持たない）・P-5.6（決定の欄の一覧と文法の写しは、割った後の便 182 が欄の決まりの生成区間へ運ぶ）。
- 出所: 判断の記録 ADR-31（持ち主の承認 2026-09-28 10:29 JST・対話面 R-8・逐語「全部承認する」・PR #371・本流 78a793f・台帳 f2-648.267）の決定 (1)（決定の欄の閉じた一覧と床）・(3)（裁定 id の文法）・(7)（条 P-12 と P-17 の機構の注の「数える範囲」を同じ取り込みで直す）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gb` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 36 本（src 9・歯の file 7・設計文書の正本 1・fixture と凍結 anchor 19）。新しい file は 2 本（頭に `+`・既存の dir `crates/folio/src/` と `crates/folio/tests/` の下）。縮む file は 4 本（頭に `-`）。消す file・新しい dir は無い。
- 門: 対象。write-set に設計文書の正本 `design-intent/constitution.yaml` が在る。本流の組み立て（78a793f の binary）に write-set 36 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 本流 78a793f**（ADR-31 と要件書 第 1.53 版が発効し、規則の表の行 R-10 と D-8 の ruling が本判断の承認の裁定 id に引き直された main）。この契約の数はすべて 78a793f の写しの実測（参考値・規則の表の行 D-13）。受付の時点の main が 78a793f と違えば、その main で数え直す。
- 実装の見本: origin の枝 `impl/d181`（commit **a2f8606**・親 78a793f）が本便の後の中身で、`git diff 78a793f a2f8606` が便の全体の差分（36 file・+736 −184・103,145 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout a2f8606 -- <write-set の file>`）。write-set の外は変えない。
- 割った便: ADR-31 の便 A は、差分が審査の上限 120,000 byte を超える（割る前の見本で 135,170 byte ＋ 歯）ので 2 便に割った。**本便 181（行 gb）= 床**（文法・決定の欄の歩き手・床の判定・fixture と既存の歯の字）、**便 182（行 gc・docs/design/delivery-182.md）= 欄の決まりの写し**（判断の記録の欄の決まり `design-intent/adr/schema.yaml` の生成区間へ、文法の字面と決定の欄の閉じた一覧を写す・fixture の写し 17 本）。順は 181 → 182（182 の見本 impl/d182 は a2f8606 の上）。二つの write-set は `crates/folio/src/adr.rs`・`crates/folio/src/ruling.rs`・`crates/folio/tests/ruling.rs`・`crates/folio/tests/graph.rs`・`tests/fixtures/schema/node-digest-anchor.txt` で重なる。
- 並行の便との重なり: base の時点で、本便の write-set を書き換える未着地の便の契約は便 182 だけ（便 179・180 は着地済み＝本流 0fb780e・63a25a9）。受付の時点で precheck が重なりを見る。

## 1. 設計

### (a) いま起きていること（base 78a793f の実測・参考値）

1. **裁定 id の形を数える所はばらばら。** 床（`folio check`）が裁定の欄に台帳の id の形があるかを見るのは 4 か所だけ: 判断の記録の承認欄（`crates/folio/src/adr.rs` の関数 check_approval・種別 N-4）・設計ノートの承認欄（`note.rs` の関数 check_meta・種別 note・値が字のときだけ＝一覧の値は素通り〔台帳 f2-648.242〕）・憲法の改訂来歴（`link.rs` の関数 amended_by・種別 N-4）・凍結 anchor の承認一覧（`anchor.rs`）。判定の関数は `adr.rs` の has_ledger_id（4 字の窓のどこでも `[a-z][0-9]-[a-z0-9]`）。別に `floor.rs` にも同じ名の has_ledger_id（前の字が頭・空白・非 ASCII の語頭だけ）が在り、外の置き場の骨格に folio2 の番号の印が無いことを見る関数 marked が使う。**規則の表の全行の ruling・憲法の発効の承認（meta.approval.ruling）・要件書と入口と天井の正本と相談窓口と索引の欄の決まりの承認欄の行（stamp）は見ない**（条 P-12 と P-17 の機構の注・台帳 f2-648.187）。
2. **決定の欄の全数（ADR-31 決定 (1) の一覧・起草役の独立でない数え＝検証役の fields.py〔yq と字の走査〕）。** folio2 の本流は 155 欄（5 正本の stamp 83〔作成とレビューの行を除く〕・憲法の発効の承認 1・改訂来歴 5・閾値の行 20・開発規律の行 17・判断の記録の承認欄 29）で、台帳の id を 1 つも持たない欄は 0（行 R-10 と D-8 は ADR-31 の取り込みで直った）。切り出しの形は notes-time 180・bead 22。tsuzuri の写し（本物の `.git` だけを写して clone した d927a57）は 56 欄（憲法 1・改訂来歴 3・閾値 23・開発規律 4・判断の記録 13・設計ノート 12）で 0 欄、形は bead 57・question 6。床の土台（`tests/fixtures/floor_base/design-intent/`）は 55 欄で、**規則の表の 14 行（初版の答えの記号 G9=A などと、便 179 の写しの字）が台帳の id を持たない**。古い形の fixture 16 組（`tests/fixtures/{adr,anchor,check,link,refs,vocab}/…/rules.yaml`）は行に ruling の欄が無い。
3. **骨格。** `folio init` の直後の置き場は、骨格が書く決定の欄 4 つ（憲法の発効の承認と、規則の表の行 R-2・R-8・R-16）の値が骨格の印 `未記入`。今の床はそれを見ず、骨格の床は「違反 0・まだ分からない 2」（凍結の基準の不在 2 つ）。
4. **base の歯（参考値）。** workspace の nextest 1016 / 1016・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 36 file。`git grep -n f181_ -- crates` は 0 件・行 id `gb` は 0 件。

### (b) 直す先 — 1 つの文法と 1 つの歩き手で決定の欄を全部数える

1. **新しい file `crates/folio/src/ruling.rs`（責務の層 1 読む）。**
   - 関数 rulings（欄の字から裁定 id を全部、出てくる順に切り出す・ADR-31 決定 (3)）: 語頭（頭か、前の byte が ASCII の英数字・`_`・`-`・`.` でない位置）の台帳の id（英小字 1・数字 1・`-`・英小字か数字の並び 1 字以上・`.数字` の段を何段でも）を 1 つとし、直後に器の問いの印（`:` 年月日 8 桁 `T` 時分 4 桁 `Z-` 連番）が続けばそれを含めて形 question、` notes` に日付（` YYYY-MM-DD`）か時刻（` HH:M` と数字か `x`）の少なくとも 1 つと任意の ` JST` が続けばそれを含めて形 notes-time、どちらも無ければ形 bead。切り出した字の終わりから続けて探す。返すのは字（欄の字のまま）・形・台帳の id の部分の 3 つ（型 Ruling）。関数 has_ruling = 1 つ以上切り出せるか。正規表現は使わない。
   - 型 Form（Question・NotesTime・Bead・名の定数 NAMES = question・notes-time・bead）。
   - 決定の欄の名の定数 11（憲法の発効の承認・改訂来歴・閾値の行・開発規律の行・判断の記録の承認欄・設計ノートの承認欄の行・5 正本の承認欄の行の stamp＝定数 STAMPS）・骨格が書く欄 SKELETON（憲法の発効の承認・閾値の行・開発規律の行）・数えない役 SKIP_ROLES（作成・レビュー）。
   - 型 Tree（歩き手が読む木: 憲法・規則の表・要件書・入口・天井の正本・相談窓口・索引の欄の決まり〔在れば〕・判断の記録・設計ノート）と型 Site（欄の名・file・在り処の字・値）と関数 sites（決定の欄を一覧の順に全部拾う）。欄を持つ入れ物（承認欄の表・行・承認欄の行）が在れば、欄が無くても 1 つに数える（値は無し）。承認欄の行は、欄 role が SKIP_ROLES の行だけを飛ばす（role の無い行は数える）。憲法の各条の supersedes_v1 は拾わない（ADR-31 決定 (1)）。
   - 単体の歯 3 本（(c) の 6）。
2. **`crates/folio/src/check.rs`。** 関数 check_dir は、設計ノートの床（note::check_note が読めた設計ノートを返すように変わる）の後に、索引の欄の決まり `graph.yaml` を在れば読み（関数 load_graph・無ければ読まない・在って読めなければ 7 本と同じ読み手が「まだ分からない」）、Tree を組んで関数 check_rulings（新しい関数）を呼ぶ。check_rulings は Site ごとに: 骨格の欄で値が字の `未記入` なら「まだ分からない」（`<file>: <在り処> が 未記入（骨格の印・裁定の前＝条 P-17.3）`）、字で has_ruling なら何もしない、字で切り出せなければ種別 **裁定 id** の違反（`<file>: <在り処>「<値>」に台帳 id が無い（決定の欄・形は adr/schema.yaml の ruling_pattern）`）、一覧か表なら違反（`… が字でない（一覧か表）＝台帳 id を切り出せない`）、無いか空の値なら違反（`… が無い＝台帳 id が無い`）。在り処の字は `meta.approval.ruling`・`条 <id> の amended_by[<n>].ruling`・`行 <id> の ruling`・`approval.ruling`（file は `adr/<id>.yaml`）・`meta.approval[<n>].ruling`（file は `design-note/<file>`）・`meta.approval[<n>].stamp`。どの形の種類で足りるかと、台帳に在るかは数えない。頭の注に 1 項。
3. **判定の関数を 1 つに寄せる。** `adr.rs` の has_ledger_id と check_approval の裁定の欄の検査・`note.rs` の check_meta の裁定の欄の検査・`link.rs` の amended_by の裁定の欄の検査を消し（決定の欄の床が同じ欄を種別 裁定 id で数える）、`floor.rs` の has_ledger_id を消して関数 marked は ruling::has_ruling を使う。`anchor.rs` の承認一覧の検査と `schema.rs` の単体の歯も has_ruling を使う。`note.rs` の check_note は読めた設計ノートの一覧を返す。`main.rs` に mod ruling。
4. **`design-intent/constitution.yaml`（条文の外・条 A-2.2・ADR-31 決定 (7)）。** 条 P-12 の機構の注の数える範囲に「決定の欄の裁定 id の形」を足し、決定の欄（ADR-31 決定 (1) の閉じた一覧）と床が数えること（1 つ以上切り出せるか・台帳に在るかと形の種類は数えない）と、5 正本の承認欄の行の対話面ほかの欄充足はまだ数えないことを書く。条 P-17 の機構の注の末尾に「床が数えるのは今在る各行の ruling に台帳の裁定 id が 1 つ以上在る形まで（ADR-31 決定 (1)・便 181 から・骨格の印は まだ分からない）。行を消す変更と、裁定 id を前の裁定のまま値を変える変更は、この形の床には見えない」を足す。字は起草の記録の mech-181.py のとおり（機構の kind・live・stage・polarity と relations は変えない）。
5. **fixture（字の期待だけ）。** 床の土台の規則の表の 14 行（R-2・R-3・R-4・R-8・R-9・R-10・R-11・R-12・D-1・D-2・D-4・D-7・D-8 の ruling の頭に初版の裁定 `f2-648.1 notes 2026-09-12 20:2x（初版の行の裁定＝憲法の発効の承認・P-17.3）・` を足す・R-19 は `f2-648 notes 2026-09-28 07:18 JST（床の土台の写し・便 179）` にする＝行の数と位置と ruling のほかの欄は不変）。古い形の fixture 16 組の規則の表の各行に `ruling: f2-648.2 notes 2026-09-12` を足す。字は起草の記録の fixtures-181.py（段 181）のとおり。土台の索引の凍結 anchor 2 本（`graph-anchor.txt` の節点の表と出力全体の値・`node-digest-anchor.txt` の 14 節点の行と要約の行）は独立の script `tests/fixtures/schema/node-digest.py` の出力と、索引の要約値の列がそれと一致することを確かめた folio の出力から組み直した（anchors-18x.py・辺の表と残差は不変）。
6. **既存の歯の字の期待（中身は変えない）。** `tests/init.rs`（骨格の まだ分からない 2 → 6 と骨格の欄 4 つの行）・`tests/mechanism_live.rs` と `tests/outside_faces.rs`（骨格の要約の 2 → 6）・`tests/note.rs`（f161_ の 2 本の裁定の欄の行を種別 裁定 id の字に）・`tests/modules.rs`（層の一覧に ruling を層 1 で）・`tests/graph.rs`（土台の索引の要約値の anchor の sha256）。
7. **変えないもの。** 判断の記録の欄の決まりの生成区間（`floor_adr.rs` の FLOOR の ruling_pattern とその注・`design-intent/adr/schema.yaml`）＝便 182 が運ぶ。書き出し（--emit-rulings・便 C）・計画のノート（便 B）・判断の表の行（便 B が決定の欄に足す）・承認欄のほかの欄の検査（who・date・verbatim・surface）・凍結 anchor の承認一覧の形・床の 7 本の一覧・要件書と判断の記録の字・規則の表の行。folio2 自身の床は合格のまま（(e) の 3）。

### (c) 歯（f181_・base で 0 件）

binary の歯は新しい `crates/folio/tests/ruling.rs` の 5 本。土台（床の土台）の写し全部を一時 dir に作り、字を 1 か所ずつ変えて素の `folio check` を撃つ。版管理は作らない（土台の写しの床は器の導出 file と版管理の 2 つが「まだ分からない」）＝歯は種別 裁定 id の違反の行（標準出力）と「まだ分からない」の行（標準エラー）を数える。

1. **f181_a_rules_row_without_a_ledger_id_is_one_violation（AC28 の前半）。** 土台の写しで種別 裁定 id の違反は 0。行 R-10 の ruling を `G16=A（受入 (f)）` にすると違反がちょうど 1 行 `[裁定 id] rules.yaml: 行 R-10 の ruling「G16=A（受入 (f)）」に台帳 id が無い（…）`、語頭でない台帳の id の形（`G16=A・folio2-648`）でも同じ 1 行で、「まだ分からない」の行は土台と同じ。`未記入` にすると違反 0・「まだ分からない」がちょうど 1 行増え、その行は `rules.yaml: 行 R-10 の ruling が 未記入（骨格の印・裁定の前＝条 P-17.3）`。**base は違反 0 ＝RED。**
2. **f181_each_kind_of_decision_field_is_read。** 決定の欄の各種類の 1 つ（憲法の発効の承認・行 D-8・ADR-2 の承認欄・要件書・入口・天井の正本・相談窓口の承認の行の stamp）の値を `発効承認` にすると、その欄の違反がちょうど 1 行（在り処の字は (b) の 2）。憲法の条 P-1 に改訂来歴（ruling `持ち主の裁定`）を足し、索引の欄の決まり `graph.yaml`（承認欄の行 2 つ・作成の行と `発効` の承認の行）を置くと、違反は改訂来歴と graph.yaml の 2 行目の 2 行だけ（作成の行は数えない）。設計ノートの承認欄は `tests/note.rs` の f161_（(b) の 6）。**base は違反 0 ＝RED。**
3. **f181_rows_without_a_decision_are_not_read。** 土台の憲法の条 P-2 の supersedes_v1 の `裁定 #4` と、要件書の作成の行 `起草` とレビューの行は違反にならない（レビューの行を `未記入` にしても 0）。作成の行の役を `確認` に替えると、その行 `srs.yaml: meta.approval[0].stamp「起草」` の違反 1 行。**base は違反 0 ＝RED。**
4. **f181_the_mark_waits_only_in_the_skeleton_fields。** 憲法の発効の承認と行 D-8 と ADR-2 の承認欄と要件書の承認の行を一度に `未記入` にすると、違反は ADR-2 と要件書の 2 行だけで、憲法と行 D-8 は「まだ分からない」の行。行 R-10 の ruling を一覧 `[f2-648.1]` にすると `が字でない（一覧か表）＝台帳 id を切り出せない`、欄を消すと `が無い＝台帳 id が無い` の違反 1 行。**base は違反 0 ＝RED。**
5. **f181_every_decision_field_of_the_base_is_counted。** 決定の欄を持つ土台の 16 file（憲法・規則の表・要件書・入口・天井の正本・相談窓口・判断の記録 10 本）の `f2-` と `s2-` を大字にすると、種別 裁定 id の違反は **55 行**（憲法 1・規則の表 27・判断の記録 10・要件書 9・入口 4・天井の正本 3・相談窓口 1）。55 と内訳は独立の実装（検証役の fields.py・yq と字の走査）が同じ変えた写しで数えた数（形なし 55）と一致する（起草の記録の indep-181.sh）。**base は違反 0 ＝RED。**
6. **単体の歯 3 本（`crates/folio/src/ruling.rs` の tests の区間）。** f181_the_three_forms_are_all_cut_in_order（`t3-hub.56:20260927T2259Z-1・f2-648 notes 2026-09-28 07:18 JST（t3-hub.1）` から question・notes-time・bead の 3 つを字と台帳の id の部分つきで順に・tsuzuri の形〔覚え書きの台帳の id が先〕も全部）・f181_the_grammar_reads_word_heads_steps_and_notes_times（`.数字` の段 3 段・時刻だけ・分の x・日付だけ・JST・notes の日時の無い notes・分が 1 桁・問いの印の欠け・語頭でない 4 通り・英大字・形でない字 11 通りと、語頭の 4 通り）・f181_the_skeleton_and_the_skipped_roles_are_closed（骨格の欄 3・数えない役 2・形の名 3）。**base は file が無く `--bin folio f181_` は 0 本（nextest の rc 4）＝RED。**
7. **RED の実測。** 歯の file 7 本だけ（新しい tests/ruling.rs と (b) の 6 の字の期待）を base に当てると 18 本が落ちる（f181_ の 5 本と、字の期待を直した既存の歯 13 本・本文は起草の記録の red-181.log）。
8. **文法の 2 実装の一致（歯の外・起草の記録）。** 見本の関数 rulings と検証役の独立の実装（ruling.py・正規表現を使わない字の走査）を、folio2 の正本と tests/fixtures の全 YAML と tsuzuri の写しの字の値 7,539 と手の字 25 に当てると、切り出し 992 が全部一致した（割れる 0・grammar-diff.sh）。

### (d) 採らなかった形

1. **1 便で運ぶ。** 割る前の見本（前の起草役の途中の見本を本流へ載せたもの）は 135,170 byte で、歯を足すと審査の上限 120,000 byte を超える。大きいのは床の土台の規則の表の 14 行（1 行が長く 21,667 byte）と、判断の記録の欄の決まりの写し 19 本（17 本の fixture・正本・生成区間の anchor）。
2. **写しを先に運ぶ（182 → 181）。** 生成区間の注が「床は決定の欄を数える」と書くのに床が数えない期間ができ、一覧の定数が使われない（dead code）。本便を先にすると、181 の着地から 182 の着地までの間、欄の決まりの ruling_pattern は前の字（`[a-z]\d-[0-9a-z]+(\.\d+)?`・今の判定より広い）のままで、決定の欄の一覧は実装の定数にだけ在る（条 P-5.6 の写しが 1 便だけ遅れる）。床の側が先に正しく数えるほうを採った。
3. **has_ledger_id を 2 つとも残し、欄ごとに今の関数で数える。** 判断の記録の承認欄（窓のどこでも）と外の置き場の印（語頭）で答えが割れうる。ADR-31 決定 (3) は 1 つの関数を求める。今の 2 置き場で割れる欄は 0（ADR-31 文脈 (3)・(e) の 3）。
4. **歯の写しで版管理を作る（git init と commit）。** 土台の床を合格 0/0 にできるが、歯は種別 裁定 id の行と「まだ分からない」の行の増減だけを見れば足りる。版管理を作らないので速く、署名の設定に依らない。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写し（見本 a2f8606）で、workspace の nextest **1024 / 1024**（1016 + f181_ の 8 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 36 file（base と比べ変わるのは `constitution.html`〔条 P-12 と P-17 の機構の注〕だけ）。字の期待を直した既存の歯は (b) の 6 のとおりで、直さないと 12 本が落ちる（graph 1・init 2・mechanism_live 2・modules 1・note 2・outside_faces 4・old-teeth-181.log）。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f181_ と note の f161_who を撃つ）。** 16 通りとも落ちる（mut-181.log・各変異の本文は mut-181-M<n>.out）。歯の略: 形 3 = three_forms・文法 = grammar・閉じ = skeleton_and_the_skipped_roles・歯 1〜5 = (c) の 1〜5・f161 = note の f161_who の 2 本。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 最初の id だけを拾う | 文法・形 3 |
| M2 問いの形を台帳の id だけの形にする | 形 3 |
| M3 作成とレビューの行も数える | 歯 1〜5・f161 |
| M4 未記入を全欄で まだ分からない にする | 歯 4・f161 |
| M5 supersedes_v1 の裁定を数える | 歯 1〜5・f161 |
| M6 語頭の判定を外す | 文法・歯 1 |
| M7 開発規律の節を読まない | 歯 2・4・5 |
| M8 一覧・表の値を通す | 歯 4 |
| M9 欄が無いを通す | 歯 4 |
| M10 索引の欄の決まりを読まない | 歯 2 |
| M11 「.数字」の段を 1 段だけ読む | 文法 |
| M12 分の x を読まない | 文法 |
| M13 JST を含めない | 文法・形 3 |
| M14 判断の記録の承認欄を読まない | 歯 2・4・5・f161 |
| M15 設計ノートの承認欄を読まない | f161 |
| M16 憲法の発効の承認を読まない | 歯 2・4・5 |

3. **外の置き場と folio2 自身。** 見本の binary で、folio2 の正本は種別 裁定 id の違反 0（床 合格 0/0）、tsuzuri の写し（d927a57）も違反 0 で床 合格 0/0（本流の binary と同じ）。決定の欄の数は独立の実装と同じ（folio2 155・tsuzuri 56・土台 55・indep-181.log）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file の `crates/folio/src/ruling.rs` と `crates/folio/tests/ruling.rs`（既存の dir の下）は頭に `+`。縮む `crates/folio/src/adr.rs`・`crates/folio/src/floor.rs`・`crates/folio/src/link.rs`・`crates/folio/src/note.rs` は頭に `-`。ほかは印なし。差分 103,145 byte（`git diff 78a793f a2f8606 | wc -c`・36 file・+736 −184）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 9 本。各行を ceil(字数 / 120) で数えて足す（空行は 1）。起草役は python と awk の 2 実装で数え、一致した（cap-18x.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/ruling.rs` | 0（新しい file） | 1500 | 337 | 1163 |
| `crates/folio/src/check.rs` | 1056 | 444 | 1105（+49） | 395 |
| `crates/folio/src/adr.rs` | 993 | 507 | 972（−21） | 528 |
| `crates/folio/src/anchor.rs` | 1071 | 429 | 1072（+1） | 428 |
| `crates/folio/src/floor.rs` | 727 | 773 | 716（−11） | 784 |
| `crates/folio/src/link.rs` | 669 | 831 | 661（−8） | 839 |
| `crates/folio/src/main.rs` | 686 | 814 | 687（+1） | 813 |
| `crates/folio/src/note.rs` | 977 | 523 | 971（−6） | 529 |
| `crates/folio/src/schema.rs` | 240 | 1260 | 241（+1） | 1259 |

3. **size は M。** src の増分は新しい file の 337 を含み S の見積 100 を超える。余地の最小は check.rs の 395 で、M の 300 の内。
4. **verify は 8 行**で、done の 8 つの塊と 1 対 1 に揃える。便の後の写しで 8 行とも rc 0。
   1. `cargo nextest run -p folio --test ruling f181_` = (c) の 1〜5（5 本）。
   2. `cargo nextest run -p folio --bin folio f181_` = (c) の 6（3 本）。
   3. `cargo nextest run -p folio --test note f161_` = 設計ノートの承認欄（f161_ の 3 本・裁定の欄は種別 裁定 id）。
   4. `cargo nextest run -p folio --test init` = 骨格の欄 4 つが「まだ分からない」（骨格の床 違反 0・まだ分からない 6）。
   5. `cargo nextest run -p folio --test mechanism_live` = 骨格の要約の字。
   6. `cargo nextest run -p folio --test outside_faces` = 骨格の要約の字。
   7. `cargo nextest run -p folio --test graph` = 土台の索引の凍結 anchor。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（ruling・note・init・mechanism_live・outside_faces・graph）は全部 write-set に在る。`tests/modules.rs` は共通の検証（workspace の nextest）で撃つ。

### (g) 門と受付

1. **門。** 冒頭のとおり対象・0（通す）。
2. **受付。** 本便を受け付けてから便 182 を受け付ける（182 の見本は本便の見本の上）。受付の先撃ち（precheck）は枝 docs/d181m（本流 78a793f の上）で契約に起因する断り 0（起草の記録の precheck-181.log）。共通の検証は同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/d181-draft.md`、script と log は同じ dir の d181-scripts（repo には入れない）。

1. 模擬: 見本 a2f8606（`git diff 78a793f a2f8606`）。憲法の注は mech-181.py、fixture は fixtures-181.py（段 181）、索引の凍結 anchor は anchors-18x.py（段 181）で組み直せる。run-181.sh（組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と要約）。
2. RED と既存の歯: red-18x.sh（歯の file だけを base に当てる）・old-teeth-181.sh（歯の file を base の字に戻す）。
3. 突然変異: mut-18x.py（段 181・16 通り・最後に戻して組み直す）。
4. 独立の数え: indep-181.sh（fields.py と見本の binary で、土台・大字にした土台・folio2・tsuzuri の写し）。文法の 2 実装: grammar-diff.sh。
5. 余地: cap-18x.sh（lines.py と lines.awk）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 判断の記録の欄の決まりの生成区間の文法と一覧の写し（便 182）・書き出し（便 C）・計画のノートと判断の表（便 B）・要件書と判断の記録と条文の字・規則の表の行・承認欄のほかの欄の検査・台帳への記帳（席）・tsuzuri の字（tsuzuri の手番）・外部 crate・新しい dir。
2. **言えないこと。** (1) 床は形だけを見る。会話の決定を覚え書きの台帳の id（例 t3-hub.1）の下に写した欄も通る（ADR-31 帰結 1・止めるのは器）。(2) 台帳の id の形は英小字 1・数字 1・`-` で始まる語頭の字で、台帳の本当の番号かは見ない（例 `a1-b2` も 1 つに数える）。(3) 決定の欄は Tree に載る木だけで、読めない正本（「まだ分からない」の file）と、判断の記録の床が読めなかった置き場の判断の記録の承認欄は数えない（その置き場の床は既に合格でない）。(4) 設計ノートの承認欄は note の床が読めた設計ノートだけ。(5) 5 正本の承認欄の行の対話面・逐語の欄充足は数えない（条 P-12 の機構の注）。(6) 行を消す変更と、前の裁定 id のまま行の値を変える変更は形の床には見えない（条 P-17 の機構の注・条 P-17.2 の差分の口は未実装）。
3. **撤退条件。** (1) 本便が要件書 FR26 / FR5 の字か憲法の条文を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `check.rs` の check_dir・`adr.rs` の check_approval・`note.rs` の check_note と check_meta・`link.rs` の amended_by・`floor.rs` の marked が base（78a793f）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) の 6 に挙げた外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が `constitution.html` のほかで 1 byte でも変われば、止めて席へ返す。(5) ADR-31 の撤退条件 (1)（欄の字を直さずに済んだ誤りが 3 件目）は着地の後に席が数える。

## 2. 範囲

- 入れる: 新しい `crates/folio/src/ruling.rs`（文法・形の種類・決定の欄の名と骨格の欄と数えない役・歩き手・単体の歯 3 本）・`check.rs` の check_rulings と load_graph と check_dir の呼び出し・has_ledger_id を 2 つとも消して has_ruling に寄せる（adr.rs・floor.rs・anchor.rs・schema.rs）・adr.rs・note.rs・link.rs の裁定の欄の検査を外す・note::check_note の返り値・mod ruling・憲法の条 P-12 と P-17 の機構の注・fixture の規則の表の ruling（土台 14 行・古い形 16 組）・土台の索引の凍結 anchor 2 本・既存の歯の字の期待 6 本・新しい歯の file（5 本）。
- 入れない: 欄の決まりの生成区間（FLOOR・adr/schema.yaml）・書き出し・計画のノート・判断の表・条文と要件書と判断の記録の字・規則の表の行・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| grammar | 文法 | `ruling.rs` の関数 rulings と has_ruling（語頭の台帳の id・question / notes-time / bead・全部を切り出す） |
| walker | 歩き手 | `ruling.rs` の関数 sites（決定の欄を一覧の順に全部拾う・作成とレビューの行と supersedes_v1 は拾わない） |
| floor | 床 | `check.rs` の関数 check_rulings（種別 裁定 id の違反と、骨格の欄の未記入の「まだ分からない」） |
| merge | 寄せ | adr.rs・floor.rs の has_ledger_id を消し、4 か所の裁定の欄の検査を決定の欄の床へ |
| notes | 注 | 憲法の条 P-12 と P-17 の機構の注の数える範囲 |
| teeth | 歯 | f181_ の 8 本（binary 5・単体 3）と既存の歯の字の期待 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない（歯は python を使わない）。新しい dir は無い。
- 前提の着地: ADR-31 と要件書 第 1.53 版（PR #371・本流 78a793f）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。便 182 を受け付ける。本流の `target/debug/folio` を組み直す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gb"
title = "決定の欄の裁定 id の形を床で数える（判断の記録 ADR-31 の便 A の前半・決定 (1)(3)(7)・要件書 第 1.53 版の FR26 の床の 2 文と AC28 の前半）: 新しい crates/folio/src/ruling.rs に裁定 id の文法の関数 rulings（語頭の台帳の id を全部切り出し、器の問いの印が続けば question、notes の日付か時刻と任意の JST が続けば notes-time、どちらも無ければ bead）と has_ruling と、決定の欄（憲法の発効の承認と改訂来歴・規則の表の全行の ruling・判断の記録と設計ノートの承認欄の ruling・要件書と入口と天井の正本と相談窓口と索引の欄の決まりの承認欄のうち役が作成とレビューでない行の stamp）を拾う歩き手 sites を置き、crates/folio/src/check.rs の check_rulings が欄ごとに、骨格が書く欄（憲法の発効の承認と規則の表の行）の値が 未記入 なら まだ分からない、1 つも切り出せない字・一覧か表の値・無い欄を種別 裁定 id の違反にする（形の種類と台帳に在るかは数えない・supersedes_v1 は数えない・索引の欄の決まり graph.yaml は在れば読む）。adr.rs と floor.rs の has_ledger_id を消して has_ruling に寄せ、adr.rs・note.rs・link.rs の裁定の欄の検査を決定の欄の床へ移し、anchor.rs も has_ruling を使う。憲法の条 P-12 と P-17 の機構の注の数える範囲を直す。床の土台の規則の表の 14 行と古い形の fixture 16 組の規則の表の行に台帳の id を足し、土台の索引の凍結 anchor 2 本と、骨格の まだ分からない の数（2 から 6）ほかの既存の歯の字の期待を直す。判断の記録の欄の決まりの生成区間の写しは便 182（行 gc）が運ぶ。歯は f181_ の 8 本（新しい crates/folio/tests/ruling.rs の 5 本と ruling.rs の単体の 3 本）。実装の見本は origin の枝 impl/d181 の commit a2f8606（親 78a793f）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 78a793f"
req = ["FR26", "FR5"]
section = "1"
write-set = ["+crates/folio/src/ruling.rs", "crates/folio/src/check.rs", "-crates/folio/src/adr.rs", "crates/folio/src/anchor.rs", "-crates/folio/src/floor.rs", "-crates/folio/src/link.rs", "crates/folio/src/main.rs", "-crates/folio/src/note.rs", "crates/folio/src/schema.rs", "+crates/folio/tests/ruling.rs", "crates/folio/tests/note.rs", "crates/folio/tests/init.rs", "crates/folio/tests/mechanism_live.rs", "crates/folio/tests/outside_faces.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "design-intent/constitution.yaml", "tests/fixtures/floor_base/design-intent/rules.yaml", "tests/fixtures/adr/effective-no-approval/rules.yaml", "tests/fixtures/adr/schema-drift/rules.yaml", "tests/fixtures/adr/two-adopted/rules.yaml", "tests/fixtures/anchor/no-anchor/rules.yaml", "tests/fixtures/anchor/root-digest-drift/rules.yaml", "tests/fixtures/check/dup-key/rules.yaml", "tests/fixtures/check/empty-field/rules.yaml", "tests/fixtures/check/unknown-section/rules.yaml", "tests/fixtures/link/adr-id-missing/rules.yaml", "tests/fixtures/link/amended-by-orphan/rules.yaml", "tests/fixtures/link/retreat-kind-drift/rules.yaml", "tests/fixtures/refs/bad-counts/rules.yaml", "tests/fixtures/refs/dangling-id/rules.yaml", "tests/fixtures/refs/orphan-rule/rules.yaml", "tests/fixtures/vocab/exemptions/rules.yaml", "tests/fixtures/vocab/unknown-word/rules.yaml", "tests/fixtures/schema/graph-anchor.txt", "tests/fixtures/schema/node-digest-anchor.txt"]
verify = ["cargo nextest run -p folio --test ruling f181_", "cargo nextest run -p folio --bin folio f181_", "cargo nextest run -p folio --test note f161_", "cargo nextest run -p folio --test init", "cargo nextest run -p folio --test mechanism_live", "cargo nextest run -p folio --test outside_faces", "cargo nextest run -p folio --test graph", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/ruling.rs の f181_ の 5 本（床の土台の写しで規則の表の行 R-10 の ruling を G16=A（受入 (f)）か語頭でない folio2-648 にすると種別 裁定 id の違反がちょうど 1 行で、未記入 にすると違反 0 で まだ分からない がちょうど 1 行増える／憲法の発効の承認・行 D-8・ADR-2 の承認欄・要件書と入口と天井の正本と相談窓口の承認の行の stamp・足した改訂来歴・置いた graph.yaml の承認の行がそれぞれ違反 1 行で graph.yaml の作成の行は数えない／supersedes_v1 の 裁定 #4 と作成とレビューの行は数えず、役を替えた行は数える／未記入 は憲法の発効の承認と規則の表の行だけ まだ分からない で ADR-2 と要件書は違反、一覧の値と無い欄は違反／土台の 16 file の f2- と s2- を大字にすると違反 55 行〔憲法 1・規則の表 27・判断の記録 10・要件書 9・入口 4・天井の正本 3・相談窓口 1〕）が緑、binary の単体の f181_ の 3 本（3 つの形を全部順に・語頭と .数字の段と notes の日時の文法・骨格の欄と数えない役と形の名）が緑、tests/note.rs の f161_ の 3 本（設計ノートの承認欄の裁定の欄は種別 裁定 id）が緑、tests/init.rs の歯の全部（骨格の床は違反 0・まだ分からない 6 で骨格の欄 4 つの行を持つ）が緑、tests/mechanism_live.rs の歯の全部が緑、tests/outside_faces.rs の歯の全部が緑、tests/graph.rs の歯の全部（土台の索引の凍結 anchor を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
