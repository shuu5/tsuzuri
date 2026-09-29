# 設計: 便 128 — 憲法の meta・前文・条・規範文・mechanism と規則の表の行の未知の欄と、mechanism の形の崩れを床で落とす（天井の 29 周目 実態 F-1・台帳 f2-648.176・憲法 N-3 の機構の注）

- 要件: FR5（構造の床を実行し 3 値で返す・要件書 第 1.37 版）。FR9 の規範文は、設計ノートの正本の形（重複キー・未知の欄・欄の非空）を構造の床（FR5）で数えると書く。本便は同じ要件 FR5 の床を、憲法と規則の表に広げる。規範文はどれも変えない。FR19（生成区間への写し）は req に入れない。本便は生成区間を 1 byte も変えないからである（(b) の 生成区間は動かない）。
- 条: N-3.1（規則の例外機構〔無効化の旗・「今回だけ」の口〕を足す変更を拒む＝行や条へ旗の欄を足す口と、機構を旗の字に置き換える口を塞ぐ）/ P-4.1（実行できなかった検査を異常なしとして扱わない＝未知の欄と形の崩れを黙って読み捨てない）/ P-5.1・P-6.3・P-6.4（閉じた一覧は型付きデータに 1 つだけ置き、手書きの写しを持たない）/ P-5.6（実装の型付きの定数が正本なら写しを生成区間へ導出する）。
- 出所: 天井の 29 周目の実態の所見 **F-1**（重さ 止める・反証で支持・所見は持ち主の home の下の `.local/share/folio2/ceiling/2026-09-24-round29/` の実態の findings.yaml、反証の理由は同じ dir の refute-notes の reality-F-1.md）。憲法 N-3 の機構の注（kind build-check・live now・polarity fail-closed）は「正本と rules の全節（top_level・meta・前文・条・規範文・mechanism・rules 行）で未知の欄を schema 検査が落とす（床は folio check）」と約束するが、床は最上位の節しか数えない。台帳 **f2-648.176**（memo・2026-09-24）の観測「規則の表の開発規律行に stage を書いても床は黙って落とす」も同じ根である。判断の記録 ADR-11 の注 (ア)（2026-09-21）も、規則の表の行の欄の定数を床が読んでいないことを記録している。独立の検証役（2026-09-24）は、条の mechanism を表でない字（無効）にしても床が合格を返すことを実測した。規則の表の開発規律行 D-11 は、この行と契約表の行の title が出所（29 周目の実態 F-1・台帳 f2-648.176・判断の記録 ADR-11 決定 (3)）を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dz` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 8 本。**新しい file は 1 本で、先頭に `+` を付けて宣言する**（歯の file 1・既に在る dir `crates/folio/tests/` に置く）。書き換える 7 本は印なし。**縮む file も消す file も無く、新しい dir も作らない。** 歯の runner `crates/folio/tests/floor_cases.rs` は本文を変えないが、verify が `--test floor_cases` で名指すので write-set に入れる。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 8 本をそのまま門に渡した実測は **0（通す・断りの字 = 設計文書の正本を書き換えない便）**（2026-09-24・本体の作業ツリーの binary・§1 (h) の 8）。印は 29 周目の不合格のものだが、対象外の便なので門の判定は印に依らない（規則の表の開発規律行 D-12 と、2026-09-22 の席の前例を当てる場面は無い）。
- 前の便: 前提の便は無い。**base = main ee714d7（便 125 の着地・骨格の命令 folio init）。この契約の数はすべて base ee714d7 の実測（参考値）である**（規則の表の行 D-13）。枝は main 4eb2d2d（便 126 の契約・`docs/design/delivery-126.md` の 1 file だけ）の上に積んだ。ee714d7 から 4eb2d2d までの差分は `crates/`・`tests/`・`design-intent/` を 1 byte も動かさない。
- 改訂: 改訂 b（2026-09-24・席の連絡）= base を 3b770a1 から ee714d7 に取り直した。数は変わらなかった。改訂 c（2026-09-24・独立の検証 blocking 1・文面 6・低い重さ 3 と席の裁定）= 範囲に mechanism と meta・前文の形の崩れ（表でない・mechanism の必須の欄の欠け）を足した（(b) の 形の崩れ・歯 5・床の場合 1 件）。組み立ての導出の Err の 4 つの形の歯を足した（歯 6・write-set に `crates/folio/tests/constitution_enums.rs`）。§1 の文面を 6 か所直した（N-3 の注と実態の合い方・数えない節・ほかの床との境界・ADR-11 の注 (ア)・利用者の置き場・FR9 の引用）。新しい関数の名を closed_fields に改め、歯の変異の値の決まりを足した。write-set は 7 本から 8 本、歯は 4 本から 6 本、床の場合は 2 件から 3 件、verify は 3 行から 4 行になった。size は S のままである。
- 並行の便との重なり: 起草の時点で契約が main に在り未着地の便（便 126・行 `dy`）の write-set は、gate.rs・stamp.rs・ceiling.rs ほかと天井の正本の fixture で、本便の 8 本と 1 本も重ならない。便 125 の骨格（folio init）が書く憲法と規則の表は、本便の閉じた一覧の内に収まり、形も崩れていない（(a) の 6）。

## 1. 設計

### (a) いま起きていること（実測・base main ee714d7）

1. **床が未知の欄を数えるのは最上位の節だけである。** `crates/folio/src/check.rs` の check_constitution は、最上位の節を置き場の憲法の schema.top_level で閉じる（unknown_sections・種別 未知の節）。そのうえで前文・条・規範文の必須の欄の一部の非空と、条の値域の欄の値を数える。meta・前文・条・規範文・mechanism の中の欄の名は数えない。check_rules は最上位の節を床の定数 RULES_TOP_LEVEL で閉じ、行の id・article・what の非空と refs だけを数える。行の欄の閉じた一覧の定数（`crates/folio/src/rules.rs` の閾値行と開発規律行の required と optional の 4 本）を読むのは床の木（`folio schema` の導出）と歯だけで、床は読まない。
2. **だから未知の欄は黙って通る。** 起草役が design-intent の写しに欄を 1 つずつ足して `folio check` を撃った。写しは器の導出 file の写しを親の contracts/ に置き、git の 1 commit にした形である（script は (h) の 1）。

| 足した欄 | 終了コード | 床の出力 |
| --- | --- | --- |
| なし（元の写し） | 0 | 合格（違反 0・まだ分からない 0） |
| meta に bogus_meta | 0 | 合格（元の写しと同じ） |
| 条 N-3 に bogus_a | 0 | 合格 |
| 条 N-3 の mechanism に bogus_m | 0 | 合格 |
| 規範文 N-3.1 に bogus_s | 0 | 合格 |
| 規則の表の行 R-2 に bogus_r | 0 | 合格 |
| 開発規律行 D-5 に disabled: true（例外の旗の形） | 0 | 合格 |
| 開発規律行 D-14 に stage: post（台帳 f2-648.176 の形） | 0 | 合格 |
| 前文に bogus_p | 1 | 改憲の違反（種別 N-4）1 件だけ。未知の欄は数えない |
| 前文の mechanism に bogus_pm | 1 | 改憲の違反 1 件だけ |
| 憲法の schema.meta.optional に bogus_meta を足す | 1 | 改憲の違反 1 件だけ |

   前文は改訂の差分の範囲（amendment_scope の precedence）が節ごと比べるので、欄を足すと改憲の違反が立つ。条・規範文・mechanism は、差分が比べる条の欄（id・title・statements・tier・binds）に足した欄が入らないので、欄を足しても立たない（規範文の行に足した欄も入らない・実測）。
3. **mechanism の形が崩れても黙って通る。** 同じ写しで条 N-3 の mechanism の値を置き換えた（script は (h) の 1 の改訂 c の分）。

| 条 N-3 の mechanism の値 | 終了コード | 床の出力 |
| --- | --- | --- |
| 字 無効 | 0 | 合格 |
| 一覧 [disabled] | 0 | 合格 |
| null | 0 | 合格 |
| 空の表 {} | 0 | 合格 |
| kind を欠いた表 | 0 | 合格 |

   条の mechanism は改訂の差分の範囲の外で、非空の検査も mechanism を見ない。check_article_enums も、表でなければ黙る。これは N-3.1 の「無効化の旗」の形そのもので、N-3 の注が名指す mechanism の上に在る。meta を一覧にした写しは、ほかの検査（参照 id・版の綴り・承認の写しの突き合わせ）が違反と「まだ分からない」を出す。規範文の行と規則の表の行が表でない形は、行の一覧の読み（check.rs の rows）が「まだ分からない」を出す。前文の mechanism の崩れは改憲の違反だけが拾う（独立の検証役の実測）。
4. **閉じた一覧はもう在る。** 憲法の正本の schema 節は、meta・precedence・article・mechanism・statement の 5 つの部位ごとに required と optional の一覧を宣言している（statement は required だけで optional の欄を持たない）。判断の記録 ADR-11 の決定 (3) の最後の文は「(ア) と (イ) の両方が当たる憲法自身の欄の決まりの節は、A-2 の門が在るので (ア)（file が正本）とする」と決めた。(ア) は、実装は実行時に読むか組み立て時に導出し、手書きの写しを持たないとする。規則の表の行の一覧は、決定 (3)(イ) と (4)② のとおり実装の型付きの定数が正本で、写しは既に規則の表の生成区間に在る（生成区間の threshold_row と discipline_row の required と optional・便 53 から）。
5. **実のデータは宣言した一覧の内に収まっている。** 起草役が実の design-intent を数えた（script は (h) の 2）。宣言した欄はどの部位でも 1 回以上使われていて、宣言の外の欄は 0・表でない holder は 0・必須の欄の欠けは 0 である。

| 部位 | 行の数（参考値） | 使われている欄 |
| --- | --- | --- |
| meta | 1 | required の 10 と optional の 2（changes_from_v0_1・changes_from_v0_2） |
| 前文と前文の mechanism | 1 | required の 5・mechanism は kind・live・note |
| 条 | 27 | required の 8・optional の relations・retreat・amended_by・supersedes_v1・note |
| 条の mechanism | 27 | kind・live・note・stage・polarity |
| 規範文 | 67 | id・pattern・strength・text |
| 閾値行 | 17 | required の 9・optional の basis・projection・same_failure・population・note・refs |
| 開発規律行 | 14 | required の 7・optional の note・refs |

6. **道具の fixture と骨格も収まっている。** folio2 の憲法の一覧を当てて、`tests/fixtures/` の下の憲法か規則の表を持つ置き場 25 本と、便 125 の骨格（ee714d7 の binary の `folio init` が書いた置き場）を数えた（script は (h) の 2）。25 本の内訳は、schema 節に 5 部位の一覧を持たない 23 本〔4 file 形の fixture と天井の束の材料〕・凍結の土台・規則の表だけを持つ check/missing-file である。どれも、宣言の外の欄は 0、表でない meta・前文・mechanism は 0、表の mechanism の kind と live の欠けは 0 である。その 23 本の憲法の schema 節は最上位の節と値域だけを持ち、5 部位の一覧を持たない。床の凍結の場合（`tests/floor_cases.yaml`）が憲法と規則の表へ足す欄（add の変異）は amended_by・statements の行・条 P-20 の欄・行 R-17 の欄で、どれも一覧の内である。
7. **必須の欄の全部を数えると fixture が一斉に落ちる。** 同じ 25 本に、5 部位と行の required の全部の有無を当てると、24 本に欠けが在る（1 本あたり参考値 4〜35 件・meta の approval・条の binds と mechanism・規範文の pattern・閾値行の stage ほか）。欠けが 0 なのは凍結の土台と骨格と実の design-intent だけである（script は (h) の 3）。

### (b) 直す先 — 閉じた一覧で未知の欄を落とし、mechanism の形の崩れを落とす

**憲法の 5 部位の一覧は、組み立て時に憲法の正本から導出する。** `crates/folio/build.rs` に新しい純粋な関数 constitution_fields を足す。憲法の正本の字を受け、schema の meta・precedence・article・mechanism・statement の 5 部位（この 5 つの名だけは導出の側が書く）について、2 つの列を返す。1 つは required の列、もう 1 つは required と optional を file の順に繋いだ閉じた列である。出力は、部位の名と 2 つの列の組を 5 つ持つ定数 1 つの Rust の source で、main が値域の導出（constitution_enums）の source の後ろに繋いで、同じ OUT_DIR の file に書く。`crates/folio/src/constitution_enums.rs` はその file を今のまま取り込むので、取り込みの行は変わらない。optional が無い部位（statement）は空の列とする。一覧を狭める側に倒れるので黙る口にはならない。次のどれかなら Err にして、組み立てを失敗させる（値域の導出と同じ形・黙って空の一覧にしない）。Err の理由の文には部位の名を入れる。

- 部位が表でない。
- required が字の一覧でない。
- optional が在って字の一覧でない。
- 同じ部位の中に同じ名が 2 度在る。

憲法の字は変えない。憲法の一覧を変える改訂（部位に欄を足す・外す）は改憲の手続き（A-2・N-4）を通り、組み立て直せば床の一覧も同じ字で動く。手書きの写しは置かない（P-6.4）。

**規則の表の行の一覧は、今の型付きの定数を床が読む。** 閾値行は `crates/folio/src/rules.rs` の閾値行の required と optional を繋いだ列、開発規律行は開発規律行の required と optional を繋いだ列で閉じる。file の生成区間の threshold_row と discipline_row は床の定数の写しとして読まない。file の側で一覧に欄を足して通す口を塞ぐためである（N-3.1・最上位の節を RULES_TOP_LEVEL で閉じた便 51 と同じ向き）。

**未知の欄を数える段。** `crates/folio/src/check.rs` に新しい非公開の関数 closed_fields を 1 つ足す（file 名・場所の字・欄の表・閉じた一覧を受け、表の鍵のうち一覧に無いものを 1 つにつき違反 1 件）。種別は要件書の項の床（check_srs_item）と同じ 未知の欄 で、字も同じ形「file: 場所 の未知の欄「鍵」」にする。設計ノートの床（`crates/folio/src/note.rs`）にも同じ字の形の非公開の関数 unknown_fields が在るが、欄の一覧の型が違い、note.rs は write-set の外なので使い回さない。呼ぶ所は次の 8 か所である。

| holder | 閉じた一覧 | 場所の字（違反の行は `[未知の欄] ` の後にこの行の字が続く） |
| --- | --- | --- |
| 憲法の meta | 導出した meta の閉じた列 | constitution.yaml: meta の未知の欄「鍵」 |
| 憲法の前文 | 導出した precedence の閉じた列 | constitution.yaml: 前文（precedence） の未知の欄「鍵」（非空の検査の場所の字と同じ） |
| 前文の mechanism | 導出した mechanism の閉じた列 | constitution.yaml: 前文（precedence）の mechanism の未知の欄「鍵」 |
| 条 X | 導出した article の閉じた列 | constitution.yaml: 条 X の未知の欄「鍵」 |
| 条 X の mechanism | 導出した mechanism の閉じた列 | constitution.yaml: 条 X の mechanism の未知の欄「鍵」 |
| 条 X の規範文 Y | 導出した statement の閉じた列 | constitution.yaml: 条 X の規範文 Y の未知の欄「鍵」 |
| 閾値行 R-n | 閾値行の列 | rules.yaml: 行 R-n の未知の欄「鍵」 |
| 開発規律行 D-n | 開発規律行の列 | rules.yaml: 行 D-n の未知の欄「鍵」 |

- 一覧は holder ごとで、部位の和集合にしない。条には在ってよい note は規範文では未知の欄、閾値行の value・stage は開発規律行では未知の欄である（開発規律行の stage が台帳 f2-648.176 の形）。
- 置き場の憲法の schema 節の一覧は読まない。道具を組み立てた folio2 の憲法の一覧で閉じる。置き場の schema 節に欄を足しても床の一覧は広がらない（歯 3）。値域を組み立てた版で持つ判断の記録 ADR-16 決定 (2)(ウ) と同じ向きで、置き場ごとに一覧を広げる口は持たない（N-3.1）。
- 例外の旗・既定に倒れる枝・一覧を切り替える設定は持たない。

**形の崩れを数える段（改訂 c・席の裁定）。** 同じ関数の呼び出しの前に、次の 2 つを種別 schema の違反にする。

| 形 | 当てる holder | 字（違反の行は `[schema] ` の後にこの行の字が続く） |
| --- | --- | --- |
| 鍵は在るが値が表でない（字・一覧・数・null） | 憲法の meta・前文・前文の mechanism・条 X の mechanism | constitution.yaml: 条 X の mechanism が表でない（meta・前文（precedence）・前文（precedence）の mechanism も同じ形） |
| 表の mechanism に、導出した mechanism の required の欄（kind・live）が無い | 前文の mechanism・条 X の mechanism | constitution.yaml: 条 X の mechanism の必須の欄 kind が無い（欄 1 つにつき 1 件） |

- 表でない holder の中の欄は数えない。違反 1 件で足りる。
- 規範文の行と規則の表の行が表でない形は、今の行の一覧の読みが「まだ分からない」を出すので、本便は重ねない（(a) の 3）。
- 必須の欄の欠けを数えるのは mechanism だけである。mechanism の kind と live は、機構がそもそも機構である条件（どの仕掛けか・いつ実在するか）で、欠けた表は無効化の旗と同じ働きをする。実の design-intent・fixture 25 本・骨格の欠けは 0 である（(a) の 6）。ほかの部位と規則の表の行の必須の欄の全部は数えない。数えると fixture 24 本が一斉に落ちるからである（(a) の 7・(i) の 2）。
- mechanism の値が空の字（""）の形は、表でない の 1 件になる。kind が空の字の形は、今の値域の検査（check_article_enums）が「憲法の値域に無い」の 1 件を出すので、必須の欄の欠けには数えない（二重に出さない）。

**生成区間は動かない。** 規則の表の行の一覧は、定数の値も床の木（rules.rs の FLOOR）も変えないので、生成区間の写しと凍結 anchor `tests/fixtures/schema/rules-region.txt` は 1 byte も動かない。憲法の一覧は憲法の正本そのものが正本なので、写しを置く生成区間を持たない（P-5.6 が掛かるのは実装の定数が正本のときだけ・値域の導出と同じ扱い）。`rules.rs` の書き換えは頭の注の 1 文だけである（「それまで FLOOR と行の欄の定数の読み手は歯だけ」を「行の欄の定数は床〔check.rs の check_rules〕も読む〔便 128〕」に直す）。

**N-3 の注との合い方。** 本便の着地で、注が列挙する 7 か所の holder そのもの（最上位の節・meta・前文・条・規範文・mechanism・規則の表の行）の欄は、注のとおり未知の欄を床が落とす。mechanism の形の崩れも落とす。ただし注の「全節」を字どおりに読むと、次の 2 つは本便の後も黙る。注の約束と実態が全部合うとは言えない（(i) の 2・§5 の申し送り）。

- 条の中の行（rationale の各行・retreat・relations・amended_by の各行）の欄。
- 最上位の節のうち 7 か所に入らない節（north_star・amendment・sources・rules_pointer・glossary_pointer）の中の欄。

**folio2 自身の結果は変わらない。** (a) の 5 のとおり実のデータは一覧の内で形も崩れていないので、`folio check --dir design-intent` は合格（違反 0・まだ分からない 0）のまま、`folio schema --dir design-intent --check` は全 file 一致のままである。注の字は変えない。

**利用者の置き場への帰結。** 床は folio2 を組み立てた一覧で閉じるので、利用者の憲法が folio2 の一覧の外の欄を使うと違反になる。利用者が A-2 の手続きで自分の schema 節を改めても、床は違反を出し続ける。判断の記録 ADR-16 決定 (2) が列挙する「道具の側に焼き込む所」(ア)〜(キ) に欄の一覧は無く、本便はその列に 1 つ足す形になる。今の M3 の範囲は利用者の憲法の移行を入れず、骨格は一覧の内に収まる（(a) の 6）。利用者の憲法を移すときは、ADR-16 決定 (2)(ウ) と同じく別の判断の記録で扱う（§5）。

### (c) 歯（新しく 6 本）

関数名は f128_ で始める（verify の絞り込みの語・base で `git grep -n 'fn f128_'` は 0 件）。歯 1〜5 は新しい file `crates/folio/tests/` の下の unknown_fields.rs に、歯 6 は既に在る `crates/folio/tests/constitution_enums.rs` に置く。

**歯 1〜5 の土台**は `crates/folio/tests/constitution_range.rs` の Work と同じ形とする。一時 dir に実の `design-intent/` の写しと親の contracts/ に器の導出 file の写し schema.toml を置き、git の 1 commit にしてから、写しの字面に変異を当てて `folio check` を撃つ。当て先はちょうど 1 か所で、一時 dir は消す。歯は標準出力の違反の行（`[` で始まる行）を全部数える。**変異で足す欄の値は、id の形（P-n・R-n・FR-n ほか）を含まない字（x・post・disabled の類）にする。** id の字を含むと、散文の言及（R-17）や参照 id の検査が別の行を出し、「ちょうど N 行」が崩れる。

1. **f128_constitution_unknown_fields_are_violations（憲法の 5 部位）。** 元の写し → 合格（違反 0・まだ分からない 0）・終了コード 0。続けて別々の写し 4 つ = meta に bogus_meta・条 N-3 に bogus_a・条 N-3 の mechanism に bogus_m・規範文 N-3.1 に bogus_s を足す。それぞれ違反の行はちょうど 1 行で、(b) の表の字（例 `[未知の欄] constitution.yaml: 条 N-3 の mechanism の未知の欄「bogus_m」`）と一致し、終了コード 1。さらに別々の写し 2 つ = 前文に bogus_p・前文の mechanism に bogus_pm を足す。それぞれ違反の行はちょうど 2 行で、改憲の違反（種別 N-4）の行と (b) の表の字の行である。**base では 4 つが合格・2 つが改憲の違反だけで落ちる＝RED。**
2. **f128_rule_row_unknown_fields_are_violations（規則の表の行）。** 別々の写し 3 つ = 行 R-2 に bogus_r・行 D-5 に disabled: true・行 D-14 に stage: post を足す。それぞれ違反の行はちょうど 1 行（`[未知の欄] rules.yaml: 行 D-14 の未知の欄「stage」` ほか）・終了コード 1。**base では 3 つとも合格で落ちる＝RED。**
3. **f128_place_schema_cannot_widen_the_lists（置き場の側で一覧を広げても通らない）。** 写し 1 つ目 = 憲法の schema.meta.optional に bogus_meta を足し、meta にも bogus_meta を足す → 違反の行はちょうど 2 行（改憲の違反と `[未知の欄] constitution.yaml: meta の未知の欄「bogus_meta」`）。写し 2 つ目 = 規則の表の生成区間の threshold_row.optional に bogus_r を足し、行 R-2 にも bogus_r を足す → 違反の行はちょうど 1 行（`[未知の欄] rules.yaml: 行 R-2 の未知の欄「bogus_r」`・床は生成区間の印を見ない）。**base では 1 つ目が改憲の違反だけ・2 つ目が合格で落ちる＝RED。** 置き場の schema 節か生成区間から一覧を読む形は、この歯が落とす。
4. **f128_lists_are_per_holder（部位ごとの一覧で、和集合でない）。** 1 つの写しに 5 か所の変異を同時に当てる = 規範文 N-3.1 に note: x（条では在ってよい欄）・条 N-3 の mechanism に relations: x（条では在ってよい欄）・開発規律行 D-5 に value: x（閾値行では在ってよい欄）・条 N-3 に bogus_a1: x と bogus_a2: x。違反の行はちょうど 5 行で、(b) の表の字が 5 つ揃い（同じ holder の 2 つの欄は 1 つずつ数える）、終了コード 1。**base では合格で落ちる＝RED。**
5. **f128_mechanism_shape_is_a_violation（mechanism の形の崩れ・改訂 c）。** 条 N-3 の mechanism の値を置き換えた別々の写し 5 つ。
   - 字 無効・一覧 [disabled]・null → それぞれ違反の行はちょうど 1 行 `[schema] constitution.yaml: 条 N-3 の mechanism が表でない`。
   - 空の表 {} → ちょうど 2 行（必須の欄 kind が無い・必須の欄 live が無い）。
   - kind を欠いた表（live: now・stage: post・polarity: fail-closed・note: x）→ ちょうど 1 行 `[schema] constitution.yaml: 条 N-3 の mechanism の必須の欄 kind が無い`。
   - どれも終了コード 1。
   さらに別の写し 1 つ = 前文の mechanism を字 無効 にする → ちょうど 2 行（改憲の違反と `[schema] constitution.yaml: 前文（precedence）の mechanism が表でない`）。**base では 5 つが合格・前文の 1 つが改憲の違反だけで落ちる＝RED**（(a) の 3）。
6. **f128_constitution_fields_refuses_broken_parts（導出の Err の 4 つの形・`crates/folio/tests/constitution_enums.rs`）。** この歯の file は build.rs を path で取り込んでいる（便 49 の歯と同じ形）。実の憲法の正本の字を constitution_fields に渡すと Ok で、source に 5 部位の名が全部出る。次の 4 つの変異の字は、それぞれ Err で理由の文に部位の名が入る。
   - schema.meta を字にする。
   - schema.statement.required を字にする。
   - schema.article.optional に数を混ぜる。
   - schema.mechanism.required に kind を 2 度書く。
   **base では関数が無いので、この歯の file が組み立たず落ちる＝RED。**

**床の凍結の場合に 3 件足す（`tests/floor_cases.yaml`・新しい組 cases_unknown_fields）。** 凍結の土台は実の design-intent から独立なので、P-10.1 の anchor になる。

- unknown-field-article: 変異 = 憲法の articles[id=P-8] に欄 bogus_a（値 x）を add で足す。期待 = 終了コード 1・字 条 P-8 の未知の欄「bogus_a」・違反 1 件。
- unknown-field-discipline-stage: 変異 = 規則の表の discipline[id=D-1] に欄 stage（値 post）を add で足す。期待 = 終了コード 1・字 行 D-1 の未知の欄「stage」・違反 1 件。
- mechanism-not-a-map: 変異 = 憲法の articles[id=P-8].mechanism の値を字 無効 に置き換える。期待 = 終了コード 1・字 条 P-8 の mechanism が表でない・違反 1 件。

場合の数 expected_cases は 143 から 146 になる（参考値）。起草役が凍結の土台の写しに同じ 3 つの変異を当てて base の binary を撃つと、3 つとも合格（終了コード 0）である＝**base で RED**（(h) の 4）。

**導出の一致の歯（common-verify が走らせる・f128_ の名を持たない）。** `crates/folio/src/constitution_enums.rs` の tests に 1 本足す。導出した 5 部位の 2 つの列が、実の憲法の正本の schema の同じ部位の required の列、および required と optional を file の順に繋いだ列と、長さ・字・並びまで一致することを見る（正本の側は歯が file から読む・定数を書かない）。base では導出の定数が無いので組み立てに失敗する。名に f128_ を持たないので、verify の絞り込みには入らない（src の unit test に絞り込みの語を置かない）。

### (d) 採らなかった形

1. **置き場の憲法の schema 節から実行時に一覧を読む（最上位の節の読みと同じ形）。** 退けた理由は 2 つある。1 つ目は、(a) の 6 の 23 本の憲法が 5 部位の一覧を持たないことである。読めない一覧を「まだ分からない」にすれば fixture を使う既存の歯が一斉に終了コード 2 に落ち、黙って通せば N-3 の穴が残る。どちらを選んでも 23 本の書き換え（write-set の外の大きな便）になる。2 つ目は、判断の記録 ADR-16 決定 (2) の「1 つの組み立て」の向き（値域は道具の側に残す）である。置き場の schema 節を広げる変更そのものは、schema が改訂の差分の範囲なので、どちらの形でも改憲の違反が立つ（歯 3 の 1 つ目の base の実測）。
2. **5 部位の一覧を check.rs に手で書く。** 憲法の schema 節と同じ一覧を 2 か所に人が書き、一致を歯で強制する形で、P-6.4 と判断の記録 ADR-11 決定 (3)(ア) の「手書きの写しを持たない」に反する。
3. **憲法に生成区間を足して一覧を写す。** 憲法の字を変える（schema 節は改訂の差分の範囲）ので改憲の手続きが要り、正本の向き（憲法が正・ADR-11 決定 (3)）も逆になる。
4. **憲法 N-3 の機構の注を実態に合わせて弱める（所見 F-1 の note のもう 1 つの直し方）。** 条文の改訂ではないが、fail-closed を約束した機構の注を「まだ分からない」へ下げる形で、N-3.1 の機構の中身（行や条へ旗の欄を足しても床が拒む）を失う。
5. **未知の欄を「まだ分からない」にする。** 閉じた一覧が組み立てた版で必ず立っているので、比較元が立たない場面は無い。N-3.1 は拒むことを求めるので違反にする。
6. **5 部位と規則の表の行の必須の欄の全部を本便で数える。** fixture 24 本に欠けが在り（(a) の 7）、書き換えが write-set の外へ大きく広がる。ADR-11 の注 (ア) が「床に足すのは後続の便」と書いた部分で、別の便にする（§5）。本便は mechanism の kind と live だけを数える。

### (e) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 1 本に `+`（`crates/folio/tests/` の下の unknown_fields.rs）。書き換える 7 本は印なし。7 本は `crates/folio/build.rs`・`crates/folio/src/check.rs`・`crates/folio/src/constitution_enums.rs`・`crates/folio/src/rules.rs`・`crates/folio/tests/constitution_enums.rs`・`crates/folio/tests/floor_cases.rs`〔本文不変・verify の scope〕・`tests/floor_cases.yaml` である。`-`（行が減る file）と、着地で消える file の印は当たらない。新しい dir は作らない。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs の 3 本（新しい .rs は src に無い）。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない・script は (h) の 5）。size S の見積は 1 file あたり 100。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 本便で増える見積（参考値） |
| --- | --- | --- | --- |
| `crates/folio/src/check.rs` | 951 | 549 | 70 |
| `crates/folio/src/constitution_enums.rs` | 86 | 1414 | 35 |
| `crates/folio/src/rules.rs` | 295 | 1205 | 0（注の 1 文の書き直し） |

   3 本とも余地は S の 100 を超える（check.rs は M の 300 も超える）。src の外の `crates/folio/build.rs` は参考に 442（余地 1058）で、増える見積は 80 である。歯の file は src の外なので余地を測らない。
3. **size は S。** 変える src は 3 本と組み立ての script 1 本で、どれも 1 file あたり 100 行を超えない見積である（改訂 c で増えた形の崩れの数えは check.rs に 30 行・build.rs に 30 行ほど）。
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test unknown_fields f128_` = (c) の歯 1〜5 の 5 本。
   2. `cargo nextest run -p folio --test constitution_enums f128_` = (c) の歯 6 の 1 本。
   3. `cargo nextest run -p folio --test floor_cases` = 床の凍結の場合（expected_cases の件数どおり・新しい 3 件を含む）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file は unknown_fields・constitution_enums・floor_cases の 3 本で、どれも write-set に在る（floor_cases は本文不変）。絞り込みの語 f128_ を関数名に持つ file は unknown_fields（5 本）と `crates/folio/tests/constitution_enums.rs`（1 本）の 2 本だけで、src には置かない。導出の一致の歯（src の constitution_enums.rs の tests）は共通の検証（`.vessel.toml` の common-verify の workspace の nextest）が走らせる。

### (f) 既存の歯のうち落ちるもの・凍結 anchor が動くか

1. **落ちる既存の歯は 0 本の見積である。** (a) の 5・6 のとおり、実の design-intent・fixture 25 本・床の凍結の場合の add の変異・骨格の置き場は、どれも閉じた一覧の内で、形の崩れも無い。値域の歯（`crates/folio/tests/check.rs` の check_constitution_enum_ で始まる歯）の変異は mechanism の中の値だけを変え、kind と live を残す。独立の検証役は改訂 b の形（未知の欄だけ）を ee714d7 に試作して、workspace の nextest が全部緑（参考値 835 本）・clippy 0 警告・`folio build --write` の出力の差 0 を実測した。改訂 c で足した形の崩れの数えは、同じ置き場の模擬の数えで 0 件（(h) の 2）だが、実装での実測はまだ無い（(i) の 3 の撤退条件 (1)）。
2. **凍結 anchor は動かない。** 凍結の土台（`tests/fixtures/floor_base/`）・生成区間の写し（`tests/fixtures/schema/` の下の *-region.txt の 9 本と要約値の anchor）・判断の記録の欄の決まりの写しは 1 byte も動かない。本便は `design-intent/` を書き換えず、`tests/fixtures/` の file を 1 本も書き換えないからである。値域の凍結 anchor（`tests/fixtures/check/enum-range-anchor.yaml`）も値域を変えないので動かない。床の凍結の場合の file は場合が 3 件増えるだけで、既存の 143 件の変異と期待は変えない。
3. **folio2 自身の結果**（`folio check --dir design-intent` の合格・`folio schema --dir design-intent --check`・`folio inject --check`・`folio build --write` の出力）は変わらない。歯 1 の最初の段が、元の写しの合格を見る。

### (g) 門（規則の表の開発規律行 D-12）

本便は `design-intent/` の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d128-draft.md`、script は同じ dir の d128_base_measure.sh・d128_base_measure_c.sh・d128_sim.py・d128_req.py（repo には入れない）。独立の検証の記録は同じ dir の d128-verify.md。

1. base の実測（(a) の 2・3）: `d128_base_measure.sh <repo> <folio の binary>` と `d128_base_measure_c.sh <repo> <folio の binary>`。写しを git の 1 commit にしてから欄を 1 つずつ足すか mechanism を置き換え、`folio check` の終了コードと違反の行を印字する。
2. 模擬の数え（(a) の 5・6）: `BUILT=design-intent/constitution.yaml python3 d128_sim.py design-intent <fixture の置き場>...`（folio2 の憲法の 5 部位の一覧と規則の表の行の一覧で、宣言の外の欄を数える）。骨格は `folio init --dir <一時 dir>/design-intent` の出力に同じ script を当てる。
3. 必須の欄の全部の欠けと形の崩れ（(a) の 6・7）: `BUILT=design-intent/constitution.yaml python3 d128_req.py design-intent <fixture の置き場>...`。
4. 床の凍結の場合の RED: 凍結の土台の写しに (c) の 3 件の変異を当てて base の binary の `folio check` → 3 つとも合格。
5. 余地: `python3 d119-draft-lines.py crates/folio/src/check.rs crates/folio/src/constitution_enums.rs crates/folio/src/rules.rs crates/folio/build.rs`（便 119 の script・同じ式）。
6. 着地の後: `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`・`folio check --dir design-intent`（合格）・`folio schema --dir design-intent --check`・`folio inject --check`。
7. 出力の差: base と後の binary で `folio build --dir design-intent --out <別々の置き場> --write` を撃ち、`diff -r` が差 0。
8. 門: 本体の作業ツリーで `folio ceiling --gate --dir design-intent --write-set <write-set の 8 本（接頭辞を剥がす）>`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 憲法・規則の表・要件書・判断の記録・語彙・天井の字（憲法 N-3 の機構の注と判断の記録 ADR-11 の注 (ア) の字を含む）。値域の導出と値域の部分集合の数え（便 49・便 122）。面の生成器。規則の表の生成区間と床の木の値。台帳への記帳（台帳 f2-648.176 の昇格と close は席）。外部 crate。**ほかの正本の床との境界**: 本便が足すのは憲法と規則の表の 8 か所の未知の欄と、4 か所の形の崩れだけである。判断の記録（`crates/folio/src/adr.rs`）・設計ノート（`crates/folio/src/note.rs` の meta・節・図の行）・要件書の項と図の行（check.rs の check_srs_item ほか）・欄の決まりの file（`crates/folio/src/floor.rs`）・天井・入口・相談窓口・語彙の欄の検査は、それぞれ別に数えていて、本便は変えない。
2. **言えないこと — 数えない欄と形。** 次は本便の後も黙る。
   - 条の中の行（rationale の各行・retreat・relations・amended_by の各行）と、meta の counts と approval の中の欄。憲法の schema 節がこれらの一覧を宣言していないので、閉じる一覧が無い。
   - 最上位の節のうち、注の 7 か所に入らない節（north_star・amendment・sources・rules_pointer・glossary_pointer）の中の欄。north_star は改訂の差分の範囲の外なので、欄を足しても改憲の違反も立たない（独立の検証役の実測）。
   - mechanism 以外の部位と規則の表の行の、必須の欄の有無。条から mechanism の鍵そのものを消す形もここに入り、黙る（(a) の 7・(d) の 6）。
   憲法 N-3 の注の「全節」は、これらの分だけ本便の後も実態より広い。
3. **撤退条件。** (1) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す（(f) の 1 は模擬の見積なので、落ちたら見積の外の欄か形が在ったことになる）。(2) 本便の後に folio2 自身の置き場の床の結果（合否・違反と「まだ分からない」の件数）か `folio build` の出力が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で `crates/folio/src/check.rs` の check_constitution か check_rules の周り、`crates/folio/build.rs` の main の憲法の段、または憲法の正本の schema の 5 部位の一覧が base と違っていたら、base を取り直して (a) の 5〜7 と余地を測り直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/build.rs` の新しい純粋な関数 constitution_fields（憲法の 5 部位の required の列と閉じた列の導出・Err の 4 つの形）と、main で値域の source の後ろに繋ぐ 1 行。`crates/folio/src/constitution_enums.rs` の導出の一致の歯 1 本と頭の注の 1 文。`crates/folio/src/check.rs` の新しい関数 closed_fields と、check_constitution・check_rules からの呼び出し（(b) の未知の欄の表の 8 か所）と、形の崩れの数え（表でない 4 か所・mechanism の必須の欄 2 か所）と頭の注の 1 文。`crates/folio/src/rules.rs` の頭の注の 1 文。新しい歯の file（f128_ の 5 本）。`crates/folio/tests/constitution_enums.rs` の f128_ の 1 本。`tests/floor_cases.yaml` の組 cases_unknown_fields の 3 件と expected_cases（146）。
- 入れない: 憲法と規則の表ほか設計文書の字・生成区間と床の木の値・凍結の土台と生成区間の写しと値域の anchor・値域の導出と部分集合の数え・mechanism 以外の必須の欄の有無の検査・(i) の 2 の holder と節の欄・ほかの正本の床（(i) の 1 の境界）・面の生成器・`crates/folio/src/note.rs`・`crates/folio/tests/floor_cases.rs` の本文・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| derive | 憲法の 5 部位の一覧の導出 | `crates/folio/build.rs` の新しい関数 constitution_fields（組み立て時・導出できなければ組み立てを失敗させる） |
| derivetest | 導出の歯 | `crates/folio/src/constitution_enums.rs` の tests（導出した列と正本の一致）と `crates/folio/tests/constitution_enums.rs` の f128_ の 1 本（Err の 4 つの形） |
| count | 未知の欄の数え | `crates/folio/src/check.rs` の新しい関数 closed_fields と check_constitution・check_rules からの呼び出し |
| shape | 形の崩れの数え | `crates/folio/src/check.rs` の check_constitution の中（meta・前文・前文の mechanism・条の mechanism が表でない・mechanism の kind と live の欠け） |
| rows | 規則の表の行の一覧 | `crates/folio/src/rules.rs` の閾値行と開発規律行の required と optional（値は不変・頭の注の 1 文） |
| cases | 床の凍結の場合 | `tests/floor_cases.yaml` の組 cases_unknown_fields の 3 件 |
| teeth | 歯 | 新しい歯の file の f128_ の 5 本 |

## 4. 検査（歯）

§1 (c)(f) と (e) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない（組み立ての script は今の yaml_rust2 を使う）。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無い。base は main ee714d7（便 125 の着地の後・§0 の 前の便）。
- 並行の便: 便 126（行 `dy`・契約は main 4eb2d2d）とは write-set が重ならない。`crates/folio/src/check.rs`・`crates/folio/build.rs`・`tests/floor_cases.yaml` を書き換える未着地の便は、起草の時点で契約表に無い。受付の時点で重なりが在れば、席が 1 本ずつ運ぶ。
- 本便の着地の後に席が見ること: 台帳 f2-648.176 を本便の行で閉じる。天井の次の周の実態の観点で、29 周目の F-1 の場所（憲法 N-3 の機構の注）が支持されないこと。
- 後の便への申し送り:
  - **N-3 の注の残り。** 注の「全節」のうち本便の後も黙る所（(i) の 2 の条の中の行・7 か所に入らない節・mechanism 以外の必須の欄）を、台帳に控えとして 1 件起票する。直し方は、注を「7 か所の holder の欄と mechanism の形」に狭める是正か、後続の便で床を広げるかのどちらかで、席が決める。次の天井の周の前に置かないと、実態の観点が同じ注を別の形で拾いうる。
  - **判断の記録 ADR-11 の注 (ア) が古くなる。** 注の「必須の欄の集合の定数…を読むのは歯と folio schema だけ」は、本便の着地の後は床（check_rules）も同じ定数を未知の欄のために読むので古くなる。必須の欄の床の便か、次の注の是正で直す。必須の欄の全部の床は、fixture 24 本の欠けの書き換え（(a) の 7）を同じ便に含める。
  - **利用者の置き場の欄の一覧。** 床が folio2 を組み立てた一覧で閉じることは、判断の記録 ADR-16 決定 (2) の焼き込む所 (ア)〜(キ) に無い 1 つを足す（(b) の 利用者の置き場への帰結）。ADR-16 の注に記帳するかは席が決める。利用者の憲法を移すときは別の判断の記録で扱う。
  - **憲法の一覧を変える改訂。** 憲法の 5 部位の一覧を変える改訂（改憲の手続き）は、組み立て直すと床の一覧も動く。実の design-intent のどこかの holder が新しい欄を使い始めたら、同じ改訂で一覧に足さないと床が落とす。
  - **規則の表の行の一覧を変える便。** `crates/folio/src/rules.rs` の定数を変え、生成区間を `folio schema --write` で導出し直し、凍結 anchor `tests/fixtures/schema/rules-region.txt` を同じ便で直す（行 D-11 の作法）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dz"
title = "天井の 29 周目の実態の所見 F-1（止める・反証で支持）と台帳 f2-648.176（規則の表の開発規律行に stage を書いても床が黙って落とす）と憲法 N-3 の機構の注（正本と rules の全節で未知の欄を schema 検査が落とす・床は folio check）の 1 便: 憲法の meta・前文・前文の mechanism・条・条の mechanism・規範文と、規則の表の閾値行・開発規律行の欄を、holder ごとの閉じた一覧で数え、一覧に無い欄を 1 つにつき種別 未知の欄 の違反 1 件にする（fail-closed・例外の口なし）。あわせて、meta・前文・前文の mechanism・条の mechanism が表でない形と、mechanism の必須の欄 kind・live の欠けを種別 schema の違反にする（無効化の旗の形を塞ぐ）。憲法の 5 部位（meta・precedence・article・mechanism・statement）の一覧は判断の記録 ADR-11 決定 (3) のとおり憲法の正本が正で、crates/folio/build.rs の新しい関数 constitution_fields が組み立て時に schema の required と optional から導出する（導出できなければ組み立てを失敗させる・手書きの写しは持たない・置き場の schema 節は読まない）。規則の表の行の一覧は crates/folio/src/rules.rs の今の型付きの定数（写しは生成区間に在る）を床が読み、file の生成区間は読まない。crates/folio/src/check.rs に新しい関数 closed_fields を足し、check_constitution と check_rules から呼ぶ。mechanism 以外の必須の欄の全部は数えない（fixture 24 本に欠けが在る・後続の便）。憲法と設計文書の字・生成区間・凍結 anchor は変えず、folio2 自身の床の結果は変わらない。歯は新しい crates/folio/tests/unknown_fields.rs の f128_ の 5 本と crates/folio/tests/constitution_enums.rs の f128_ の 1 本と、tests/floor_cases.yaml の新しい組の 3 件（凍結の土台の条 P-8 の未知の欄・条 P-8 の mechanism が表でない・開発規律行 D-1 の stage）と、導出の一致の歯 1 本"
req = ["FR5"]
section = "1"
write-set = ["crates/folio/build.rs", "crates/folio/src/check.rs", "crates/folio/src/constitution_enums.rs", "crates/folio/src/rules.rs", "+crates/folio/tests/unknown_fields.rs", "crates/folio/tests/constitution_enums.rs", "crates/folio/tests/floor_cases.rs", "tests/floor_cases.yaml"]
verify = ["cargo nextest run -p folio --test unknown_fields f128_", "cargo nextest run -p folio --test constitution_enums f128_", "cargo nextest run -p folio --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "unknown_fields の f128_ の歯 5 本（元の写しは合格で、憲法の meta・条・条の mechanism・規範文に足した未知の欄はそれぞれ違反ちょうど 1 行、前文と前文の mechanism に足した欄は改憲の違反と未知の欄の 2 行／規則の表の行 R-2 の bogus_r・行 D-5 の disabled・行 D-14 の stage はそれぞれ未知の欄の違反ちょうど 1 行／置き場の憲法の schema.meta.optional と規則の表の生成区間の threshold_row.optional を広げても未知の欄の違反は消えない／規範文の note・条の mechanism の relations・開発規律行の value・条の 2 つの欄を同時に足すと違反ちょうど 5 行で、一覧は holder ごと／条 N-3 の mechanism を字・一覧・null にするとそれぞれ 表でない の違反ちょうど 1 行、空の表で kind と live の欠けの 2 行、kind を欠いた表で 1 行、前文の mechanism を字にすると改憲の違反と 表でない の 2 行）が緑、constitution_enums の f128_ の歯 1 本（導出の関数が実の正本で Ok・部位が表でない・required が字の一覧でない・optional に字でない値・同じ名の 2 度の 4 つで Err と部位の名）が緑、床の凍結の場合が tests/floor_cases.yaml の expected_cases（146）どおりで新しい 3 件（条 P-8 の未知の欄・条 P-8 の mechanism が表でない・開発規律行 D-1 の stage）が終了コード 1、clippy が 0 警告で、workspace の nextest が全部緑（導出の一致の歯・値域の歯・folio2 自身の置き場の合格の歯を含む）で CI が通り、着地の後の main で folio check --dir design-intent が 合格（違反 0・まだ分からない 0）を返す"
<!-- contracts:end -->
