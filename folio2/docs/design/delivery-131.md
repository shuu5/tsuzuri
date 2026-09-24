# 設計: 便 131 — folio check が機構がまだ無い条を標準エラーに 1 行で並べる（判断の記録 ADR-23 決定 (3) の便・判定と終了コードと標準出力は不変）

- 要件: FR5（構造の床を実行し 3 値で返す・要件書 第 1.41 版）。FR5 の規範文は `folio build` の 3 値の口を書き、この行を求めていない。本便の行は **3 値の外の 1 行**で、判定・違反とまだ分からないの数・終了コード・標準出力は 1 byte も変えないので、FR5 の規範文とも受入基準 AC3 とも食い違わない。契約表の行の req は FR5 の 1 つ（main に在る id・床の出力の口の要件）。規範文はどれも変えない。
- 条: P-3.3（床の合格を、完成や天井の合格として扱わない＝機構がまだ無い条を合格の陰に隠さない）/ P-4.1（検査・生成が実行できなかった結果を「異常なし」として扱わない）/ P-5.1（閉じた一覧は型付きデータ＝機構の種別と段の値は組み立てた値域の型で読む）/ P-10.1（独立した凍結 anchor）。
- 出所: 判断の記録 **ADR-23**（憲法 第 1.4 版・発効 2026-09-25・持ち主の承認・対話面 R-8・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-25 07:03 JST）の決定 (3)「folio check は、置き場の憲法の条のうち機構の種別が機械が拒む（reject）か検査で落とす（build-check）で、実在の段が「いま」（now）でない条を、標準エラーに 1 行で並べる」と、承認要求 `docs/design/adr-23.md` の §決定 (3) の便の要点（契約表の起草の材料）と §承認の後に席が書く欄 の 16。憲法の schema 節の機構の段の規則（schema.mechanism_live_rule・第 1.4 版）は「機構がまだ無い条として床の判定（違反・まだ分からない）に数えず、folio check の出力と面の機構の欄に条ごとに表に出す」と書く。本便はその「folio check の出力」の口である（面の機構の欄は便 132）。規則の表の開発規律行 D-11 は、この行と契約表の行の title が ADR-23 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ed` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 4 本。**新しい file は 1 本で、先頭に `+` を付けて宣言する**（歯の file 1・既に在る dir `crates/folio/tests/` に置く）。縮む file も消す file も無く（`-` は当たらない）、新しい dir も作らない。歯の runner `crates/folio/tests/floor_cases.rs` は本文を変えないが、verify が `--test floor_cases` で名指すので write-set に入れる。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 4 本を base の binary（main ed966c1）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は ADR-23 の発効の取り込み（憲法 第 1.4 版・凍結 anchor v1.4・一括 19 と ADR-22 の発効と同じ取り込み）で、本流 ed966c1 に着地済み。**base = main ed966c1。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 132（行 `ee`・面の名札）とは write-set が 1 本も重ならない。起草役は両便を base に同時に当てた写しで workspace の nextest が全部緑（参考値 866 本）・床 4 本 rc 0 を実測した。席は本便 → 便 132 の順で逐次に受け付ける（`crates/folio/tests/sheet.rs` の一時 dir は process id を持たず、同じ host で 2 本の共通の検証を同時に撃つと偶発の Failed が出うる・便 129 / 130 の検証役の実測）。ADR-23 決定 (4) により、本便と便 132 の 2 本が着地するまで本流の上で天井の周の束を組まない。
- 改訂 b（2026-09-25・独立の検証 d131-verify.md〔条件付き支持・blocking 1・文面 4〕と席の裁定）: 歯 1 に、種別が human-review の条の live を M1 にした写しを足した（blocking 1＝一覧の条件のうち種別の絞りを数える歯が無く、check.rs から種別の絞りを外した突然変異でも workspace の nextest が全部緑だった）。(a) の 4 に種別の組の実測を、(c) の末尾に歯 2・3 が足した code を数えないことを、(i) の 2 に凍結が成功する道の行を歯が数えないことを足した（文面 2・3）。write-set（4 本）・歯（4 本）・verify（3 行）・size・base の RED の数え（歯 1 と 4）は変えていない（模擬と突然変異を撃ち直した）。

## 1. 設計

### (a) いま起きていること（実測・base ed966c1）

1. **機構がまだ無い条は、どこにも表に出ない。** 実の憲法（第 1.4 版）の 27 条のうち、機構の種別が reject か build-check で live が now でない条は 6 本（P-11 reject M1・P-13 build-check M1・P-15 build-check M1・P-17 reject M1・P-18 build-check M1・A-3 reject M1）。`folio check --dir design-intent` の出力は標準出力の要約の 1 行「folio check: 合格（違反 0・まだ分からない 0）」だけで、終了コード 0。6 本の名はどこにも出ない（第 1.4 版の規則「folio check の出力に条ごとに表に出す」と食い違う・ADR-23 決定 (4) の周の前提）。
2. **出力の組み立ては main.rs の 1 か所。** `crates/folio/src/main.rs` の命令 check の枝は、`crates/folio/src/check.rs` の check_dir（床を回して Report と材料 Materials を返す）→ 凍結の後始末（`crates/folio/src/freeze.rs` の after）→ 断りの道（after が Refused のとき、標準エラーに「folio check: <理由>」を出して終了コード 1 で抜ける・要約の行は出さない）→ 違反の行（標準出力・差分の印字の口 --emit-amends では標準エラー）→「# まだ分からない: 」の行（標準エラー）→ 要約の行（標準出力・--emit-amends では標準エラー）→ 凍結の口の知らせ（標準エラー）か差分の行（標準出力）、の順で出す。Materials は凍結 anchor の列の結果・判断の記録・現行の id の 3 欄で、組み立てるのは check_dir だけ、読むのは main.rs だけである（`crates/folio/src/site.rs` の組み立ての床は Materials を捨てる）。
3. **値域の型は在る。** 組み立て時に憲法の値域から導出した型（`crates/folio/src/constitution_enums.rs` が取り込む MechanismKind と MechanismLive・名から型を引く関連 fn from_name）を、check.rs は ce の名で既に使う。面の生成器（`crates/folio/src/face_constitution.rs`）も同じ型で機構の欄を読む。
4. **写しの実測（起草役・base の binary）。**

| 置き場 | 当たる条（憲法の条の並び・括弧は live の値） | folio check の終了コード・標準出力 |
| --- | --- | --- |
| 実の design-intent | P-11（M1）・P-13（M1）・P-15（M1）・P-17（M1）・P-18（M1）・A-3（M1）の 6 本 | 0・要約「合格（違反 0・まだ分からない 0）」 |
| 床の凍結の土台 `tests/fixtures/floor_base/`（第 1.0 版）の写し | P-1（M0）・P-3（M0）・P-4（M0）・P-7（delivery-0）・P-10（M0）・P-11（delivery-0）・P-12（M0）・P-13（M0）・P-15（M1）・P-17（delivery-0）・P-18（delivery-0）・A-3（M1）・N-1（M0）・N-5（M0）の 14 本 | 0・要約「合格（違反 0・まだ分からない 0）」・標準エラーは空 |
| 同じ写しに差分の印字（--emit-amends） | 同じ 14 本 | 0・標準出力は比較元の注の 1 行・標準エラーは要約の 1 行 |
| 同じ写しに同じ版の凍結（--freeze-anchor） | — | 1・標準出力は空・標準エラーは「folio check: 版 v1.0 は最新 anchor v1.0 より新しくない（同じ版は上書きしない・版を上げてから）」の 1 行（断りの道） |
| folio init の骨格（git の 1 commit の後） | 0 本（唯一の条の live は now） | 2・要約「まだ分からない（違反 0・まだ分からない 2）」（凍結の基準が無い 2 件） |

   写しは一時 dir に design-intent/ として作り、親に器の導出 file `contracts/schema.toml` を写し、git の init と 1 commit をした形である。

   **種別の絞りの実測。** 種別が human-review か none で live が now でない条は、実の正本にも土台にも面の fixture にも 0 本である。土台の写しで種別が human-review の最初の条（P-16）の live を M1 にすると、終了コード 0・要約は合格のままで、一覧は 14 本のまま（P-16 は載らない）。種別の絞りを外した突然変異の binary では P-16（M1）を足した 15 本になる（起草役と検証役の実測）。
5. **床の凍結の場合は標準エラーの字を数えない。** `tests/floor_cases.yaml` の場合（参考値 146 件）のうち終了コード 0 を期待する 21 件（ADR-23 の承認要求の検証の数え）を含め、runner `crates/folio/tests/floor_cases.rs` は終了コードと、期待の字を標準出力か標準エラーに含むかだけを見る。行を足した写しで runner の 11 本は全部緑だった（(e) の 1）。
6. **base の歯と床（参考値）。** workspace の nextest 861 / 861。床 4 本（check 合格 0/0・inject 68 行 / 7,912 byte 一致・schema 9 file 一致・derive 一致 1）はどれも rc 0。`git grep -n 'f131_' -- crates` は 0 件。

### (b) 直す先 — 材料に一覧を載せ、要約の行の直前に 1 行

1. **一覧を組む関数（check.rs）。** `crates/folio/src/check.rs` に crate の中だけで見える関数 not_yet_live_articles を足す。憲法の木を受け、articles の各行について、mechanism の kind を MechanismKind の from_name で、live を MechanismLive の from_name で引き、kind が Reject か BuildCheck で live が Now でない条を、憲法の条の並びのまま「<条の id>（<live の字>）」の字の列にして返す。articles が無い・一覧でない、条の mechanism・kind・live・id が無いか字でない、kind か live が組み立てた値域に無い条は数えない（その崩れは値域の検査〔便 55・122〕と形の検査〔便 128〕が別に違反か まだ分からない として数える＝同じ崩れを 2 度出さない）。前文（precedence）の機構は条でないので数えない。値の字（reject・build-check・now）を関数の中に書かない（P-5.1・値域の型の名で持つ）。
2. **材料の欄（check.rs）。** Materials に欄 not_yet_live（字の列）を足し、check_dir が正本 7 file を読めたときに、読んだ憲法の木（load_all の結果・床が数えるのと同じ木）を 1 の関数に渡して埋める。読めなければ空の列（そのときは床が まだ分からない を出す）。憲法を 2 度読まない。
3. **出す所（main.rs）。** 命令 check の枝で、「# まだ分からない: 」の行を全部出した直後、要約の行の直前に、列が空でなければ標準エラーへ次の 1 行を出す（列が空なら何も出さない）。

   ```
   # 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: P-11（M1）・P-13（M1）・P-15（M1）・P-17（M1）・P-18（M1）・A-3（M1）
   ```

   頭の字は `# 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: ` で、その後に列を「・」で繋ぐ（ADR-23 決定 (3) の下限の字と同じ）。差分の印字の口・凍結の口（--freeze-anchor・--freeze-ids・--freeze-start）も同じ道を通るので同じ行を出す。断りの道（Refused・要約の行より前に終了コード 1）はこの段より前に抜けるので出さない。
4. **変えないもの。** 判定（Report の中身と verdict）・違反とまだ分からないの数・終了コード・標準出力の全部（要約の行と違反の行と差分の行）・「# まだ分からない: 」の行・`folio build` の床の行（site.rs は Materials を捨てる）・面・設計文書。Report には何も足さない。
5. **folio2 自身の結果。** `folio check --dir design-intent` は終了コード 0・標準出力は要約の 1 行のままで、標準エラーに (b) の 3 の 1 行が出る（起草役の実測）。`folio build --dir design-intent --out <置き場> --write` の出力は base と 1 byte も変わらない（30 file・起草役の実測）。

### (c) 歯（新しく 4 本・新しい file `crates/folio/tests/` の下の mechanism_live.rs）

関数名は f131_ で始める（verify の絞り込みの語・base で 0 件）。土台は `crates/folio/tests/freeze_root.rs` の Work と同じ形（一時 dir に design-intent/ として写し・親に `contracts/schema.toml` を写し・git init と 1 commit・歯の終わりに消す・一時 dir の名に process id を入れる）。歯は folio の code を呼ばず、命令を撃って標準出力・標準エラー・終了コードを見る。

**凍結 anchor（P-10.1）。** 土台の一覧の字を、歯の中の定数に手で書く（(a) の 4 の 14 本・folio の code から組まない）。folio2 自身の憲法の一覧（6 本）の定数は、条の機構の段の欄の変更を審査に引き出すための**意図した 2 つ目の写し**である（実の正本の本数や字に依らない歯の向き〔便 43・44〕の例外・ADR-23 決定 (3)）。歯の file の頭の注にその旨を 1 文で書く。

1. **f131_floor_base_lists_the_articles_before_the_summary。** 土台の写しに folio check → 終了コード 0・標準出力はちょうど「folio check: 合格（違反 0・まだ分からない 0）」の 1 行・標準エラーはちょうど 14 本の一覧の 1 行。同じ写しに --emit-amends → 終了コード 0・標準エラーの行はちょうど 2 行で、一覧の行・要約の行の順（一覧の行が要約の行の直前）・標準出力に一覧の頭の字が無い。さらに、土台の写しで種別が human-review の最初の条（P-16）の live を now から M1 に置き換えた写し（置き換える字が無ければ前提が崩れたとして落ちる）→ 終了コード 0・標準出力は同じ要約の 1 行・一覧の行はちょうど 14 本の 1 行のまま（種別の絞りを数える唯一の歯・(a) の 4 の実測）。**base では標準エラーが空で落ちる＝RED**（起草役の実測）。種別の絞りを外した突然変異では、human-review の写しで一覧が 15 本になって落ちる（起草役の実測・歯 1 本だけが落ちる）。
2. **f131_no_line_when_no_article_waits_for_its_mechanism（当たらないことの歯）。** 土台の写しの憲法の全条の live の値（M0・delivery-0・M1・adr）を now に置き換えた写し（機構の欄は凍結 anchor の写しの 5 欄の外なので、改憲の違反は立たない）→ 終了コード 0・標準出力は 1 と同じ要約の 1 行・標準エラーは空。folio init の骨格の写し → 終了コード 2・標準出力はちょうど「folio check: まだ分からない（違反 0・まだ分からない 2）」の 1 行・一覧の行が無い。**base でも緑**（行を出さない側の歯・判定と終了コードと標準出力が行を足す前と同じことを数える）。
3. **f131_a_refused_freeze_prints_no_line（当たらないことの歯）。** 土台の写しに --freeze-anchor → 終了コード 1・標準出力は空・標準エラーに一覧の行が無く、断りの字「同じ版は上書きしない」が在る。**base でも緑。**
4. **f131_folio2_constitution_lists_the_six_articles（意図した写し）。** 実の design-intent に folio check → 標準エラーの一覧の行はちょうど 1 行で、(b) の 3 の字（6 本・並び・段の値まで）と一致する。条の live を now から別の値へ戻す変更も、機構が着地して now に改める変更も、同じ取り込みの要求でこの定数を直さないと落ちる（ADR-23 の撤退条件 (2) の決定的な受け皿）。終了コードは見ない（実の置き場の合否は別の歯が見る）。**base では行が無く落ちる＝RED。**

歯 2 と 3 は行を出さない側の歯で、足した code を数えない（base でも緑・判定と終了コードと標準出力が行を足す前と同じことを数える）。足した code を数えるのは歯 1 と 4 である。

### (d) 採らなかった形

1. **要約の行に数を足す（ADR-23 の案 e）。** 要約の字を見る歯が 21 か所・10 file に在り（ADR-23 の検証の数え）、ADR-23 決定 (3) が「標準出力は 1 byte も変えない」と決めた。
2. **Report に種別を足して違反か まだ分からない にする。** 第 1.4 版の規則は「床の判定に数えない」と書く。
3. **main.rs で憲法の file をもう 1 度読んで組む（ADR-23 の検証の試作の形）。** 床が数えた木と別の読みになり（P-6.3 の向き）、読めないときの扱いが 2 通りになる。値の字を main.rs に書くことにもなる。
4. **`folio build` の床の行にも出す。** ADR-23 決定 (3) が触らないと決めた（読み方 ② の「表に出す所」に数えない）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで workspace の nextest は 865 / 865（base 861 + 歯 4）・clippy 0 警告。床の凍結の場合の runner（11 本・場合 146 件）も、folio check の標準出力を見る歯（要約の字を見る 21 か所・10 file）も、Materials を読む所（main.rs だけ）も落ちない。歯のうち標準エラーの全体が空であることを見る既存の歯は、実測で 1 本も落ちなかった。
2. **凍結 anchor は動かない。** 床の凍結の土台・生成区間の写し・値域の anchor・`tests/floor_cases.yaml` は 1 byte も変えない。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 1 本に `+`（`crates/folio/tests/` の下の mechanism_live.rs）。書き換える 2 本（`crates/folio/src/check.rs`・`crates/folio/src/main.rs`）と本文不変の 1 本（`crates/folio/tests/floor_cases.rs`・verify の scope）は印なし。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/check.rs` | 1,002 | 498 | 1,029（+27） |
| `crates/folio/src/main.rs` | 655 | 845 | 662（+7） |

   2 本とも余地は S の見積 100 を超える。新しい歯の file は src の外なので余地を測らない（参考値 190 行）。
3. **size は S。** src の増分は 1 file あたり 30 行に満たない見積。
4. **verify は 3 行**で、done の 3 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test mechanism_live f131_` = (c) の歯 4 本。
   2. `cargo nextest run -p folio --test floor_cases` = 床の凍結の場合（件数と終了コードの期待が不変・参考値 11 本）。
   3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は mechanism_live（新しい file）と floor_cases（本文不変）の 2 本で、どちらも write-set に在る。絞り込みの語 f131_ を関数名に持つ file は新しい file だけで、src には置かない。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付の順。** 前提の ADR-23 の発効の取り込みの後に受け付ける。便 132 とは write-set が重ならないが、席は本便 → 便 132 の順で逐次に受け付ける（§0 の 並行の便との重なり）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d131-draft.md`、模擬の差分は同じ dir の d131-scripts（repo には入れない）。

1. 模擬: 差分 a.patch を base に当て、workspace の nextest（865 / 865）・clippy・床 4 本・`folio build --write` の出力の差 0。便 132 の差分 b.patch と両方を当てても 866 / 866。
2. RED: 歯の file だけを base に当てると、歯 1 と 4 が落ち、歯 2 と 3 は緑。突然変異: 差分を当てた写しで check.rs の一覧の関数から種別の絞りを外すと、歯 1 だけが落ちる。
3. 写しの実測（(a) の 4）: 土台と骨格の写しを一時 dir に作り、base と後の binary で folio check を撃つ（旗なし・--emit-amends・--freeze-anchor）。
4. 余地: 持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d119-draft-lines.py` を repo の根で当てる（便 119 の script・同じ式）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 面の名札（便 132）。`folio build` の床の行。憲法・規則の表・要件書ほか設計文書の字。6 本の条の機構そのもの（P-11・P-13・P-15・P-17・P-18・A-3 の検査の口・どれも台帳の控えか条の注が持つ）。床の凍結の場合と土台。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 次の天井の周が、この 1 行を第 1.4 版の規則の「folio check の出力に条ごとに表に出す」の充足と読むかは、確率的な審査なので言えない（ADR-23 の撤退条件 (3)）。行は標準エラーに出るので、標準出力だけを読む消費者には見えない（ADR-23 決定 (3) の置き場の決め）。凍結が成功する道（--freeze-anchor・--freeze-ids・--freeze-start）で行が出ることを数える歯は無い。main.rs の上では差分の印字の口と同じ道を通り、要約の行の前に出す。
3. **撤退条件。** (1) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に folio2 自身の床の結果（合否・違反とまだ分からないの件数・終了コード・標準出力）か `folio build` の出力が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で、main.rs の命令 check の枝か check.rs の check_dir と Materials、または憲法の 6 本の条の機構の欄が base と違っていたら、base を取り直して (a) の 4 と歯 4 の定数を測り直してから運ぶ。(4) 本便が決定 (3) の字で着地できない（契約の審査で落ちる・行 R-7 の回数を超えて失敗する）ときは、席が ADR-23 の撤退条件 (5) で持ち主に問う。

## 2. 範囲

- 入れる: `crates/folio/src/check.rs` の関数 not_yet_live_articles と Materials の欄 not_yet_live と check_dir の 2 行。`crates/folio/src/main.rs` の命令 check の枝の標準エラーの 1 行。新しい歯の file（f131_ の 4 本）。
- 入れない: 判定・Report・終了コード・標準出力・`folio build`・面・設計文書・床の凍結の場合と土台・`crates/folio/tests/floor_cases.rs` の本文・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| list | 機構がまだ無い条の一覧 | `crates/folio/src/check.rs` の not_yet_live_articles（値域の型で引く・憲法の条の並び） |
| carry | 材料の欄 | check.rs の Materials の not_yet_live（check_dir が床と同じ木から埋める） |
| line | 標準エラーの 1 行 | `crates/folio/src/main.rs` の命令 check の枝（要約の行の直前・0 本なら出さない） |
| teeth | 歯 | 新しい歯の file の f131_ の 4 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: ADR-23 の発効の取り込み（憲法 第 1.4 版）。
- 並行の便: 便 132（行 `ee`）と write-set が重ならない（§0）。受付は本便 → 便 132 の逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。便 132 の着地の後に本流で天井の周を組む（ADR-23 決定 (4)）。条の機構の段を改める取り込みの要求では、歯 4 の定数も同じ要求で直す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ed"
title = "判断の記録 ADR-23（憲法 第 1.4 版・発効 2026-09-25・裁定 id = 台帳 f2-648 notes 2026-09-25 07:03 JST）の決定 (3) の便: folio check が、置き場の憲法の条のうち機構の種別が reject か build-check で live が now でない条を、憲法の条の並びに id と live の値の括弧で、標準エラーに 1 行で並べる（頭の字は # 機構がまだ無い条（床の判定の外・憲法 schema.mechanism_live_rule）: ・0 本なら出さない・# まだ分からない の行の後で要約の行の直前・差分の印字と凍結の口も同じ・凍結を断る道では出さない）。crates/folio/src/check.rs に値域の型で引く一覧の関数と材料の欄を足し、check_dir が床と同じ憲法の木から埋め、crates/folio/src/main.rs が出す。判定・違反とまだ分からないの数・終了コード・標準出力・folio build・面・設計文書は変えない。歯は新しい crates/folio/tests/mechanism_live.rs の f131_ の 4 本（床の凍結の土台の 14 本の行と要約の直前と種別の絞り・0 本の写しと骨格で行が無く判定と標準出力が同じ・断りの道で行が無い・folio2 自身の憲法の 6 本を固定する意図した写し）"
req = ["FR5"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/src/main.rs", "+crates/folio/tests/mechanism_live.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --test mechanism_live f131_", "cargo nextest run -p folio --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "mechanism_live の f131_ の歯 4 本（床の凍結の土台の写しで終了コード 0・標準出力は要約の 1 行だけ・標準エラーはちょうど一覧の 1 行で、差分の印字の口では一覧の行が要約の行の直前／全条の live を now にした写しと folio init の骨格では一覧の行が無く、終了コードと標準出力は行を足す前と同じ／同じ版の凍結を断る道では一覧の行が無い／種別が human-review の条の live を M1 にした写しでも一覧が土台の 14 本のまま／実の design-intent の一覧の行が §1 (b) の 3 の 6 本の字と一致）が緑、床の凍結の場合の runner が件数と終了コードの期待どおりに緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）と終了コード 0 を返す"
<!-- contracts:end -->
