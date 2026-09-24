# 設計: 便 126 — 天井の周の引き金を規範の欄の要約値で測り、印に引き金の要約値の欄を足し、門は印の引き金の要約値で古さを判定する（ADR-18 決定 (1)(4)(5)(7) の便ア）

- 要件: FR20（天井の門・要件書 第 1.36 版で改めた規範文。印の中身の列に引き金の要約値・門の古さの判定を印の引き金の要約値へ・0 を返すときに印の後に変わった節点の数を理由の行に添える・引き金の要約値は天井の正本の生成区間が写す規範の欄の一覧から印と門が同じ関数で測る）/ 受入基準 AC18（門の場合を 6 から 7 へ・引き金は同じで正本の他の字が違う → 通す）。契約表の行の req は FR20 の 1 つ（main に在る id）。FR19（床の定数から欄の決まりの file の生成区間へ写す）も本便が天井の正本の生成区間を書き換える要件だが、規範文は変えないので req に入れない。
- 条: P-5.1・P-5.6（規則・型の一覧は実装の型付きの定数に置き、その写しを設計文書の置き場へ決定的に導出する＝規範の欄の一覧の正本は天井の床の定数・写しは天井の正本の生成区間）/ P-15.2（止める仕掛けの判定の式は同じ関数で確かめる＝印と門が同じ 1 つの関数で引き金の要約値を測る）/ P-4.1・P-4.2（測れない・数えられないを「異常なし」にしない・読まれていない字を「まだ読まれていない」として表に出す）/ P-3.3（床の合格を天井の合格として扱わない）/ P-10.1・P-10.2（独立した凍結 anchor・生成物どうしの突き合わせを唯一の合格判定にしない）/ P-6.2（生成物を手で直さない＝印は周の後に席が --stamp で書き直す）
- 出所: 判断の記録 **ADR-18**（案 a・発効 2026-09-24 11:51 JST・持ち主の承認・対話面 R-8・逐語「全部推奨で」・裁定 id = 台帳 f2-648 notes 2026-09-24 11:51 JST）の決定 (1)「周の引き金を規範の欄の要約値が変わることに絞る。規範の欄は 5 つの閉じた一覧。一覧の正本は実装の型付きの定数に置き、天井の正本の末尾の生成区間へ写しを導出する」・決定 (4)「印に引き金の要約値の欄を足す。印と門は同じ 1 つの関数で引き金の要約値を測る」・決定 (5)「門の式を改める。通すときに印の正本の要約値が今と違えば、理由の行に印の後に引き金の外の変更が在る（節点 k 個・次の引き金の周が読む）を添える」と、決定 (7) と判断の記録の承認要求 docs/design/adr-18.md §5 の便の見積の **便ア**（規範の欄の型付きの定数と引き金の要約値の関数・印の欄・門の式と理由の行・天井の正本の生成区間への導出・歯と凍結の写し・size M）。台帳 f2-648.145（ADR-13 の便の列の ⑤ の 2 本目・周の引き金の判定の口）は本便が置き換える（台帳の書き換えは席）。規則の表の開発規律行 D-11 は、この行と契約表の行の title が ADR-18 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dy` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 37 本。**新しい file は無い**（`+` は当たらない）。縮む file も消す file も無く（`-` は当たらない）、新しい dir も作らない。`~` は使わない。
- 門: 本便は天井の正本 `design-intent/ceiling.yaml` の生成区間を書き換えるので天井の門の対象である。起草役が base main ee714d7 の作業ツリー（本便の枝）で write-set 37 本（接頭辞なし）を `folio ceiling --gate --dir design-intent --write-set …` に渡すと **2（まだ分からない・断りの字は 印が古い）**（2026-09-24・main ee714d7 で build した binary）。src の 1 本（`crates/folio/src/gate.rs`）だけを渡すと 0（通す・設計文書の正本を書き換えない便）。印（design-intent/preview/ceiling-stamp.yaml）は天井の 28 周目（合格）のもので、29 周目は合格でなく印を付け直していない。その後の判断の記録 ADR-17・ADR-18・ADR-19 の発効と一括 16 で正本が変わったので古い。受付の時点の main で撃ち直す。**印が古いままなら、規則の表の行 D-12 の持ち主の裁定の前例（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes・便 101・103・117・119・123 で当てた裁定）を席が当てて受付ける**（判断の記録 ADR-19 決定 (4) はこの前例を本便の着地の時点で置き換えるので、本便の受付の時点ではまだ有効）。当てたときは、受付のときに席がその裁定 id を台帳の notes に記帳する（§1 (h)）。
- 前の便: **base = main ee714d7（判断の記録 ADR-17 の発効・一括 16・ADR-18 の発効〔16ed6a2〕・ADR-19 の発効〔2df6d80・規則の表の行 D-14 新設・要件書 第 1.37 版〕・便 125 の着地〔骨格の命令 folio init・ee714d7〕の後）。この契約の数はすべて参考値である**（規則の表の行 D-13）。改訂 b で起草の土台 694751e（ADR-18 の起草の枝）から載せ替え、数を全部 ee714d7 で数え直した。628c547 から ee714d7 までに変わった `crates/` のうち write-set に在るのは `crates/folio/src/main.rs`（便 125 の命令 init・門の呼び出しの周りは変わらない）だけで、ほか（src の hello.rs・init.rs と歯の file）は write-set の外である。design-intent のうち write-set に在る design-intent/ceiling.yaml は生成区間の外（頭の注・v0.15・承認欄の 2 行）だけが変わった。生成区間・要件 FR20 の規範文・受入基準 AC18 は第 1.37 版でも第 1.36 版と同じ字で、§1 の設計は動かない。受付の時点の main で数え直す。
- **ADR-19 との時刻の点:** 判断の記録 ADR-19 の撤退条件を数える期間の起点と、2026-09-22 00:0x JST の持ち主の裁定（印が古くても設計文書の便を門を経ずに出す）の置き換え（ADR-19 決定 (4)・規則の表の行 D-14 の注）は、どちらも本便の着地の時刻である（ADR-18 の撤退条件の窓は、本便の着地の後に引き金の要約値を持つ印が初めて付いた時刻から 7 日）。便イ（便 127・面の天井の名札の添え書き・`crates/folio/src/face.rs`）は本便の着地の後に受付ける。
- 分割: **1 便で運ぶ。** 決定 (4)(5) は「印と門が同じ 1 つの関数で測る」ことが肝で、関数と印の欄と門の式を別の便に割ると、印に欄が在って門が読まない（または逆の）木が main に着地する。§1 (g) の余地の表のとおり、触る src の 6 本はどれも size M の見積 300 を上回る余地を持つ（最も狭いのは `crates/folio/src/anchor.rs` で参考値 467・本便はそこを 1 行しか触らない）。
- 改訂 b（2026-09-24・独立の検証 d126-verify.md〔条件付き支持・blocking 1・文面 6・低い重さ 4〕への直し）: 枝を main ee714d7（便 125 の着地の後）へ載せ替え（差分は本 file 1 本）、base と参考値を ee714d7 で数え直した（余地・写しの本数 20・区間の byte・歯の数 835・発効した判断の記録 19 本・生成区間の波及 5 本）。文面 1〜6 と低い重さ 1〜4 を全部当てた（§1 (b)(c)(e)(g)(h) と §0）。write-set（37 本）・歯の本数（9）・verify（10 行）は変えていない。done は歯 8 の見る字を足した。

## 1. 設計

### (a) いま起きていること（実測・base main ee714d7）

1. **引き金は型付きの欄として無く、門の式の中に在る。** 門（`crates/folio/src/gate.rs` の run）は、印の欄 sources（観点の reads が指す文書の file の全文を `<dir>` からの相対 path の byte 順に連結した sha256・同じ file の sources_digest）が今の正本の同じ値と違えば 2（印が古い）を返す。だから注や平易文が 1 字変わっても、設計文書を書き換える次の便の前に周が要る。印（`crates/folio/src/stamp.rs` の derive）は sources を門と同じ関数で測る（便 104）。印の欄は閉じた一覧 round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes で、引き金の欄は無い。
2. **天井の正本の生成区間に引き金の一覧は無い。** 天井の床の定数（`crates/folio/src/ceiling.rs` の FLOOR）の最上位は 16 欄（top_level から refute_note まで・注 8 つ）で、`folio schema --write` がそれを design-intent/ceiling.yaml の末尾の生成区間へ写す。凍結 anchor は `tests/fixtures/schema/ceiling-region.txt`（参考値 27 行・3,176 byte）。
3. **生成区間の写しは 21 か所。** design-intent/ceiling.yaml と、`tests/fixtures/` の下の ceiling.yaml 20 本（区間は 20 本とも design-intent と byte 一致）。組み直す手順（この 2 つの一覧が同じ 20 本になる）:

   ```
   find tests crates -name '*.yaml' | xargs grep -l '所見を出した文脈から独立して中立に検証する' | sort
   find tests -name ceiling.yaml | sort
   ```

   `crates/folio/tests/ceiling.rs` の歯 f102_every_ceiling_copy_carries_the_frozen_region_and_the_same_documents が、design-intent と `tests/fixtures` の下の ceiling.yaml を全部歩いて、生成区間が凍結 anchor と byte 一致することを数える。
4. **印の後に変わった節点を数える材料は印に在る。** 印の欄 nodes（節点ごとの要約値の表）と rest（残差の要約値）は `crates/folio/src/graph.rs` の stamp_table が組む（便 99）。門はいま nodes と rest を読まない。graph.rs は責務の層 3（導出する）、gate.rs は層 2（検査する）で、層の歯（`crates/folio/tests/modules.rs`）は下の層から上の層への名指しを 1 本も認めない（上がる辺の凍結の一覧は空）。
5. **憲法の条文の写しの関数は anchor.rs に在る。** 凍結 anchor の写し（`crates/folio/src/anchor.rs` の project）は、範囲の各節と、条の 5 欄（判断の記録の欄の決まりの床の定数 anchor.projection_article_fields = id・title・tier・binds・statements）と規範文の 4 欄（同 statement_fields = id・text・pattern・strength）を型付きの木で写す。非公開の関数である。
6. **門の対象の実測。** §0 の 門。

### (b) 直す先 — 規範の欄の閉じた一覧と引き金の要約値の関数

1. **一覧の正本は天井の床の定数。** `crates/folio/src/ceiling.rs` に、ADR-18 決定 (1) の 5 つを文書の id ごとの型付きの定数で置き、床の木 FLOOR の末尾（refute_note の後）に欄 trigger と注 trigger_note を足して、葉を同じ定数の配列に向ける（同じ一覧を 2 回書かない）。文書の id はどれも読む文書の閉じた集合（同じ file の DOCUMENT_IDS）に在る。
   - constitution: 条の欄 = id・title・tier・binds・statements、規範文の欄 = id・text・pattern・strength。**この 2 つは凍結 anchor の写しと同じ配列**にする: 判断の記録の欄の決まりの床の定数（`crates/folio/src/floor_adr.rs`）の anchor.projection_article_fields と statement_fields の字面の配列を、同じ file の名前つきの定数（crate の中だけで見える）に上げ、floor_adr.rs の床の木と ceiling.rs の trigger の両方がそれを指す（判断の記録の欄の決まりの生成区間の字は 1 byte も変わらない）。
   - adr: 状態の値 = 判断の記録の床の定数の値域 effective_status（accepted・retired・floor_adr.rs の EFFECTIVE_STATUS を指す）、欄 = id・status・decision・retreat・amends・revises・supersedes・superseded_by（決定 (1) ② の「置き換えの欄」は判断の記録の欄の決まりの supersedes と superseded_by）。
   - srs: 行の一覧の節 requirements と nonfunctional の欄 = id・shall・strength・pattern・when・verify・milestone、acceptance の欄 = id・title・verifies・red_test.sentence（点は入れ子の欄＝判定の文）、constraints の欄 = id・text、丸ごと取る節 = goals・scope・scope_m1・scope_m3（最上位の目的と範囲の節と M ごとの範囲の節）。
   - rules: 行の一覧の節 = thresholds・discipline、欄 = id・article・what・value・kind・status・stage・population・same_failure・projection。**決定 (1) ④ の主の字（注・裁定・来歴を除く中身の欄）に従い、括弧の列挙に無い projection（行 R-2 の注入の写しの取り方）も中身に数える。** 数えないのは note・ruling・ruled_at・refs と basis（実測の来歴）。
   - ceiling: 丸ごと取る節 = weights、documents の欄 = id・file、viewpoints の欄 = id・reader・question・reads（観点の名 name は数えない）。
2. **生成区間に入る 18 行（逐語・`folio schema --write` の導出と、凍結 anchor を直す独立の script の両方がこの字になる）。** 天井の正本の生成区間の refute_note の行の直後、終わりの印の行の直前に入る。区間の外は 1 byte も変わらない。

   ```
     trigger:
       constitution:
         articles: [id, title, tier, binds, statements]
         statements: [id, text, pattern, strength]
       adr:
         status: [accepted, retired]
         fields: [id, status, decision, retreat, amends, revises, supersedes, superseded_by]
       srs:
         requirements: [id, shall, strength, pattern, when, verify, milestone]
         nonfunctional: [id, shall, strength, pattern, when, verify, milestone]
         acceptance: [id, title, verifies, red_test.sentence]
         constraints: [id, text]
         whole: [goals, scope, scope_m1, scope_m3]
       rules:
         sections: [thresholds, discipline]
         fields: [id, article, what, value, kind, status, stage, population, same_failure, projection]
       ceiling: {whole: [weights], documents: [id, file], viewpoints: [id, reader, question, reads]}
     trigger_note: 周の引き金の閉じた一覧（規範の欄）。文書の id ごとに、節の名と各行から取る欄（点は入れ子の欄）。whole は節を丸ごと、adr は status の値の判断の記録ごとに fields、rules は sections の各行の fields、constitution は凍結 anchor の写しと同じ条の欄と規範文の欄。この写しを決まった順に並べた要約値が引き金の要約値で、印と門が同じ関数で測る。何にするかの裁定の正本は判断の記録 ADR-18 決定 (1)
   ```

   床の導出の体裁（`crates/folio/src/floor.rs` の幅 100 字の規則）で、この 18 行になる（起草役が幅の規則を写した小さな script と、捨てる試作の `folio schema --write` の両方で同じ 18 行を得た・区間は参考値 45 行・4,537 byte）。
3. **引き金の要約値の関数を gate.rs に 1 つ足す**（crate の中だけで見える・名は散文で trigger_digest・引数は置き場と天井の正本の読み手の型 Ceiling）。印（stamp.rs）と門（gate.rs）は必ずこの関数を呼ぶ（P-15.2・便 104 の sources_digest と同じ形）。式は次のとおりで、どの手順で失敗しても Err（理由 1 つ）を返す。
   1. 文書の id を天井の正本の documents の行（同じ file の documents）で file に解く。
   2. 各文書を型付きで読む（`crates/folio/src/cursor.rs` の load と同じ読み＝無い・読めない・UTF-8 でない・重複キー・parse できないは Err）。
   3. 写しの木を組む。最上位は文書の id を鍵にした表で、値は次のとおり。欄や節が無いときは null。行の一覧の節が一覧でない・行が表でないときは Err。
      - constitution: anchor.rs の project を範囲 articles だけで呼んだ結果（articles を鍵にした表）。**anchor.rs の変更は project を crate の中で見える関数にする 1 行だけ**で、写しの式は 1 字も変えない。この形では articles と条の statements が無いときは null でなく空の一覧になる（project の今の式・独立の実装もこれに合わせる）。
      - adr: 置き場の adr の file（dir 形）の直下の .yaml を、名 schema.yaml（判断の記録の欄の決まり・adr.rs と mentions.rs も外す）を除いて名の byte 順に読み、状態が adr の status の値のどれかで承認欄 approval が表のもの（anchor.rs の is_effective と同じ判定）だけを、fields の欄の表にして並べた一覧。**この判定は ADR-18 決定 (1) ② の字（状態が accepted か retired）より承認欄の 1 条件だけ狭いが、床が合格の木では同じ集合になる**（承認欄の無い accepted は床が落とす・凍結の土台 tests/fixtures/adr/effective-no-approval）。
      - srs と ceiling: 行の一覧の節は、節の名を鍵に、各行を欄の表にした一覧（行の順は正本の順）。whole の節は、節の名を鍵に、正本の木を丸ごと。点を含む欄（red_test.sentence）は入れ子を辿った値で、鍵は点を含む字のまま。
      - rules: sections の各節の名を鍵に、各行を fields の欄の表にした一覧。
   4. 写しの木を型付きの木の正規化（`crates/folio/src/yaml.rs` の canonical・キー順固定・空白なし・非 ASCII はそのまま・json の字面）の字にし、その byte の sha256 を「sha256 <16 進 64 字>」の形で返す（sources と同じ形）。
4. **folio2 自身で測れる。** 起草役が式を PyYAML で写した見積の script（repo に入れない・§1 (j)）で、base main ee714d7 の design-intent に (b) の 2 の 18 行を足した写しへ当てると組め、発効した判断の記録 19 本（参考値・ADR-19 までの 19 本）と 4 文書の写しを数えた。凍結の土台 `tests/fixtures/ceiling/bundle/source/` にも当てて組める（判断の記録 2 本はどちらも proposed なので adr は空の一覧）。

### (c) 直す先 — 印の欄と門の式

1. **印に欄 trigger を足す**（`crates/folio/src/stamp.rs` の derive）。欄 sources の直後の 1 行「trigger: sha256 <16 進>」で、値は (b) の 3 の関数の返り。印の欄の閉じた一覧は round・at・verdict・sources・trigger・faces・viewpoints・refutes・reads・rest・nodes になる（要件 FR20 の規範文の並びと同じ）。関数が Err なら、今の欄と同じく印を書かずに まだ分からない（終了 2）。他の欄の字と導出は 1 字も変えない（(f) の 4 の歯 stamp_writes_the_frozen_shape が凍結 anchor で確かめる）。file の頭の注の欄の一覧の字も同じ並びに直す。
2. **門の式**（`crates/folio/src/gate.rs` の run・判定の順は次のとおりで、最初に当たったもので決まる）。
   1. write-set に設計文書の正本が 1 つも無い → 0「設計文書の正本を書き換えない便」（今と同じ・同じ file の is_design_source は変えない）。
   2. 印が無い → 2「印が無い」。印が読めない → 2（今と同じ）。天井の正本が読めない → 2（今と同じ）。
   3. 印に欄 trigger が無い → 2「印が古い（引き金の要約値の欄が無い）」（本便の着地の直後の今の印はこれ・次の周の --stamp で付け直す）。
   4. 今の引き金の要約値が測れない → 2「引き金の要約値が測れない: <理由>」。
   5. 印の trigger と今の値が違う → 2「印が古い（引き金の要約値が違う）」。
   6. まだ分からない の観点が在る → 2、不合格 の観点が在る → 1（今と同じ字・同じ順＝まだ分からない が先）。
   7. 今の正本の要約値（sources_digest）が測れない → 2（今と同じ理由の字）。印の sources と同じ → 0「印が 4 観点とも合格・引き金の要約値が同じ・正本の要約値が同じ」。
   8. 違う → 印の欄 nodes と rest を読み、今の正本の節点ごとの要約値の表と残差の要約値と突き合わせる。印の nodes が読めない → 2「印の節点の表が読めない」。今の表が組めない → 2「印の後に変わった節点が数えられない: <理由>」（P-4.1＝数えられないものを数えずに通さない）。組めたら 0「印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る（節点 k 個・次の引き金の周が読む）」。k は、印の表と今の表の片方にだけ在る id と、両方に在って要約値が違う id の数の和。残差の要約値も違うときは括弧の中を「節点 k 個と節点の外の字・次の引き金の周が読む」にする（節点の外だけが変わると k が 0 になるので、0 個とだけ言って黙らない・P-4.2）。
   - 字「印が古い」と「正本の要約値が同じ」は今の理由の字に在り、残す（既存の歯と席の受付の手順が読む）。
   - **4・7・8 の 2（引き金の要約値が測れない・正本の要約値が測れない・節点の表が読めない / 組めない）は、要件 FR20 の 3 値の列挙の外で、条 P-4.1（実行できなかった結果を異常なしとして扱わない）による補完である。** FR20 の字だけを読むと合格で引き金が同じ場合は 0 と読めるが、測れないまま 0 を返すと、FR20 が 0 のときに課す節点の数の添え書きも書けない。
3. **今の節点の表は門の外から渡す。** 節点の表を組む関数（graph.rs の stamp_table・層 3）を gate.rs（層 2）から名指すと上がる辺になる。そこで門の口 run に、置き場と天井の正本から（残差の要約値・節点の表）を返す関数を受け取る引数を 1 つ足し、命令の入口（`crates/folio/src/main.rs`・層 5）が graph.rs の stamp_table を渡す（main.rs の変更は呼び出しの 1 行）。印の側（stamp.rs・層 3）は今どおり stamp_table を直に呼ぶ。層の歯（`crates/folio/tests/modules.rs`）の割り当てと上がる辺の一覧は変えない（新しい辺は gate→ceiling・gate→anchor・gate→cursor の同じ層か下向きだけ）。
4. **印の読み手の他の口は変えない。** 面の天井の名札が読む口（stamp.rs の marks）は欄 trigger を読まない（名札の添え書きは便イ）。

### (d) 採らなかった形

1. **引き金の一覧を正本 YAML の人が書く節に置く。** 決定 (1) が正本を実装の型付きの定数と定め、写しを生成区間に導出する（ADR-11 と同じ形）。採らない。
2. **引き金の要約値を行の逐語（byte）で測る。** 注の行の字下げや流れの形の書き換えで引き金が動く。欄の値を型付きの木で写して正規化する（凍結 anchor の写しと同じ式）。採らない。
3. **門が graph.rs の stamp_table を直に呼ぶ。** 上がる辺になり層の歯が落ちる（(c) の 3）。gate.rs を層 3 へ移すのは ADR-15 の層の割り当ての改訂になる。採らない。
4. **印の nodes が無ければ k を数えずに通す。** 読まれていない字の量を言えないまま通すことになる（P-4.2）。今の --stamp は nodes を必ず書くので、無いのは手で組んだ印だけである。採らない。
5. **憲法の条文の写しを schema 節と前文まで広げる（凍結 anchor の範囲 amendment_scope と同じ）。** 決定 (1) ① の字は条文の 5 欄で、前文と schema 節の改訂は条 A-2 の手順で判断の記録の発効（② の引き金）を伴う。採らない（撤退条件 (3) の穴が見つかれば持ち主に問う）。
6. **stamp-expected.yaml に引き金の要約値を凍結する。** 印の周の土台（`crates/folio/tests/stamp.rs` の Round）は観点の reads を組み替えた写しで、読む欄を違えた周（便 104 の歯）も同じ anchor を使う。reads は引き金の ⑤ に入るので周ごとに値が違う。anchor からは他の要約値と同じく落とし、値は門の歯（(e)）で独立の実装と突き合わせる。採らない。

### (e) 歯（新しく 9 本・関数名は f126_ で始める）

verify の絞り込みの語は f126_（base で `git grep -n 'f126_' -- crates` は 0 件）。

**凍結 anchor と独立の実装（P-10.1・P-10.2）。** 生成区間の凍結 anchor `tests/fixtures/schema/ceiling-region.txt` は、起草役の独立の置き換えの script（§1 (j)・folio を呼ばない）で (b) の 2 の 18 行を足して直し、`folio schema --write` の導出と byte 一致することを確かめる。**引き金の要約値の独立の実装**は歯の file `crates/folio/tests/gate.rs` に置く: 凍結 anchor の ceiling-region.txt の trigger の欄を yaml-rust2（folio が既に依存している crate・歯の file からも使える）で読み、写しの置き場の文書を yaml-rust2 で読んで (b) の 3 の式どおりに写しの木を組み、キー順固定・空白なし・非 ASCII そのままの json の字にして、外の命令 sha256sum（子の処理）で測る。folio の code を 1 行も呼ばない。

**歯の土台（門の歯）。** 今の歯の file と同じく、凍結の束の正本 `tests/fixtures/ceiling/bundle/source/` を一時 dir の design-intent/ へ写す。印は凍結 fixture stamp-pass / stamp-fail / stamp-unknown.yaml を写し、欄 sources と trigger の仮の値（64 字の 0）を、fresh のときは独立の実装で測った今の値に置き換える。節点の数を見る歯では、写しの置き場に命令 folio graph --print を撃って節点の行の 4 列目（要約値）から nodes の表を組み、rest の仮の値と一緒に印の末尾に足す（k の期待は編集の作りから決まる＝1 つの要件の平易文だけを変えれば 1）。

**門の歯（`crates/folio/tests/gate.rs`・6 本）。**

1. **注の側だけの編集は通し、変わった節点の数を添える。** fresh の印（nodes つき）を置き、要件書の写しの要件 FR1 の平易文 plain の字を変えて write-set に要件書 → 0・字「通す」「引き金の外の変更」「節点 1 個」。**base では正本の要約値が違うので 2（印が古い）＝ RED。**
2. **規範の欄の編集は 5 種とも古いと言う。** fresh の印を置き、写しごとに 1 か所ずつ変える: 憲法の規範文 P-1.1 の text／要件 FR1 の規範文 shall／規則の表の行 R-1 の what／天井の正本の観点 fidelity の問い question／判断の記録 ADR-2 の状態を accepted にして承認欄 approval の表を足す（発効） → どれも 2・字「印が古い」「引き金の要約値が違う」。**base では、天井の正本の問いの場合は 0（通す・この土台の観点は天井の正本を読まないので今の門は変化を見ない）、他の 4 場合は 2 だが理由の字が「印が古い」だけ ＝ RED。**
3. **引き金の外の編集は通す。** fresh の印（nodes つき）を置き、写しごとに 1 か所ずつ変える: 憲法の条 P-1 の平易文 plain／受入基準 AC1 の平易文／規則の表の行 R-1 の注 note／行 D-1 の裁定 ruling／要件書の meta の版 version／判断の記録 ADR-2（proposed）の決定 decision／設計ノート full.yaml の 1 字／天井の正本の documents の行 srs の注 note／観点 fidelity の名 name → どれも 0・字「通す」（読む文書の file を変えた 7 か所は字「引き金の外の変更」も・天井の正本だけを変えた 2 か所は字「正本の要約値が同じ」＝この土台の観点は天井の正本を読まない）。**base では読む文書の file を変えた 7 か所が 2（印が古い）＝ RED。**
4. **引き金の欄の無い印は古い。** sources だけを fresh にし、trigger の行を落とした印 → 2・字「印が古い」「引き金の要約値の欄が無い」。**base では sources が同じなので 0 ＝ RED。**（本便の着地の直後の実の印の形）
5. **引き金の要約値が測れなければ まだ分からない。** fresh の印を置き、憲法の写しの末尾に同じ最上位の鍵 articles の行を足す（重複キーで型付きに読めない・sources は byte を連結するだけなので測れる）→ 2・字「引き金の要約値が測れない」。**base では sources が違うので 2（印が古い）で、字が無い ＝ RED。**
6. **印に節点の表が無ければ通さない。** fresh の印（nodes なし）を置き、歯 1 と同じ平易文の編集 → 2・字「印の節点の表が読めない」。**base では 2（印が古い）で、字が無い ＝ RED。**

**印の歯（`crates/folio/tests/stamp.rs`・2 本・土台は今の周 Round）。**

7. **印は sources の直後に trigger を持つ。** 4 観点合格の周で --stamp → 最上位の欄の並びが round・at・verdict・sources・trigger・faces・viewpoints・refutes・reads・rest・nodes で、trigger の値が「sha256 」と 16 進の小文字 64 字。**base では欄 trigger が無い ＝ RED。**
8. **自分で書いた印を、注の側の編集の後も門が通し、規範の側の編集で古いと言う（印と門が同じ関数・本物の rest で残差の比較も縛る）。** 4 観点合格の周で --stamp → 門（write-set = 写しの要件書）は 0 で字「正本の要約値が同じ」→ 写しの要件書の meta の版 version だけを変えて門 → 0 で字「節点 0 個と節点の外の字」→ version を元の字に戻し、要件 FR1 の平易文を変えて門 → 0 で字「節点 1 個・次の引き金の周が読む」を持ち、字「節点の外の字」を持たない → 同じ要件の規範文を変えて門 → 2 で字「引き金の要約値が違う」。**base では version の編集の後が 2 ＝ RED。**

**床の定数の単体の歯（`crates/folio/src/ceiling.rs`・1 本）。**

9. **一覧の名がどれも実在の名を指し、範囲の節を取りこぼさない。** 関数名は f126_ で始め、字 ceiling_floor を含めない（含めると verify 9 の歯の数が 2 から 3 に変わる）。 trigger の文書の id がどれも DOCUMENT_IDS に在る／要件書の節の名（行の一覧の節と whole）がどれも要件書の最上位の節の閉じた一覧（`crates/folio/src/check.rs` の SRS_TOP_LEVEL）に在り、その一覧のうち goals と、scope か scope_ で始まる名が全部 whole に在る（M の範囲の節が増えたら落ちる）／規則の表の sections が規則の表の最上位の閉じた一覧（`crates/folio/src/rules.rs` の RULES_TOP_LEVEL）に在る／天井の正本の節の名が CEILING_TOP_LEVEL に在る／adr の status の値が判断の記録の床の定数の値域 effective_status と同じ並び／constitution の 2 つの配列が判断の記録の床の定数の anchor.projection_article_fields と statement_fields と同じ並び。各行から取る欄の名（milestone・verify・value ほか）が実在の欄かはこの歯は数えない。その字は凍結 anchor ceiling-region.txt との byte 一致（単体の歯 ceiling_floor_derives_the_frozen_anchor_byte_for_byte）が縛り、今の 18 行の欄の名は起草役と検証役が実の design-intent で全部実在すると数えた。**base では定数が無く組めない ＝ RED。**

**base で RED は 9 本とも。** 起草役は実装を組んでいないので、RED の根拠は上の各歯の base の返り（今の門の式・今の印の欄）から読んだ見込みである。歯の効きの見込み（検証役が実装の後に変異を当てて確かめる）:

| 当てた形 | 落ちる歯（見込み） |
| --- | --- |
| 門が今の sources の比較のまま | 1・3・4・8（と 2・5・6 の理由の字） |
| 引き金の写しが判断の記録を数えない | 2（判断の記録の発効の場合） |
| 引き金の写しが規則の表の what を数えない | 2（規則の表の場合） |
| 引き金の写しが注や平易文も数える | 1・3・8 |
| 印と門が別の関数で測る（印が sources の値を trigger に書く） | 8 と、門の歯の fresh の場合の全部（独立の実装の値と違う） |
| 印の nodes が無いときに k を数えずに通す | 6 |
| trigger の欄が無い印を通す | 4 |
| 一覧から scope_m3 を落とす | 9 |
| 残差の要約値を比べない・節点の外の字をいつも添える・k が 0 のときに 0 個とだけ言う | 8 |

**受入基準 AC18 の 7 場合と歯の対応。**

| AC18 の場合 | 持つ歯 |
| --- | --- |
| 設計文書の置き場の file を触らない → 0 | 今の gate_passes_a_delivery_that_touches_no_design_intent・gate_ignores_preview_and_retired_paths |
| 合格で同じ引き金の要約値 → 0 | 今の gate_passes_when_the_stamp_is_all_pass_and_fresh（補助が trigger を独立の実装の値にする）・歯 8 の 1 段目 |
| 合格で引き金の要約値は同じで正本の他の字が違う → 0（節点の数を添える） | 歯 1・3・8（新しい場合） |
| 不合格 → 1 | 今の gate_stops_on_a_failed_viewpoint |
| 引き金の要約値が違う → 2 | 今の gate_is_unknown_when_the_stamp_is_stale（trigger が仮の値）・歯 2・4・8 の最後の段 |
| まだ分からない の観点 → 2 | 今の gate_is_unknown_on_an_unknown_viewpoint |
| 印が無い → 2 | 今の gate_is_unknown_without_a_stamp |

凍結の束と所見から導出した印が凍結の写しと一致することは、今の stamp_writes_the_frozen_shape が持つ（欄 trigger は周ごとに違うので落として比べる・§1 (d) の 6）。

### (f) 既存の歯のうち落ちるもの・凍結 anchor が動くか・直し方

1. **生成区間の写しの波及（実測・捨てる試作）。** 起草役が base main ee714d7 の写しの木に、床の定数に (b) の 2 の欄を足す変更だけを当て、`folio schema --write` と独立の置き換えの script で生成区間 21 か所と凍結 anchor を直して workspace の nextest を撃つと、**落ちる既存の歯は 5 本**（参考値・base の 835 本のうち・830 本は緑）。同じ木の `folio check --dir design-intent` は合格（違反 0・まだ分からない 0）。起草の土台 694751e で撃ったときに環境で落ちた 4 本（ADR-17 の着地の前の木を着地の後の版管理の中で撃ったことによる違反 1 件）は、ee714d7 では出ない。

   | 歯 | 直し方 |
   | --- | --- |
   | `crates/folio/tests/schema_docs.rs` の schema_check_matches_the_real_ceiling_file_and_its_frozen_digest・schema_check_fails_on_one_byte_drift_inside_the_ceiling_region・schema_write_restores_the_ceiling_region_and_is_idempotent | 天井の正本の生成区間の定数 CEILING_REGION_ の行数・byte 数・要約値の 3 つを、直した anchor を `wc -l -c` と sha256sum で測り直した値に（参考値 45 行・4,537 byte）。頭の注に便 126 の 1 行を足す。**この歯の file は歯 f89_schema_teeth_are_split_and_under_the_cap が器の式で 1,200 行以下に抑える（base 参考値 1,144）ので、足す注は 2 行までにする** |
   | `crates/folio/tests/graph.rs` の f99_the_independent_script_matches_the_anchor | 凍結 anchor `tests/fixtures/schema/node-digest-anchor.txt` を独立の script `tests/fixtures/schema/node-digest.py` で床の凍結の土台から組み直し（土台の天井の正本は観点が天井の正本を読むので、その file は残差に入る）、定数 F99_ANCHOR_SHA256 を sha256sum で測り直した値に。動くのは anchor の末尾の 2 行（残差の要約値と byte 数の行）だけで、行数と byte 数は変わらない（参考値 192 行・3,007 byte・残差と合計の byte の桁は変わらない）。定数の上の注に便 126 の 1 行を足す |
   | `crates/folio/src/ceiling.rs` の単体の歯 ceiling_floor_notes_are_outside_the_diff | 床の木の注の数の期待を 8 から 9 に（trigger_note）。床の突き合わせは注を読まないことを見る歯の本文は変えない |

2. **印と門の式の波及（見込み）。** 実装の後に、歯の file の補助を直さないと落ちる既存の歯は 6 本の見込み。
   - `crates/folio/tests/gate.rs` の gate_passes_when_the_stamp_is_all_pass_and_fresh・gate_stops_on_a_failed_viewpoint・gate_is_unknown_on_an_unknown_viewpoint（印の trigger が仮の値のままだと 2 になる）→ 印を置く補助が fresh のときに trigger も独立の実装の値に置き換える（歯の本文は変えない）。gate_is_unknown_when_the_stamp_is_stale は trigger も仮の値のまま置くので「印が古い（引き金の要約値が違う）」で緑のまま。
   - `crates/folio/tests/stamp.rs` の f99_the_stamp_carries_the_node_table（最上位の欄の並びの期待に trigger を足す）・stamp_writes_the_frozen_shape と f104_the_stamp_survives_viewpoints_reading_different_fields（印から要約値を落とす補助が trigger の行も落とす・(d) の 6）。**凍結 anchor stamp-expected.yaml は 1 byte も動かない**（歯 stamp_writes_the_frozen_shape と f99_the_stamp_carries_the_node_table の行数と byte 数の期待 34 行・1,342 byte がそれを確かめる）。
   - f104_the_gate_passes_the_stamp_it_just_wrote は理由の字「正本の要約値が同じ」を残すので緑のまま。
3. **凍結 fixture の印 3 本**（`tests/fixtures/ceiling/findings/` の stamp-pass・stamp-fail・stamp-unknown.yaml）に、sources の行の直後の 1 行「trigger: sha256 」と 0 の 64 字を足す（仮の値・歯の補助が置き換える）。ほかの行は変えない。
4. **動く凍結 anchor は 2 本**（ceiling-region.txt と node-digest-anchor.txt）と、生成区間の写しの 20 本。ほかの凍結 anchor（束の anchor bundle-anchor.txt・stamp-expected.yaml・判断の記録と設計ノートの欄の決まりの写し・面の凍結 fixture・索引の anchor・floor_base の要件書と規則の表）は 1 byte も動かない。試作で束の歯（bundle）・所見の歯（findings）・面の歯・床の組（floor_cases）が緑のままだったことを確かめた（凍結の束の観点は天井の正本を読まないので、生成区間の変化は束に入らない）。判断の記録の欄の決まりの生成区間（adr/schema.yaml）は floor_adr.rs の配列を名前つきの定数に上げても字が変わらない＝その凍結 anchor の単体の歯 adr_floor_derives_the_frozen_anchor_byte_for_byte が共通の検証で確かめる。
5. 本便の後の木で workspace の nextest は全部緑（参考値 844 本 = base 835 + 新しい歯 9）・clippy 0 警告・`folio check` 合格・`folio schema --check` 9 file 一致・`folio inject --check` 0。

### (g) 大きさ・verify と done の対応

1. **write-set の印。** 37 本とも印なし（新しい file・消す file・縮む file は無い）。書き換える src 6 本・歯の file 5 本（うち本文を変えないのは `crates/folio/tests/ceiling.rs` の 1 本）・凍結 fixture の印 3 本・凍結 anchor 2 本・天井の正本 1 本・生成区間の写し 20 本。新しい dir は作らない。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない・script は §1 (j)）。size M の見積は 1 file あたり 300。

   | file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 本便の後の見込み |
   | --- | --- | --- | --- |
   | `crates/folio/src/gate.rs` | 251 | 1,249 | 約 450〜500（引き金の関数・判定の 8 段・節点の数え） |
   | `crates/folio/src/stamp.rs` | 341 | 1,159 | 約 345 |
   | `crates/folio/src/ceiling.rs` | 557 | 943 | 約 620 |
   | `crates/folio/src/anchor.rs` | 1,033 | 467 | 1,033（可視性の 1 行） |
   | `crates/folio/src/floor_adr.rs` | 511 | 989 | 約 515 |
   | `crates/folio/src/main.rs` | 655 | 845 | 655（呼び出しの 1 行） |

3. **size は M。** 増えるのは gate.rs（引き金の要約値の関数・印の欄の読み・門の判定の 8 段・節点の数え）と ceiling.rs（型付きの定数・床の木の 2 欄・単体の歯 1 本）。歯の file は src の外なので余地を測らない（gate.rs の歯の file は独立の実装と 6 本で約 250 行増える見込み）。
4. **verify は 10 行**で、done の 10 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test gate f126_` = (e) の門の歯 6 本。
   2. `cargo nextest run -p folio --test stamp f126_` = (e) の印の歯 2 本。
   3. `cargo nextest run -p folio --bin folio f126_` = (e) の床の定数の単体の歯 1 本（folio は実行 file だけの crate なので `--bin folio`）。
   4. `cargo nextest run -p folio --test gate` = 門の歯の全部（今の 7 本と新しい 6 本）。
   5. `cargo nextest run -p folio --test stamp` = 印の歯の全部（今の 11 本と新しい 2 本・凍結 anchor stamp-expected.yaml の一致を含む）。
   6. `cargo nextest run -p folio --test schema_docs` = 生成区間の歯の全部（天井の正本の定数を直した 3 本と歯 f89 の行数の上限を含む）。
   7. `cargo nextest run -p folio --test ceiling` = 天井の正本の床の歯の全部（写しが全部凍結 anchor と byte 一致する歯 f102 を含む）。
   8. `cargo nextest run -p folio --test graph f99_` = 節点の要約値の歯（node-digest-anchor.txt と独立の script の一致を含む）。
   9. `cargo nextest run -p folio --bin folio ceiling_floor` = ceiling.rs の単体の歯 2 本（床の木の導出が凍結 anchor と byte 一致・注の数）。
   10. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す gate・stamp・schema_docs・ceiling・graph の 5 本は全部 write-set に在る（ceiling は本文不変でも置く）。`--bin folio` の絞り込みの語 f126_ と ceiling_floor を関数名に持つ src は `crates/folio/src/ceiling.rs` だけ（base で `git grep -n 'fn [a-z0-9_]*ceiling_floor' -- crates/folio/src` は 2 件・ともに ceiling.rs。f126_ は 0 件で、単体の歯 9 は ceiling.rs に置く）で、write-set に在る。

### (h) 門（規則の表の開発規律行 D-12）

本便は天井の正本の生成区間を書き換えるので門の対象で、実測は base main ee714d7 で **2（まだ分からない・印が古い）**（§0 の 門）。行 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。**受付の時点の門が 2（印が古い）なら、行 D-12 の持ち主の裁定の前例（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes）を席が当てて受付ける**（受付の時点の門が 0 なら当てない）。同じ前例を当てたのは便 101・103・117・119・123 である。席は受付のときに、当てた前例の裁定 id を台帳の notes に記帳する。本便が書き換える正本の中身は天井の正本の生成区間の 18 行（床の定数の写し）だけで、人が書く字は 1 字も変わらない。**判断の記録 ADR-19 決定 (4) は、この前例の裁定を本便の着地の時点で置き換える**（着地の後の設計文書の便は、新しい門の式で判定する）。行 D-12 の注は「便アの着地までは…門が まだ分からない を返せば、引き金の外の変更の後でも周を回す（門を外す口は持たない）」と書くが、**行 D-14 の注（main 2df6d80・前例は ADR-18 の便アの着地まで有効で、着地の時点で D-14 の窓の定めに置き換わる）が、本便の受付についてはその「周を回す」より先に立つ**。

### (i) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 面の天井の名札の添え書き（便イ・便 127・`crates/folio/src/face.rs`）。完成の判定（ADR-18 決定 (6)・機械の口は無く、印の sources の比較の意味は変えない）。手順の引き金 2 つ（決定 (3)・規則の表の行 D-12 と席の作法）。実の印 design-intent/preview/ceiling-stamp.yaml の付け直し（生成物・次の周で席が --stamp で書く）。要件 FR20 の注と天井の正本の頭の注の「口は後続の便で入る」の字（人が書く字・着地の後に席が直す）。門の単位を便の file の単位へ狭める口（台帳 f2-648.146・決定 (7) のとおり本便の着地の後に要否を問い直す）。外部 crate。
2. **言えないこと。** 歯の独立の実装は yaml-rust2（YAML 1.2 の読み）で読み、folio の型付きの読みは PyYAML と同じ読み（YAML 1.1）に揃えてある。yes・no・on・off のような字が規範の欄の値そのものに現れると 2 つの読みが割れるが、凍結の土台 bundle/source の規範の欄には無い（歯の土台の中でだけ比べる）。実の design-intent の値の割れは歯では言えない（folio2 自身の引き金の要約値は印と門が同じ関数で測るので、割れても判定は揺れない）。
3. **まだ分からない — 節約の実測。** 周がいくつ減るかは ADR-18 の撤退条件の窓（本便の着地の後に引き金の要約値を持つ印が初めて付いた時刻から 7 日）で席が数える。本便の歯では言えない。
4. **撤退条件。** (1) 本便の後に (f) の 1 と 2 に挙げた歯のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(2) 実の design-intent で引き金の要約値が組めない（関数が Err を返す）なら止めて席へ返す（着地の後の門が全部 まだ分からない になる）。(3) 受付の時点の main で、天井の正本の生成区間の写しの数（§1 (a) の 3 の find）が 20 本でない・gate.rs の run か stamp.rs の derive か ceiling.rs の FLOOR の末尾か graph.rs の stamp_table の返りの形が書き換わっている・要件 FR20 の規範文か受入基準 AC18 が第 1.36 版の字と違う、のどれかなら、写しを数え直し、式を測り直してから運ぶ。(4) 判断の記録 ADR-18 の発効の字（決定 (1) の 5 つの一覧）が base の字と違って取り込まれたら、(b) の 1 と 2 の一覧と 18 行を改訂してから運ぶ。

### (j) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の `.local/share/folio2/handoff-2026-09-24/d126-draft.md`、生成区間の置き換えの script は同じ dir の d126-draft-anchor.py（足す 18 行は d126-draft-region.txt）、引き金の要約値の見積の script は d126-draft-trigger.py（PyYAML・repo には入れない）、行数の script は d119-draft-lines.py（便 119 と同じ式）。

1. base の写し: `git worktree add --detach <写し> ee714d7`（改訂 b の土台・起草の土台は 694751e）。
2. 写しの数: §1 (a) の 3 の find（20 本）。
3. 生成区間の独立の置き換え: `python3 d126-draft-anchor.py d126-draft-region.txt <写し 20 本> tests/fixtures/schema/ceiling-region.txt`。床の定数を直した binary の `folio schema --dir design-intent --write` の区間と byte 一致を見る。
4. 節点の anchor: `python3 tests/fixtures/schema/node-digest.py tests/fixtures/floor_base/design-intent`（直した写しの土台で・末尾の 2 行だけが動く）。
5. 引き金の要約値の見積: `python3 d126-draft-trigger.py <置き場>`（3 の後の、生成区間を直した置き場で・design-intent と bundle/source。直す前の置き場には欄 trigger が無いので読めない）。
6. RED: 歯の file 3 本（gate・stamp の f126_ と ceiling.rs の単体の歯）だけを当てて verify の 1〜3 を撃つ（9 本とも落ちるか組めない）。
7. 門: `folio ceiling --gate --dir design-intent --write-set <write-set の 37 本>`。
8. 余地: `python3 d119-draft-lines.py <src 6 本>`。

## 2. 範囲

- 入れる: `crates/folio/src/ceiling.rs` に規範の欄の型付きの定数と床の木の 2 欄（trigger・trigger_note）と単体の歯 1 本・注の数の期待。`crates/folio/src/floor_adr.rs` の条の欄と規範文の欄の 2 つの配列を名前つきの定数に上げる（床の木の字は同じ）。`crates/folio/src/anchor.rs` の project を crate の中で見える関数にする 1 行。`crates/folio/src/gate.rs` に引き金の要約値の関数・印の欄 trigger・nodes・rest の読み・門の判定の順（§1 (c) の 2）・節点の数え・節点の表を受け取る引数。`crates/folio/src/stamp.rs` の印の欄 trigger の 1 行と頭の注。`crates/folio/src/main.rs` の門の呼び出しの 1 行。天井の正本の生成区間 21 か所（design-intent と fixture の写し 20 本）と凍結 anchor 2 本。凍結 fixture の印 3 本の仮の行。歯の file 4 本の新しい歯と補助と定数。
- 入れない: 面の天井の名札（便イ）。完成の判定・手順の引き金。実の印の付け直し。天井の正本の人が書く節（meta・weights・documents・viewpoints）と頭の注。要件書・判断の記録・憲法・規則の表・語彙の人が書く字。層の割り当ての表と上がる辺の一覧（modules.rs）。門の単位の狭め（f2-648.146）。新しい file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| list | 規範の欄の一覧 | `crates/folio/src/ceiling.rs` の型付きの定数と床の木の欄 trigger（正本）と、天井の正本の生成区間の写し |
| digest | 引き金の要約値 | `crates/folio/src/gate.rs` に足す関数（印と門が共有する唯一の式・凍結 anchor の写しの式 project を借りる） |
| stamp | 印の欄 | `crates/folio/src/stamp.rs` の derive が sources の直後に書く trigger の 1 行 |
| gate | 門の式 | `crates/folio/src/gate.rs` の run の判定の順と理由の行（節点 k 個の添え書き・節点の表は main.rs が渡す） |
| copies | 写しと anchor | 生成区間の写し 21 か所・凍結 anchor 2 本・凍結 fixture の印 3 本 |
| teeth | 歯 | f126_ の 9 本と、直す既存の歯（生成区間の 5 本と印と門の補助で直る 6 本） |

## 4. 検査（歯）

§1 (e)(f) と (g) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない（歯の独立の実装は folio が既に依存する yaml-rust2 と外の命令 sha256sum を使う）。新しい dir は無い。
- 前提の着地: 判断の記録 ADR-18 の発効（main 16ed6a2・要件書 第 1.36 版・規則の表の行 D-12・語彙の周の引き金・天井の正本 v0.15）と ADR-19 の発効（main 2df6d80・行 D-14・要件書 第 1.37 版）。どちらも着地済み。§0 の 前の便。
- 並行の便: 便 125（④-2）は ee714d7 で着地済みで、本便の base に入っている（`crates/folio/src/main.rs` の重なりは解けた）。重なる file は受付の時点の main で字面を測り直す（§1 (i) の撤退条件 (3)）。便イ（便 127）は本便の後。
- 本便の着地の後に席が見ること: 実の印は引き金の欄を持たないので、設計文書の便の門は 2（印が古い〔引き金の要約値の欄が無い〕）を返す＝**次の周で --stamp を撃ち直して印を付け直す**（ADR-18 決定 (7) の順）。ADR-19 決定 (4) で 2026-09-22 の裁定は着地の時点で置き換わるので、付け直すまで設計文書の便は止まる。要件 FR20 の注と天井の正本の頭の注の「口は後続の便で入る」の字（次の版）。台帳 f2-648.145 の置き換えと f2-648.146 の問い直し。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dy"
title = "判断の記録 ADR-18 決定 (1)(4)(5)(7) の便ア（裁定 id = 台帳 f2-648 notes 2026-09-24 11:51 JST・案 a）: 天井の周の引き金を規範の欄の要約値で測る。crates/folio/src/ceiling.rs に規範の欄の 5 つの閉じた一覧（憲法の条文の 5 欄と規範文の 4 欄・発効した判断の記録の決定と撤退条件と改訂と置き換えの欄・要件と非機能要件の規範文ほか 7 欄と受入基準の判定の文ほか 4 欄と制約と目的と範囲の節・規則の表の行の注と裁定と来歴を除く中身の欄・天井の正本の重さと文書の行の id と file と観点の読み手と問いと読む欄）を型付きの定数で置き、床の木の末尾に欄 trigger と注を足して、folio schema --write で天井の正本の生成区間へ 18 行を写す（写し 20 本と凍結 anchor ceiling-region.txt は独立の script で揃える・憲法の 2 つの配列は crates/folio/src/floor_adr.rs の同じ配列を名前つきの定数に上げて指す）。crates/folio/src/gate.rs に引き金の要約値の関数を 1 つ足し（文書を型付きで読み、一覧の欄だけを写した木を正規化して sha256・憲法は crates/folio/src/anchor.rs の凍結 anchor の写しの関数を crate の中で見える形にして借りる）、印（crates/folio/src/stamp.rs）は sources の直後に trigger の 1 行を書き、門はこの関数で古さを判定する: 印に trigger が無いか今と違えば まだ分からない（印が古い）、測れなければ まだ分からない、まだ分からない と不合格の観点の順は今のまま、合格で正本の要約値も同じなら通す、正本の要約値だけが違えば印の nodes と rest を今の表（crates/folio/src/main.rs が graph.rs の stamp_table を渡す）と突き合わせ、印の後に引き金の外の変更が在る（節点 k 個・次の引き金の周が読む）を理由の行に添えて通す（節点の表が読めない・組めないは まだ分からない）。歯は f126_ の 9 本（crates/folio/tests/gate.rs に 6 本・引き金の要約値の独立の実装を yaml-rust2 と sha256sum で持つ／crates/folio/tests/stamp.rs に 2 本／ceiling.rs の単体の歯 1 本）で、直す既存の歯は生成区間の定数の 5 本（schema_docs の 3 本・graph の f99 の 1 本と node-digest-anchor.txt・ceiling.rs の注の数の 1 本）と、印と門の歯の補助で直る 6 本。凍結 fixture の印 3 本に trigger の仮の行を足す。受付は ADR-18 の発効が main に着地した後で、受付の時点の門が印が古いなら行 D-12 の持ち主の裁定の前例（2026-09-22）を当てる"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/gate.rs", "crates/folio/src/stamp.rs", "crates/folio/src/ceiling.rs", "crates/folio/src/anchor.rs", "crates/folio/src/floor_adr.rs", "crates/folio/src/main.rs", "crates/folio/tests/gate.rs", "crates/folio/tests/stamp.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/ceiling.rs", "crates/folio/tests/graph.rs", "tests/fixtures/ceiling/findings/stamp-pass.yaml", "tests/fixtures/ceiling/findings/stamp-fail.yaml", "tests/fixtures/ceiling/findings/stamp-unknown.yaml", "tests/fixtures/schema/ceiling-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "design-intent/ceiling.yaml", "tests/fixtures/adr/effective-no-approval/ceiling.yaml", "tests/fixtures/adr/schema-drift/ceiling.yaml", "tests/fixtures/adr/two-adopted/ceiling.yaml", "tests/fixtures/anchor/no-anchor/ceiling.yaml", "tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "tests/fixtures/ceiling/bundle/source/ceiling.yaml", "tests/fixtures/check/dup-key/ceiling.yaml", "tests/fixtures/check/empty-field/ceiling.yaml", "tests/fixtures/check/missing-file/ceiling.yaml", "tests/fixtures/check/unknown-section/ceiling.yaml", "tests/fixtures/face/ceiling.yaml", "tests/fixtures/floor_base/design-intent/ceiling.yaml", "tests/fixtures/link/adr-id-missing/ceiling.yaml", "tests/fixtures/link/amended-by-orphan/ceiling.yaml", "tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "tests/fixtures/refs/bad-counts/ceiling.yaml", "tests/fixtures/refs/dangling-id/ceiling.yaml", "tests/fixtures/refs/orphan-rule/ceiling.yaml", "tests/fixtures/vocab/exemptions/ceiling.yaml", "tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio --test gate f126_", "cargo nextest run -p folio --test stamp f126_", "cargo nextest run -p folio --bin folio f126_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test ceiling", "cargo nextest run -p folio --test graph f99_", "cargo nextest run -p folio --bin folio ceiling_floor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f126_ の門の歯 6 本（注の側だけの編集は通し節点 1 個を添える／規範の欄の 5 種の編集は引き金の要約値が違うで まだ分からない／引き金の外の 9 か所の編集は通す／trigger の無い印は印が古い／引き金の要約値が測れなければ まだ分からない／印に節点の表が無ければ通さない）が緑、f126_ の印の歯 2 本（印は sources の直後に trigger を持つ／自分で書いた印を、meta の版だけの編集の後は節点 0 個と節点の外の字で通し、平易文の編集の後は節点 1 個で節点の外の字を添えずに通し、規範文の編集で引き金の要約値が違うと言う）が緑、f126_ の床の定数の単体の歯 1 本（一覧の名がどれも実在の文書と節を指し範囲の節と凍結 anchor の写しの配列と発効の値域を取りこぼさない）が緑、門の歯の全部（今の 7 本を含む）が緑、印の歯の全部（凍結 anchor stamp-expected.yaml の一致を含む）が緑、生成区間の歯の全部（schema_docs・天井の正本の定数を直した 3 本と行数の上限を含む）が緑、天井の正本の床の歯の全部（写しが全部凍結 anchor と byte 一致する歯を含む）が緑、節点の要約値の歯 f99_ の全部（直した node-digest-anchor.txt と独立の script の一致を含む）が緑、ceiling.rs の床の木の単体の歯 2 本（凍結 anchor と byte 一致・注の数）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、folio check と folio schema --check と folio inject --check が着地の後の main で 0 を返す"
<!-- contracts:end -->
