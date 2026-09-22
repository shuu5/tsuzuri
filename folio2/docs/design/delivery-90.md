# 設計: 便 90 — 要件書の項に判断の記録だけを受ける欄 adrs を足し、受け皿が無くて除外した 13 対を書き写す（FR19 / NFR3）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から導出する）/ NFR3（参照は必ずつながる）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.2（生成物を手で直さない）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）
- 出所: 判断の記録 ADR-13 決定 (3-b)（ア）。設計ノート docs/design/graph-and-incremental-ceiling.md §8 の便の列の 1 本目（G0-a1）。後付けの設計ノート docs/design/edge-retrofit-2026-09-22.md §4.1 の 1 が「受け皿の欄が無い」で落ちた 14 対を席へ返しており、独立の検証役はそのうち要件書の側だけを先に、判断の記録つきで出すことを推している。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cm が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規の file は無い・縮む file も無い）。
- 門: 本便は design-intent の下の正本（srs.yaml）を書き換えるので天井の門の対象である。席が実測した `folio ceiling --gate` は 2（まだ分からない・印が古い）を返す。持ち主の裁定 D-12（2026-09-21 23:45 JST「推奨で進めて」）により門の外で受ける。

## 1. 目的と中身

要件書の項は、どの判断の記録から生まれたかを注の散文でしか言えない。指す先を型付きの欄に持たないので、機械は「この判断を直したら、その判断が生んだ要件を読み直せ」と言えない。本便は要件の行に判断の記録だけを受ける欄 adrs を足し、後付けの 1 回目が受け皿の無さで除外した対のうち 13 対を書き写す（席の裁定で 1 対を外した・(b)）。欄の決まりの正本は実装の型付きの定数（P-5.6）で、設計文書の側の生成区間へは `folio schema --write` が導出する。欄は増やすが、**何でも受ける汎用の欄にはしない**（判断の記録の id だけに閉じる）。

### (a) 実測（2026-09-22・main 3d499b1）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = check.rs 795・余地 **705**／face_srs.rs 1398・余地 **102**／adr.rs 1243・余地 257／link.rs 657・余地 843。歯 = tests/check.rs 862・余地 **638**／tests/schema_docs.rs 1029・余地 **471**。本便が触る src は check.rs の 1 本だけである。
- 要件の行の欄の閉じた一覧の正本は crates/folio/src/check.rs の定数 5 本。SRS_ITEM_TEXT 7 語・SRS_ITEM_LIST 3 語・SRS_ITEM_OPTIONAL 3 語（126 行）・SRS_ITEM_VERIFY_TEXT 2 語・SRS_ITEM_VERIFY_LIST 1 語。同じ file の SRS_FLOOR（71 行〜）の requirement_row の群がその 5 本を木として持ち、`folio schema` はこの木を srs.yaml の生成区間へ導出する。
- 欄を数える場所は check_srs_item（668 行〜）。行の鍵を 4 本の一覧（TEXT・LIST・OPTIONAL・VERIFY）で閉じ、どれにも当たらない鍵は種別 未知の欄 で落とす。OPTIONAL の欄は在ってもよい欄で、**中身は今どれも見ていない**（rules も同じ）。
- 要件書の basis の欄は条の id だけを受ける。面の生成器 face_srs.rs の 1029 行が各項を ctx.article で条として解き、制約の行も同じ関数（1149 行）を通る。条以外を書くと面が「まだ分からない」に落ちる（後付けの 1 回目の実測・歯 29 本が赤になった）。だから basis を広げず、別の欄を足す。
- **判断の記録の id の形を解く口は既に在る。** adr.rs の is_basis_id（1093 行・pub(crate)）が 条・要件・規則の表の行・判断の記録 の 4 形の全体一致を返し、要件書の図の行の refs（check.rs 744 行）がすでにこの口を呼んでいる。ADR- で始まる字については、is_basis_id はほかの 3 形の接頭辞（P- / A- / N- ／ FR・NFR・AC・CON・GOAL ／ R- / D-）のどれにも当たらないので、同じ file の is_adr_id（1087 行）と同値である。**ADR- で始まることを前置すれば、既に在る口だけで判断の記録に狭まる。adr.rs は 1 字も触らない。**
- **実在の突き合わせも既に在る。** link.rs の references（308 行〜）が srs.yaml の節を全部歩き（population の絞りは無い）、scan_adr_ids（276 行〜）が拾った ADR-n が adr/ の記録に無ければ種別 A-2 の違反を出す。walk（257 行〜）は一覧の中へも降りるので、adrs の一覧の項もこの網の内側に入る（席が当てて実測した。存在しない id を 1 つ書くと違反 1 件・終了コード 1）。**link.rs も 1 字も触らない。**
- 生成区間の実測（席が当てて測り、戻した）。閉じた一覧に 1 語足すと srs.yaml の生成区間は **29 行のまま 1168 byte から 1174 byte**（+6）になり、変わる行は optional の 1 行だけである。要約値は 3937340f77713b087b33ca43b8fe866ca310b97cd04e5e1f4ce9967e19b724e1 から **7fc1ed83deb5193ffffa57061707b7d912919b51e2ff542d17a97fb2f026ef59** に変わる。ほかの 7 file の生成区間は 1 byte も動かない（`folio schema --write` が 変わらない を 7 行、書いた を 1 行出す）。
- 生成区間の凍結の定数は 便 89 で tests/schema_docs.rs へ移っており、srs.yaml の側の値は 2 か所に在る。F77_REGIONS（72 行〜）の 1 組目の byte 数と要約値、および F86_SRS_BYTES（898 行）と F86_SRS_SHA256（899 行）。凍結 anchor は tests/fixtures/schema/srs-region.txt（29 行・1168 byte）。
- 同じ file の f86_region_lists_every_group_of_the_row（949 行の組）は、生成区間の 4 群の語数を [7, 3, 3, 2, 1] で凍結し、**さらに正本の要件の行に実際に現れる欄の和集合が生成区間の 4 群と一致すること**を見る。したがって閉じた一覧に adrs を足すだけでは落ち、**少なくとも 1 行が実際に adrs を持って初めて緑になる**（本便は 13 行に書くので満たす）。
- tests/check.rs の F86_REGION_OPTIONAL（829 行）は、生成区間の optional の行を写しの側の変異の当て先として持つ。生成区間が変わるので同じ字に直す（この定数を使う歯 f86_unknown_field_cannot_be_loosened_from_the_file は、file の側で一覧を緩めても閉じた一覧は緩まないことを見る歯で、見るものは変わらない）。
- 土台の写し tests/fixtures/floor_base/design-intent/srs.yaml と tests/floor_cases.yaml（expected_cases 141）は **1 byte も動かない**。欄が増えるだけで既存の期待が動かないことを、席が当てた木で 141 件を回して実測した（141 件とも期待どおり）。
- id の一覧の凍結 anchor design-intent/anchors/ids-v1.24.yaml は、projection.fields のとおり shall と title の要約値だけを持つ。adrs はその写しの外なので **1 byte も動かない**（当てた木で `folio check` が 違反 0・まだ分からない 0）。
- 面の歯と面の凍結の写し（tests/fixtures/face/srs.yaml・tests/fixtures/face/expected-srs.html）も動かない。当てた木で workspace 全体の nextest が 708 件すべて緑だった（本便の前の 704 件 ＋ f90_ の 4 本）。落ちたのは (d)(g) が名指す 6 本だけで、どれも凍結の値と変異の当て先の字である。
- 歯の関数名の接頭辞。`grep -rn 'fn f90_' crates/folio/tests` は今 0 本（使われている最大は f89_）。

### (b) 足す欄と、書き写す 13 対

欄の名は **adrs**。要件の行（requirements と nonfunctional）の**任意**の欄で、値は判断の記録の id の一覧である。名を adrs とし、何でも受ける汎用の欄（refs）にしないのは、根拠の欄との使い分けが人の判断になると、機械で数えられる形に閉じるという向き（条 N-2）から遠ざかるからである（判断の記録 ADR-13 決定 (3-b)（ア））。

書き写す 13 対は次のとおり。出所はどれも、その要件の注の中の判断の記録の言及である。設計ノート §4.1 の 1 が挙げた 14 対のうち 1 対（NFR2 → ADR-7）は下の理由で外す。

| 要件 | adrs | 注の中の出所 |
| --- | --- | --- |
| FR9 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (1)」 |
| FR10 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (2)・(3)」 |
| FR11 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (4)」 |
| FR12 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (7)」 |
| FR13 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (5)・(6)」 |
| FR14 | ADR-3 | 注の 1 文目「判断の記録 ADR-3 決定 (9)」 |
| FR15 | ADR-4 | 注の 1 文目「判断の記録 ADR-4」 |
| FR16 | ADR-5 | 注の「生成器は手書きのまま（判断の記録 ADR-5 決定 (3)…）」 |
| FR17 | ADR-8 | 注の 1 文目「判断の記録 ADR-8 決定 (1)(2)(3)」 |
| FR18 | ADR-8 | 注の 1 文目「判断の記録 ADR-8 決定 (3)(4)(5)」 |
| FR19 | ADR-9 | 注の 1 文目「判断の記録 ADR-9・ADR-11 決定 (4)①」 |
| FR20 | ADR-8 | 注の「門の形の裁定と撤退条件は判断の記録 ADR-8 の注が持つ」 |
| NFR1 | ADR-4 | 注の「R-1 の母集団に入れない（判断の記録 ADR-4 決定 (5)）」 |

**この 13 対はどちら向きの辺も持たない。** 席は相手の判断の記録の basis を全部読み、13 とも要件の側の id を持っていないことを実測した（2026-09-22）。

書く場所は行の basis の直後（rules を持つ行では rules の直後）で、1 行 1 欄とする。欄の値の順は表のとおりで、13 行とも 1 id である。

**除外 1 対（NFR2 → ADR-7）。** この対の唯一の出所は NFR2 の注の末尾の「判断の記録 ADR-7 の案 c の理由と同じ見立て」で、後付けの設計ノート docs/design/edge-retrofit-2026-09-22.md §3.3 の 3（先例の引き合い＝ X と同じ運び方・X と同形にする）と同じ種別である。後付けの 1 回目はこの種別を辺にしないと決め、実際に 11 対を §4 の E6 で外している。**同じ種別を片方だけ辺にしない**ので、本便も外す。ほかの 13 対は「この要件はその判断から生まれた」か「その判断が要件の範囲を切っている」形で、種別が違う。判断の記録 ADR-13 決定 (3-b)（ア）が言う **14 という数は起草時の実測の値**であり、その出所である設計ノート §4.1 の 1 は「どの要件も注の 1 文目が 判断の記録 ADR-n 決定 (k) で始まる」と述べているが、NFR2 の注はその形ではない（席が字面を読み直して実測）。**本便の確定値は 13 対である。** 数の差 1 は判断の記録の改訂を要しない（決定 (3-b) が定めるのは欄を足すことと受ける相手を判断の記録に閉じることで、対の数はその根拠の実測値である）。席の裁定 2026-09-22。

**逆向きの辺が既に在る 5 対も書かない。** FR1 → ADR-6・FR7 → ADR-12・FR16 → ADR-7・FR19 → ADR-11・NFR3 → ADR-3 は、相手の判断の記録の basis に要件の側の id が既に在る（席の実測）。判断の記録 ADR-13 決定 (2) が伝播を両向きと定めているので、逆向きが在る対に順向きの辺を重ねても近傍は 1 つも広がらない。FR16 と FR19 が 2 本目の id を持たないのはこの理由である。

### (c) 床の側の判定（check.rs だけを触る）

1. SRS_ITEM_OPTIONAL を 3 語から 4 語へ増やし、milestone と rules の間ではなく **rules と note の間に adrs を置く**（id の一覧を持つ任意の欄を並べる）。生成区間の optional の行はこの順で出る。
2. check_srs_item の末尾に、adrs の中身を見る枝を 1 本足す。欄が無い・null なら何もしない。一覧なら各項を見て、**ADR- で始まり、かつ adr::is_basis_id が真** でなければ種別 schema の違反を 1 件出す（(a) のとおり、この 2 つの組は is_adr_id と同値である）。一覧でなければ種別 schema の違反を 1 件出す。これは要件書の図の行の refs（744 行）と同じ形で、受ける id を判断の記録だけに狭めた枝である。
3. **実在は数えない。** (a) のとおり link.rs の references が srs.yaml を全部歩いて種別 A-2 で数えるので、ここで重ねると 1 つの誤りが 2 件になる。

新しい定数・新しい旗・新しい module は持たない。例外の口（無効化の旗・今回だけの口）も持たない（N-3.1）。

### (d) 生成区間と凍結 anchor

`folio schema --write` が srs.yaml の生成区間を 1174 byte に書き直す。`schema.rs` の TARGETS は既に srs.yaml（check::SRS_FLOOR）を持つので **schema.rs も main.rs も 1 行も触らない**。要件 FR19 の対象も増えない。

凍結 anchor tests/fixtures/schema/srs-region.txt は、生成器にも検査側にも依らずに組む（P-10.2）。組み方は、いまの anchor の optional の 1 行だけを新しい字に替えることで、OS の道具だけで足りる。席はその手順で組んだ写しが `folio schema --write` の出力と 1 byte 違わないことを実測し、byte 数（1174）と要約値（7fc1ed83deb5193ffffa57061707b7d912919b51e2ff542d17a97fb2f026ef59）を OS の道具 sha256sum で独立に出した（2026-09-22）。**本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**

### (e) 面の側 — 本便では出さない

要件書の面に adrs の chip を出すのは **本便では運ばない**。face_srs.rs の余地は **102** で、器の受付は size S の見積（100 行）をぎりぎり通すだけである。chip を 1 群足すと余地は 90 前後になり、次に面を直す便がどの大きさでも受けられなくなる。設計ノート docs/design/graph-and-incremental-ceiling.md §8 の G0-a1 は「収まらなければ面へ出すのを G7b（file を割る）の後に回す」と先に断っており、本便はその断りに従う。

面へ出さないので、面の生成器は adrs を読まない。面は自分が読む欄だけを描くので、知らない欄が 1 つ増えても落ちない（席が当てた木で、面の歯・面の凍結の写し・parts の検査を含む nextest 708 件が全部緑）。要件と判断の記録の行き来は、判断の記録の面の側の根拠の章から今までどおり辿れる。

### (f) 要件書の版

meta.version を v1.27 に上げ、effective_version は v1.26 のままにする（承認待ちの起草）。status_note の末尾に 1 件足し、meta.approval に 作成 の行だけを足す（承認の行は持ち主の承認の後）。**要件の規範文・平易文・確かめ方・受入の判定の式は 1 字も触らない。** 動くのは 13 行の adrs の欄と meta の 3 か所と生成区間の 1 行だけである。

### (g) 歯（関数名は f90_ で始める・置き場は crates/folio/tests/check.rs）

tests/check.rs の Work は実の design-intent を一時 dir へ写し、器の導出 file を根に置き、git init と 1 commit を行う。本便の歯はその写しを使う（新しい歯の file も新しい dir も作らない）。

1. f90_the_real_srs_carries_the_adrs_field — 実の要件書の写しで素の床が 終了コード 0・違反 0。写しの srs.yaml の adrs の行がちょうど **13 本**で、各項が ADR- の後に 1〜9 で始まる数字列だけの形であり、その id の正本 adr/&lt;id&gt;.yaml が写しに在る。本便の前の main には adrs の行が 1 本も無いので **赤い歯**。
2. f90_an_id_that_is_not_an_adr_is_a_violation — 写しの FR19 の行の adrs の値を FR5 に替えて素の床 → 終了コード 1・違反 1 件で、行に FR19 と adrs と FR5 と 判断の記録の id の形でない が出る。本便の前の main には変異の当て先が無いので **赤い歯**。
3. f90_an_adr_that_does_not_exist_is_a_violation — 同じ行の値を実在しない番号に替えて素の床 → 終了コード 1・違反 1 件で、行に adrs と その番号が実在しない が出る（(a) のとおり link.rs の網が受け持つことを歯で押さえる）。同じ理由で **赤い歯**。
4. f90_adrs_that_is_not_a_list_is_a_violation — 同じ行を一覧でない字に替えて素の床 → 終了コード 1・違反 1 件で、行に FR19 と adrs が一覧でない が出る。同じ理由で **赤い歯**。

変異の当て先は FR19 の行の adrs の 1 行とする。実の要件書でこの値を持つ行は 1 本だけで、tests/check.rs の変異の口は当て先がちょうど 1 か所であることを自分で確かめる（当て先が 1 か所でなければ歯が落ちる）。

回帰は verify の 2 行目で見る。tests/check.rs（本便の前の 37 本 ＋ f90_ の 4 本 = 41 本）と tests/schema_docs.rs（23 本・本数は変わらない）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない。

### (h) 大きさ

src は crates/folio/src/check.rs の 1 本だけ（795・余地 **705**・+28 行の見込み）。歯は crates/folio/tests/check.rs（862・余地 638・+63 行）と crates/folio/tests/schema_docs.rs（1029・余地 471・**行数は不変**で凍結の 4 つの値だけが変わる）。凍結 anchor tests/fixtures/schema/srs-region.txt は 29 行のまま 1 行の字が変わる。size **S**（触る src の余地は 705 で、S の見積 100 を大きく上回る）。新しい file も新しい dir も無く、縮む file も無い。外部 crate は増やさない。

### (i) 本便が運ばないもの・撤退条件

- 面の側の表示（(e)）。file を割る便の後に回す。
- 規則の表の行が条以外を指す欄（33 対）と、判断の記録が生んだものを記す帰結の欄（12 対）。判断の記録 ADR-13 決定 (3-b) の（イ）（ウ）で、行 R-4 の双方向の式と母集団に触るので別便（G0-a2）。
- 要件の項がほかの要件・制約を指す欄（12 対）。判断の記録 ADR-13 決定 (3-b) が本便の後の数えを見てから決めると保留している。
- 規則の表の行 R-17 を機械が数える歯（G0'）。受け皿の無い対が残っているあいだに歯を入れると床が落ち続けるので、（イ）（ウ）の後である。
- 憲法の条文・規則の表の行・語彙・判断の記録の本文・ほかの 7 file の生成区間・face_srs.rs・adr.rs・link.rs・schema.rs・main.rs・土台の写し・tests/floor_cases.yaml・id の一覧の凍結 anchor・CI の yml。
- 撤退条件: adrs の欄が機械で数えられる形に閉じていられなくなったとき（受ける相手を判断の記録の外へ広げたい対が出て、欄の意味が人の判断に戻るとき）は、(c) 2 の枝を check.rs から外し、欄を SRS_ITEM_OPTIONAL から落として 13 行を消す。便 1 本で戻せる。欄を残したまま受ける相手を広げる形は取らない（広げた瞬間に basis と同じ使い分けの問題が戻るため）。

## 2. 範囲

- 入れる: 閉じた一覧に 1 語・床の枝 1 本・要件書の 13 行と meta の 3 か所・生成区間の 1 行・凍結 anchor 1 行・凍結の定数 4 つと変異の当て先 2 つの直し・f90_ の歯 4 本。
- 入れない: 面の側の表示・規則の表と判断の記録の欄・要件どうしの欄・行 R-17 の歯・憲法と規則の表と語彙・要件の規範文と平易文と受入の判定の式・adr.rs と link.rs と face_srs.rs と schema.rs と main.rs・土台の写しと tests/floor_cases.yaml・id の一覧の凍結 anchor・新しい file と新しい dir。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| field | 新しい欄 | 要件の行の adrs（判断の記録の id だけを受ける任意の一覧） |
| floor | 床の枝 | crates/folio/src/check.rs の閉じた一覧 1 語と check_srs_item の枝 1 本 |
| rows | 書き写し | design-intent/srs.yaml の 13 行と meta（版・来歴・作成の行） |
| region | 生成区間 | srs.yaml の生成区間の optional の 1 行（folio schema --write が書く） |
| anchor | 凍結 anchor | tests/fixtures/schema/srs-region.txt（29 行・1174 byte） |
| pins | 凍結の定数 | tests/schema_docs.rs の 4 つの値と tests/check.rs の変異の当て先 |
| teeth | 歯 | crates/folio/tests/check.rs の f90_ 4 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は .vessel.toml の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cm"
title = "要件書の項（requirements と nonfunctional）に判断の記録だけを受ける任意の欄 adrs を足し、後付けの 1 回目が受け皿の無さで除外した 13 対を書き写す。欄の決まりの正本は実装の型付きの定数（check.rs の閉じた一覧）で、srs.yaml の生成区間へ folio schema --write で導出する。床は id の形（ADR- で始まり adr::is_basis_id が真）と一覧であることを数え、実在は既に在る link.rs の網に任せる。面には当面出さない（face_srs.rs の余地 102）"
req = ["FR19", "NFR3"]
section = "1"
write-set = ["crates/folio/src/check.rs", "design-intent/srs.yaml", "tests/fixtures/schema/srs-region.txt", "crates/folio/tests/check.rs", "crates/folio/tests/schema_docs.rs"]
verify = ["cargo nextest run -p folio --test check f90_", "cargo nextest run -p folio --test check --test schema_docs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f90_ の歯 4 本（実の要件書に adrs の行が 13 本在り各 id の正本が実在して床が終了コード 0・判断の記録でない id で違反 1 件・実在しない番号で違反 1 件・一覧でない値で違反 1 件）が全部緑、tests/check.rs と tests/schema_docs.rs の既存の歯が全部緑（生成区間 29 行 1174 byte と凍結 anchor が byte 一致し、生成区間の 4 群の語数が 7 / 3 / 4 / 2 / 1 で正本の要件の行に現れる欄の和集合と一致する）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
