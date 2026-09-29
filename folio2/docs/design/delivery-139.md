# 設計: 便 139 — 入口の棚の判断の記録のカードに、番号と見出しの一覧を折りたたみで出す（見出しは正本の title の逐語）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は入口の面の棚の判断の記録のカードに、正本の欄 title（判断の記録の見出し）を読んで番号と見出しの一覧を足すだけで、FR4 の規範文も部品も色も様式の定義（folio.css）も変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する＝見出しは正本の title を切らずに escape だけして出す）/ P-2.4（部品の閉じた一覧＝部品も class も足さず、判断の記録の面の根拠の一覧と同じ ul.basis と、入口の面が既に使う折りたたみ details.note だけで組む）/ P-2.3（design token と様式の定義は 1 か所＝folio.css を変えない）/ P-5.1（折りたたみの名札は型付きの定数で持つ）/ P-6.2（生成物を手で直さない＝面の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 天井の 33 周目（2026-09-25・本流 94e6d3e）の読みやすさの所見 **F-6**（重さ 直す・場所 index の shelf.documents.adr）。一括 20 の仕分け（`docs/design/batch20-triage.md`）の便の候補 **B-1**。台帳の控え **f2-648.205**。カードに題が無いという同じ指摘は 3・5・27・28・30・33 周目に立ち、33 周目で 6 回目（仕分けの数え）。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `el` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 8 本（書き換える 6 本 + 本文が変わらない verify の scope 2 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-25・席の指示と裁定）: 便 138 の着地（main 021b7a4）を受けて枝を main に載せ替え、base を 021b7a4 に一本化した（便 138 の前の base の数の列と、便 138 の着地を待つ字を外した）。数はすべて 021b7a4 の写しで撃ち直した（nextest 897 → 901・face_index の歯 22 → 26・`face_index_read.rs` 623 → 626）。凍結 anchor 2 本の手の直しが 021b7a4 の生成器の出力と byte で一致することを確かめ直した。模擬の差分は 021b7a4 の上で作り直した。§0 の並行の便との重なりを着地後の形にした。席は起草役の問い 8 点を全部推奨どおりに裁定した（見出しは切らない・details.note で畳む・番号の列は残す・判断の記録のカードだけ当たり判定を外し folio.css は触らない・id の数の昇順・S・ADR-5 の撤退条件 ② に当たらない・行ごとの状態の名札は出さない）。walk は前置せず、着地後の面を持ち主への次の報告で見せる（席の裁定）。設計・歯・write-set・verify・done・size は変えない。
- 改訂 c（2026-09-25・独立の検証 d139-verify.md〔条件付き支持・blocking 1・数値は B-1 のほかは全部一致〕）: blocking B-1 = 当たり判定 sc-hit が折りたたみと一覧を覆い最新の記録へ飛ぶという字は実測で偽（base の 4 枚とも 0×0 で効いていない）。席の裁定で外す決定は残し、理由を今は効いていない・様式の定義が意図どおりに直ったときに一覧の押した先を覆わないための先回り・今の見た目と押した先は不変、に直した（§1 (a) の 5・(b) の 2・4・6・(d) の 4・(i) の 2・契約表の title）。様式の定義の潜在の不具合は台帳の控え f2-648.213 として §5 に書いた。任意の提案から、(f) の 2 に行数の上限の歯 schema_docs.rs の 700、(e) の 4 に変異 2 行（行き先を全部最新へ・畳まない・起草役の実装でも落ちることを実測）、(i) の 2 に開いた一覧の最長の行の高さを足した。数値・write-set 8・verify 6・done・size S は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 8 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は便 138 の着地（本流 021b7a4）。**base = main 021b7a4。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 138（行 ek・台帳 f2-648.210）は着地済み（main 021b7a4）。本便の write-set と共に持つ 4 本（`crates/folio/src/face_index_read.rs`・`crates/folio/tests/face_index.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`）は着地後の形で数えた。便 140（一括 20 の B-5・起草中・`crates/folio/src/floor_note.rs` と床の fixture）とは重なり 0 である。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 021b7a4・数は参考値）

1. **カードは番号だけを並べる。** 入口の面の生成器 `crates/folio/src/face_index.rs` の関数 adr_rows は、判断の記録の card に 2 行を出す。1 行目は本数と範囲と状態ごとの数（● 23 本・ADR-1〜ADR-23（発効 23））。2 行目は更新の日付・23 本の番号のリンク（class xref・行き先 adr-n.html）を中黒でつないだ列・開く →（最も新しい番号 ADR-23 の面へ）・card 全体の当たり判定（class sc-hit・行き先は開く → と同じ・(a) の 5 のとおり今は効いていない）である。見出しの字はどこにも無い。
2. **正本の見出しは読んでいるが捨てている。** 読み手 `crates/folio/src/face_index_read.rs` の関数 records は、判断の記録の正本ごとに欄 title を必須として読み（escape した字）、値を捨てる。型 Record は id・状態の名札・日付だけを持つ（便 26 が title を必須の欄にした）。
3. **正本の見出しの長さ（base の design-intent/adr の 23 本・空白を含む字数）。** 最長は ADR-7 の 171 字、最短は ADR-15 の 33 字、中央は 89 字、23 本の合計は 2,011 字。100 字を超えるのは 6 本（ADR-4・6・7・8・10・12）。区切りの字（前後に空白の付いたダッシュ）を持つのは 6 本（ADR-3・7・8・18・19・20）で、区切りの前だけに切っても合計は 1,636 字・最長は 106 字である。
4. **カードの幅（起草役がブラウザで測った参考値）。** 棚の置き場の格子は、横 1,400 の窓で card 1 枚の幅が約 320 で、判断の記録の card の高さは 392 である。幅 390 の窓（縦 1 列）では 397 である。
5. **当たり判定は今は効いていない。** 様式の定義の意図（部品目録 parts.json の sc-hit = 棚カードの全面リンク）は card 全体を覆う当たり判定だが、生成器が sc-hit を class sc-row の行の中に置くため、行の中のリンクを上に出す規則（folio.css 177 行・詳細度が高い）が当たり判定の絶対配置の規則（176 行）に勝ち、4 枚の card とも当たり判定の大きさは 0×0 で効いていない（独立の検証役の base の実測）。card の右上の空きを押しても何も起きない。この様式の定義の潜在の不具合は本便の範囲の外で、台帳の控え f2-648.213 で扱う（§5）。
6. **base の歯（参考値）。** workspace の nextest 897 / 897・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file。`git grep -n 'f139_' -- crates` は 0 件。歯の file の本数は face_index 22・face_index_shelf 11・badge 16・site 13。

### (b) 直す先 — 読み手が見出しを持ち、カードの最後に番号と見出しの一覧の折りたたみを足す

1. **読み手（`crates/folio/src/face_index_read.rs`）。** 型 Record に見出しの欄（escape した正本の title の逐語）を足し、関数 records が読んで捨てていた値をそこへ入れる。欄の必須・escape・並べ方（id の数の昇順）は今のまま。
2. **カード（`crates/folio/src/face_index.rs` の関数 adr_rows）。** 字は次のとおり。1 行目は base のまま。
   - 2 行目（更新・番号のリンクの列・開く →）は base のまま残し、**card 全体の当たり判定だけを外す**（様式の定義の意図どおりに当たり判定が効けば、折りたたみの名札と一覧は sc-row の外なので覆われる。今は効いていないが（(a) の 5）、効くように直ったときに一覧の行き先が最新の記録に化けないよう、判断の記録のカードからだけ先に外す。外しても今の面の見た目と押した先は変わらない）。
   - 2 行目の後（card の最後）に、既存の折りたたみ details.note を 1 つ置く。名札は 番号と見出しの一覧（<本数> 本）。中身は既存の ul.basis（判断の記録の面の根拠の一覧と同じ形）で、1 本 1 行・id の数の昇順・各行は番号のリンク（class xref・行き先 adr-n.html）と見出しの逐語（span）である。既定は畳んだまま。
   - 名札の字 番号と見出しの一覧 は `face_index.rs` の型付きの定数に置く。
   - 生成物の形（fixture の写し・記録 1 本）: 2 行目の末尾から当たり判定の a が消え、その後に details の開き（summary と ul の開きまで）の 1 行・li の 1 行・ul と div と details の閉じの 1 行が続く。
3. **見出しは切らない（逐語）。** 正本の title を escape するだけで出す。区切りの字の前だけにする形は採らない（(d) の 1）。長い見出しを短くするのは正本の側の直し（一括 20 で ADR-21・ADR-22 の見出しを短くしたのと同じ置き場）であり、面が別の規則で切ると面と正本の字が食い違う。
4. **畳む理由（起草役の実測）。** 一覧を畳まずに card に置いた最初の模擬では、card の高さが横 1,400 の窓で 3,283（base の 392 の 8 倍余り）になり、棚の図（figure の docset）が縦に伸びた。畳むと閉じたときの高さは 441（base より 49 高い・名札の 1 行ぶん）、開くと 3,964 である。幅 390 の窓では閉じて 444・開いて 3,841 で、横のはみ出しは出ない。開いたときの折りたたみの名札と一覧の番号と見出しを押した先が、それぞれ折りたたみ自身と各記録の面であること（最新の記録へ飛ばないこと）を確かめた。当たり判定を残しても同じである（base の当たり判定は効いていない・検証役の反実仮想の実測）。
5. **変えないもの。** 1 行目・2 行目の番号の列と開く → の行き先・ほかの 3 枚の card（憲法・要件書・設計ノート）の当たり判定・棚の図の見出しと凡例と数・読む順番・相談窓口・支度表の節・脚・入口の正本 index.yaml・様式の定義 folio.css と folio-ui.js・部品目録・判断の記録と設計ノートの面・床・設計文書。
6. **契約で決めること（起草役の判断）。**
   - **見出しを切るか: 切らない**（(b) の 3・(d) の 1）。
   - **並びの順: id の数の昇順**（base の番号の列と、判断の記録の面の前後の案内と同じ順）。
   - **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は正本に在る欄（title）を入口の面が読んで捨てていたのを出す中身の直しで、見た目だけの直しではない。部品（data-component）も class も色も余白も足さず、様式の定義を 1 字も変えない。当たり判定を外すのは、今は効いていない要素を判断の記録のカードから除く先回りで、面の見た目も押した先も変えない。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137・138 の契約（撤退条件 ② に当たらないとした読み）と同じである。仮に数えても、部品 shelf-card について見た目だけを直した便は、撤退条件 ② の数えに名指した契約（便 15・16・20・27・28・35・36）に無い（便 16 は入口の面を作った便）ので、1 本目で上限 2 に届かない。要件書の面の見た目の直しの数え（上限 2 に到達済み）には関わらない。持ち主の walk（面の承認）は前置しない。着地後の面を持ち主への次の報告で見せる（席の裁定・折りたたみの中の見た目は開いて見ないと分からないため）。
   - **部品の閉じた一覧（P-2.4）: 新しい class は無い。** 使う class（note・basis・xref）は様式の定義に在り、実の正本で組んだ入口の面に `folio parts --check --page index=<面>` を撃つと合格（違反 0・まだ分からない 0）である（起草役の実測）。

### (c) 歯（関数名 f139_・base で 0 件・どれも `crates/folio/tests/face_index.rs`・binary 経由）

1. **f139_adr_card_lists_each_number_with_its_title_in_number_order。** 入口の写し（記録は fixture の ADR-2 の 1 本）に、ADR-2 を歯の中で写して id を ADR-10・title を 十番目の記録 にした記録を足した面で、折りたたみが card にちょうど 1 つ在り、中身が 名札 番号と見出しの一覧（2 本）・ADR-2 の行（見本の判断の記録）・ADR-10 の行（十番目の記録）の順（字の順なら ADR-10 が先）の逐語で、1 行目の要約は ADR-2〜ADR-10（提案中 2）、折りたたみは更新の行の後で card の最後に在る。期待の字は歯の中の手書き。**base では折りたたみが無い＝RED。**
2. **f139_adr_title_is_escaped_verbatim_and_not_cut。** 写しの ADR-2 の title を、山括弧のタグと and 記号と区切りの字とその後ろの字を持つ字（<b>前</b> と 後 — 長い見出しの後半も切らずに出す）にした面で、一覧の行が escape した逐語でちょうど 1 つ在り、区切りの後ろも残り、面に生のタグが無い。**base では一覧が無い＝RED。**
3. **f139_adr_card_has_no_hit_area_and_opens_the_newest。** 記録 2 本の写しの面で、判断の記録の card に当たり判定が無く、2 行目が 更新 2026-09-06・ADR-2 と ADR-10 の番号のリンクの列・開く →（adr-10.html）の逐語でちょうど 1 つ在り、面の当たり判定はほかの 3 枚の card の 3 つだけ。**base では当たり判定が在る＝RED。**
4. **f139_real_adr_card_lists_every_record_title。** 実の正本の入口の面で、design-intent/adr の全部の判断の記録を yaml-rust2 で直に読み（生成側から独立した物差し）、id の数の昇順の全行（番号のリンクと escape した title の逐語）を名札の本数とともに組んだ期待の折りたたみが card にちょうど 1 つ在り、行の数が記録の本数と等しい。**base では一覧が無い＝RED。**

fixture は新しい file を足さない。2 本目の記録は歯の中で fixture の ADR-2 を写して作る（歯の file の既存の口 index_fixture_copy・index_from・edit と同じ形）。

### (d) 採らなかった形

1. **見出しを区切りの字の前だけに切る（短い見出し）。** 区切りを持つのは 23 本のうち 6 本で、切っても合計 2,011 字が 1,636 字・最長 171 字が 106 字になるだけで、短い一覧にはならない（(a) の 3）。面に正本と別の字を出す規則（どこで切るか）を型付きの定数として持つことになり、区切りの前が決定の中身かどうかは見出しごとに違う。見出しが長いことは正本の側の所見として一括で直すほうが、面と正本の字が 1 つに揃う（P-6.1・P-6.3）。
2. **一覧を畳まずに card に置く。** (b) の 4 のとおり、card が base の 8 倍余りの高さになり、棚の図の釣り合いが崩れる。
3. **番号の列を消して、一覧だけにする。** 番号を知っている読み手（持ち主と席）が 1 回で面へ行ける今の導線を 2 回の操作に変える。base の番号の列と入口の census の歯（面へのリンクが id の順に中黒で並ぶ）は今のまま残る。
4. **一覧を sc-row の行の並び（1 本 1 段落）で出し、当たり判定を残す。** 番号のリンクは当たり判定の上に出るが、様式の定義の意図どおりなら見出しの字を押すと最新の記録へ飛ぶ（今は当たり判定が効いていないので飛ばない）。行の中で長い見出しは番号の次の段へ折り返し、番号と見出しの位置が揃わない。
5. **様式の定義に折りたたみや一覧を当たり判定の上に出す規則を足す。** 様式の定義は設計文書の置き場（design-intent/preview/folio.css）に在り、門の対象で、面の見た目の直しにもなる。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **落ちる既存の歯は 5 本で、4 本は凍結 anchor 2 本の手の直しで、1 本は歯の本文の 1 か所で直る（起草役の実測）。** 実装だけを base に当てた写し（歯の file と凍結の面は base のまま）で、workspace の nextest は 897 本のうち 5 本が落ちる。

| 歯 | 直し方 |
| --- | --- |
| face_index の face_index_write_matches_the_frozen_fixture・face_index_write_with_a_sheet_matches_the_frozen_fixture | 凍結の面 2 本の手の直し（本文は変えない） |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures | 同上（expected-index.html と expected-index-sheet.html を読む） |
| site の site_write_matches_the_frozen_fixture | 同上（expected-index.html を読む） |
| face_index_shelf の face_index_shelf_adr_card_is_readable_with_one_record | 歯の本文の 1 か所: 在ることを求める字の 3 つ目（card の当たり判定）を一覧の行（ADR-2・見本の判断の記録）に替え、当たり判定が無いことを 1 行で確かめる |

   入口の census の歯（面へのリンクが id の順に中黒で並ぶ）は、2 行目の番号の列を残すので落ちない。実装と凍結の面 2 本を当てた写し（歯を除く）で 897 本中 1 本（face_index_shelf の上の 1 本）だけが落ちる。本便の差分の全部を base に当てた写しで、workspace の nextest は 901 / 901（base 897 + f139_ 4）・clippy 0 警告・床 4 本 rc 0・face_index 26 / 26・face_index_shelf 11 / 11・badge 16 / 16・site 13 / 13。
2. **動く凍結 anchor は 2 本で、1 か所ずつ。** `tests/fixtures/face/expected-index.html` と `tests/fixtures/face/expected-index-sheet.html` の判断の記録の card の 2 行目の 1 行が、当たり判定の a を外した 1 行と、折りたたみの 3 行（details の開き・ADR-2 の行・閉じ）の 4 行になる（起草役は手で書き換え、生成器の出力と byte で一致した）。ほかの面の凍結 anchor（憲法・要件書・判断の記録 2 本・設計ノート）・床の凍結の土台・天井の束の凍結 anchor は 1 byte も変えない。判断の記録の面の歯 f74_glossary_chip_is_verbatim_the_index_chip は expected-index.html の付録の札を読むが、札の字は変わらず緑のまま（common-verify で撃つ）。
3. **RED（起草役の実測）。** 歯と凍結の面だけを base に当てた写しで、901 本中 9 本が落ちる＝f139_ の 4 本全部と、(e) の 1 の 5 本（凍結の面 4 本・face_index_shelf の 1 本は実装が無いので逆向きに落ちる）。
4. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変える・f139_ の 4 本と face_index_shelf の 11 本を撃つ）。** 9 通りとも 1 本以上が落ちる（M8・M9 は改訂 c で足した）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 見出しを出さない | (c) の 1・2・4・face_index_shelf |
| M2 一覧を逆の順に | (c) の 1・4 |
| M3 当たり判定を残す | (c) の 3・face_index_shelf |
| M4 見出しを escape しない | (c) の 2 |
| M5 見出しを区切りの字の前で切る | (c) の 2・4 |
| M6 名札に本数を出さない | (c) の 1・4 |
| M7 番号をリンクにしない | (c) の 1・2・4・face_index_shelf |
| M8 一覧の行き先を全部最新の記録へ | (c) の 1・4 |
| M9 畳まない（details を出さない） | (c) の 1・4 |

5. **folio2 自身の面の変化（起草役の実測）。** `folio build --dir design-intent --out <置き場> --write` の出力は 30 file のまま、base と違うのは index.html の 1 枚だけで、判断の記録の card の 2 行目の 1 行が 26 行（2 行目・折りたたみの開き・23 本の行・閉じ）になる。組み立てた入口の面は `folio parts --check` に合格する。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 8 本とも印なし（書き換える 6 本 = `crates/folio/src/face_index.rs`・`crates/folio/src/face_index_read.rs`・`crates/folio/tests/face_index.rs`・`crates/folio/tests/face_index_shelf.rs`・`tests/fixtures/face/expected-index.html`・`tests/fixtures/face/expected-index-sheet.html`／本文不変の 2 本 = `crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。書き換える src の file に行数の上限を持つ歯（歯 f100 の形）は無い（上限を持つ歯は `crates/folio/tests/face_srs.rs` の 1100〔face_srs.rs が相手〕と `crates/folio/tests/schema_docs.rs` の 700〔tests/schema.rs が相手〕で、どちらも write-set に当たらない）。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_index.rs` | 773 | 727 | 793（+20） | 707 |
| `crates/folio/src/face_index_read.rs` | 623 | 877 | 626（+3） | 874 |

   余地はどれも S の見積 100 を超える。歯の file と fixture は src の外なので余地を測らない（参考値 face_index 846 → 965・face_index_shelf 516 → 518・expected-index.html 160 → 163・expected-index-sheet.html 164 → 167）。
3. **size は S。**
4. **verify は 6 行**で、done の 6 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test face_index f139_` = (c) の 1〜4（4 本）。
   2. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（凍結の面 2 本との byte 一致・census・parts --check を含む・参考値 26 本）。
   3. `cargo nextest run -p folio --test face_index_shelf` = 棚の歯の全部（(e) の 1 の 1 本を含む・参考値 11 本）。
   4. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   5. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_index・face_index_shelf・badge・site の 4 本で、どれも write-set に在る。絞り込みの語 f139_ を関数名に持つ歯は `crates/folio/tests/face_index.rs` だけで write-set に在る（src の単体の歯は無い）。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d139-draft.md`、模擬の差分と script は同じ dir の d139-scripts（repo には入れない）。

1. 模擬: 受付の時点の main の写しの根で apply-139.py（実装）・anchor-139.py（凍結の面 2 本）・apply-139-teeth.py（歯）を撃つ。どれも字の置き換えで、base 021b7a4 に当たる（その差分が c139.patch）。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -r`・`folio parts --check --page index=<面>`・verify の 6 行を撃つ。
2. RED: 歯と凍結の面だけを当てると (e) の 3 のとおり。実装だけを当てると (e) の 1 の 5 本が落ちる。
3. 突然変異: mut-139.sh（(e) の 4）。
4. 余地: lines-139.py と lines-139.awk（同じ式の 2 実装）を写しの根で write-set の src の file に当てる。
5. 高さ: 組み立てた入口の面を loopback で配り、ブラウザで判断の記録の card の高さを折りたたみの開閉で測る（(b) の 4）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 見出しを短くする正本の直し（長い見出しは次の周の所見と一括で扱う）。一覧に状態（提案中・廃止）の名札を行ごとに出すこと（今の 23 本は全部発効で、状態ごとの数は 1 行目に在る）。判断の記録の面の側の一覧（所見が挙げたもう 1 つの置き場）。様式の定義の直し。設計文書の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 畳んだ一覧で所見 F-6 が解けたと読むかは次の周の読みやすさの観点でしか分からない（一覧が既定で閉じているので、開かない読み手には今と同じく番号の列が見える）。当たり判定を外した card は、様式の定義の card の規則（開く → を持つ card は全体に指の形の cursor）により、何も無い所でも指の形になるが押しても何も起きない。base の 4 枚の card も当たり判定が効いていないので既に同じで、本便の差分ではない。開いた一覧は幅 320 の card で、最長の見出しの 1 行が約 285 の高さになる（横 1,400・見出しの欄が約 200 の幅のため・検証役の実測）。面の見た目が持ち主に受け入れられるかは walk でしか分からない。横 1,400 の窓で入口の面が横にはみ出すのは base から在る事実で、本便の差分ではない（起草役の実測・base も同じ幅）。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 5 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と、index.html のほかの file で 1 byte でも違うか file 数が違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、関数 records が欄 title を読まなくなっているか、判断の記録の card に当たり判定が無くなっているか、入口の凍結 anchor 2 本の判断の記録の card の 2 行目が base と違えば、(b) と (e) を数え直してから運ぶ（欄 title が必須でなくなっていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/face_index_read.rs` の型 Record の見出しの欄と関数 records の読み。`crates/folio/src/face_index.rs` の名札の定数 1 つと関数 adr_rows の当たり判定を外すことと折りたたみ。`crates/folio/tests/face_index.rs` の歯 4 本。`crates/folio/tests/face_index_shelf.rs` の既存の歯 1 本の 1 か所。入口の凍結の面 2 本の 1 か所ずつ。
- 入れない: 入口の正本 index.yaml・様式の定義 folio.css と folio-ui.js・部品目録・ほかの 3 枚の card・1 行目と 2 行目の番号の列・判断の記録と設計ノートの面・床・設計文書・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| reader | 判断の記録の読み | `crates/folio/src/face_index_read.rs` の型 Record の見出しの欄・関数 records |
| card | 入口の棚の判断の記録のカード | `crates/folio/src/face_index.rs` の名札の定数・関数 adr_rows の折りたたみと当たり判定 |
| anchor | 入口の凍結の面 | `tests/fixtures/face/expected-index.html` と `tests/fixtures/face/expected-index-sheet.html` の 1 か所ずつ |
| teeth | 歯 | face_index の f139_ 4 本・face_index_shelf の既存の歯 1 本の 1 か所 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 便 138（base = main 021b7a4）。
- 並行の便: 便 138 は着地済み（§0）。受付は逐次。
- 台帳の控え f2-648.213: 入口の棚の card の当たり判定 sc-hit が、様式の定義の規則どうしの詳細度で 4 枚とも効いていないこと（§1 (a) の 5・folio.css は設計文書の置き場で門の対象・直すと入口のほかの 3 枚の操作が変わるので本便に入れない）。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.205）を閉じる。次の周の読みやすさの観点が、畳んだ一覧で所見 F-6 を解けたと読むか、長い見出し（最長 171 字）を正本の側の所見として立てるかを見る。判断の記録の面の側に一覧を置くか（所見が挙げたもう 1 つの置き場）を台帳の控えに残すかを決める。着地後の面を持ち主への次の報告で見せる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "el"
title = "一括 20 の B-1（天井の 33 周目 読みやすさ F-6・同じ指摘は 6 回目・台帳 f2-648.205）: 入口の棚の判断の記録のカードが番号だけを 23 個並べ、読み手が番号から何の判断か取れない問題を直す。読み手（crates/folio/src/face_index_read.rs）の型 Record に正本の title の逐語（escape 済み）を持たせ、カード（crates/folio/src/face_index.rs の関数 adr_rows）の最後に、既存の折りたたみ details.note（名札 番号と見出しの一覧（<本数> 本）・名札は型付きの定数）の中に既存の ul.basis で 1 本 1 行・id の数の昇順・番号のリンク（adr-n.html）と見出しの逐語（切らない）を出す。カード全体の当たり判定 sc-hit は様式の定義の意図どおりに効けば折りたたみと一覧を覆うので（今は sc-row の中のリンクの規則に負けて効いていない）、判断の記録のカードからだけ外す。1 行目と 2 行目の番号の列と 開く → はそのまま。部品・class・様式の定義・ほかのカード・設計文書は変えない。入口の凍結 anchor 2 本（expected-index.html・expected-index-sheet.html）の 1 か所ずつを手で直し、face_index_shelf の既存の歯 1 本の当たり判定の字を一覧の行に替える。歯は face_index の f139_ 4 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main 021b7a4（便 138 の着地の後）・受付の時点の main で数え直す"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_index.rs", "crates/folio/src/face_index_read.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/face_index_shelf.rs", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --test face_index f139_", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face_index_shelf", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_index の f139_ の 4 本（記録 2 本の写しで折りたたみがカードに 1 つだけ在り 名札 番号と見出しの一覧（2 本）と ADR-2・ADR-10 の行が数の順に見出しの逐語で並び更新の行の後でカードの最後に在る・山括弧と and 記号と区切りの字を持つ見出しが escape した逐語で切られずに出る・判断の記録のカードに当たり判定が無く 2 行目の番号の列と 開く →（最新の記録）が今の形で残りほかの 3 枚のカードの当たり判定は 3 つ・実の正本の全部の判断の記録の見出しが id の数の昇順で逐語に並び行の数が記録の本数と等しい）が緑、face_index の歯の全部（入口の凍結の面 2 本との byte 一致・census・parts --check を含む）が緑、face_index_shelf の歯の全部（記録 1 本のカードが一覧の行を持ち当たり判定を持たないことを含む）が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで違う file は index.html の 1 枚だけ"
<!-- contracts:end -->
