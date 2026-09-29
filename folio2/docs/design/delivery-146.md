# 設計: 便 146 — 判断の記録・設計ノート・入口の面の鮮度の札と、5 面の図の札・足の行の日付を、便 145 の口で揃える

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）・FR9（設計ノートを 1 つの型で生成する）・FR16（判断の記録の面を正本から逐語で生成する）。本便は 5 面の頭と図の札と足の行の日付と、その日付の名（承認 か 生成）を、正本に在る欄（判断の記録と設計ノート = 承認欄の日付・入口と要件書 = 承認欄の最後の 承認 の行・憲法 = 便 144 の今の版の承認）から組むように直すだけで、3 つの要件の規範文も面の部品・色・並びも様式の定義も変えない。契約表の行の req は FR4・FR9・FR16 の 3 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する）/ P-6.3（同じ内容を 2 つの面が持つときは一方を正本とする＝1 枚の面の日付は 1 つの口から出す）/ P-4.2（判定できないものは まだ分からない として表に出す＝便 138 と便 144 の札をそのまま使う）/ P-2.1（生成器は 1 つ＝日付の名の 2 値は共有の口 1 か所で決める）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-6.2（生成物を手で直さない＝凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 便 145（台帳 f2-648.218・行 er）の起草役の問い 6（席の裁定 = 控え）と、便 145 の独立の検証の非 blocking の提案 1（図の札と足の行）。台帳の控え **f2-648.219**（notes に提案 1 を足した 2026-09-26 03:52 JST の行）。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `es` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 25 本（書き換える 19 本 + 本文が変わらない verify の scope 6 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 25 本を base の binary（写しの組み立て）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便を当てた写しの binary でも同じ。
- 前の便: 前提の着地は無い。**base = main 973d274（便 145 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草役は `git archive 973d274` の写しを組み立てて測った（本流の作業ツリーの binary は使っていない）。
- 並行の便との重なり: base の時点で、契約の枝のうち main に着地していない便は無い（便 145 は着地済み・起草役の実測）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 973d274・数は参考値）

1. **便 145 の後に残る形。** 便 145 は憲法と要件書の面の頭の 2 か所（鮮度の札・版の札）を承認の日付にした。残りは次の 3 つの形である。
   - 判断の記録・設計ノート・入口の 3 面の鮮度の札は、今も 生成 <日付> を出す。日付の出どころは、判断の記録が記録の欄 date（起草の日）、設計ノートと入口が欄 meta.generated（最初に生成した日）である。
   - 図の札（要件書の図 1〜3・憲法の図 1・入口の棚）は <版> <meta.generated> を出す。便 145 が誤りとして直した版の札と同じ形（版の横に初回の生成日）である。
   - 足の行（ft-plain・folio が生成した の文の中）は 5 面とも <版>（<日付>）を出す。日付に名が無く、文の中では面を組んだ日と読める。
2. **出どころ（関数）。**

| 所 | 面 | 字を組む関数 | 渡している日付 |
| --- | --- | --- | --- |
| 鮮度の札 | 判断の記録 | `crates/folio/src/face_adr.rs` の head → 共有の口 Frame::head | 記録の欄 date |
| 鮮度の札 | 設計ノート | `crates/folio/src/face_note.rs` の head → Frame::head | meta.generated |
| 鮮度の札 | 入口 | `crates/folio/src/face_index.rs` の head → Frame::head | meta.generated |
| 版の札 | 設計ノート | face_note.rs の cover | meta.generated |
| 図の札 | 要件書 図 1〜3 | `crates/folio/src/face_srs.rs` の figure_close | meta.generated |
| 図の札 | 憲法 図 1 | `crates/folio/src/face_constitution.rs` の amendment_chapter | meta.generated |
| 図の札 | 入口の棚 | face_index.rs の shelf | meta.generated |
| 足の行 | 憲法・要件書・判断の記録・設計ノート | 共有の口 `crates/folio/src/face.rs` の Frame::foot と foot_aside | 憲法・要件書・設計ノートは meta.generated・判断の記録は記録の欄 date |
| 足の行 | 入口 | face_index.rs の foot（Frame を使わない自前の 1 行） | meta.generated |

   便 145 の後、Frame::head は 生成 の名で Frame::head_dated へ委ねるだけの口で、呼ぶのは判断の記録・設計ノート・入口の 3 面である。
3. **folio2 自身の面の字（`folio build --dir design-intent --out <置き場> --write` の出力 30 file）。** 30 file のうち様式と script の 2 file を除く 28 枚が次の形を持つ。

| 面 | 所 | base の字 | 正本に在る承認の日付 |
| --- | --- | --- | --- |
| 判断の記録 ADR-23 | 鮮度の札（18 行目） | 生成 2026-09-24 · ADR-23（発効） | 承認欄 2026-09-25（表紙の状態も 承認 2026-09-25） |
| 判断の記録 ADR-23 | 足の行（301 行目） | 判断の記録 ADR-23（2026-09-24） | 同上 |
| 判断の記録（23 枚とも） | 鮮度の札・足の行 | 生成 <記録の日付>・<記録の日付> | 23 本とも発効で承認欄が在る |
| 入口 | 鮮度の札（18 行目） | 生成 2026-09-17 · v0.5（発効） | 承認欄の最後の 承認 の行 2026-09-24（版 v0.5） |
| 入口 | 棚の札（90 行目） | 棚 · v0.5 2026-09-17 · index.yaml | 同上 |
| 入口 | 足の行（155 行目） | 入口 v0.5（2026-09-17） | 同上 |
| 設計ノート example・figures | 鮮度の札・版の札・足の行 | 生成 2026-09-17 / 2026-09-19 など | 無い（2 本とも見本 status example・承認欄を持たない） |
| 憲法 | 図 1 の札（414 行目） | 図 1 · v1.4 2026-09-12 · constitution.yaml | 今の版の承認 2026-09-25（便 144） |
| 憲法 | 足の行（579 行目） | 憲法 v1.4（2026-09-12） | 同上 |
| 要件書 | 図 1〜3 の札（87・135・321 行目） | 図 n · v1.44 2026-09-12 · srs.yaml | 最後の 承認 の行 2026-09-26 |
| 要件書 | 足の行（1028 行目） | 要件書 v1.44（2026-09-12） | 同上 |

   同じ面の上で、鮮度の札か表紙の状態の承認の日付と、図の札か足の行の初回の日付が食い違って見える（台帳 f2-648.219 の観測）。入口の憲法と要件書のカード（便 144）・判断の記録の表紙の 日付 の枡（記録の欄 date に名 日付 が付く）は、名と日付が合っている。
4. **正本の欄の形。** 判断の記録の承認欄は 1 つの表 approval（date を持つ・発効した判断に必須・判断の記録の欄の決まり）。設計ノートの承認欄も 1 つの表 approval（date を持つ・発効と廃止に必須・設計ノートの欄の決まり）。入口の承認欄は憲法・要件書と同じ行の一覧 meta.approval（役 role と日付 when）で、床は入口の承認欄を数えない（`crates/folio/src/entrance.rs` の頭の注）＝発効の入口に承認欄が無い置き場も床を通る。
5. **fixture。** 面の fixture の判断の記録 ADR-2（`tests/fixtures/face/adr/ADR-2.yaml`）は提案中・date 2026-09-06・承認欄なし。設計ノート full（`tests/fixtures/face/design-note/full.yaml`）は draft・generated 2026-09-18。入口（`tests/fixtures/face/index.yaml`）は draft・generated 2026-09-03・承認欄なし。憲法は draft・generated 2026-09-01。要件書は発効・generated 2026-09-01・最後の 承認 の行 2026-09-05。凍結 anchor 7 本（`tests/fixtures/face/` の expected.html・expected-srs.html・expected-index.html・expected-index-sheet.html・expected-note.html・expected-adr.html・expected-site-adr-2.html）は、足の行に名の無い日付を持ち、要件書の anchor は図 1〜3 の札に v0.3 2026-09-01 を持つ。天井の束の凍結 anchor `tests/fixtures/ceiling/bundle/faces/` は 120 byte ほどの見本で、面の生成器の出力と比べない。
6. **base の歯（参考値）。** workspace の nextest 924 / 924・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file（1,946,018 byte）。`git grep -n 'f146_' -- crates` は 0 件。歯の file の本数は face_srs 23・face_constitution 25・face_adr 37・face_note 30・face_index 29。行数の上限の歯は face.rs 1,250（`crates/folio/tests/face_labels.rs`）と face_srs.rs 1,100（`crates/folio/tests/face_srs.rs`）。

### (b) 直す先 — 日付の名と日付の組を 1 か所で決め、5 面の頭・図の札・足の行が同じ組を使う

1. **日付の組の口（`crates/folio/src/face_labels.rs`・face.rs が再輸出する）。**
   - 関数 named を足す（承認の日付の Option と、無いときに読む日付の読み手を受ける）: Some なら（承認・その日付）、None なら（生成・読み手の日付）。名の値域は 承認 と 生成 の 2 つだけで、ここ 1 か所で決める。承認の日付が在るときは読み手を呼ばない。便 145 の dated は named へ委ねる（読み手 = meta.generated・出力は不変）。
   - 関数 approval_date を足す（欄の表と、承認を読まない状態の一覧を受ける）: 状態がその一覧に在るか、承認欄 approval が無ければ None。そうでなければ approval.date（escape 済み）。判断の記録（一覧 = 提案中）と設計ノート（一覧 = draft と見本）が使う。
   - 関数 last_approval（便 145）は、欄 meta に承認欄が無ければ None を返す（今は Err）。入口の承認欄は床が数えないので、無いときは生成日に落として面を書く。要件書の面は承認欄の章が承認欄を必ず読むので、要件書の出力と断りは変わらない。ただし入口の面の要件書のカードは同じ関数を呼ぶので、発効の要件書に承認欄が無い置き場では、base の「まだ分からない: srs.yaml.meta: 欄 approval が無い」で止まる形から、入口の面を書いてカードに生成日を出す形に変わる（folio build の結果は要件書の面の断りで まだ分からない のまま・検証役の実測）。
2. **共有の口 Frame（`crates/folio/src/face.rs`）。**
   - Frame::foot と foot_aside の日付の引数を、鮮度の札と同じ（名・日付）の組にする。足の行の字は <文書の名> <版>（<名> <日付>）になる。
   - Frame::head を外す。5 面とも head_dated を呼ぶようになり、残すと使われない口として clippy が警告を出す（`-D warnings` で落ちる）。head_dated の字と引数は変えない。
3. **判断の記録の面（`face_adr.rs`）。** 鮮度の札と足の行の組を named（approval_date〔提案中は読まない〕・無ければ記録の欄 date）から取る。発効と廃止で承認欄が在れば 承認 <承認欄の日付>、提案中か承認欄の無い廃止は 生成 <記録の日付>（base と同じ字）。表紙の 日付 の枡・機械のための面の date・承認欄の章・状態の行は変えない。
4. **設計ノートの面（`face_note.rs`）。** 鮮度の札・版の札（<版> / <日付>）・足の行の組を named（approval_date〔draft と見本は読まない〕・無ければ meta.generated）から取る。今の folio2 の 2 本は見本なので、変わるのは足の行の名だけ。
5. **入口の面（`face_index.rs`）。** 鮮度の札・棚の札（棚 · <版> <日付> · index.yaml）・足の行の組を、要件書と同じ口 dated と last_approval（draft か承認欄か 承認 の行が無ければ生成日）から取る。
6. **要件書の面（`face_srs.rs`）。** 図 1〜3 の札の日付を版の札と同じ dated と last_approval の日付に、足の行を同じ組にする（face_srs.rs の正規化行数は増えない）。
7. **憲法の面（`face_constitution.rs`）。** 鮮度の札と足の行が同じ組を使うよう、便 145 の head の中の式を関数 dated_of に出し、foot も呼ぶ。図 1 の札の日付を版の札と同じ Approved の date（draft は生成日・便 144 の定め）にする（amendment_chapter に Approved を渡す）。
8. **folio2 自身の面の字（本便の後）。**

| 版の立場 | 鮮度の札 | 版の札（設計ノート）・図の札・棚の札 | 足の行 |
| --- | --- | --- | --- |
| 承認の日付が引ける（発効・廃止で承認欄が在る） | 承認 <承認の日付> · <版>（今の名札） | <版> <承認の日付>（版の札は <版> / <承認の日付>） | <文書の名> <版>（承認 <承認の日付>） |
| 引けない（draft・見本・提案中・承認欄の無い発効の入口と要件書） | 生成 <生成日か記録の日付>（base のまま） | base のまま | <文書の名> <版>（生成 <同じ日付>） |

   1 枚の面の上の日付は、鮮度の札・版の札・表紙の状態・図の札・足の行のどれも、同じ 1 つの（名・日付）から出る。便 138 と便 144 の札（起草・承認待ち・効く版はまだ分からない）の出し方は変えない。
9. **変えないもの。** 部品・class・様式の定義・章と節の並び・HTML のタグの並び。表紙の状態の字・承認欄の章・判断の記録の表紙の 日付 の枡・機械のための面。入口のカードの字（判断の記録のカードの 更新 <記録の日付の最大> を含む＝(i) の 1）。床。設計文書。凍結 anchor の列（`design-intent/anchors/`）。天井の束の凍結 anchor。
10. **契約で決めること（起草役の判断）。**
   - **足の行は日付に名を添える（(d) の 1・2 は採らない）。** 足の行は folio が生成した の文の中なので、名の無い日付は面を組んだ日と読める。承認の日付を名なしで置くとその誤読が広がる。名を添えれば鮮度の札と同じ組を 1 つの口から出せる（P-6.3）。draft の面も 生成 の名が付き、凍結 anchor 7 本の足の行が 1 行ずつ変わる。
   - **図の札と棚の札は名を添えず、版の札と同じ日付にする。** 便 145 の問い 1 の裁定（版の札は <版> / <日付> のまま日付を替える）と同じ形で、図の札は版の札と同じ版の印である。draft の面の図の札は base と 1 byte も変えない。
   - **提案中の判断の記録は、記録の欄 date を 生成 の名のまま出す。** 面の fixture の凍結 anchor の鮮度の札を変えず、名の値域を 2 つに保つ。記録の日付は起草の日で、正本に在る欄の逐語である（(i) の 2）。
   - **設計ノートと入口の、承認の日付が引けない面は 生成 <日付> のまま出す（消さない）。** 生成日は正本に在る欄の逐語で、名が付けば偽りにならない。消すと draft の面で版の日付が読めなくなる。
   - **発効の入口に承認欄が無い置き場は生成日に落とす。** 床が入口の承認欄を数えないので、Err にすると base で書けた面が まだ分からない に変わる。名は 生成 なので偽りにならない。
   - **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は、正本に在る承認の日付を面が読まずに初回の日付や記録の日付を出していた中身の正しさの直し（P-6.1）で、部品・色・余白・並び・様式の定義・名札の class を 1 つも変えない。起草役の実測で、変わった 28 枚の変わった行は、どれもタグの並びが base の同じ行と一致し（字だけが違う）、`folio parts --check` も合格した（(e) の 5）。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137・138・139・141・144・145 の契約と同じ読みである。仮に本便を見た目の直しと数えても、判断の記録の面は便 27 に次ぐ 2 本目で上限を超えない。要件書の面は便 35（図の番号・部品 figure-panel）・便 36（用語集）の 2 本に届いており、面ごとに数える読みなら 3 本目になる（本便は figure-panel の中の figcaption の字を変える）。その読みを席が採るなら本便は 🔴 の候補で、持ち主の walk（面の承認）を前置する。部品ごとに数えても、見た目の直しと数えるなら鮮度の札（freshness-stamp）は便 138・144・145・本便で 4 本目になり上限を超える（figure-panel は便 35 と本便の 2 本目）。よって 当たらない の根拠は、本便が見た目でなく中身の正しさの直しである（部品・色・余白・並び・class・タグの並びを 1 つも変えず、面が読む正本の欄を直す）という前段の読みだけである。席はこの読みを採る（便 138・144・145 と同じ判定・持ち主への次の報告で明記する）。

### (c) 歯（関数名 f146_・base で 0 件）

1. **f146_named_and_approval_date_pick_the_approval（単体の歯・`crates/folio/src/face_labels.rs` の既存の tests の区間 face_labels_tests）。** named が Some で（承認・その日付）を返し読み手を呼ばないこと・None で（生成・読み手の日付）・読み手の Err をそのまま返すこと。approval_date が発効と廃止と設計ノートの発効で承認欄の日付（escape 済み）を返し、提案中・見本・draft と承認欄の無い表は None、状態の無い表は Err。承認欄の無い発効の meta で last_approval が None・dated が（生成・生成日）。**base では口が無く組み立てが通らない＝RED。**
2. **f146_srs_figure_captions_and_foot_follow_the_last_approval（`crates/folio/tests/face_srs.rs`・binary 経由・面の fixture の写し）。** 4 通りの写しで図 1〜3 の札と足の行がちょうど 1 つずつ在ること: 変異なし（v0.3 2026-09-05・承認 2026-09-05）、承認の行を 1 つ足した写し（2026-09-09）、承認の行を消した写し（v0.3 2026-09-01・生成 2026-09-01）、draft の写し（生成 2026-09-01）。**base では図の札が 2026-09-01・足の行に名が無い＝RED。**
3. **f146_real_srs_figure_captions_and_foot_follow_the_last_approval（同・実の正本）。** 歯の側の手書きの読み（yaml-rust2 で承認欄を直に読み、最後の 承認 の行の when を escape する）の日付で、図 1〜3 の札が <版> <日付> で 1 つずつ・足の行が 要件書 <版>（承認 <日付>）で 1 つ・生成日の図の札と名の無い生成日の足の行が無いこと。**base では 2026-09-12＝RED。**
4. **f146_constitution_figure_caption_and_foot_follow_the_current_approval（`crates/folio/tests/face_constitution.rs`・実の置き場の写しと面の fixture の写し）。** 便 144 の歯の側の手書きの読み（source_rows・source_meta）の今の版の承認の日付で、図 1 の札と足の行（承認 <日付>）が 1 つずつ・生成日の図の札と名の無い生成日の足の行が無いこと。面の fixture の憲法は draft で 図 1 · v0.9 2026-09-01 と（生成 2026-09-01）、発効にした写しで 2026-09-02 と（承認 2026-09-02）。**base では足の行に名が無い＝RED。**
5. **f146_adr_stamp_and_foot_follow_the_approval（`crates/folio/tests/face_adr.rs`・面の fixture の写し）。** 提案中（生成 2026-09-06）、提案中に承認欄を足した写し（読まない＝生成 2026-09-06）、発効にして承認欄 2026-09-08 を足した写し（鮮度の札と足の行が 承認 2026-09-08・表紙の 日付 の枡は 2026-09-06 のまま・生成 の札が無い）、廃止で承認欄の在る写し（承認 2026-09-08）と無い写し（生成 2026-09-06）。**base では発効の写しが 生成 2026-09-06 を出す＝RED。**
6. **f146_real_adr_faces_date_the_approval（同・実の正本の全本数）。** 実の判断の記録の file を全部（base で 23 本）、歯の側の手書きの読み（承認欄の date・無ければ記録の date と名 生成）で、鮮度の札の頭と足の行がちょうど 1 つずつ在ること。承認欄を持つ記録を 1 本以上数えること。**base では 23 本とも 生成 <記録の日付>＝RED。**
7. **f146_note_stamp_cover_and_foot_follow_the_approval（`crates/folio/tests/face_note.rs`・面の fixture の写し）。** draft（生成 2026-09-18・v0.1 / 2026-09-18・足の行 生成 2026-09-18）、発効にして承認欄 2026-09-21 を足した写し（承認 2026-09-21 が 3 か所・生成日の鮮度の札が無い）、draft のまま承認欄を足した写し（読まない）。**base では発効の写しが生成日を出す＝RED。**
8. **f146_index_stamp_shelf_and_foot_follow_the_last_approval（`crates/folio/tests/face_index.rs`・面の fixture の写し）。** draft（生成 2026-09-03・棚 · v0.1 2026-09-03・生成 2026-09-03）、発効にして行の並び 作成・承認・承認・作成 の承認欄を足した写し（最後の 承認 の行 2026-09-07 が 3 か所）、発効で承認欄の無い写し（面が書け、生成 2026-09-03）。**base では発効の写しが生成日を出す＝RED。**
9. **f146_real_index_stamp_shelf_and_foot_follow_the_last_approval（同・実の正本）。** 歯の側の手書きの読みの最後の 承認 の行の日付で、鮮度の札の頭・棚の札・足の行がちょうど 1 つずつ・生成日の鮮度の札と棚の札が無いこと。**base では 生成 2026-09-17＝RED。**

fixture は新しい file を足さない。変異は歯の中で一時 dir の写しを書き換える（歯の file の既存の口 fixture_srs・real_copy・mutated・index_fixture_copy と同じ形）。正本の読みは歯の側の手書き（生成側の口を呼ばない・P-10.1）。

**既存の歯の期待の字の直し（全数）。** 0 本。便 138・144・145 の歯は鮮度の札・版の札・表紙の状態・カードだけを見るので、本便の後もそのまま緑である（起草役の実測・(e) の 1）。落ちる既存の歯は凍結 anchor 7 本の手直しだけで直る。

### (d) 採らなかった形

1. **足の行の日付を外す（<文書の名> <版> だけ）。** 同じ 28 枚と凍結 anchor 7 本が変わり、日付の写しが 1 つ減る。ただし印刷した面や写した面で、どの版のどの日付の正本から生成したかが足の行から読めなくなる。
2. **足の行に承認の日付のときだけ名を添える。** draft の面の凍結 anchor は変わらないが、同じ足の行の形が面の状態で割れ、名の無い日付が面を組んだ日と読める誤読も残る。
3. **図の札・棚の札も名を添える（<版>（承認 <日付>））か日付を外す。** 便 145 の版の札の形（<版> / <日付>）と割れる。
4. **提案中の判断の記録の鮮度の札の名を 日付（表紙の枡の名）にする。** 名の値域が 3 つになり共有の口の 2 値を崩す。面の fixture の凍結 anchor の鮮度の札も変わる。
5. **Frame::head を残して 3 面を head のままにし、日付だけを替える。** 名が 生成 のまま承認の日付を出すことになり、承認の日付を生成日と偽る（便 145 の (d) の 2 と同じ理由）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **実装だけを base に当てた写し（歯の file と凍結 anchor は base のまま）で、workspace の nextest は 924 本のうち 14 本が落ちる（起草役の実測）。** どれも凍結 anchor 7 本の手直しで直る。

| 歯 | 本数 | 直し方 |
| --- | ---: | --- |
| face の face_write_matches_the_frozen_fixture（憲法）・face_adr の face_adr_write_matches_the_frozen_fixture と face_adr_figure_chapter_is_the_only_difference_from_the_figureless_face・face_note の face_note_write_matches_the_frozen_fixture と face_note_figure_chapter_is_the_only_difference_from_the_figureless_face・face_index の face_index_write_matches_the_frozen_fixture と face_index_write_with_a_sheet_matches_the_frozen_fixture | 7 | 凍結 anchor の足の行 |
| face_srs_body の face_srs_write_matches_the_frozen_fixture・face_srs_figure の face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face・face_srs の f118_handwritten_scope_m3_adds_one_escaped_block と f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required と f138_equal_versions_keep_the_effective_labels | 5 | 要件書の凍結 anchor（図 1〜3 の札と足の行） |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures・site の site_write_matches_the_frozen_fixture | 2 | 凍結 anchor 7 本 |

   本便の差分の全部を base に当てた写しで、workspace の nextest は 933 / 933（base 924 + 単体 1 + face_srs 2 + face_constitution 1 + face_adr 2 + face_note 1 + face_index 2）・clippy 0 警告・床 4 本 rc 0・`folio parts --check`（build の出力 28 枚の面）合格（違反 0・まだ分からない 0）。
2. **動く凍結 anchor は 7 本で、足の行 1 行ずつと要件書の図の札 3 か所。** 起草役は script で 1 か所ずつ字を置き換え（生成器を呼ばない）、生成器の出力と byte で一致した。

| 凍結 anchor | 直す所 | byte | 行数 | sha256 の頭 8 字 |
| --- | --- | --- | --- | --- |
| expected.html（憲法・draft） | 足の行 憲法 v0.9（2026-09-01）→（生成 2026-09-01） | 25,729 → 25,736 | 224 のまま | c5fef562 → 26ba781c |
| expected-srs.html（要件書・発効） | 図 1〜3 の札 v0.3 2026-09-01 → v0.3 2026-09-05・足の行 →（承認 2026-09-05） | 34,276 → 34,283 | 325 のまま | 61486a0b → 67d6db49 |
| expected-index.html（入口・draft） | 足の行 →（生成 2026-09-03） | 12,082 → 12,089 | 123 のまま | 349709db → e8a6c19d |
| expected-index-sheet.html（同・支度表あり） | 同上 | 12,560 → 12,567 | 126 のまま | 8f92ee15 → 844b8c3e |
| expected-note.html（設計ノート・draft） | 足の行 →（生成 2026-09-18） | 18,981 → 18,988 | 245 のまま | d450dd42 → d78fe28c |
| expected-adr.html（判断の記録・提案中） | 足の行 →（生成 2026-09-06） | 16,669 → 16,676 | 227 のまま | 5a2c9904 → 5daa5441 |
| expected-site-adr-2.html（組み立ての出力） | 同上 | 16,659 → 16,666 | 227 のまま | a58ebe67 → e3e0f996 |

   byte の差は 7（生成 か 承認 の 2 字と空白 1 字）。draft と提案中の anchor の鮮度の札・図の札・版の札は 1 byte も変えない。天井の束の凍結 anchor・床の凍結の土台・凍結 anchor の列は変えない。
3. **RED（起草役の実測）。** binary の歯と凍結 anchor だけを base に当てた写し（単体の歯を除く 932 本）で 22 本が落ちる＝binary の f146_ の 8 本すべて（(c) の 2〜9）と、凍結 anchor に掛かる 14 本（(e) の 1 の表）。単体の歯を足すと組み立てが通らない（named と approval_date が無い・E0425 が 4 つ）。
4. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変え、verify の歯の file 11 本と単体の歯を撃つ）。** 15 通りとも 1 本以上が落ちる。

| 変異 | 落ちる歯（主なもの） |
| --- | --- |
| M1 判断の記録の組を記録の日付に戻す | (c) の 5・6（2 本） |
| M2 提案中でも承認欄を読む | (c) の 5（1 本） |
| M3 draft の設計ノートでも承認欄を読む | (c) の 7（1 本） |
| M4 設計ノートの版の札を生成日に戻す | (c) の 7（1 本） |
| M5 入口の鮮度の札を生成日に戻す | (c) の 8・9（2 本） |
| M6 入口の棚の札を生成日に戻す | (c) の 8・9（2 本） |
| M7 入口の足の行を生成日に戻す | (c) の 8・9（2 本） |
| M8 要件書の図の札を生成日に戻す | (c) の 2・3 と要件書の凍結 anchor の 7 本（9 本） |
| M9 要件書の足の行を生成日に戻す | (c) の 2・3 と凍結 anchor の 7 本（9 本） |
| M10 憲法の図 1 の札を生成日に戻す | (c) の 4（1 本） |
| M11 憲法の足の行を生成日に戻す | (c) の 4（1 本） |
| M12 共有の口の足の行から名を落とす | 5 面の f146_ と凍結 anchor 7 本に掛かる歯（18 本） |
| M13 承認欄の無い meta を Err にする | (c) の 1・8（2 本） |
| M14 named の承認の名を 生成 にする | 便 145 の歯を含む 25 本 |
| M15 approval_date が状態を見ない | (c) の 1・5・7（3 本） |

5. **folio2 自身の面の変化（起草役の実測）。** base の写しで `folio build --dir design-intent --out <置き場> --write` の出力は 30 file のままで、byte は 1,946,018 → 1,946,214（28 枚 × 7 byte）。違う file は 28 枚（判断の記録 23・設計ノート 2・入口・憲法・要件書）で、様式と script の 2 file は byte で一致する。どの面も行数は変わらず、変わった行はどれもタグの並びが base の同じ行と一致する（起草役の script で行ごとに確かめた）。

| 面 | 変わる行 | base | 本便の後 |
| --- | --- | --- | --- |
| 判断の記録 ADR-23 | 18 行目・301 行目 | 生成 2026-09-24 · ADR-23（発効）／ADR-23（2026-09-24） | 承認 2026-09-25 · ADR-23（発効）／ADR-23（承認 2026-09-25） |
| 判断の記録（他の 22 枚） | 18 行目と足の行 | 生成 <記録の日付>／（<記録の日付>） | 承認 <承認欄の日付>／（承認 <承認欄の日付>） |
| 入口 | 18・90・155 行目 | 生成 2026-09-17／棚 · v0.5 2026-09-17／入口 v0.5（2026-09-17） | 承認 2026-09-24／棚 · v0.5 2026-09-24／入口 v0.5（承認 2026-09-24） |
| 設計ノート example・figures | 足の行 | example v0.1（2026-09-17）／figures v0.2（2026-09-19） | （生成 2026-09-17）／（生成 2026-09-19） |
| 憲法 | 414・579 行目 | 図 1 · v1.4 2026-09-12／憲法 v1.4（2026-09-12） | 図 1 · v1.4 2026-09-25／憲法 v1.4（承認 2026-09-25） |
| 要件書 | 87・135・321・1028 行目 | 図 n · v1.44 2026-09-12／要件書 v1.44（2026-09-12） | 図 n · v1.44 2026-09-26／要件書 v1.44（承認 2026-09-26） |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 25 本とも印なし。書き換える 19 本 = `crates/folio/src/face.rs`・`crates/folio/src/face_labels.rs`・`crates/folio/src/face_constitution.rs`・`crates/folio/src/face_srs.rs`・`crates/folio/src/face_adr.rs`・`crates/folio/src/face_note.rs`・`crates/folio/src/face_index.rs`・`crates/folio/tests/face_srs.rs`・`crates/folio/tests/face_constitution.rs`・`crates/folio/tests/face_adr.rs`・`crates/folio/tests/face_note.rs`・`crates/folio/tests/face_index.rs`・凍結 anchor 7 本（`tests/fixtures/face/expected.html`・`tests/fixtures/face/expected-srs.html`・`tests/fixtures/face/expected-index.html`・`tests/fixtures/face/expected-index-sheet.html`・`tests/fixtures/face/expected-note.html`・`tests/fixtures/face/expected-adr.html`・`tests/fixtures/face/expected-site-adr-2.html`）。本文不変の 6 本 = `crates/folio/tests/face.rs`・`crates/folio/tests/face_srs_body.rs`・`crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/face_labels.rs`（verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 7 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face.rs` | 1,124 | 376 | 1,113（−11） | 387 |
| `crates/folio/src/face_labels.rs` | 519 | 981 | 569（+50・うち単体の歯 約 30） | 931 |
| `crates/folio/src/face_constitution.rs` | 1,156 | 344 | 1,165（+9） | 335 |
| `crates/folio/src/face_srs.rs` | 1,058 | 442 | 1,058（±0） | 442 |
| `crates/folio/src/face_adr.rs` | 913 | 587 | 920（+7） | 580 |
| `crates/folio/src/face_note.rs` | 1,010 | 490 | 1,018（+8） | 482 |
| `crates/folio/src/face_index.rs` | 793 | 707 | 797（+4） | 703 |

   余地はどれも S の見積 100 を超える（最小は face_constitution.rs の 335）。行数の上限の歯は face.rs が 1,113 ≤ 1,250（`crates/folio/tests/face_labels.rs`・verify の 17）、face_srs.rs が 1,058 ≤ 1,100（`crates/folio/tests/face_srs.rs` の f100_face_srs_is_split_and_under_the_cap・verify の 7）で緑。歯の file と凍結 anchor は src の外なので余地を測らない（参考値 wc -l で face_srs 997 → 1,075・face_constitution 1,249 → 1,307・face_adr 1,323 → 1,410・face_note 1,058 → 1,105・face_index 1,083 → 1,160）。
3. **size は S。** 2 便に割る案（(i) の 3・§5）は採らない。
4. **verify は 18 行**で、done の 18 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f146_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_srs f146_` = (c) の 2・3（2 本）。
   3. `cargo nextest run -p folio --test face_constitution f146_` = (c) の 4（1 本）。
   4. `cargo nextest run -p folio --test face_adr f146_` = (c) の 5・6（2 本）。
   5. `cargo nextest run -p folio --test face_note f146_` = (c) の 7（1 本）。
   6. `cargo nextest run -p folio --test face_index f146_` = (c) の 8・9（2 本）。
   7. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（便 138・145 の歯・凍結 anchor との一致・行数の上限を含む・参考値 25 本）。
   8. `cargo nextest run -p folio --test face_constitution` = 憲法の面の歯の全部（便 144・145 の歯を含む・参考値 26 本）。
   9. `cargo nextest run -p folio --test face_adr` = 判断の記録の面の歯の全部（凍結 anchor との一致を含む・参考値 39 本）。
   10. `cargo nextest run -p folio --test face_note` = 設計ノートの面の歯の全部（凍結 anchor との一致を含む・参考値 31 本）。
   11. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（凍結 anchor 2 本と要件書・憲法のカードの歯を含む・参考値 31 本）。
   12. `cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture` = 憲法の凍結 anchor との byte 一致。
   13. `cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture` = 要件書の凍結 anchor との byte 一致。
   14. `cargo nextest run -p folio --test face_srs_figure face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face` = 同じ凍結 anchor から図の章を抜いた字との一致。
   15. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い 5 面の凍結 anchor 7 本との byte 一致。
   16. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   17. `cargo nextest run -p folio --test face_labels` = face.rs の行数の上限 1,250。
   18. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告（外した Frame::head が残っていないことを含む）。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_srs・face_constitution・face_adr・face_note・face_index・face・face_srs_body・face_srs_figure・badge・site・face_labels の 11 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f146_ を関数名に持つ src は `crates/folio/src/face_labels.rs` だけで write-set に在る。base では verify の 1〜6 が 0 本の実行で rc 4、7〜18 が rc 0（起草役の実測）。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d146-draft.md`、模擬の差分と script は同じ dir の d146-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-146.py（実装）・apply-146-teeth.py（歯）・anchor-146.py（凍結 anchor 7 本）を撃つ。その差分が c146.patch。workspace の nextest・clippy・床 4 本（floor4.sh）・`folio build --write` の出力の file 数と base との `diff -rq`・`folio parts --check`・verify の 18 行（verify-146.sh）を撃つ。
2. RED: r146-teeth.patch（binary の歯と凍結 anchor だけ）を base に当てると (e) の 3 のとおり。実装だけを当てると (e) の 1 の 14 本が落ちる。
3. 突然変異: mut-146.sh（(e) の 4）。
4. 余地: lines-146.py と lines-146.awk（同じ式の 2 実装）を write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 入口の判断の記録のカードの 更新 <記録の日付の最大>（今は 更新 2026-09-24・最も新しい承認は ADR-23 の 2026-09-25＝同じ形の食い違いが残る・カードの 更新 の意味を決める便が要る＝台帳 f2-648.220 に控えた）。入口の設計ノートのカードの 更新 <生成日の最大>（2 本とも見本なので今は食い違わない）。判断の記録の表紙の 日付 の枡と機械のための面の date（名が合っている）。承認欄の章・表紙の状態・入口の憲法と要件書のカードの字。凍結 anchor の列。設計文書の字（meta.generated と記録の欄 date の名と意味を含む）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 提案中の判断の記録の鮮度の札と足の行は、記録の欄 date（起草の日）を 生成 の名で出す（base と同じ・生成日と起草の日は今の正本では同じ日だが、欄の意味は同じではない）。承認欄の無い廃止の判断の記録も同じ。要件書と入口の日付は承認欄の最後の 承認 の行で、効いている版の行とは限らない（今の正本では一致する）。入口の承認欄は床が数えないので、発効の入口に承認欄が無ければ 生成 の名で生成日を出す（P-4.2 の読みで「承認の日付はまだ分からない」と出す形は採らない＝便 145 の要件書と同じ扱い）。日付は正本の字をそのまま出し、形（年-月-日）を面で確かめない。足の行の名は何の日付かを示すだけで、面を組んだ日（生成の実行の日）は出さない（正本に無い値なので P-6.3 により出さない）。次の周で読みやすさの観点がこの食い違いを解けたと読むかは周の結果でしか分からない。面の見た目が持ち主に受け入れられるかは walk でしか分からない。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 14 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と比べて file 数が変わるか、様式と script の 2 file が 1 byte でも違うか、変わった面の変わった行のタグの並びが変わるか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、Frame::head_dated と foot の引数の形・便 145 の last_approval と dated の定め・判断の記録と設計ノートの承認欄の形が base と違えば、(b) と (f) を数え直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の named・approval_date・last_approval の承認欄の無いときの扱い・単体の歯 1 本。`crates/folio/src/face.rs` の Frame::foot と foot_aside の（名・日付）の組と Frame::head を外すこと。`crates/folio/src/face_adr.rs`・`crates/folio/src/face_note.rs`・`crates/folio/src/face_index.rs` の鮮度の札（設計ノートは版の札・入口は棚の札も）と足の行の日付。`crates/folio/src/face_srs.rs` と `crates/folio/src/face_constitution.rs` の図の札と足の行の日付。歯の file 5 本の f146_ の歯 8 本。凍結 anchor 7 本の手直し。
- 入れない: 入口のカードの字・部品目録・名札の class・様式の定義・床・設計文書・凍結 anchor の列・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| named | 日付の組の口 | `crates/folio/src/face_labels.rs` の named・approval_date・last_approval |
| frame | 共有の口 | `crates/folio/src/face.rs` の Frame::head_dated と Frame::foot・foot_aside |
| heads | 3 面の頭 | 判断の記録・設計ノート・入口の鮮度の札（設計ノートの版の札・入口の棚の札） |
| captions | 図の札 | 要件書の図 1〜3・憲法の図 1・入口の棚 |
| foot | 足の行 | 5 面の ft-plain |
| anchor | 凍結 anchor | `tests/fixtures/face/` の expected 7 本 |
| teeth | 歯 | 単体の f146_ 1 本・binary の f146_ 8 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 973d274）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.219）を閉じる。入口の判断の記録のカードの 更新 の日付を承認の日付に揃えるかを決める（§1 (i) の 1＝控えの候補）。着地後の面の頭と足の行を持ち主への次の報告で見せるかを決める。次の周で、読みやすさの観点が面の日付の食い違いを所見に立てないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "es"
title = "便 145 の問い 6 の控えと独立の検証の提案 1（台帳 f2-648.219）: 判断の記録・設計ノート・入口の面の鮮度の札が 生成 <記録の日付か初回の生成日> を出し、同じ面の承認の日付と食い違って見える問題と、要件書の図 1〜3・憲法の図 1・入口の棚の札が版の横に初回の生成日を出し、5 面の足の行が名の無い日付を出す問題を直す。日付の組の口（crates/folio/src/face_labels.rs）に、承認の日付が在れば（承認・その日付）・無ければ（生成・代わりの日付）を返す named と、承認欄が 1 つの表（判断の記録・設計ノート）の承認の日付を返す approval_date（提案中・draft・見本は読まない）を足し、便 145 の dated は named へ委ね、last_approval は承認欄の無い meta で None を返す（入口の承認欄は床が数えない）。共有の口 Frame（crates/folio/src/face.rs）の foot と foot_aside の日付を（名・日付）の組にし、使われなくなる head を外す。判断の記録（crates/folio/src/face_adr.rs）・設計ノート（crates/folio/src/face_note.rs・版の札も）・入口（crates/folio/src/face_index.rs・棚の札も）の鮮度の札と足の行、要件書（crates/folio/src/face_srs.rs）と憲法（crates/folio/src/face_constitution.rs）の図の札と足の行が、その面の同じ組を使う。図の札と棚の札は名を添えず版の札と同じ日付、足の行は名を添える。部品・class・タグの並び・表紙の状態・承認欄の章・入口のカード・設計文書は変えない。凍結 anchor 7 本の足の行と要件書の図の札を手で直す（既存の歯の期待の字の直しは 0 本）。歯は単体の f146_ 1 本と face_srs 2 本・face_constitution 1 本・face_adr 2 本・face_note 1 本・face_index 2 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main 973d274・受付の時点の main で数え直す"
req = ["FR4", "FR9", "FR16"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_labels.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/face_index.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face_index.rs", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-site-adr-2.html", "crates/folio/tests/face.rs", "crates/folio/tests/face_srs_body.rs", "crates/folio/tests/face_srs_figure.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs", "crates/folio/tests/face_labels.rs"]
verify = ["cargo nextest run -p folio --bin folio f146_", "cargo nextest run -p folio --test face_srs f146_", "cargo nextest run -p folio --test face_constitution f146_", "cargo nextest run -p folio --test face_adr f146_", "cargo nextest run -p folio --test face_note f146_", "cargo nextest run -p folio --test face_index f146_", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_srs_figure face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_labels", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f146_（named が承認の日付で名 承認 と代わりの日付を読まないこと・無ければ名 生成 と代わりの日付・approval_date が発効と廃止で承認欄の日付を escape して返し提案中・draft・見本と承認欄の無い表で None・承認欄の無い発効の meta で last_approval が None）が緑、face_srs の f146_ の 2 本（面の fixture の写し 4 通りと実の要件書で図 1〜3 の札と足の行が最後の承認の行か生成日に従う）が緑、face_constitution の f146_ の 1 本（実の置き場の写しで図 1 の札と足の行が今の版の承認の日付・面の fixture の憲法が draft なら生成日と名 生成・発効なら初回の承認）が緑、face_adr の f146_ の 2 本（面の fixture の写しで提案中は記録の日付と名 生成・発効と廃止は承認欄の日付と名 承認・表紙の日付の枡は不変、実の判断の記録の全本数で鮮度の札と足の行が承認欄の日付）が緑、face_note の f146_ の 1 本（draft と承認欄の在る draft は生成日・発効は承認欄の日付が鮮度の札と版の札と足の行に）が緑、face_index の f146_ の 2 本（面の fixture の写しで draft と承認欄の無い発効は生成日・発効は最後の承認の行、実の入口で鮮度の札と棚の札と足の行が最後の承認の行の日付）が緑、face_srs の歯の全部が緑、face_constitution の歯の全部が緑、face_adr の歯の全部が緑、face_note の歯の全部が緑、face_index の歯の全部が緑、face の憲法の凍結 anchor との byte 一致が緑、face_srs_body の要件書の凍結 anchor との byte 一致が緑、face_srs_figure の図の章を抜いた字との一致が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、face_labels の face.rs の行数の上限が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで様式と script の 2 file は byte 一致・変わった面の変わった行のタグの並びは不変"
<!-- contracts:end -->
