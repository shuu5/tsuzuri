# 設計: 便 144 — 憲法の面に今の版の承認の日付と版ごとの承認の行（判断の記録への導線）を出し、入口の棚の憲法と要件書のカードの更新の日付を承認の日付にする

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は憲法の面の頭の 3 か所（鮮度の札・表紙の状態・承認欄のリード）と承認欄の行、入口の棚の憲法と要件書のカードの更新の行の字を、正本に在る欄（判断の記録の amends と承認欄・要件書の承認欄の行）から組むように直すだけで、FR4 の規範文も面の部品・色・並びも様式の定義も変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する＝版ごとの承認は判断の記録の正本の欄から逐語で出す）/ P-6.3（同じ内容を 2 つの面が持つときは一方を正本とする＝版ごとの承認の正本は判断の記録で、凍結 anchor の承認一覧はその写しなので読まない）/ P-4.2（判定できないものは まだ分からない として表に出す＝今の版を名指す発効した判断が無ければ頭に 効く版はまだ分からない と出す）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-6.2（生成物を手で直さない＝入口の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 天井の 36 周目（2026-09-25）の読みやすさの所見 **F-2**（重さ 直す・場所 憲法の meta.approval）。一括 22 の仕分け（`docs/design/batch22-triage.md`）の便の候補 **B-1**。台帳の控え **f2-648.217**。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eq` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 12 本（書き換える 8 本 + 本文が変わらない verify の scope 4 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 12 本を本流の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提の着地は無い。**base = main 118c2f4（一括 22 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。本流の作業ツリーの binary は 2fbc1af の組み立てで、2fbc1af から 118c2f4 までに `crates/` で変わったのは歯の file `crates/folio/tests/face_style.rs` の新設だけ（src と Cargo.lock は不変・起草役の `git diff --stat` の実測）なので、面の出力は 118c2f4 の組み立てと同じである。
- 並行の便との重なり: base の時点で、契約の枝のうち main に着地していない便は無い（便 143 は着地済み・起草役の実測）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 118c2f4・数は参考値）

1. **憲法の正本の欄。** 憲法の正本 `design-intent/constitution.yaml` の欄 meta は、版 version（v1.4）・状態 status（effective）・生成日 generated（2026-09-12）・承認欄 approval を持つ。承認欄は初回の発効の承認の 1 件（2026-09-12・逐語 承認する）だけで、版の番号を持たない。第 1.1〜1.4 版の承認は憲法の正本の欄には無く、版を上げた判断の記録の正本（`design-intent/adr/ADR-n.yaml`）の欄 amends（改訂の各項が上がる版 version を持つ）と承認欄 approval（who・date・ruling・verbatim・surface）に在る。
2. **版を上げた発効した判断は 4 本（起草役の実測）。** 発効した判断は、欄の決まり `adr/schema.yaml` の effective_status（accepted か retired）で承認欄が表のもの（床の口 anchor.rs の is_effective と同じ定義）。

| 判断の記録 | amends が名指す版 | 承認の日付 | 逐語 |
| --- | --- | --- | --- |
| ADR-11 | v1.1 | 2026-09-20 | 承認する |
| ADR-12 | v1.2 | 2026-09-21 | 承認する |
| ADR-17 | v1.3 | 2026-09-24 | 全部推奨で |
| ADR-23 | v1.4 | 2026-09-25 | すべて承認する |

   各条の改訂来歴の欄 amended_by には ADR-11・ADR-12・ADR-17 が在るが、第 1.4 版の ADR-23 は schema 節と前文だけを改めたので、どの条の amended_by にも無い（前文の欄の決まりは amended_by を持たない）。凍結 anchor（`design-intent/anchors/constitution-v1.N.yaml`）の承認一覧は、凍結の時点で判断の記録の承認欄を写したもの（freeze.rs の build）で、正本ではない。
3. **憲法の面の字（`folio build --dir design-intent --out <置き場> --write` の constitution.html）。**

| 所 | base の字 | 読める意味 |
| --- | --- | --- |
| 鮮度の札 | 生成 2026-09-12 · v1.4（発効・拘束力あり） | 生成日は初回の日付 |
| 表紙の版の札 | 版 v1.4 / 2026-09-12 | v1.4 が 2026-09-12 の版と読める |
| 表紙の状態 | 発効・拘束力あり（承認 2026-09-12） | v1.4 が 2026-09-12 に承認されたと読める（誤り） |
| 承認欄 | 作成の行と、初回の承認の 1 行だけ | 第 1.1〜1.4 版の承認と判断の記録へ辿れない |

   表紙の状態の日付は、生成器 `crates/folio/src/face_constitution.rs` の関数 cover が meta.approval.date を読むので初回の日付になる。承認欄（関数 approval）も meta.approval の 1 行だけを出す。ADR-17 は §6 の改訂の例（条 N-6 の amended_by）から辿れるが、ADR-23 は面のどこにも承認と結びついて出ない（所見 F-2 の証拠の字）。
4. **入口の棚のカード（index.html）。** 憲法のカードの更新の行は 更新 2026-09-12・v1.4、要件書のカードは 更新 2026-09-12・v1.44 で、どちらも生成器 `crates/folio/src/face_index_read.rs` の関数 updated が meta.generated（初回の生成日）と version を出す。要件書の承認欄の最後の承認の行は 2026-09-26（v1.44）で、要件書の面の表紙の状態は 発効・拘束力あり（承認 2026-09-26）と出している（便 138 の後の形）。判断の記録のカードの更新（記録の日付の最大）と設計ノートのカードの更新（生成日の最大）は本便の範囲の外。
5. **fixture。** 面の fixture の憲法（`tests/fixtures/face/constitution.yaml`）は status draft・generated 2026-09-01・承認欄 2026-09-02、fixture の判断の記録 2 本（`tests/fixtures/face/adr/ADR-1.yaml`・`ADR-2.yaml`）はどちらも提案中で amends を持たない。面の fixture の要件書は status effective・generated 2026-09-01・最後の承認の行 2026-09-05。
6. **base の歯（参考値）。** workspace の nextest 911 / 911・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file。`git grep -n 'f144_' -- crates` は 0 件。歯の file の本数は face_constitution 19・face_index 26・face_index_sheet 14。

### (b) 直す先 — 版を上げた発効した判断を読む口を 1 つ置き、憲法の面の頭と承認欄・入口のカード 2 枚で使う

1. **読み手の口（`crates/folio/src/face_constitution_read.rs` に足す）。**
   - 型 Amend（判断 1 本 × amends が名指す版 1 つ・id・版・承認欄の who・date・verbatim・ruling・surface・どの字も escape 済み）と関数 amendments（置き場の根を受ける）: `adr/` の直下の ADR-<数>.yaml を番号の昇順に読み、状態が欄の決まりの effective_status の値（床の口 `crate::adr::floor_strs` の effective_status を読む＝値を写さない）で承認欄が表の判断だけについて、amends の表の項の version を出た順に重ねずに集め、1 行ずつ返す。提案中の判断・承認欄の無い判断・amends の無い判断は読まない。`adr/` が dir でなければ空を返す（(b) の 5）。
   - 型 Approved（版の立場・日付・今の版を名指す判断の id の一覧）と関数 approved（欄 meta と Amend の行を受ける）: status を文書の状態の表（既存の DOC_STATUS）で引き、表の外は Err。effective でなければ Draft（行を読まず、日付は generated）。effective なら、version と字が同じ版を名指す行が在れば Effective（日付はその行の承認の日付の最も新しいもの・id は行の順）、行が 1 つも無ければ Effective（日付は meta.approval.date・初回の発効の版）、行は在るが今の版を名指すものが無ければ Unknown（日付は在る承認の最も新しいもの〔meta.approval.date を含む〕）。版の新旧は比べない（便 138 と同じ・字が同じ行だけを拾う）。版の立場の型は便 138 の Standing（`crates/folio/src/face_labels.rs`・名札 効く版はまだ分からない）をそのまま使い、Pending は出さない（憲法は効いている版の欄を持たない）。
2. **憲法の面（`face_constitution.rs`）の 4 か所。** 関数 derive で amendments と approved を 1 度だけ呼び、次の 4 か所に渡す。Effective（初回の版）と Draft は base と 1 byte も変えない。

| 所 | Effective（今の版を名指す判断が在る） | Unknown（今の版を名指す判断が無い） |
| --- | --- | --- |
| 鮮度の札 | base のまま（v1.4（発効・拘束力あり）） | <version>（効く版はまだ分からない） |
| 表紙の状態 | 発効・拘束力あり（承認 <日付>・判断の記録 <id を ・ でつなぐ>） | 発効・拘束力あり（承認 <在る承認の最も新しい日付>）・効く版はまだ分からない |
| 承認欄のリード | base のまま（発効・拘束力あり） | 発効・拘束力あり（効く版はまだ分からない） |
| 承認欄の行 | 作成の行と初回の承認の行の後に、Amend の行を番号の順に 1 行ずつ足す | 同じ（在る行だけ） |

   鮮度の札とリードと表紙の Unknown の字は便 138 の Standing の口（stamp・lead・cover）をそのまま呼ぶ。足す承認の行は既存の承認の行と同じ部品と class（approval-block の中の sign・role・who・when・stamp）で、字は 承認 ／ <who> ／ <date> · 版 <version> · 逐語「<verbatim>」 ／ 判断の記録: <id>／裁定: <ruling>／対話面: <surface> ／ 名札 発効 の 5 つの枡。行は状態によらず出す（版を上げた発効した判断の承認は事実なので）。判断の記録の番号は面の後処理（便 135 の口 face::link_ids）が行き先の面 adr-n.html へのリンクにするので、表紙の状態と承認の行の番号から判断の記録の面へ辿れる（本便は包みを書かない）。本流の面の変化は (e) の 5。
3. **入口の棚のカード（`face_index_read.rs`）。** カードの更新の行の名は 更新 のまま、日付だけを次に替える。版の字と便 138 の札は今のまま。
   - 憲法のカード（関数 constitution_card）: approved の日付・version・札（Standing の card）。Effective は 更新 <今の版の承認の日付>・<version>、Unknown は 更新 <在る承認の最も新しい日付>・<version>（効く版はまだ分からない）、Draft は base のまま（更新 <generated>・<version>）。
   - 要件書のカード（関数 srs_card）: 便 138 の Standing が Draft でなければ、承認欄の最後の 承認 の行の when（要件書の面の表紙の状態と同じ式）、承認の行が無いか Draft なら generated。後ろの札（起草・承認待ち・発効は <効いている版>／効く版はまだ分からない）は変えない。
   - 関数 updated（generated と version をつなぐだけ）は使う所が無くなるので消す。入口の面の口 `crates/folio/src/face_index.rs` の derive は amendments を 1 度呼び、読み手の関数 context に憲法の正本と一緒に渡す（引数の数を増やさないように、憲法の引数を正本と行の組にする。形は実装が決めてよいが、clippy の引数の数の警告を出さない）。
4. **変えないもの。** 憲法の面の題・鮮度の札の生成日の字と名（共有の口 Frame::head・5 面共有）・表紙の版の札（版 <version> / <generated>）・§6 の改訂の例・脚・部品と class・様式の定義・章と節の並び。入口の面の頭・判断の記録と設計ノートのカード。要件書の面（`face_srs.rs` は触らない）。判断の記録と設計ノートの面。床。設計文書。凍結 anchor の列（`design-intent/anchors/`）。
5. **契約で決めること（起草役の判断）。**
   - **版ごとの承認は判断の記録の正本から読む（凍結 anchor の承認一覧は読まない）。** anchor の承認一覧は凍結の時点の写しで（P-6.3）、版を上げる枝の上では判断の記録が先に在る。憲法の正本に版ごとの承認の欄を足す形は設計文書の一括（門の対象）が先に要る（(d) の 1）。
   - **今の版を名指す発効した判断が無ければ、面は書き、頭に 効く版はまだ分からない と出す（P-4.2）。** 導出しない（終了コード 2）形は便 138 と同じ理由で採らない。版を上げた判断が 1 本も無い正本（初回の版だけの正本・外部の利用者の憲法）は、今の版の承認が meta.approval なので Effective で、base と字が変わらない。
   - **`adr/` が dir でない写しでは行を空にし、面は今の字で書く。** 起草役は `adr/` が読めなければ Err（入口の面と同じ）にした模擬も撃ち、既存の歯 3 本（`crates/folio/tests/face.rs` の count_word_uses_tsu_only_up_to_nine と face_check_has_three_values・`crates/folio/tests/face_srs.rs` の f84_missing_field_terms_leaves_the_face_unchanged）が判断の記録の置き場を持たない写しで憲法の面を組んで落ちた。憲法の面は今まで `adr/` を要さないので、その約束を本便で広げない。`folio init` の雛形と本流の置き場は `adr/` を持つ。
   - **入口のカードの名は 更新 のまま。** 名を 発効 に替えると、便 138 の札の形（更新 <日付>・v1.42（起草・承認待ち・発効は v1.41））と並べたときに、起草中の版が発効と読める。生成 に替えると所見の求める承認の日付が出ない。承認の日付は、その文書が最後に効力のある形で変わった日なので 更新 の意味と合う。
   - **便 138 の歯 2 本の期待の字の日付（3 か所）を直す。** 要件書のカードの日付が fixture の生成日 2026-09-01 から最後の承認の行 2026-09-05 に替わるので、`crates/folio/tests/face_index.rs` の f138_srs_card_marks_a_pending_version と f138_srs_card_marks_an_unknown_effective_version の期待の字の 2026-09-01 を 2026-09-05 にする（札の字と歯の意図は変えない）。
   - **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は正本に在る欄（判断の記録の amends と承認欄・要件書の承認の行）を面が読まずに誤った日付を出していた中身の正しさの直し（P-6.1・P-4.2）で、部品・色・余白・様式の定義・名札の class を 1 つも変えない。承認欄に足す行は既存の部品と class の行である。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137・138・139・141 の契約（撤退条件 ② に当たらないとした読み）と同じである。仮に数えても、撤退条件 ② の数えに名指した契約（便 15・16・20・27・28・35・36）に、憲法の面の部品（doc-cover-band・approval-block）か入口の shelf-card の見た目だけを直した便は無い（便 14 は憲法の面を、便 16 は入口の面を作った便・起草役の `git grep` の実測）ので、1 本目で上限 2 に届かない。持ち主の walk（面の承認）は章と部品を変えないので要しないと起草役は読む（承認欄に 4 行が加わるので、着地後の面を次の報告で見せることを勧める・求めるかは席が決める）。

### (c) 歯（関数名 f144_・base で 0 件）

1. **f144_approved_follows_the_decisions_naming_the_version（単体の歯・`crates/folio/src/face_constitution.rs` の既存の tests の区間 face_constitution_tests）。** approved が、行の無い発効の正本で Effective と meta.approval.date、今の版を名指す行が 2 本なら Effective と新しい日付と id 2 つ（行の順）、名指す行が無ければ Unknown と在る承認の最も新しい日付（meta.approval.date が新しければそれ）、draft は行を読まず Draft と生成日、版の字を escape した字どうしで比べること、表の外の状態 retired の Err を確かめる。**base では型と関数が無く、組み立てが通らない＝RED。**
2. **f144_cover_names_the_approval_of_the_current_version（`crates/folio/tests/face_constitution.rs`・binary 経由・実の置き場の写し）。** 歯の側の手書きの読み（yaml-rust2 で判断の記録を直に読み、発効した判断の amends が今の版を名指す行を集める）から組んだ表紙の状態 発効・拘束力あり（承認 <日付>・判断の記録 <id>）がちょうど 1 つ在り、初回の日付の（承認 <初回の日付>）が無く、鮮度の札とリードは今の字。**base では（承認 2026-09-12）＝RED。**
3. **f144_approval_lists_every_amending_version（同）。** 承認欄の行が 作成・初回の承認・版ごとの承認の順で、版ごとの行が歯の側の手書きの読みから組んだ字（(b) の 2 の 5 つの枡）と 1 行ずつ等しく、数が 2 + 行の数。**base では 2 行だけ＝RED。**
4. **f144_unknown_when_no_decision_names_the_version（同）。** 写しの版の欄を先に進めた面（<version>-next）と、今の版を名指す判断を提案中に戻した面の 2 通りで、鮮度の札・表紙の状態・承認欄のリードが (b) の 2 の Unknown の列の字でちょうど 1 つずつ在り、今の版を発効と出す札が無く、提案中に戻した判断の承認の行が無い。**base では今の版を発効と出す＝RED。**
5. **f144_without_amending_decisions_the_first_approval_stays（同）。** 版を上げた判断を全部提案中に戻した写しで、表紙の状態が（承認 <初回の日付>）・鮮度の札が今の字・承認欄が 2 行。面の fixture の写しから `adr/` を消しても面を書き（終了コード 0）、承認欄は 2 行。守りの歯（base でも緑・直しが初回の版の字を崩すか、`adr/` の無い写しで面を止めたら落ちる）。
6. **f144_constitution_card_shows_the_approval_of_the_current_version（`crates/folio/tests/face_index.rs`・binary 経由）。** 入口の写しの憲法を effective にし、発効した判断 ADR-3（承認 2026-09-07）を置いた面で、憲法のカードが amends の版 v0.9 なら 更新 2026-09-07・v0.9、v0.8 なら 更新 2026-09-07・v0.9（効く版はまだ分からない）、ADR-3 を置かなければ 更新 2026-09-02・v0.9 でちょうど 1 つ。**base では 更新 2026-09-01・v0.9＝RED。**
7. **f144_srs_card_shows_the_last_approval（同）。** 変異なしの写しで要件書のカードが 更新 2026-09-05・v0.3、憲法のカード（draft）が 更新 2026-09-01・v0.9、承認の行を 1 つ足した写し（2026-09-09）で 更新 2026-09-09・v0.3、要件書を draft にした写しで 更新 2026-09-01・v0.3。**base では 更新 2026-09-01・v0.3＝RED。**
8. **f144_real_cards_follow_the_approvals（同・実の正本）。** 実の入口の面の憲法のカードが歯の側の手書きの読みの今の版の承認の日付と version で、要件書のカードが承認欄の最後の承認の行の日付と version で、それぞれちょうど 1 つ始まる。**base では生成日＝RED。**

fixture は新しい file を足さない。変異は歯の中で一時 dir の写しを書き換える（歯の file の既存の口 real_copy・index_fixture_copy・index_with_srs・edit と同じ形）。判断の記録の正本の読みは歯の側の手書き（生成側の口を呼ばない・P-10.1）。

### (d) 採らなかった形

1. **憲法の正本の承認欄を版ごとの一覧にする（設計文書の一括）。** 同じ事実（版ごとの承認）を判断の記録と憲法の 2 つの正本が持つことになり（P-6.3・P-6.4）、門の対象の一括と持ち主の承認を前置にする。判断の記録に在る欄を面が読めば足りる。
2. **凍結 anchor の承認一覧を読む。** anchor は判断の記録の承認欄の写しで（P-6.3）、版を上げる枝の上では凍結の前に面を組む。
3. **各条の amended_by から版ごとの承認を拾う。** amended_by は版の番号を持たず、前文と schema 節の改訂（ADR-23）を持たない（(a) の 2）。
4. **入口のカードの名を 発効 か 生成 に替える。** (b) の 5 のとおり。
5. **`adr/` が読めなければ面を導出しない（終了コード 2）。** (b) の 5 のとおり、既存の歯 3 本が落ち、憲法の面の今の約束を広げる。
6. **鮮度の札の生成日と表紙の版の札の日付も直す。** 鮮度の札は 5 面共有の口 Frame::head で、表紙の版の札は要件書の面も同じ形（版 v1.44 / 2026-09-12）なので、憲法の面だけを直すと面の間で形が割れる。2 面の同じ形として台帳の控えに残す（§5）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **落ちる既存の歯は 6 本で、どれも本便の凍結 anchor 2 本の 1 か所ずつと便 138 の歯 2 本の日付の字で直る（起草役の実測）。** 実装だけを base に当てた写し（歯の file と凍結 anchor は base のまま）で、workspace の nextest は 911 本のうち 6 本が落ちる。面の fixture の要件書のカードの日付が 2026-09-01 から 2026-09-05 に替わるためである。

| 歯 | 直し方 |
| --- | --- |
| face_index の face_index_write_matches_the_frozen_fixture・face_index_write_with_a_sheet_matches_the_frozen_fixture | 凍結 anchor 2 本の要件書のカードの更新の行の日付 1 か所ずつ（本文は変えない） |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures | 同上 |
| site の site_write_matches_the_frozen_fixture | 同上 |
| face_index の f138_srs_card_marks_a_pending_version・f138_srs_card_marks_an_unknown_effective_version | 期待の字の日付 3 か所（(b) の 5） |

   本便の差分の全部を base に当てた写しで、workspace の nextest は 919 / 919（base 911 + 単体 1 + face_constitution 4 + face_index 3）・clippy 0 警告・床 4 本 rc 0・`folio parts --check`（憲法と入口の面）合格・face_constitution 23 / 23・face_index 29 / 29・face_index_sheet 14 / 14。
2. **動く凍結 anchor は 2 本で、1 か所ずつ。** `tests/fixtures/face/expected-index.html` と `tests/fixtures/face/expected-index-sheet.html` の要件書のカードの更新の行（どちらも 45 行目）の 更新 2026-09-01・v0.3 を 更新 2026-09-05・v0.3 にする（起草役は script で 1 か所ずつ置き換え、生成器の出力と byte で一致した）。byte 数は 12,082 と 12,560 のまま（日付の 1 字の差だけ）。憲法の凍結 anchor `tests/fixtures/face/expected.html`（fixture の憲法は draft）・要件書・判断の記録・設計ノートの面の凍結 anchor・床の凍結の土台・天井の束の凍結 anchor は 1 byte も変えない。
3. **RED（起草役の実測）。** 歯と凍結 anchor 2 本だけを base に当てた写し（単体の歯を除く 918 本）で 12 本が落ちる＝binary の f144_ の 7 本のうち 6 本（(c) の 2・3・4・6・7・8）と、便 138 の歯 2 本（期待の字を先に直したため）と、凍結 anchor 4 本（(e) の 1 の表の上の 3 行）。(c) の 5 は守りの歯で緑。単体の歯は組み立てが通らない。
4. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変える・f144_ の 8 本を撃つ）。** 9 通りとも 1 本以上が落ちる。

| 変異 | 落ちる f144_ |
| --- | --- |
| M1 表紙の日付を初回の承認に戻す | (c) の 2・4 |
| M2 承認欄の版ごとの行を出さない | (c) の 3 |
| M3 今の版を名指す判断が無くても Effective | (c) の 1・4・6 |
| M4 発効していない判断も読む | (c) の 4・5 |
| M5 今の版の承認の日付を最も新しいものでなく最初の行から取る | (c) の 1・4・6 |
| M6 憲法のカードの日付を生成日に戻す | (c) の 6・8 |
| M7 要件書のカードの日付を生成日に戻す | (c) の 7・8 |
| M8 draft でも判断を読む | (c) の 1 |
| M9 鮮度の札に今の版の承認の立場を出さない | (c) の 4 |

5. **folio2 自身の面の変化（起草役の実測）。** base の本流で `folio build --dir design-intent --out <置き場> --write` の出力は 30 file のままで、違う file は constitution.html と index.html の 2 枚だけ（`diff -rq`）。constitution.html は表紙の状態の 1 行が 発効・拘束力あり（承認 2026-09-25・判断の記録 ADR-23）になり、承認欄に第 1.1〜1.4 版の 4 行（ADR-11・12・17・23 のリンク付き）が加わる（−1 行 / +5 行）。index.html は憲法のカードが 更新 2026-09-25・v1.4、要件書のカードが 更新 2026-09-26・v1.44 になる（−2 行 / +2 行）。鮮度の札・表紙の版の札・リードは今の字のまま（本流は Effective）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 12 本とも印なし（書き換える 8 本 = `crates/folio/src/face_constitution_read.rs`・`crates/folio/src/face_constitution.rs`・`crates/folio/src/face_index_read.rs`・`crates/folio/src/face_index.rs`・`crates/folio/tests/face_constitution.rs`・`crates/folio/tests/face_index.rs`・`tests/fixtures/face/expected-index.html`・`tests/fixtures/face/expected-index-sheet.html`／本文不変の 4 本 = `crates/folio/tests/face.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/face_index_sheet.rs`・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 4 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_constitution.rs` | 1,042 | 458 | 1,119（+77・うち単体の歯 約 50） | 381 |
| `crates/folio/src/face_constitution_read.rs` | 268 | 1,232 | 387（+119） | 1,113 |
| `crates/folio/src/face_index.rs` | 790 | 710 | 792（+2） | 708 |
| `crates/folio/src/face_index_read.rs` | 626 | 874 | 634（+8） | 866 |

   余地はどれも S の見積 100 を超える。行数の上限の歯（face.rs の 1,250・face_srs.rs の 1,100・face_srs_items.rs の 600）は本便の write-set の file に当たらない。歯の file と凍結 anchor は src の外なので余地を測らない（参考値 face_constitution 821 → 1,012・face_index 973 → 1,071・凍結 anchor 163 と 167 は不変）。
3. **size は S。**
4. **verify は 10 行**で、done の 10 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f144_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_constitution f144_` = (c) の 2〜5（4 本）。
   3. `cargo nextest run -p folio --test face_index f144_` = (c) の 6〜8（3 本）。
   4. `cargo nextest run -p folio --test face_constitution` = 憲法の面の歯の全部（参考値 23 本）。
   5. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（入口の凍結 anchor 2 本との byte 一致と便 138 の歯を含む・参考値 29 本）。
   6. `cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture` = 憲法の凍結 anchor expected.html との byte 一致（本便で変えないことの確かめ）。
   7. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   8. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   9. `cargo nextest run -p folio --test face_index_sheet` = 支度表の入った入口の面の歯の全部（参考値 14 本）。
   10. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_constitution・face_index・face・badge・site・face_index_sheet の 6 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f144_ を関数名に持つ src は `crates/folio/src/face_constitution.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門・起草役の模擬の binary でも同じ）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d144-draft.md`、模擬の差分と script は同じ dir の d144-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-144.py（実装）・apply-144-teeth.py（歯と便 138 の歯の日付）・anchor-144.py（凍結 anchor 2 本）を撃つ。その差分が c144.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -rq`・`folio parts --check`・verify の 10 行を撃つ。
2. RED: apply-144-teeth.py と anchor-144.py だけを base に当て、単体の歯の区間を戻すと (e) の 3 のとおり（差分は r144-teeth.patch）。実装だけを当てると (e) の 1 の 6 本が落ちる。
3. 突然変異: mut-144.sh（(e) の 4）。
4. 余地: lines-144.py と lines-144.awk（同じ式の 2 実装）を write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 鮮度の札の生成日の字と名・表紙の版の札（憲法と要件書の 2 面の同じ形・(d) の 6）。前文と schema 節の改訂来歴を §0 に出すこと（憲法の欄の決まりの precedence は amended_by を持たない・承認欄の ADR-12・17・23 の行から辿れる）。判断の記録と設計ノートのカードの更新の日付。要件書の面。凍結 anchor の列。設計文書の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 承認の日付は判断の記録の承認欄の日付で、憲法の正本の版を上げた取り込みの日付ではない（同じ日のはずだが面は突き合わせない）。1 回の承認で複数の判断を発効した形（例 ADR-22 と ADR-23）では、行は版を名指す判断の分だけ出る。版の字が同じかどうかだけを見るので、判断の記録が未来の版を名指していても（版上げの前）今の版の承認は変わらず、その行は承認欄に出る。初回の承認の行は版の番号を持たない（正本の欄に無い）。要件書のカードの日付は承認欄の最後の 承認 の行で、効いている版の行とは限らない（要件書の面の表紙の状態と同じ式）。入口の面は、要件書が発効のとき要件書の承認欄（meta.approval）を必須として読み、無ければ組めない（今までは要件書の面だけが要した欄・folio face --face index だけを撃つ利用者には新しい失敗の口）。最も新しい日付は年-月-日の字の大小で決める（判断の記録の承認欄の日付は床が年-月-日に閉じる〔floor_adr.rs の DATE_FORMAT〕・要件書の承認欄の when は床が形を閉じていないので年-月-日でない字が入れば順が崩れる〔今の正本は全行が年-月-日〕）。枝の上の Pending の札とは 更新 <日付>・<版の欄>（起草・承認待ち・発効は <効いている版>）と並び、日付は効いている版の承認の日付である。次の周で読みやすさの観点が F-2 を解けたと読むかは周の結果でしか分からない。面の見た目が持ち主に受け入れられるかは walk でしか分からない。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 6 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と比べて file 数が変わるか、constitution.html と index.html のほかの file が 1 byte でも違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、憲法の正本が版ごとの承認の欄を持つようになっていたか、判断の記録の欄の決まりの amends・approval・effective_status の形が base と違えば、(b) と (f) を数え直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face_constitution_read.rs` の Amend・amendments・Approved・approved。`crates/folio/src/face_constitution.rs` の鮮度の札・表紙の状態・承認欄のリードと行・単体の歯 1 本。`crates/folio/src/face_index_read.rs` の憲法と要件書のカードの更新の行（関数 updated を消す）。`crates/folio/src/face_index.rs` の amendments の呼び出し。`crates/folio/tests/face_constitution.rs` の歯 4 本。`crates/folio/tests/face_index.rs` の歯 3 本と便 138 の歯 2 本の日付の字。入口の凍結 anchor 2 本の 1 か所ずつ。
- 入れない: 共有の口 Frame（`face.rs`）・鮮度の札の生成日・表紙の版の札・要件書の面・判断の記録と設計ノートの面とカード・部品目録・名札の class・様式の定義・床・設計文書・凍結 anchor の列・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| reader | 版を上げた判断の読み手 | `crates/folio/src/face_constitution_read.rs` の Amend・amendments・Approved・approved |
| head | 憲法の面の頭と承認欄 | 鮮度の札・表紙の状態・承認欄のリードと版ごとの承認の行（`crates/folio/src/face_constitution.rs`） |
| card | 入口の棚の憲法と要件書のカード | 更新の行の日付（`crates/folio/src/face_index_read.rs`・`crates/folio/src/face_index.rs`） |
| anchor | 入口の凍結 anchor | `tests/fixtures/face/expected-index.html`・`tests/fixtures/face/expected-index-sheet.html` の 1 か所ずつ |
| teeth | 歯 | 単体の f144_ 1 本・face_constitution の f144_ 4 本・face_index の f144_ 3 本・便 138 の歯 2 本の日付 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 118c2f4）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.217）を閉じる。憲法と要件書の 2 面の同じ形（鮮度の札の 生成 <初回の日付> と表紙の版の札 版 <version> / <generated>）を台帳の控えとして残すか、次の面の便で直すかを決める（§1 (d) の 6）。前文の改訂来歴を面に出すかは、憲法の欄の決まりに precedence の amended_by を足す一括の要否と一緒に決める。着地後の憲法の面の承認欄を持ち主への次の報告で見せるかを決める（§1 (b) の 5）。次の周で、読みやすさの F-2 と同じ所見が立たないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eq"
title = "一括 22 の B-1（天井の 36 周目 読みやすさ F-2・台帳 f2-648.217）: 憲法の面が第 1.4 版の承認の日付を初回の 2026-09-12 と見せ、第 1.1〜1.4 版を決めた判断の記録へ辿れず、入口の棚の憲法と要件書のカードも初回の生成日を更新と出す問題を直す。読み手（crates/folio/src/face_constitution_read.rs）に、発効した判断の記録（状態が欄の決まりの effective_status で承認欄が表）の amends が名指す版を判断の番号の順に 1 行ずつ読む口 amendments（adr/ が dir でなければ空）と、今の版の承認を決める口 approved（draft は生成日・今の版を名指す行が在れば Effective でその承認の最も新しい日付と id・行が無ければ初回の承認・名指す行が無ければ Unknown）を足す。憲法の面（crates/folio/src/face_constitution.rs）の表紙の状態を 発効・拘束力あり（承認 <日付>・判断の記録 <id>）にし、承認欄に版ごとの承認の行（<日付> · 版 <版> · 逐語・判断の記録・裁定・対話面・名札 発効）を既存の部品と class で足し、Unknown なら鮮度の札・表紙の状態・承認欄のリードに便 138 の名札 効く版はまだ分からない を出す（面は書く・P-4.2）。入口の棚（crates/folio/src/face_index_read.rs と crates/folio/src/face_index.rs）の憲法のカードの更新の日付を今の版の承認の日付に、要件書のカードの日付を承認欄の最後の承認の行の日付にする（名は 更新 のまま・draft は生成日のまま・便 138 の札は変えない）。鮮度の札の生成日・表紙の版の札・要件書の面・部品・class・様式の定義・設計文書は変えない。入口の凍結 anchor 2 本の要件書のカードの日付 1 か所ずつを手で直し、便 138 の歯 2 本の期待の字の日付 3 か所を直す。歯は単体の f144_ 1 本と face_constitution の f144_ 4 本と face_index の f144_ 3 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main 118c2f4・受付の時点の main で数え直す"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_constitution_read.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_index_read.rs", "crates/folio/src/face_index.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face_index.rs", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "crates/folio/tests/face.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs", "crates/folio/tests/face_index_sheet.rs"]
verify = ["cargo nextest run -p folio --bin folio f144_", "cargo nextest run -p folio --test face_constitution f144_", "cargo nextest run -p folio --test face_index f144_", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_index_sheet", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f144_（今の版の承認の口が、行の無い発効の正本で初回の承認・今の版を名指す行が在れば Effective と最も新しい日付と id・名指す行が無ければ Unknown と在る承認の最も新しい日付・draft は行を読まず生成日・版の字は escape した字どうしで比べ・表の外の状態は Err）が緑、face_constitution の f144_ の 4 本（実の置き場の写しで表紙の状態が今の版を名指す発効した判断の承認の日付と id を出し初回の日付を出さない・承認欄が作成と初回の承認の後に版ごとの承認の行を判断の番号の順に歯の側の手書きの読みと同じ字で並べる・版の欄を先に進めた写しと今の版を名指す判断を提案中に戻した写しで鮮度の札と表紙の状態と承認欄のリードが 効く版はまだ分からない を出し提案中の判断の行を出さない・版を上げた判断が無い写しと adr/ の無い fixture の写しで今の字のまま面を書く）が緑、face_index の f144_ の 3 本（憲法のカードが今の版の承認の日付・名指す判断が無ければ 効く版はまだ分からない・判断が無ければ初回の承認・draft は生成日、要件書のカードが承認欄の最後の承認の行の日付・draft は生成日、実の正本の 2 枚のカードが歯の側の手書きの読みの日付と版で始まる）が緑、face_constitution の歯の全部が緑、face_index の歯の全部（入口の凍結 anchor 2 本との byte 一致と便 138 の歯を含む）が緑、face の憲法の凍結 anchor expected.html との byte 一致が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、face_index_sheet の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで違う file は constitution.html と index.html の 2 枚だけ"
<!-- contracts:end -->
