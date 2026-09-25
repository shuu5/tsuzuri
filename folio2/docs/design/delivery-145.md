# 設計: 便 145 — 憲法と要件書の面の表紙の鮮度の札と版の札の日付を、初回の生成日から効いている版の承認の日付にする

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は憲法と要件書の 2 面の頭の 2 か所（鮮度の札・表紙の版の札）の日付と、鮮度の札の日付の名を、正本に在る欄（憲法 = 便 144 の今の版の承認・要件書 = 承認欄の最後の承認の行）から組むように直すだけで、FR4 の規範文も面の部品・色・並びも様式の定義も変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する）/ P-6.3（同じ内容を 2 つの面が持つときは一方を正本とする＝表紙の日付は表紙の状態と同じ読みの口から出し、入口のカードも同じ口を使う）/ P-4.2（判定できないものは まだ分からない として表に出す＝便 138 と便 144 の札をそのまま使う）/ P-2.1（生成器は 1 つ＝共有の口 Frame の鮮度の札の字を面ごとに書き分けない）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-6.2（生成物を手で直さない＝要件書の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 便 144（台帳 f2-648.217・行 eq）の起草役の問い 3（席の裁定 = 控え）と独立の検証の非 blocking の提案 1。一括 22 の仕分け B-1 の文は要件書の表紙の 更新 <初回の生成日>・<版> も直す先に挙げている。台帳の控え **f2-648.218**。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `er` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 14 本（書き換える 8 本 + 本文が変わらない verify の scope 6 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 14 本を base の binary（写しの組み立て）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便を当てた写しの binary でも同じ。
- 前の便: 前提の着地は無い。**base = main 7572ab4（便 144 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草役は `git archive 7572ab4` の写しを組み立てて測った（本流の作業ツリーの binary は使っていない）。
- 並行の便との重なり: base の時点で、契約の枝のうち main に着地していない便は無い（便 144 は着地済み・起草役の実測）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 7572ab4・数は参考値）

1. **表紙の日付の出どころ。** 憲法と要件書の面の頭には日付が 3 か所ある。鮮度の札（部品 freshness-stamp・site-bar の中）と、表紙の版の札（部品 doc-cover-band の cover-meta の 版 の枡）と、表紙の状態（cover-status）である。表紙の状態は便 138 と便 144 で承認の日付を出すようになったが、残りの 2 か所は正本の欄 meta.generated（その文書を最初に生成した日）をそのまま出す。
   - 鮮度の札: 共有の口 `crates/folio/src/face.rs` の Frame::head が 生成 <b><generated></b> · <b><版></b>（<名札>）の字を組む。Frame::head は 5 面（入口・憲法・要件書・判断の記録・設計ノート）が共有し、日付の名 生成 は口の中に固定の字で在る。憲法の面は `crates/folio/src/face_constitution.rs` の関数 head、要件書の面は `crates/folio/src/face_srs.rs` の関数 head が meta.generated を渡す。
   - 版の札: 憲法の面は face_constitution.rs の関数 cover、要件書の面は face_srs.rs の関数 cover が <version> / <generated> を組む。
2. **folio2 自身の面の字（`folio build --dir design-intent --out <置き場> --write` の出力）。**

| 面 | 所 | base の字 | 読める意味 |
| --- | --- | --- | --- |
| 憲法 | 鮮度の札 | 生成 2026-09-12 · v1.4（発効・拘束力あり） | v1.4 が 2026-09-12 の生成と読める |
| 憲法 | 版の札 | v1.4 / 2026-09-12 | v1.4 が 2026-09-12 の版と読める（誤り） |
| 憲法 | 表紙の状態 | 発効・拘束力あり（承認 2026-09-25・判断の記録 ADR-23） | 便 144 の後の形 |
| 要件書 | 鮮度の札 | 生成 2026-09-12 · v1.44（発効・拘束力あり） | 同上 |
| 要件書 | 版の札 | v1.44 / 2026-09-12 | v1.44 が 2026-09-12 の版と読める（誤り） |
| 要件書 | 表紙の状態 | 発効・拘束力あり（承認 2026-09-26） | 便 138 の後の形 |

   同じ表紙の上で、版の札の 2026-09-12 と表紙の状態の承認の日付が 2 行違いで食い違って見える（台帳 f2-648.218 の観測）。入口の棚のカードは便 144 で 更新 2026-09-25・v1.4 と 更新 2026-09-26・v1.44 になっており、表紙の 2 か所だけが初回の日付のまま残る。
3. **正本の値。** 憲法の meta.generated は 2026-09-12・今の版 v1.4 を名指す発効した判断は ADR-23（承認 2026-09-25）で、便 144 の読み手の口 approved（`crates/folio/src/face_constitution_read.rs`・Approved の date）がその日付を返す。要件書の meta.generated は 2026-09-12・承認欄の最後の 承認 の行は 2026-09-26（版 v1.44）。meta.generated は正本に在る欄なので、生成 の名で出すこと自体は逐語である。食い違いの原因は、読み手が鮮度の札の日付を今の版の日付と読むのに、欄が初回の日付を持つことにある。
4. **判断の記録と設計ノートの面も同じ形を持つ（本便の範囲の外）。** 判断の記録 ADR-23 の面の鮮度の札は 生成 2026-09-24 · ADR-23（発効）で、表紙の状態は 発効・拘束力あり（承認 2026-09-25）である。入口の面の鮮度の札は 生成 2026-09-17 · v0.5（発効）。本便は依頼どおり憲法と要件書の 2 面だけを直し、他の 3 面の出力は 1 byte も変えない（(i) の 1・§5）。
5. **fixture。** 面の fixture の憲法（`tests/fixtures/face/constitution.yaml`）は status draft・版 v0.9・generated 2026-09-01・初回の承認 2026-09-02。面の fixture の要件書（`tests/fixtures/face/srs.yaml`）は status effective・版 v0.3・effective_version v0.3・generated 2026-09-01・承認欄の最後の 承認 の行 2026-09-05。要件書の凍結 anchor `tests/fixtures/face/expected-srs.html` は 18 行目の鮮度の札に 生成 2026-09-01、32 行目の版の札に v0.3 / 2026-09-01 を持つ。
6. **base の歯（参考値）。** workspace の nextest 919 / 919・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file。`git grep -n 'f145_' -- crates` は 0 件。歯の file の本数は face_srs 21・face_constitution 23。行数の上限の歯は face.rs 1,250（`crates/folio/tests/face_labels.rs`）と face_srs.rs 1,100（`crates/folio/tests/face_srs.rs`）。

### (b) 直す先 — 表紙の日付を承認の日付にする口を 1 組置き、2 面の鮮度の札と版の札と入口の要件書のカードで使う

1. **表紙の日付の口（`crates/folio/src/face_labels.rs` に足す・face.rs が再輸出する）。**
   - 関数 last_approval（欄 meta を受ける）: 便 138 の standing が Draft なら None。そうでなければ承認欄 approval の行のうち役 role が 承認 の最後の行の when（escape 済み）を返し、承認 の行が無ければ None。今の要件書の面の表紙の状態（face_srs.rs の cover）と入口の要件書のカード（`crates/folio/src/face_index_read.rs` の srs_card）が同じ式を 2 か所に書いているのを、この口 1 つにまとめる。
   - 関数 dated（欄 meta と承認の日付の Option を受ける）: Some なら（承認・その日付）、None なら（生成・meta.generated）の組を返す。日付の名は 2 つの字（承認・生成）だけで、表の外の値は出ない。
2. **共有の口 Frame（`crates/folio/src/face.rs`）。** 関数 head_dated を足す。引数は head と同じで、日付の引数だけが（日付の名・日付）の組になる（引数の数は 7 のまま・clippy の引数の数の警告を出さない）。鮮度の札の字は <日付の名> <b><日付></b> · <b><版></b>（<名札>）。既存の head は（生成・generated）で head_dated へ委ねるだけにする。入口・判断の記録・設計ノートの面は今までどおり head を呼ぶので、3 面の出力は 1 byte も変わらない（(e) の 5）。
3. **憲法の面（`face_constitution.rs`）。**
   - 関数 head: 文書が発効なら dated に Approved の date（便 144 の今の版の承認の日付・今の版を名指す判断が無ければ在る承認の最も新しい日付）を、draft なら None を渡し、head_dated を呼ぶ。
   - 関数 cover: 版の札の日付を Approved の date にする。draft の Approved の date は meta.generated（便 144 の定め）なので、draft の面は base と 1 byte も変えない。
4. **要件書の面（`face_srs.rs`）。**
   - 関数 head: dated に last_approval を渡し、head_dated を呼ぶ。
   - 関数 cover: 版の札の日付を dated の日付にし、表紙の状態の承認の日付も last_approval から取る（base の同じ式のループを口の呼び出しに置き換える・表紙の状態の字は変えない）。
5. **入口の棚の要件書のカード（`face_index_read.rs` の srs_card）。** 日付を dated と last_approval から取る（base と同じ値・入口の面の出力は 1 byte も変えない）。使わなくなる型 Standing の輸入を外す。
6. **2 面の頭の字（本便の後）。**

| 版の立場 | 鮮度の札 | 版の札 | 表紙の状態 |
| --- | --- | --- | --- |
| Draft | 生成 <generated> · <版>（今の名札） | <版> / <generated>（base のまま） | base のまま |
| Effective | 承認 <今の版の承認の日付> · <版>（発効・拘束力あり） | <版> / <今の版の承認の日付> | base のまま（同じ日付） |
| Unknown | 承認 <在る承認の最も新しい日付> · <版>（効く版はまだ分からない） | <版> / 同じ日付 | base のまま（同じ日付） |
| Pending（要件書だけ） | 承認 <最後の承認の行の日付> · <効いている版>（発効・拘束力あり・<版> は起草・承認待ち） | <版> / 同じ日付 | base のまま（同じ日付） |
| 発効だが承認の行が無い（要件書だけ） | 生成 <generated> · <版>（発効・拘束力あり） | <版> / <generated> | 発効・拘束力あり（base のまま） |

   1 つの表紙の上の日付は、鮮度の札・版の札・表紙の状態のどれも同じ 1 つの日付になる。便 138 と便 144 の札（起草・承認待ち・効く版はまだ分からない）の出し方は変えない。
7. **変えないもの。** 部品・class・様式の定義・章と節の並び・表紙の状態の字・承認欄・便 138 と便 144 の札の字。入口・判断の記録・設計ノートの面（鮮度の札の 生成 の字を含む）。入口の憲法と要件書のカードの字。床。設計文書。凍結 anchor の列（`design-intent/anchors/`）。憲法の凍結 anchor `tests/fixtures/face/expected.html`（fixture の憲法は draft）。
8. **契約で決めること（起草役の判断）。**
   - **鮮度の札の日付の名は、日付が承認の日付なら 承認、生成日なら 生成 にする。** 日付だけを承認の日付に替えて名を 生成 のまま残すと、承認の日付を生成日と偽ることになる（P-6.1）。名を消して日付だけにすると、draft の面では何の日付か読めない。名の字は既存の字（承認は表紙の状態と承認欄の役の名・生成は今の鮮度の札の名）で、部品と class は変えない。
   - **版の札は <版> / <日付> の形のまま、日付を鮮度の札と同じにする。** 依頼の候補の向きで、表紙の日付が 1 つに揃う。設計ノートの面の版の札（<版> / <generated>）と形が揃ったまま残る。日付を消して <版> だけにする形は (d) の 1。
   - **要件書の承認の日付は、承認欄の最後の 承認 の行の when にする（今の表紙の状態と入口のカードと同じ式）。** 効いている版 effective_version を名指す行を探す形にすると、同じ表紙の状態と入口のカードの式も替える便になり、範囲が広がる。今の正本では、最後の 承認 の行が効いている版 v1.44 の行である。
   - **版を上げる枝の上（Pending）でも同じ日付を出す。** 版の札は版の欄（起草中の版）と最後の承認の日付を並べるので、起草中の版がその日に承認されたと読める余地がある。ただし鮮度の札と表紙の状態が同じ行で <版> は起草・承認待ち と出すので、表紙の上で食い違いは無い。base の <版> / <初回の生成日> より誤読の幅は狭い（(i) の 2）。
   - **入口のカードの式を同じ口にまとめる。** 表紙と入口のカードが同じ事実（要件書の承認の日付）を別の 2 つの式で持つと、片方だけが替わったときに面の間で日付が割れる（P-6.3）。まとめても入口の面の出力は 1 byte も変わらない（(e) の 5）。
   - **共有の口は head を残して head_dated を足す。** head の引数を組に替えると、入口・判断の記録・設計ノートの 3 面の生成器（face_index.rs・face_adr.rs・face_note.rs）も write-set に入る。委ねる形なら 3 面の生成器を 1 字も触らずに 3 面の出力が変わらないことを示せる。
   - **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は、正本に在る欄（便 144 の今の版の承認・要件書の承認欄）を面の頭が読まずに初回の日付を出していた中身の正しさの直し（P-6.1）で、部品・色・余白・並び・様式の定義・名札の class を 1 つも変えず、HTML の構造も変えない（出力の差は 2 面の 2 行ずつの字だけ・(e) の 5）。鮮度の札の名の字が替わるのは、日付の意味を正本に合わせるためである。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137・138・139・141・144 の契約と同じ読みである。要件書の面の見た目の直しの数え（上限 2 に到達済み）は、便 35（章 09 の図の番号・部品 figure-panel）と便 36（用語集・章 08）の 2 本で、どちらも本便の部品（freshness-stamp・doc-cover-band）ではない。仮に本便を見た目の直しと数えても、同じ部品についての 1 本目で上限に届かない。持ち主の walk（面の承認）は章と部品を変えないので要しないと起草役は読む（求めるかは席が決める）。

### (c) 歯（関数名 f145_・base で 0 件）

1. **f145_last_approval_and_dated_read_the_approval_rows（単体の歯・`crates/folio/src/face_labels.rs` の既存の tests の区間 face_labels_tests）。** 行の並びが 作成・承認・承認・作成 の欄 meta で、発効（版が揃う・効いている版が違う・効いている版が無い）の 3 通りとも last_approval が最後の 承認 の行の日付を返し、dated が（承認・その日付）を返すこと。承認 の行が無い発効と draft は None と（生成・generated）。日付の字が escape 済みであること。表の外の状態 retired の Err。**base では口が無く、組み立てが通らない＝RED。**
2. **f145_srs_cover_dates_follow_the_last_approval（`crates/folio/tests/face_srs.rs`・binary 経由・面の fixture の写し）。** 6 通りの写しで、鮮度の札の日付の名と日付・版の札の字が (b) の 6 の表の字でちょうど 1 つずつ在ること。変異なし（承認 2026-09-05・v0.3 / 2026-09-05・生成日の字が無い）、承認の行を 1 つ足した写し（2026-09-09・表紙の状態も同じ日付）、最後に 作成 の行を足した写し（2026-09-05 のまま）、承認の行を消した写し（生成 2026-09-01・表紙の状態は 発効・拘束力あり）、版の欄を v0.4 に進めた写し（承認 2026-09-05・v0.4 / 2026-09-05）、draft の写し（生成 2026-09-01）。**base では 生成 2026-09-01 と v0.3 / 2026-09-01＝RED。**
3. **f145_real_srs_cover_dates_follow_the_last_approval（同・実の正本）。** 歯の側の手書きの読み（yaml-rust2 で承認欄を直に読み、最後の 承認 の行の when を escape する）の日付で、鮮度の札が 承認 <日付> で始まり、版の札が <版> / <日付> でちょうど 1 つずつ在り、生成日の鮮度の札が無いこと。**base では 生成 2026-09-12＝RED。**
4. **f145_constitution_cover_dates_follow_the_current_approval（`crates/folio/tests/face_constitution.rs`・binary 経由・実の置き場の写し）。** 便 144 の歯の側の手書きの読み（source_rows・source_meta）から組んだ今の版の承認の日付で、鮮度の札が 承認 <日付> · <版>、版の札が <版> / <日付> でちょうど 1 つずつ在り、生成日の鮮度の札と版の札が無いこと。版の欄を先に進めた写しでは版の札が <版>-next / <在る承認の最も新しい日付>、版を上げた判断を全部提案中に戻した写しでは版の札と鮮度の札が初回の承認の日付。**base では v1.4 / 2026-09-12＝RED。**
5. **f145_fixture_constitution_dates_by_status（同・面の fixture の写し）。** fixture の憲法を draft のままにした写しで鮮度の札が 生成 2026-09-01 · v0.9（未承認・拘束力なし）・版の札が v0.9 / 2026-09-01、発効にした写し（adr/ なし＝初回の承認の版）で 承認 2026-09-02 · v0.9（発効・拘束力あり）・v0.9 / 2026-09-02。**base では発効の写しが生成日を出す＝RED（draft の半分は守り）。**

fixture は新しい file を足さない。変異は歯の中で一時 dir の写しを書き換える（歯の file の既存の口 fixture_srs・real_copy・to_proposed と同じ形）。正本の読みは歯の側の手書き（生成側の口を呼ばない・P-10.1）。

**既存の歯の期待の字の直し（全数・歯の意図は変えない）。**

| 歯の file | 歯 | 直す字 |
| --- | --- | --- |
| face_srs.rs | 便 138 の口 stamp（f138_pending・f138_equal・f138_missing_effective_version が使う） | 鮮度の札の 生成 2026-09-01 を 承認 2026-09-05 に（口 stamp_dated を足し、draft は 生成 2026-09-01 のまま） |
| face_srs.rs | f138_draft_does_not_read_effective_version | 口 stamp_dated で 生成 2026-09-01 を明示（字は base と同じ） |
| face_srs.rs | f138_real_srs_head_follows_the_meta | 鮮度の札の日付の部分を 生成 <b> 固定で切り出す所を、名と日付の部分ごと切り出す形に（札の中身の検査は不変） |
| face_constitution.rs | 便 144 の口 stamp_of（f144_cover・f144_unknown の 2 か所・f144_without が使う） | 生成 <generated> を 承認 <その写しの今の版の承認の日付>（f144_cover は今の版の承認・f144_unknown は在る承認の最も新しい日付・f144_without は初回の承認）に。使わなくなる generated の束縛を外す |

### (d) 採らなかった形

1. **版の札の日付を消して <版> だけにする。** 表紙の日付は 2 か所（鮮度の札・表紙の状態）で足り、Pending の誤読の余地も無くなる。ただし draft の面（fixture の憲法・憲法の凍結 anchor）も字が替わり、設計ノートの面の版の札（<版> / <日付>）と形が割れる。依頼の候補の向き（日付を承認の日付に）でも表紙の日付は揃う。
2. **鮮度の札の名を 生成 のまま日付だけを替える／名を 発効 にする。** 前者は承認の日付を生成日と偽る。後者は draft と Pending の札（起草・承認待ち）と並べたときに、起草中の版が発効と読める（便 144 が入口のカードの名で採らなかった理由と同じ）。
3. **Frame::head の引数を（名・日付）の組に替えて 5 面すべての呼び出しを直す。** 3 面の生成器が write-set に入り、3 面の出力が変わらないことを生成器の字の読みでも示す必要が出る。
4. **要件書の承認の日付を、効いている版を名指す承認の行から取る。** (b) の 8 のとおり、表紙の状態と入口のカードの式も替える便になる。
5. **判断の記録・設計ノート・入口の面の鮮度の札も同じ便で直す。** 依頼の範囲（2 面）の外で、判断の記録の面は欄の決まり（承認欄の形）が違う。控えとして §5 に残す。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **実装だけを base に当てた写し（歯の file と凍結 anchor は base のまま）で、workspace の nextest は 919 本のうち 13 本が落ちる（起草役の実測）。** どれも要件書の凍結 anchor 1 本の 2 か所と、(c) の表の期待の字の直しで直る。

| 歯 | 本数 | 直し方 |
| --- | ---: | --- |
| face_srs_body の face_srs_write_matches_the_frozen_fixture・face_srs_figure の face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face・badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures・site の site_write_matches_the_frozen_fixture・face_srs の f118_handwritten_scope_m3_adds_one_escaped_block と f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required | 6 | 要件書の凍結 anchor の 2 か所（本文は変えない） |
| face_srs の f138_pending・f138_equal（凍結 anchor との一致も含む）・f138_missing_effective_version・f138_real_srs_head_follows_the_meta | 4 | (c) の表（と凍結 anchor） |
| face_constitution の f144_cover・f144_unknown・f144_without | 3 | (c) の表 |

   本便の差分の全部を base に当てた写しで、workspace の nextest は 924 / 924（base 919 + 単体 1 + face_srs 2 + face_constitution 2）・clippy 0 警告・床 4 本 rc 0・`folio parts --check`（憲法と要件書の面）合格（違反 0・まだ分からない 0）・face_srs 23 / 23・face_constitution 25 / 25・face_index 29 / 29。
2. **動く凍結 anchor は 1 本で、2 か所。** `tests/fixtures/face/expected-srs.html` の 18 行目の鮮度の札の 生成 2026-09-01 を 承認 2026-09-05 に、32 行目の版の札の v0.3 / 2026-09-01 を v0.3 / 2026-09-05 にする（起草役は script で 1 か所ずつ置き換え、生成器の出力と byte で一致した）。byte 数は 34,276 のまま（生成 と 承認 は同じ byte 数・日付は 1 字の差）・行数 325 のまま・sha256 の頭 8 字は 1ec51ee2 から 61486a0b。憲法の凍結 anchor expected.html（fixture の憲法は draft で字が変わらない）・入口 2 本・判断の記録 2 本・設計ノート 1 本の凍結 anchor・床の凍結の土台・天井の束の凍結 anchor は 1 byte も変えない。
3. **RED（起草役の実測）。** 歯と凍結 anchor だけを base に当てた写し（単体の歯を除く 923 本）で 16 本が落ちる＝binary の f145_ の 4 本すべて（(c) の 2〜5）と、期待の字を先に直した便 138 の歯 3 本（f138_pending・f138_equal・f138_missing_effective_version）と便 144 の歯 3 本と、要件書の凍結 anchor に掛かる 6 本（(e) の 1 の表の 1 行目）。単体の歯は組み立てが通らない（last_approval と dated が無い）。
4. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変え、verify の歯の file を撃つ）。** 11 通りとも 1 本以上が落ちる。

| 変異 | 落ちる歯（本便の f145_ と主なもの） |
| --- | --- |
| M1 要件書の鮮度の札の日付を生成日に戻す | (c) の 2・3 と要件書の凍結 anchor の 6 本・f138 3 本（11 本） |
| M2 要件書の版の札の日付を生成日に戻す | (c) の 2・3 と凍結 anchor の 6 本・f138_equal（9 本） |
| M3 憲法の鮮度の札の日付を生成日に戻す | (c) の 4・5 と f144 3 本（5 本） |
| M4 憲法の版の札の日付を生成日に戻す | (c) の 4・5（2 本） |
| M5 最後でなく最初の 承認 の行を読む | (c) の 1・2・3 と入口の f144 2 本（5 本） |
| M6 役が 承認 でない行も読む | (c) の 1・2（2 本） |
| M7 draft でも承認の行を読む | (c) の 1・2 と f138_draft・入口の f144 1 本（4 本） |
| M8 生成日でも名を 承認 にする | (c) の 1・2・5 と f138_draft・badge・site（6 本） |
| M9 draft の憲法でも名を 承認 にする | (c) の 5・badge・site（3 本） |
| M10 共有の口 head（3 面が使う）の名を 承認 にする | 入口の凍結 anchor 2 本・badge・site（4 本＝3 面が変わらないことの歯） |
| M11 入口の要件書のカードの日付を生成日にする | 入口の f138 2 本・f144 2 本・凍結 anchor 2 本・badge・site（8 本） |

5. **folio2 自身の面の変化（起草役の実測）。** base の写しで `folio build --dir design-intent --out <置き場> --write` の出力は 30 file のままで、違う file は constitution.html と srs.html の 2 枚だけ（`diff -rq`）。index.html・判断の記録の面 23 枚・設計ノートの面 2 枚・様式と script の 2 file は byte で一致する。

| 面 | 所 | base | 本便の後 |
| --- | --- | --- | --- |
| 憲法 | 鮮度の札（18 行目） | 生成 2026-09-12 · v1.4（発効・拘束力あり） | 承認 2026-09-25 · v1.4（発効・拘束力あり） |
| 憲法 | 版の札（30 行目） | v1.4 / 2026-09-12 | v1.4 / 2026-09-25 |
| 要件書 | 鮮度の札（18 行目） | 生成 2026-09-12 · v1.44（発効・拘束力あり） | 承認 2026-09-26 · v1.44（発効・拘束力あり） |
| 要件書 | 版の札（32 行目） | v1.44 / 2026-09-12 | v1.44 / 2026-09-26 |

   2 面とも −2 行 / +2 行で、表紙の状態の日付（2026-09-25・2026-09-26）と揃う。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 14 本とも印なし（書き換える 8 本 = `crates/folio/src/face.rs`・`crates/folio/src/face_labels.rs`・`crates/folio/src/face_constitution.rs`・`crates/folio/src/face_srs.rs`・`crates/folio/src/face_index_read.rs`・`crates/folio/tests/face_srs.rs`・`crates/folio/tests/face_constitution.rs`・`tests/fixtures/face/expected-srs.html`／本文不変の 6 本 = `crates/folio/tests/face_srs_body.rs`・`crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/face_index.rs`・`crates/folio/tests/face_labels.rs`・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 5 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face.rs` | 1,111 | 389 | 1,124（+13） | 376 |
| `crates/folio/src/face_labels.rs` | 447 | 1,053 | 514（+67・うち単体の歯 約 45） | 986 |
| `crates/folio/src/face_constitution.rs` | 1,153 | 347 | 1,155（+2） | 345 |
| `crates/folio/src/face_srs.rs` | 1,063 | 437 | 1,058（−5） | 442 |
| `crates/folio/src/face_index_read.rs` | 635 | 865 | 629（−6） | 871 |

   余地はどれも S の見積 100 を超える。行数の上限の歯は face.rs が 1,124 ≤ 1,250（`crates/folio/tests/face_labels.rs`・verify の 11）、face_srs.rs が 1,058 ≤ 1,100（`crates/folio/tests/face_srs.rs` の f100_face_srs_is_split_and_under_the_cap・verify の 4）で緑。face_srs.rs は上限まで 42 と狭いので、実装は face_srs.rs に字を足さず口を face_labels.rs に置く（(b) の 1）。歯の file と凍結 anchor は src の外なので余地を測らない（参考値 face_srs 880 → 980・face_constitution 1,143 → 1,230）。
3. **size は S。**
4. **verify は 12 行**で、done の 12 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f145_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_srs f145_` = (c) の 2・3（2 本）。
   3. `cargo nextest run -p folio --test face_constitution f145_` = (c) の 4・5（2 本）。
   4. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（便 138 の歯・凍結 anchor との一致・行数の上限を含む・参考値 23 本）。
   5. `cargo nextest run -p folio --test face_constitution` = 憲法の面の歯の全部（便 144 の歯を含む・参考値 25 本）。
   6. `cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture` = 要件書の凍結 anchor との byte 一致。
   7. `cargo nextest run -p folio --test face_srs_figure face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face` = 同じ凍結 anchor から図の章を抜いた字との一致。
   8. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い 5 面の凍結 anchor 7 本との byte 一致（入口・判断の記録・設計ノートの面が変わらないことを含む）。
   9. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   10. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（入口の凍結 anchor 2 本と、要件書のカードの便 138・144 の歯を含む・参考値 29 本）。
   11. `cargo nextest run -p folio --test face_labels` = face.rs の行数の上限 1,250。
   12. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_srs・face_constitution・face_srs_body・face_srs_figure・badge・site・face_index・face_labels の 8 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f145_ を関数名に持つ src は `crates/folio/src/face_labels.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d145-draft.md`、模擬の差分と script は同じ dir の d145-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-145.py（実装）・apply-145-teeth.py（歯と便 138・144 の歯の期待の字）・anchor-145.py（要件書の凍結 anchor 2 か所）を撃つ。その差分が c145.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -rq`・`folio parts --check`・verify の 12 行（verify-145.sh）を撃つ。
2. RED: r145-teeth.patch（歯と凍結 anchor だけ）を base に当てると (e) の 3 のとおり。実装だけを当てると (e) の 1 の 13 本が落ちる。
3. 突然変異: mut-145.sh（(e) の 4）。
4. 余地: lines-145.py と lines-145.awk（同じ式の 2 実装）を write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 入口・判断の記録・設計ノートの面の鮮度の札（生成 <generated> のまま・判断の記録の面は承認の日付と食い違って見える同じ形を持つ＝(a) の 4）。要件書の面の図 1〜3 の札（図 1 · v1.44 2026-09-12 · srs.yaml の形）と憲法の面の図 1 の札、2 面の足の行（要件書 v1.44（2026-09-12）の形・folio が生成した の文の中）＝版の横に初回の生成日を置く同じ形が残る（台帳の控え f2-648.219 に含める）。表紙の状態・承認欄・入口のカードの字。要件書の承認の日付の式（最後の 承認 の行のまま）。凍結 anchor の列。設計文書の字（meta.generated の欄の名と意味を含む）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 版を上げる枝の上（Pending）の要件書の版の札は、起草中の版と、効いている版の承認の日付を並べる（鮮度の札と表紙の状態が同じ行で起草・承認待ちと出す）。Unknown の憲法の日付は今の版の承認でなく在る承認の最も新しい日付である（便 144 の表紙の状態と同じ）。要件書の日付は承認欄の最後の 承認 の行で、効いている版の行とは限らない（今の正本では一致する）。日付は正本の字をそのまま出し、形（年-月-日）を面で確かめない。発効の要件書に 承認 の行が 1 つも無い置き場（床は落とさない・写しで実測）では、鮮度の札と版の札は 生成 の名で生成日を出す（名は正しいが、P-4.2 の読みで「承認の日付はまだ分からない」と出す形は採らない＝base の表紙の状態と同じ扱い）。Unknown の憲法（版の欄を先に進めた置き場）の版の札は <次の版> / <在る承認の最も新しい日付> の並びになり、要件書の Pending と同じ形で、同じ表紙の鮮度の札と表紙の状態が「効く版はまだ分からない」と出す。鮮度の札の名は、何の日付かを示すだけで、面を組んだ日（生成の実行の日）は出さない（正本に無い値なので P-6.3 により出さない）。次の周で読みやすさの観点がこの食い違いを解けたと読むかは周の結果でしか分からない。面の見た目が持ち主に受け入れられるかは walk でしか分からない。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 13 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と比べて file 数が変わるか、constitution.html と srs.html のほかの file が 1 byte でも違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、Frame::head の引数の形・便 144 の Approved の date の定め・要件書の承認欄の形が base と違えば、(b) と (f) を数え直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の last_approval・dated・単体の歯 1 本。`crates/folio/src/face.rs` の Frame::head_dated（head はそこへ委ねる）。`crates/folio/src/face_constitution.rs` と `crates/folio/src/face_srs.rs` の鮮度の札と版の札の日付（要件書の表紙の状態は同じ口から）。`crates/folio/src/face_index_read.rs` の要件書のカードの日付を同じ口から。`crates/folio/tests/face_srs.rs` の歯 2 本と便 138 の歯の期待の字。`crates/folio/tests/face_constitution.rs` の歯 2 本と便 144 の歯の期待の字。要件書の凍結 anchor の 2 か所。
- 入れない: 入口・判断の記録・設計ノートの面の字・部品目録・名札の class・様式の定義・床・設計文書・凍結 anchor の列・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| dated | 表紙の日付の口 | `crates/folio/src/face_labels.rs` の last_approval・dated |
| frame | 共有の口 | `crates/folio/src/face.rs` の Frame::head_dated（head は委ねる） |
| head | 2 面の頭 | 鮮度の札と版の札（`crates/folio/src/face_constitution.rs`・`crates/folio/src/face_srs.rs`） |
| card | 入口の要件書のカード | `crates/folio/src/face_index_read.rs` の srs_card の日付（出力は不変） |
| anchor | 要件書の凍結 anchor | `tests/fixtures/face/expected-srs.html` の 2 か所 |
| teeth | 歯 | 単体の f145_ 1 本・face_srs の f145_ 2 本・face_constitution の f145_ 2 本・便 138 と 144 の歯の期待の字 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 7572ab4）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.218）を閉じる。判断の記録の面（鮮度の札 生成 <generated> と表紙の状態の承認の日付）・設計ノートと入口の面の鮮度の札を同じ形に揃えるかを決める（§1 (a) の 4・(i) の 1＝控えの候補）。着地後の 2 面の頭を持ち主への次の報告で見せるかを決める。次の周で、読みやすさの観点が表紙の日付の食い違いを所見に立てないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "er"
title = "便 144 の問い 3 の控えと独立の検証の提案 1（台帳 f2-648.218・一括 22 の B-1 の残り）: 憲法と要件書の面の表紙の鮮度の札（生成 <初回の生成日> · <版>）と版の札（<版> / <初回の生成日>）が初回の生成日を出し、同じ表紙の状態の承認の日付と食い違って見える問題を直す。表紙の日付の口（crates/folio/src/face_labels.rs）に、要件書の承認欄の最後の 承認 の行の日付を返す last_approval（draft か行が無ければ None）と、承認の日付が在れば（承認・その日付）・無ければ（生成・生成日）を返す dated を足す。共有の口 Frame（crates/folio/src/face.rs）に日付の名と日付を選ぶ head_dated を足し、既存の head は 生成 で委ねる（入口・判断の記録・設計ノートの 3 面の出力は 1 byte も変えない）。憲法の面（crates/folio/src/face_constitution.rs）の鮮度の札と版の札の日付を便 144 の今の版の承認の日付（draft は生成日のまま）に、要件書の面（crates/folio/src/face_srs.rs）の鮮度の札と版の札の日付を承認欄の最後の承認の行の日付（無ければ生成日）にし、鮮度の札の日付の名を承認の日付なら 承認・生成日なら 生成 にする。要件書の表紙の状態と入口の要件書のカード（crates/folio/src/face_index_read.rs）の日付も同じ口から取る（字は不変）。表紙の状態・承認欄・部品・class・様式の定義・設計文書は変えない。要件書の凍結 anchor の 2 か所を手で直し、便 138 と便 144 の歯の期待の字を直す。歯は単体の f145_ 1 本と face_srs の f145_ 2 本と face_constitution の f145_ 2 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main 7572ab4・受付の時点の main で数え直す"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_labels.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_index_read.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_constitution.rs", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face_srs_body.rs", "crates/folio/tests/face_srs_figure.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/face_labels.rs"]
verify = ["cargo nextest run -p folio --bin folio f145_", "cargo nextest run -p folio --test face_srs f145_", "cargo nextest run -p folio --test face_constitution f145_", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_srs_figure face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face_labels", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f145_（表紙の日付の口が、発効の 3 通りで承認欄の最後の 承認 の行の日付と名 承認・承認の行の無い発効と draft で None と名 生成 と生成日・字は escape 済み・表の外の状態は Err）が緑、face_srs の f145_ の 2 本（面の fixture の写し 6 通りで鮮度の札の日付の名と日付・版の札が最後の承認の行か生成日に従い、実の要件書の鮮度の札と版の札が歯の側の手書きの読みの最後の承認の日付を出し生成日を出さない）が緑、face_constitution の f145_ の 2 本（実の置き場の写しで鮮度の札と版の札が今の版の承認の日付を出し生成日を出さない・版の欄を進めた写しと版を上げた判断の無い写しで版の札が在る承認の最も新しい日付と初回の承認の日付・面の fixture の憲法が draft なら生成日で発効なら初回の承認の日付）が緑、face_srs の歯の全部（便 138 の歯と凍結 anchor との一致と行数の上限を含む）が緑、face_constitution の歯の全部（便 144 の歯を含む）が緑、face_srs_body の要件書の凍結 anchor との byte 一致が緑、face_srs_figure の図の章を抜いた字との一致が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、face_index の歯の全部（入口の凍結 anchor 2 本と要件書のカードの歯を含む）が緑、face_labels の face.rs の行数の上限が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで違う file は constitution.html と srs.html の 2 枚だけ"
<!-- contracts:end -->
