# 設計: 便 123 — 器の導出 file を版管理の根で解く（床と面の生成器で 1 つの式）と、生成区間の注から条 id N-3 を外す（ADR-16 決定 (7) の ④ の 1 便目）

- 要件: FR10（契約表の欄は器の導出 file から読む・注は版 B〔要件書 第 1.34 版〕で「探す場所を置き場の親の固定から版管理の根へ・無ければ親に倒す・床と面の生成器で 1 つの関数を共有」と書いた）/ FR19（床の定数から欄の決まりの file の生成区間へ写す＝本便は 5 file の生成区間の注の字を変える）。規範文はどれも変えない。契約表の行の req は FR10 と FR19 の 2 つ（どちらも main に在る id）。
- 条: P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する＝解く式を 1 つにする）/ P-4.1・P-4.2（読めないものを合格にせず「まだ分からない」に出す）/ P-5.6（実装の型付きの定数の写しを設計文書の置き場へ決定的に導出する）/ P-10.1（独立した凍結 anchor）/ P-7.1（条の番号を利用者の側で予約しない＝利用者の床が数える要件書と語彙の生成区間の注が folio2 の条 id を名指さない）
- 出所: 判断の記録 **ADR-16**（発効 2026-09-23 17:39 JST・持ち主の承認・対話面 R-8・逐語「裁定は両方承認する」・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (2)(キ)「器の導出 file の解決先を、検査する置き場の親の場所の固定から、置き場を含む版管理の根（git toplevel）へ改める。床の読み（note.rs）と面の生成器の同じ読み（face_note.rs）を同じ便で直す。解く関数は 1 つにして床と面の生成器で共有し、版管理の根が在ればそこ、無ければ置き場の親に倒す」と決定 (2)(オ)「生成区間の注から条 id を外す（注は床の定数の字面なので folio2 の便で直す・folio2 の床の結果は変わらない）」と、決定 (7) の便の列の **④「骨格の命令・案内の 1 行・生成区間の注から条 id を外す・器の導出 file を版管理の根で解く〔床と面の生成器の 2 か所〕（中・1〜2 便）」** の 2 便の側の 1 便目。2 便目（骨格の命令 folio init と案内の 1 行）は `docs/design/delivery-125.md`（行 `dx`）。規則の表の開発規律行 D-11 は、この行と契約表の行の title が ADR-16 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dv` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 23 本。**新しい file は 1 本で、先頭に `+` を付けて宣言する**（歯の file 1）。縮む file も消す file も無く、新しい dir も作らない。
- 門: 本便は要件書・語彙・規則の表・相談窓口・索引の欄の決まりの 5 file の生成区間を書き換えるので天井の門の対象である。起草役が write-set 23 本を `folio ceiling --gate --write-set …` に渡すと **2（まだ分からない・断りの字は 印が古い）**（2026-09-24・本便の後の binary・base 2d90e8d）。base の binary に要件書 1 本だけを渡しても同じ 2 なので、印は本便の前から古い。src の 1 本（note.rs）だけを渡すと 0（通す・設計文書の正本を書き換えない便）。その後 main a70477e（天井の 28 周目の印の付け直し・4 観点 合格）で同じ write-set 23 本を撃つと **0（通す・印が 4 観点とも合格・正本の要約値が同じ）** になった（起草役の実測・本便の後の binary）。受付の時点の main で撃ち直す（便 ② ③ が正本の生成区間を書き換えると印は再び古くなりうる）。**席の裁定（2026-09-24）: 受付の時点で門が「印が古い」なら、規則の表の行 D-12 の持ち主の裁定の前例（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes・便 101・103・117・119 で当てた裁定）を当てて受付ける。** 当てたときは、受付のときに席がこの前例の裁定 id を台帳の notes に記帳する（§1 (g)）。
- 前の便: 便 119（行 `dr`・導出物の独立の命令 folio derive）と便 120（行 `ds`）は着地済み。**base = main 8066e4c（改訂 c で rebase した）。crates・tests・contracts は 2d90e8d から 1 字も変わらない（git diff で確かめた）ので、この契約の数は base 2d90e8d の実測（参考値・検証役が a70477e で再現）をそのまま使い、受付の時点の main で数え直す**（規則の表の行 D-13）。
- 受付の順序: **版 B（要件書 第 1.34 版）は main 2d90e8d に在る**（FR10 の注が本便の形を書く版）。ADR-16 決定 (7) の順に、**便 ②（`docs/design/delivery-121.md`〔と 124〕・列の根の表と始まりの凍結）→ 便 ③（`docs/design/delivery-122.md`・値域の部分集合）→ 本便 → 便 125（④-2）** の順で受付ける。門は §0 の 門 の席の裁定（行 D-12 の前例）で受付ける。本便の write-set のうち `crates/folio/src/check.rs` と `tests/floor_cases.yaml` は便 ③ も触る。本便が check.rs で変えるのは要件書と語彙の生成区間の床の木の注の字 2 つだけで、ほかの行は 1 字も変えない（§1 (h) の 5）。便 ③（行 `du`・枝 docs/d122 d97173c）が check.rs で触るのは頭の注・use の 1 行・憲法の段の検査の関数群と新しい関数で、要件書と語彙の床の木は触らない。`tests/floor_cases.yaml` で便 ③ が触るのは共有の節（shared）の 3 か所で、本便が触るのは組 nested-git-root の 1 行（§1 (f)）＝hunk は重ならない。起草役の実測: base 2d90e8d に便 ② ③ の起草の patch（d121-measured.patch・d122-measured.patch）と本便の patch を順に当てると衝突なしに当たり、workspace の nextest は 816/816（参考値・検証役の再現）。重なる file が在れば受付の時点の main で字面を測り直す（§1 (i) の撤退条件 (3)）。
- 分割: ADR-16 決定 (7) の ④ を 2 便に割る。本便（④-1・小）= 解決先の式 + 注から条 id を外す。便 125（④-2・中）= 骨格の命令 folio init + 案内の 1 行。席の推奨の割り方は案内の 1 行を ④-1 に置いていたが、**案内の 1 行は便 125 に移した**: 本便だけが着地した木で案内の 1 行が folio init を名指すと、まだ無い命令を案内する期間ができる（P-4.1 の向き・行き止まりを直す便が別の行き止まりを作る）。案内の 1 行の受入基準 AC27 も FR22 を確かめる受入基準である。

## 1. 設計

### (a) いま起きていること（実測・base main 2d90e8d）

1. **器の導出 file の解決先は置き場の親に固定されている。** 床の読み手（`crates/folio/src/note.rs` の load_external）と面の生成器の同じ読みの写し（`crates/folio/src/face_note.rs` の load_external）が、それぞれ別に、置き場（--dir）の親 dir の `contracts/schema.toml` を読む。式は 2 か所に在り、字面の写しである。導出の命令（`crates/folio/src/derive.rs`）は床の読み手を共有している。
2. **置き場を版管理の根の直下に置かないと読めない。** 版管理の根に器の導出 file を置き、置き場を 2 段下（例 design-intent/folio2）に置くと、床は「contracts/schema.toml: 器の導出 file が読めない」の「まだ分からない」を返す（判断の記録 ADR-16 文脈 (3)(キ)・(4) の実測・起草役の再測でも同じ）。
3. **版管理の根を解く口は版管理の照合の側に在る。** `crates/folio/src/gitcheck.rs` の照合が命令 git の rev-parse --show-toplevel を子 process で撃ち、置き場そのものが版管理の根なら違反（床の script と別の版管理では照合にならない）、根が置き場の上に無ければ違反、git が無い・読めないは「まだ分からない」にする。git の撃ち方（環境変数 GIT_* を渡さない・待ち上限 20 秒）はこの file の小さな関数 1 つが持つ。
4. **生成区間の注が条 id を名指す。** 最上位の節の閉じた一覧の注 top_level_note の字「最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。」が、要件書・語彙・規則の表・相談窓口・索引の欄の決まりの 5 つの床の木（`crates/folio/src/check.rs` の要件書と語彙の床の木・`crates/folio/src/rules.rs`・`crates/folio/src/intake.rs`・`crates/folio/src/graph.rs`）に同じ字で在り、5 file の生成区間と凍結 anchor 5 本（`tests/fixtures/schema/` の下の srs-region.txt・vocabulary-region.txt・rules-region.txt・intake-region.txt・graph-region.txt）に写っている。組み直す手順:

   ```
   git grep -n 'ほかの節は床が落とす・N-3' -- crates design-intent tests/fixtures
   ```

5. **床が数えるのは要件書と語彙の 2 つだけ。** 参照 id の解決（`crates/folio/src/refs.rs`）は要件書と語彙の全部（生成区間を含む）を母集団に数え、規則の表は生成区間を除き、相談窓口と索引の欄の決まりは数えない。起草役が骨格の試作（便 125 の §1 (c)・条 1 本の置き場）に base の binary で生成区間を書いて床を撃つと、**違反はちょうど 2 件**（要件書と語彙の schema.top_level_note の「id N-3 が実在しない」）で、本便の後の binary で同じ試作を撃つと違反 0（まだ分からないは凍結 anchor の 2 つの基準の不在だけ）。
6. base の workspace の nextest は全部緑（参考値 796 本・`cargo nextest run --workspace --no-tests=fail --no-fail-fast`）。

### (b) 直す先 — 解決の式を 1 つにする

1. **版管理の根を返す小さな関数**を `crates/folio/src/gitcheck.rs` に足す（crate の中だけで見える）。置き場を正規化し、同じ file の git の撃ち方で rev-parse --show-toplevel を撃ち、終了コード 0 の出力の末尾の改行を落として正規化した path を返す。git が無い・版管理の外・待ち上限を超えた・正規化できない は「無い」を返す。照合の本体（check_git）は 1 字も変えない。
2. **器の導出 file の path を解く関数**を `crates/folio/src/note.rs` に足し、床と面の生成器が共有する唯一の式にする。式は次の順で、最初に当たったもので決まる。
   1. 版管理の根が在る（(b) の 1 の関数が path を返す・置き場そのものが版管理の根の形を含む）→ 根の下の `contracts/schema.toml`。
   2. 版管理の根が無い（版管理の外・git が無い・待ち上限を超えた・正規化できない）→ 置き場の親の下の `contracts/schema.toml`（今の式と同じ・親が無ければ理由の字「正本の置き場の親 dir が無い」を返す）。
   - 1 で解いた path に file が無くても、親へは倒さない（探す先は 1 つ・黙って別の file を読まない・P-4.1）。
   - **置き場そのものが版管理の根の形は、ADR-16 決定 (2)(キ) の字どおり版管理の根（＝置き場そのもの）の下を読む**（席の裁定 2026-09-24: 親に倒すのは根が無いときだけ・特別な場合分けを足さない）。この形は床の版管理の照合が「照合にならない」違反にする形で、本便の後は違反の字に加えて、置き場の下に器の導出 file が無ければ「器の導出 file が読めない」の「まだ分からない」も出る。床の並び（読めないが在れば まだ分からない が先に立つ）により終了コードは 1 から 2 に変わる＝floor_cases の組 nested-git-root の期待を 2 に改める（違反の字の期待はそのまま・§1 (f)）。
3. **床の読み手**（`crates/folio/src/note.rs` の load_external）は、親の計算をこの関数の呼び出しに替えるだけで、読みの本体・断りの字（器の導出 file が読めない・symlink は認めない・先頭の字・行の欄）は 1 字も変えない。導出の命令（`crates/folio/src/derive.rs`）は床の読み手を共有しているので、字を変えずに同じ式に従う。
4. **面の生成器の読み**（`crates/folio/src/face_note.rs` の load_external）も、親の計算を同じ関数の呼び出しに替える（層 4 から層 2 への名指し・下向き）。面の生成器が自前に持つ器の導出 file の path の定数は、床の定数 `crates/folio/src/floor_note.rs` の同じ定数（pub(crate)）の use に替えて消す（同じ字を 2 か所に持たない・起草役の実測 patch は改訂 b でこの形）。先頭の字・行の鍵・行の欄の 3 つの字面の写しは本便では動かさない（ADR-16 決定 (2)(キ) が 1 つにするのは解く関数）。
5. **命令の説明の字。** `crates/folio/src/main.rs` の命令 derive の --dir の説明の 1 行（その親 dir の contracts/schema.toml を読む）を、版管理の根（無ければ置き場の親 dir）の contracts/schema.toml を読む、に直す。命令の一覧・旗・振り分けは変えない。
6. **folio2 自身では結果が変わらない。** folio2 の版管理の根は design-intent の親なので、1 で解く path は今の path と同じ file である（歯 6 が実の repo で確かめる・受付の器の作業木でも作業木の根が design-intent の親）。起草役の実測（base 2d90e8d）: 本便の後の binary で `folio check --dir design-intent` は合格・`folio schema --check` は 0（9 file 一致）・`folio inject --check` は 0・`folio derive --dir design-intent --out ../contracts --check` は 0・base と本便の後の binary で `folio build --write` を別の配信先へ書き `diff -r` で比べて差 0（参考値 23 file）・`folio graph --print` の出力の要約値も前後で一致。

### (c) 直す先 — 生成区間の注から条 id を外す

1. 5 つの床の木の注 top_level_note の先頭の文を「最上位の節の閉じた一覧（ほかの節は床が落とす）。」に直す（字「・N-3」を落とすだけ・2 文目以降は 1 字も変えない）。床の突き合わせは注を読まない（注を剥がして比べる）ので、床の判定の式は変わらない。
2. **5 つとも直す理由。** 床が数えるのは要件書と語彙の 2 つだけ（(a) の 5）だが、5 つは同じ文の写しで、2 つだけ直すと同じ文が 2 通りになる（前文の順位「網羅より揃い」）。広げる理由はこれだけである。**本便の後も、生成区間のほかの注には folio2 の id が残る**（起草役と検証役の実測: 判断の記録の欄の決まりの注に P-1・R-7・P-12.2・N-4・A-2・P-10.1 ほか、設計ノートの欄の決まりの注 profile_note に N-3、索引の欄の決まりに ADR-13・P-6.3、規則の表に P-5.2・R-4、天井の正本に ADR-8・N-5、入口に ADR-5）。床はそれらを数えないので、骨格の命令（便 125）が書く生成区間にも残る。それを外すのは本便の外（§1 (i) の 1）。
3. **写し 10 か所（全数・(a) の 4 の手順で数えた）。**
   - 5 file の生成区間（design-intent の srs.yaml・vocabulary.yaml・rules.yaml・intake.yaml・graph.yaml）: `folio schema --dir design-intent --write` の導出（各 1 行・区間の外は 1 byte も変えない）。書いた後の `folio schema --check` は 0（9 file 一致）・`folio check` は合格。
   - 凍結 anchor 5 本: **実装から独立に**、字面の置き換え 1 か所ずつを script で当てて直し（script は §1 (j)）、`folio schema --write` の導出と byte 一致することを確かめる（生成物の写しで anchor を作らない・P-10.2）。行数は 5 本とも不変、byte 数は各 6 減る（参考値・`wc -l -c` で測り直せる）。
   - 床の凍結の土台（`tests/fixtures/floor_base/` の下）の要件書と語彙と規則の表は、この注の字を持たない（土台は生成区間の前の版の写し）ので直さない。節点の要約値の anchor（`tests/fixtures/schema/node-digest-anchor.txt`）は土台だけを数えるので動かない。
4. **写しに出ない変更。** 最上位の節の閉じた一覧・欄の集合・値域・判定の式は変えない。

### (d) 採らなかった形

1. **根に file が無ければ親も探す（2 か所を順に探す）。** 置き場の親と版管理の根の両方に別の中身の器の導出 file が在るとき、どちらを読んだかが字で見えず、根の file を消すと黙って親の file に替わる（P-4.1）。ADR-16 決定 (2)(キ) の「無ければ親に倒す」は版管理の根が無いときの倒し方と読む。採らない。
2. **置き場そのものが版管理の根なら根と数えず親に倒す（改訂 a の形）。** ADR-16 決定 (2)(キ) の字（版管理の根が在ればそこ・無ければ親）に無い場合分けを足し、floor_cases の組の期待を保つために式に例外の枝を持つことになる。席の裁定（2026-09-24）で採らない（§1 (e) の変異 M2 が歯 5 と floor_cases で落ちる）。
3. **面の生成器の読みを床の読み手に畳む（読みの写しを全部消す）。** ADR-16 決定 (2)(キ) の範囲は解く関数で、読みの本体の畳み込みは面の生成器の欄の順（name だけ）と床の欄の型（name・need・shape）の違いを揃える別の直しになる。採らない（後の便の候補・§5）。
4. **解く関数を `crates/folio/src/gitcheck.rs` に置く。** 解く式は器の導出 file の置き場の決まり（床の定数の path）と組で、床の読み手の隣に置けば導出の命令も同じ file から引ける。gitcheck.rs は版管理の根を返すだけにする。採らない。
5. **要件書と語彙の 2 つの注だけを直す。** (c) の 2。採らない。
6. **解決先を環境変数か旗で選ばせる。** 検査される側が読む file を選べる口になる（ADR-16 決定 (4) の物差し・N-3.1）。採らない。

### (e) 歯（新しく 7 本・置き場は新しい file `crates/folio/tests/` の下の resolve.rs）

関数名は f123_ で始める（verify の絞り込みの語・base で `git grep -n 'fn f123_'` は 0 件）。**凍結 anchor（P-10.1）** は既に在る手書きの設計ノート `tests/fixtures/design-note/derive-anchor.yaml`（契約表を持ち図を持たない・便 119 の凍結の対の片方）と repo の `contracts/schema.toml` の写しで、生成器の出力を写さない。

**歯の土台。** 一時 dir の下の置き場（`<一時 dir>/<段>/design-intent`）に実の design-intent の写しと上の設計ノートを置き、器の導出 file の写しを置く場所と git の init と commit の位置を歯ごとに変えて、`folio check --dir` と `folio face --face note --id derive-anchor --dir … --out … --write` を撃つ（一時 dir は落ちても消す・git は環境変数 GIT_* を継承しない）。

1. **版管理の根が置き場の 2 段上なら、床は根の写しを読み、親の壊れた写しを読まない。** 根に repo の写し・置き場の親に壊れた写し（先頭 schema = 2）・根で git の init と commit → 床は 0 で、字「器の導出 file が読めない」が無い。**base では親の壊れた写しを読んで 2 ＝ RED。**
2. **同じ形で面の生成器も根の写しを読む。** 面の命令は 0 で面を書く。**base では 2 ＝ RED。**
3. **版管理の根に写しが無ければ、親に在っても読まない。** 置き場の親にだけ repo の写し・根で git → 床は 2 で字「器の導出 file が読めない」・面も 2 で同じ字・面の出力 file は作られない。**base では親を読んで床 0・面 0 ＝ RED。**
4. **版管理の無い写しは置き場の親を読む。** git の無い一時 dir の根に写し → 床は 2 で、字「版管理（git）が無いか読めない」が在り、字「器の導出 file が読めない」が無い（床の「まだ分からない」は版管理の不在だけ＝今と同じ）・面は 0。base でも緑（不変の歯）。
5. **置き場そのものが版管理の根なら、その根（置き場そのもの）の下を読み、親は読まない。** 置き場の中で git の init と commit・置き場の親に写し → 床は 2 で、字「器の導出 file が読めない」と字「design-intent 自体が版管理の根」（照合の違反）の両方が在る・面は 2 で同じ字・面の出力 file は作られない。続けて置き場の下の `contracts/schema.toml` に写しを置くと面は 0。**base では親を読んで床に字「器の導出 file が読めない」が無く・面 0 ＝ RED。**
6. **folio2 自身: 版管理の根は design-intent の親。** repo の design-intent で rev-parse --show-toplevel を撃ち、正規化した path が repo の根と等しい。base でも緑（(b) の 6 の前提を見る歯）。
7. **5 file の生成区間の注は条 id を名指さない。** 実の 5 file の生成区間の top_level_note の行が字「最上位の節の閉じた一覧（ほかの節は床が落とす）。」を持ち、字 N-3 を持たない。**base では字が在って落ちる ＝ RED。**

**base で RED は 5 本**（1・2・3・5・7）、不変を見る歯が 2 本（4・6）。起草役の実測（base 2d90e8d に歯の file だけを当てて `cargo nextest run -p folio --test resolve f123_ --no-fail-fast`）で 1・2・3・5・7 が落ち、4・6 が緑。

歯の効き（起草役が本便の後の木に変異を 1 つずつ当て、歯の file resolve.rs・floor_cases・face_note・schema_docs と `--bin folio` の単体の歯を撃った・改訂 b で全行を測り直した・script は d123-draft-mut.py・逐語は d123-draft-mut.out）:

| 当てた形 | 落ちる歯 |
| --- | --- |
| 本便の形 | なし |
| M1 解決先を常に置き場の親にする（前の形） | 1・2・3・5 と floor_cases の組 nested-git-root |
| M2 置き場そのものが版管理の根なら親へ倒す（改訂 a の形） | 5 と floor_cases の組 nested-git-root |
| M3 根に file が無ければ親へ倒す | 3・5 と floor_cases の組 nested-git-root |
| M4 面の生成器だけ親に固定する（共有しない） | 2・3・5 |
| M5 床だけ親に固定する | 1・3・5 と floor_cases の組 nested-git-root |
| M6 版管理が無ければ親へ倒さない | 4 と face_note の歯の多数（計 21 本） |
| M7 生成区間を書き直さない（床の木と anchor は直す） | 7 と `crates/folio/tests/schema_docs.rs` の生成区間の歯（計 19 本） |
| M8 床の木の注だけ据え置く（生成区間と anchor は直す） | `crates/folio/tests/schema_docs.rs` の生成区間の歯と `crates/folio/src/rules.rs` の単体の歯 rules_floor_derives_the_frozen_anchor_byte_for_byte（計 17 本）。歯 7 は実の design-intent の file を読むので落ちない |

### (f) 既存の歯のうち落ちるもの・凍結 anchor が動くか・直し方

起草役が、本便の src と生成区間と anchor だけを当てて歯の file と組の表（`tests/floor_cases.yaml`）を base のままにした木で workspace の nextest を撃つと、**落ちる既存の歯は 12 本**（参考値・base 2d90e8d の 796 本のうち）。11 本は `crates/folio/tests/schema_docs.rs` の中で、生成区間の byte 数と要約値の定数による。1 本は `crates/folio/tests/floor_cases.rs` の floor_cases_all_pass_with_folio で、組 nested-git-root が rc 2（期待 1）になる（§1 (b) の 2 の裁定の帰結）。

| 歯（file の指定の無い行は `crates/folio/tests/schema_docs.rs` の名） | 直し方 |
| --- | --- |
| f77_regions_match_the_frozen_anchors・f77_drift_in_each_region_fails・f77_check_covers_the_three_files | 3 file（要件書・語彙・相談窓口）の表 F77_REGIONS の byte 数と要約値を、anchor を `wc -c` と sha256sum で測り直した値に（行数は同じ） |
| f86_srs_region_matches_the_new_anchor | 上の表の要件書の行を引く定数なので、表を直せば緑 |
| schema_check_matches_the_real_rules_file_and_its_frozen_digest・schema_check_fails_on_one_byte_drift_inside_the_rules_region・schema_write_restores_the_rules_region_and_is_idempotent・f85_rules_region_matches_the_new_anchor | 規則の表の byte 数と要約値の定数を直す。f85_ の歯の本文に字で書かれた行数・byte 数・要約値の 3 か所は、同じ定数（RULES_REGION_ の 3 つ）を引く形に直す（同じ値を 2 か所に持たない） |
| f95_the_graph_schema_region_matches_the_anchor・f95_a_drift_in_the_graph_region_fails・f95_the_command_now_sees_nine_files | 索引の欄の決まりの byte 数と要約値の定数を直す |
| `crates/folio/tests/floor_cases.rs` の floor_cases_all_pass_with_folio（組の表 `tests/floor_cases.yaml` の組 nested-git-root） | 組の表の 1 行の expect_rc を 1 から 2 に改め、why に「便 123 から器の導出 file もその根の下で探すので 読めない が先に立って 2・違反の字は出る」を足す。expect_msg（字 design-intent 自体が版管理の根）はそのまま。歯の file の本文と組の数は変えない |

- `crates/folio/tests/schema_docs.rs` の頭の注に便 123 の 3 行（何を直し、定数を何で測り直したか・f85_ の字を定数へ寄せたこと）を足す。値は anchor を測り直した値（D-13 のとおり契約には写さない）。
- **組の表 `tests/floor_cases.yaml` は凍結の fixture（P-10.1）である。** 本便が改めるのは組 nested-git-root の期待の終了コードだけで、改める根拠は §1 (b) の 2 の席の裁定（ADR-16 決定 (2)(キ) の字どおり読む）の帰結である。違反の字の期待は残るので、組が見る「置き場そのものを版管理の根にすると落ちる」は同じ歯で見続ける。
- 単体の歯（`crates/folio/src/rules.rs` の rules_floor_derives_the_frozen_anchor_byte_for_byte ほか各床の木と anchor の byte 一致の歯）は、床の木と anchor を同じ便で直すので緑のまま。
- 変異 M2（改訂 a の形）で落ちる floor_cases の組 nested-git-root は、本便の形と改めた期待で緑。
- **凍結 anchor が動くのは 5 本**（(c) の 3）。ほかの凍結 anchor（判断の記録と設計ノートの欄の決まりの写し・天井の束・所見・面の凍結 fixture・節点の要約値・floor_base の土台）は 1 byte も動かない。
- 本便の後の木で workspace の nextest は全部緑（参考値 803 本 = base 2d90e8d の 796 + 新しい歯 7）・clippy 0 警告・`folio check` 合格・`folio schema --check` 9 file 一致。

### (g) 門（規則の表の開発規律行 D-12）

本便は 5 file の生成区間を書き換えるので天井の門の対象で、実測は base 2d90e8d で **2（まだ分からない・印が古い）**、天井の 28 周目の印の後の main a70477e で **0（通す）**（§0 の 門）。行 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。**席の裁定（2026-09-24）: 行 D-12 の持ち主の裁定の前例（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes）を本便に当てて、印が古いまま受付ける（受付の時点の門が 0 なら当てない）。** 同じ前例を当てたのは便 101・103（2026-09-22）と便 117・119（2026-09-24）である。席は受付のときに、当てた前例の裁定 id を台帳の notes に記帳する。本便が書き換える正本の中身は 5 file の生成区間の注の 5 行（字「・N-3」を落とすだけ）で、人が書く字は 1 字も変わらない。

### (h) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 1 本に `+`（`crates/folio/tests/` の下の resolve.rs）。書き換える 22 本は印なし（うち本文を変えない歯の file 2 本・組の表 `tests/floor_cases.yaml` 1 本を含む）。消す印と縮める印は当たらない。新しい dir は作らない。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない・script は §1 (j)）。size S の見積は 1 file あたり 100。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 本便の後（参考値） |
| --- | --- | --- | --- |
| `crates/folio/src/note.rs` | 903 | 597 | 916 |
| `crates/folio/src/face_note.rs` | 1,011 | 489 | 1,010 |
| `crates/folio/src/gitcheck.rs` | 395 | 1,105 | 403 |
| `crates/folio/src/check.rs` | 889 | 611 | 889 |
| `crates/folio/src/rules.rs` | 295 | 1,205 | 295 |
| `crates/folio/src/intake.rs` | 252 | 1,248 | 252 |
| `crates/folio/src/graph.rs` | 738 | 762 | 738 |
| `crates/folio/src/main.rs` | 632 | 868 | 632 |

3. **size は S。** 変える src は 8 本で、行が増えるのは note.rs（約 13 行）と gitcheck.rs（約 8 行）だけ。ほかは字の置き換え（face_note.rs は定数 1 つを消して use を 1 行足す）。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test resolve f123_` = (e) の新しい歯 7 本。
   2. `cargo nextest run -p folio --test schema_docs` = 5 file の生成区間と凍結 anchor の歯（直した 11 本を含む全部）。
   3. `cargo nextest run -p folio --test floor_cases` = 床の組の全部（参考値 142 組・組 nested-git-root の改めた期待 2 を含む）。
   4. `cargo nextest run -p folio --test face_note` = 面の生成器の歯（版管理の無い写しで親を読む形の全部）。
   5. `cargo nextest run -p folio --bin folio rules_floor` = `crates/folio/src/rules.rs` の単体の歯（床の木と凍結 anchor の byte 一致・folio は実行 file だけの crate なので `--bin folio`）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file のうち resolve と schema_docs は write-set に在る。floor_cases と face_note は本文を変えず、**write-set に入れる**（器の scope の規則: `--test X` が名指す tests/X.rs は本文不変でも write-set に置く）＝ write-set に `crates/folio/tests/floor_cases.rs` と `crates/folio/tests/face_note.rs` を印なしで挙げる。`--bin folio` の絞り込みの語 rules_floor を関数名に持つ src の file は `crates/folio/src/rules.rs` の 1 本だけ（base で `git grep -n 'fn [a-z0-9_]*rules_floor' -- crates/folio/src` が 3 件・すべて rules.rs）で、write-set に在る。`crates/folio/src/check.rs` の注の字 2 つと `tests/floor_cases.yaml` の 1 行は便 ③ の write-set と file が重なる（hunk は重ならない・§0 の 受付の順序）。

### (i) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 骨格の命令 folio init と案内の 1 行（便 125）。要件書・判断の記録・憲法・規則の表の人が書く字（要件 FR10 の注の「口は未実装である（2026-09-24 時点）」の字の直しは、着地の後の要件書の版）。面の生成器の読みの本体の畳み込み（(d) の 3）。図の道具（vendor/archify）の置き場の解決（面の生成器は今も置き場の親を見る・M3 の外）。設計ノートの欄の決まりの生成区間の注 profile_note の「・N-3」と、ほかの生成区間の注に残る folio2 の id（§1 (c) の 2・床が数えない・利用者の置き場にも書かれる・直すなら各判断の記録の出所の字と揃える別の直し）。外部 crate。
2. **言えないこと。** 歯は git の init と commit を撃てる環境で版管理の根の解決を見る。一時 dir（std の temp_dir・環境変数 TMPDIR）そのものが別の版管理の中に在る環境では、版管理の無い写しの歯 4 と、git を撃たない既存の面の歯の写しが外の版管理の根を解く（`crates/folio/tests/` の多数の歯の写しは TMPDIR の下に置かれ、TMPDIR が repo の中なら根の `contracts/schema.toml` を読む）。起草役と器の作業木の実測では TMPDIR は空（既定の /tmp）で当たらない。
3. **まだ分からない — 利用者の repo での実物。** scribe2 の置き場 design-intent/folio2 と repo の根の器の導出 file の組で床が読めることは、歯 1 の形（2 段上の根）で見る。scribe2 の repo そのものに撃つのは利用者の判定点 ①（ADR-16 決定 (1)・受入基準 AC23）で、folio2 の歯では言えない。
4. **撤退条件。** (1) 本便の後に (f) の 12 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(2) `folio build` の出力が本便の前後で 1 byte でも変わったら止めて席へ返す（(b) の 6）。(3) 受付の時点の main で、`crates/folio/src/check.rs` の要件書と語彙の床の木の注か、5 つの床の木の注の字か、`crates/folio/src/note.rs` と `crates/folio/src/face_note.rs` の器の導出 file の読み手の親の計算の周りか、`tests/floor_cases.yaml` の組 nested-git-root の行が書き換わっていたら（便 ② ③ の着地を含む）、変更の逐語と写しの 10 か所と組の行を測り直してから運ぶ。(4) 席か持ち主が、置き場そのものが版管理の根の形を親へ倒す形（改訂 a・(d) の 2）に戻すと裁定したら、組 nested-git-root の期待を 1 に戻し、歯 5 の期待を直す改訂を先に起こす。

### (j) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は 持ち主の home の `.local/share/folio2/handoff-2026-09-24/d123-measured.patch`（改訂 b で撃ち直した・base 2d90e8d に当たる・repo には入れない）。起草の記録は同じ dir の d123-draft.md、anchor の置き換えの script は d123-draft-anchor.py、変異の script は d123-draft-mut.py（逐語は d123-draft-mut.out）、行数の script は d119-draft-lines.py（便 119 と同じ式）。

1. base の写し: `git worktree add --detach <写し> 2d90e8d`。
2. RED: 歯の file（`crates/folio/tests/` の下の resolve.rs）だけを当てて `cargo nextest run -p folio --test resolve f123_ --no-fail-fast` → 1・2・3・5・7 が落ち、4・6 が緑。
3. 落ちる既存の歯: patch のうち `crates/folio/src`・`design-intent`・`tests/fixtures` の差分だけを当てて workspace の nextest → (f) の 12 本だけが落ちる。
4. 全部: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）・`folio check --dir design-intent`（合格）・`folio schema --dir design-intent --check`（0）。
5. anchor の独立: base の 5 本の anchor に d123-draft-anchor.py を当て、本便の後の 5 file の生成区間と byte 一致を見る。
6. 変異: `python3 d123-draft-mut.py <作業木> <target dir>`（(e) の表）。
7. 出力の差: base と後の binary で `folio build --write` を別の配信先へ・`diff -r`。
8. 門: `folio ceiling --gate --write-set <write-set の 23 本（接頭辞を剥がす）>`。
9. 余地: `python3 d119-draft-lines.py <file…>`。

## 2. 範囲

- 入れる: `crates/folio/src/gitcheck.rs` に版管理の根を返す関数 1 つ。`crates/folio/src/note.rs` に器の導出 file の path を解く関数 1 つと、床の読み手の親の計算の置き換えと use の 2 行。`crates/folio/src/face_note.rs` の読みの親の計算の置き換えと、自前の path の定数を消して床の定数を use する 1 行と、頭の注の 1 行。`crates/folio/src/main.rs` の命令 derive の --dir の説明の 1 行。5 つの床の木（`crates/folio/src/check.rs` の 2 つ・`crates/folio/src/rules.rs`・`crates/folio/src/intake.rs`・`crates/folio/src/graph.rs`）の注 top_level_note の字「・N-3」。5 file の生成区間の各 1 行（`folio schema --write`）。凍結 anchor 5 本の各 1 行。新しい歯の file（f123_ の 7 本）。`crates/folio/tests/schema_docs.rs` の定数と歯 1 本の字の値を定数へ寄せる置き換えと頭の注。組の表 `tests/floor_cases.yaml` の組 nested-git-root の 1 行（期待の終了コードと why）。
- 入れない: 骨格の命令と案内の 1 行（便 125）。床の判定の式と断りの字。器の導出 file の読みの本体と面の生成器の読みの本体の畳み込み。版管理の照合（check_git）。設計ノートの欄の決まりの注 profile_note。要件書と判断の記録と憲法と規則の表と語彙の人が書く字。天井の正本。新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| root | 版管理の根 | `crates/folio/src/gitcheck.rs` に足す小さな関数（git の toplevel を正規化して返す） |
| resolve | 解決の式 | `crates/folio/src/note.rs` に足す器の導出 file の path を解く関数（床・導出の命令・面の生成器が共有する唯一の式） |
| floor | 床の読み手 | `crates/folio/src/note.rs` の load_external（親の計算だけを置き換える） |
| face | 面の生成器の読み | `crates/folio/src/face_note.rs` の load_external（親の計算だけを置き換える） |
| note | 生成区間の注 | 5 つの床の木の top_level_note と、5 file の生成区間と、凍結 anchor 5 本 |
| teeth | 歯 | 新しい歯の file の f123_ の 7 本と、直す既存の歯 12 本（schema_docs の 11 本と組の表の 1 行） |

## 4. 検査（歯）

§1 (e)(f) と (h) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない（git は子 process で撃つ今の形）。新しい dir は無い。
- 前提の着地: 版 B（要件書 第 1.34 版）・便 ②（`docs/design/delivery-121.md`〔と 124〕）・便 ③（`docs/design/delivery-122.md`）。§0 の 受付の順序。
- 並行の便: 便 125（④-2・`docs/design/delivery-125.md`）は本便の後に受付ける（`crates/folio/src/main.rs` を両方が触る・本便は命令 derive の説明の 1 行だけ）。
- 後の便への申し送り: 面の生成器の読みの本体（先頭の字・行の鍵・行の欄の写し）を床の読み手に畳む直し（(d) の 3）。図の道具の置き場の解決。設計ノートの欄の決まりの注 profile_note の条 id。
- 本便の着地の後に席が見ること: 要件 FR10 の注の「口は未実装である」の字（次の要件書の版）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dv"
title = "判断の記録 ADR-16 決定 (2)(キ)(オ) と (7) の ④ の 1 便目（裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）: 器の導出 file contracts/schema.toml の解決先を、置き場の親の固定から、置き場を含む版管理の根へ改める。crates/folio/src/gitcheck.rs に版管理の根（git の toplevel を正規化した path・無ければ無い）を返す関数を足し、crates/folio/src/note.rs に器の導出 file の path を解く関数を 1 つ足して、床の読み手（note.rs）と面の生成器の読み（crates/folio/src/face_note.rs）がそれを共有する（導出の命令は床の読み手を共有しているので同じ式に従う）。式は、版管理の根が在ればその下（置き場そのものが根でも同じ・席の裁定 2026-09-24）、無ければ置き場の親の下で、根の下に file が無くても親へは倒さない。読みの本体と断りの字は変えない。folio2 自身では根が design-intent の親なので結果は変わらない。あわせて、要件書・語彙・規則の表・相談窓口・索引の欄の決まりの 5 つの床の木の注 top_level_note から条 id N-3 の名指しを外し、5 file の生成区間（folio schema --write）と凍結 anchor 5 本（独立の置き換えの script）を揃える。歯は新しい crates/folio/tests/resolve.rs に 7 本（版管理の根の写しを床と面が読む・根に無ければ親を読まない・版管理の無い写しは親を読む・置き場そのものが根なら置き場の下を読み親を読まない・folio2 の根は design-intent の親・注が条 id を名指さない）で、落ちる既存の歯 12 本（crates/folio/tests/schema_docs.rs の生成区間の byte 数と要約値の定数 11 本と、組の表 tests/floor_cases.yaml の組 nested-git-root の期待の終了コードを 1 から 2 に改める 1 本）を直す。受付は便 ② ③ の着地の後で、受付の時点の門が印が古いなら行 D-12 の持ち主の裁定の前例（2026-09-22）を当てる"
req = ["FR10", "FR19"]
section = "1"
write-set = ["crates/folio/src/note.rs", "crates/folio/src/face_note.rs", "crates/folio/src/gitcheck.rs", "crates/folio/src/check.rs", "crates/folio/src/rules.rs", "crates/folio/src/intake.rs", "crates/folio/src/graph.rs", "crates/folio/src/main.rs", "+crates/folio/tests/resolve.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/floor_cases.rs", "crates/folio/tests/face_note.rs", "tests/floor_cases.yaml", "design-intent/srs.yaml", "design-intent/vocabulary.yaml", "design-intent/rules.yaml", "design-intent/intake.yaml", "design-intent/graph.yaml", "tests/fixtures/schema/srs-region.txt", "tests/fixtures/schema/vocabulary-region.txt", "tests/fixtures/schema/rules-region.txt", "tests/fixtures/schema/intake-region.txt", "tests/fixtures/schema/graph-region.txt"]
verify = ["cargo nextest run -p folio --test resolve f123_", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test floor_cases", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --bin folio rules_floor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f123_ の歯 7 本（版管理の根が置き場の 2 段上なら床は根の写しを読み親の壊れた写しを読まず合格／同じ形で面の生成器も根の写しを読んで面を書く／根に写しが無ければ親に在っても床と面が まだ分からない で面を書かない／版管理の無い写しは親を読み床の まだ分からない は版管理の不在だけで面は合格／置き場そのものが版管理の根なら置き場の下を読み親を読まず床は照合の違反と読めないの両方で面は書かない／folio2 の版管理の根は design-intent の親／5 file の生成区間の注が条 id N-3 を名指さない）が緑、5 file の生成区間と凍結 anchor の歯（schema_docs の全部）が緑、床の組の全部（floor_cases・組 nested-git-root の改めた期待を含む）が緑、面の生成器の歯（face_note）が緑、rules.rs の床の木と凍結 anchor の byte 一致の単体の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、folio check と folio schema --check と folio derive --dir design-intent --out ../contracts --check が着地の後の main で 0 を返し、folio build の出力が前と byte 一致する"
<!-- contracts:end -->
