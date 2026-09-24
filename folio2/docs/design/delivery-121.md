# 設計: 便 121 — 列の根を憲法の名ごとの表で照らし（写しは置き場の名の行だけ）、憲法の列と id の一覧がどちらも無い置き場で 2 つを同時に書く凍結の命令 folio check --freeze-start を足す（FR23・FR24・ADR-16 決定 (2)(ア)(イ) と (7) の ②）

- 要件: FR23（憲法の列の根を、憲法の名ごとの表で照らす）/ FR24（憲法の列と id の一覧がどちらも無い置き場で、2 つを同時に凍結する）/ 受入基準 AC20・AC21（その 2 本の歯）。**どれも要件書 第 1.34 版（版 B・PR #291）で新設された id で、版 B は main 2d90e8d で着地（FR23・FR24・AC20・AC21 は枝と 1 字も違わない・撤退条件 (5) は発動しない）。** 契約表の行の req は FR23 と FR24 の 2 つ。FR19（床の定数から欄の決まりの file の生成区間へ写す）も本便が写しの形を変える要件だが、規範文は変えないので req に入れない（§1 (d)）。
- 条: P-5.6（実装の型付きの定数の写しを設計文書の置き場へ決定的に導出する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-4.1（実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ N-1.1（管理下の対象を回復不能に削除しない＝凍結は上書きしない）/ N-3.1（規則の例外機構を足さない＝新しい旗はどの検査も数えから外さない）
- 出所: 判断の記録 **ADR-16**（発効 2026-09-23 17:39 JST・持ち主の承認・対話面 R-8・逐語「裁定は両方承認する」・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (2)(ア)「列の根は、床の定数を憲法の名（検査される置き場の憲法の正本の meta.id）から根の要約値を引く表に広げる。最初の版は表に持たせず、全行で共通の今の値のまま。folio2 の行は 1 字も変えない。表に無い名の憲法の列の根は、凍結の旗が要約値の全桁を出して断り、既に在る列なら床が違反として落とす。表の写しは検査される置き場の名の行だけ」と (イ)「始まりの凍結は、憲法の列と id の一覧がどちらも 0 本の置き場でだけ成り立つ凍結の命令とし、2 つを同時に書く（旗の名は便で決める）。どの検査も数えから外さない。前提は 2 つの基準の不在を除くすべての検査が 0 違反で まだ分からない も無いこと。どちらか 1 本でも在れば断る。今ある 2 つの旗の前提は変えない」と、決定 (7) の便の列の **②「列の根の表（写しは自分の行だけ）と始まりの凍結の命令（中・1〜2 便）」**。判断の記録 ADR-2 決定 (3)（列の根の digest を床の定数と一致させる）は ADR-16 決定 (2)(ア) が広げた。規則の表の開発規律行 D-11（P-5.6 が掛かる定数を足す便・値を変える便は根拠の判断の記録か裁定 id を名指す）は、この行と契約表の行の title が ADR-16 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dt` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 32 本。**新しい file は 1 本（歯の file）で、先頭に `+` を付けて宣言する。** 縮む file も消す file も無く、新しい dir も作らない。`~` は使わない。
- 門: 本便は `design-intent/adr/schema.yaml`（判断の記録の欄の決まり）の生成区間を書き換えるので天井の門の対象である。受付の時点の main で patch の前の木に write-set 32 本を渡して撃つ（main a70477e = 28 周目の印で 0 = 通す・参考値）。0 なら D-12 の裁定は要らない。本便が着地すると印が古くなり、後続の便 123 / 125 の門は 2 になる見込み（席が D-12 の前例で受ける）。起草役が 2026-09-24 に planner の作業ツリー（main a70477e・patch の前）で `folio ceiling --dir design-intent --gate --write-set …` を撃ち直して 0（通す・印が 4 観点とも合格・正本の要約値が同じ）。patch を当てた後の木で撃つと、patch 自身が判断の記録の欄の決まりの要約値を変えるので必ず 2 になる（門の撃ち方ではない）。§1 (h)。
- 前の便: 判断の記録 ADR-16 決定 (7) の前提 = ADR-15 の列と一括 14（着地済み）・要件書の版 A（第 1.31 版・発効済み）・便 ①（要件書の最上位の節の閉じた一覧に M3 の範囲の節・着地済み）・版 B（要件書 第 1.34 版・main 2d90e8d）。便 119（行 `dr`・導出物の命令）と便 120（行 `ds`・main 1029382 と afabdf8）は着地済みで、本便の write-set と 1 本も重ならない。**base = main a70477e。この契約の数はすべて参考値である**（規則の表の行 D-13）。起草役が main afabdf8 で測り、検証役が main 2d90e8d と a70477e で撃ち直して同じ値だった（afabdf8 から a70477e までの差は版 B と天井の印の付け直しで、`crates/`・`tests/`・`contracts/` の file を 1 本も変えない）。起草の途中まで base を c33ee44 に取っていたが、c33ee44 から afabdf8 までの差（便 120）も本便の触る file を 1 本も変えないので、src の行数・変異・配信の組み立ての差は c33ee44 に本便の patch を当てた木でも同じ値だった（nextest の本数だけが便 120 の 4 本の分だけ違う）。
- **受付の順序。** 要件の id は main に在る＝版 B は main 2d90e8d で着地（FR23・FR24・AC20・AC21 は枝と 1 字も違わない・撤退条件 (5) は発動しない）。ADR-16 決定 (7) の順は ② → ③ → ④ で、本便が ② である。便 ③（`docs/design/delivery-122.md`・行 `du`）と便 ④（`docs/design/delivery-123.md` と 125・骨格の命令と器の導出 file の解決先）との境界は §1 (j)。
- 分割: **1 便で運ぶ。** §1 (i) の余地の表のとおり、触る src の 9 本はどれも size M の見積 300 を上回る余地を持つ（最も狭いのは `crates/folio/src/anchor.rs` で参考値 496）。ADR-16 決定 (7) の ② の見積「1〜2 便」の 1 便の側で、列の根の表と始まりの凍結は同じ照らし（列の根）を共有するので、分けると照らしの口が 2 便に跨る。
- 並行の便との重なり: 便 ③ の write-set（`crates/folio/src/check.rs`・`tests/floor_cases.yaml` ほか）とは 1 本も重ならない（本便は `crates/folio/src/check.rs` を触らない形にした・§1 (e) の 5）。便 ④（delivery-123）とは `crates/folio/src/main.rs` の 1 本、④-2（delivery-125）とは `crates/folio/src/main.rs` と `crates/folio/src/schema.rs` の 2 本が重なる。決定 (7) の順で本便が先に着地し、④ の側が base を取り直す（§5）。

## 1. 設計

### (a) いま起きていること（実測・base main a70477e）

1. **列の根は床の定数の値 1 つで、憲法の名を見ない。** 判断の記録の欄の決まりの床の定数（`crates/folio/src/floor_adr.rs` の FLOOR）の anchor の節の root_digest が、folio2 の憲法 第 1.0 版の凍結 anchor の digest を 1 つ持つ。読むのは 2 か所で、床の列の検査（`crates/folio/src/anchor.rs` の check_anchor・索引の最初の項の digest と比べて違えば違反）と凍結の命令（`crates/folio/src/freeze.rs`・最初の版を凍結するときだけ、組んだ木の digest と比べて違えば違反で書かない）。どちらも憲法の meta.id を読まない。
2. **写しは 19 か所に在る（全数）。** 判断の記録の欄の決まりの生成区間（`design-intent/adr/schema.yaml`・`folio schema --write` が `crates/folio/src/floor.rs` の導出で書く）・その凍結 anchor（`tests/fixtures/schema/adr-region.txt`・生成区間と byte 一致・`crates/folio/src/adr.rs` の単体の歯と `crates/folio/tests/schema.rs` が読む）・歯の置き場の欄の決まりの写し 17 本（`tests/fixtures` の下の adr/schema.yaml・床は注でない欄を FLOOR と突き合わせる・生成区間の印を持たない）。17 本のうち 16 本は憲法の名が fixture-constitution の置き場で、1 本（`tests/fixtures/floor_base/design-intent/adr/schema.yaml`・床の凍結の土台）だけが folio2-constitution。土台の写しの全 byte は節点の要約値の anchor（`tests/fixtures/schema/node-digest-anchor.txt`）の残差の 2 行が数える。組み直す手順:

   ```
   find tests/fixtures -path '*adr/schema.yaml' | sort
   git grep -n 'root_digest' -- crates tests design-intent
   for d in $(find tests/fixtures -path '*adr/schema.yaml' | sed 's#/adr/schema.yaml##'); do grep -m1 '^  id:' $d/constitution.yaml; done | sort | uniq -c
   ```

3. **憲法の名を書き換えても床は変わらない。** 床の凍結の土台の写し（git の中）の憲法の meta.id を別の名に書き換えて `folio check` を撃つと **0（合格・違反 0・まだ分からない 0）**。列の根の digest の比べは名を見ないので、どの名の置き場でも folio2 の値 1 つと比べる。
4. **2 つの凍結の旗は、両方 0 本の置き場で互いに断る。** 床の凍結の土台の写しから `anchors/` を消して git の中で撃つと、`--freeze-anchor` も `--freeze-ids` も **凍結しない（違反 0・まだ分からない 1）**。`--freeze-anchor` は憲法の列の 0 本の「まだ分からない」だけを数えから外し、id の一覧の 0 本の「まだ分からない」（`crates/folio/src/ids.rs` の check_ids）は外さない。`--freeze-ids` はその逆である（ADR-16 の文脈 (2)(イ) の実測と同じ）。
5. **id の一覧だけが在る置き場（手で置いた・消し残した）では、憲法の列の根は凍結できない。** 土台の写しから憲法の列（anchor と索引）だけを消して commit し、`--freeze-anchor` を撃つと **1（違反 1 = anchor が版管理の HEAD か履歴に在ったので列を始め直す凍結は認めない）**。版管理の照合（`crates/folio/src/gitcheck.rs`）は anchors/ の下の id の一覧の file も anchor の在った印に数える。逆の順（憲法の列を先）は id の一覧の 0 本で凍結しない（上の 4）。**今の 2 つの旗だけでは、両方 0 本の置き場から始められる順が無い。**
6. **表に無い名の概念が無い。** 歯の置き場 `tests/fixtures/anchor/root-digest-drift/`（名 fixture-constitution・自分自身と一致する digest の列）の床は、列の根の digest が床の定数と違うの違反 1 件で落ちる（`crates/folio/tests/anchor.rs` の anchor_root_digest_drift_fails が違反 1 件と字 列の根・床の定数 を見る）。
7. base の workspace の nextest は全部緑（参考値 796 本・base a70477e・`cargo nextest run --workspace --no-tests=fail --no-fail-fast`）。`folio check --dir design-intent` は合格、`folio schema --dir design-intent --check` は 9 file 一致。

### (b) 直す先 — 列の根の表（FR23・ADR-16 決定 (2)(ア)）

1. **表の型。** `crates/folio/src/floor_adr.rs` に定数 ROOT_DIGESTS を足す。型は字の対の閉じた一覧（鍵 = 検査される置き場の憲法の正本の meta.id・値 = その列の根の digest の 16 進 64 字）で、初版の行は 1 つ = 鍵 folio2-constitution・値は今の root_digest の値そのまま（**1 字も変えない**）。行を足すのは folio2 の便で、契約表の行か設計ノートが利用者の持ち主の承認の裁定 id を名指す（規則の表の行 D-11・ADR-16 決定 (2)(ア)・便 ⑦）。最初の版（anchor の節の first_version）は表に持たせず、全行で共通の今の値のまま。
2. **床の定数の木。** FLOOR の anchor の節の欄 root_digest（値 1 つ）を、欄 root_digests（置き場の名で行を選ぶ表）に替える。`crates/folio/src/floor.rs` の床の木の型 Floor に、置き場の名で行を選ぶ表の変種を 1 つ足す（中身は上の字の対の一覧を名指す）。この変種の写しと突き合わせは **鍵が置き場の名と等しい行だけ**（0 行か 1 行）を扱い、名が無いか表に無ければ空の表とする。
3. **引く口。** `crates/folio/src/adr.rs` に、憲法の名（無いこともある）から表の値を引く関数 root_digest と、置き場の `constitution.yaml` の meta.id を読む関数 place_name（読めなければ理由の字・床の欄の決まりの突き合わせと `folio schema` が同じ関数で読む）を足す。床の定数の値の読み口（floor_val）で anchor の節の root_digests を読むと None になる（値でなく表なので）。
4. **床の照らしの式。** `crates/folio/src/anchor.rs` の check_anchor は、憲法の meta.id（字でなければ無い）で表を引き、索引の最初の項（n = 0）の digest を次の 3 つに分ける。
   - 表に行が無い → 違反（種別 anchor）。字は **列の根**・**憲法の名（無ければ meta.id が無い の断り）**・**表に無い**・**索引の digest の全桁（16 進 64 字）** を持つ（今の字の 列の根・床の定数 も含む）。
   - 行が在って値と違う → 違反（種別 anchor）。今の字に **列の根の表の <名> の行** を足した字（と違う を含む）。
   - 一致 → 何もしない。
   最初の版の綴りの照らし（first_version）と、そのほかの列の検査は 1 字も変えない。
5. **凍結の命令の照らし。** `crates/folio/src/freeze.rs` の `--freeze-anchor` が列の根を凍結するとき（最新の anchor が無い）の照らしを、同じ表の引きに替える。表に行が無い → 違反（種別 anchor）で、字は **列の根**・**憲法の名**・**表に無い**・**旗の綴り --freeze-anchor**・**組んだ木の digest の全桁（表に行を足す値）**・**凍結しない** を持つ。行が在って違う → 今の字（digest の先頭 12 字の対）に **列の根の表の <名> の行** を足した違反。どちらも今と同じく、違反を積んで 0 違反でないので書かない（終了 1）。照らしの関数は始まりの凍結（(c)）と共有する。
6. **folio2 の床は変わらない。** folio2 の置き場の憲法の名は folio2-constitution で、表の値は今の root_digest と同じなので、folio2 の床の結果（合否・違反と「まだ分からない」の件数）は変わらない（ADR-16 の撤退条件 (2) の物差し・歯 1 と実の置き場の床の歯）。
7. **名の書き換えは落ちる。** 置き場の憲法の meta.id を書き換えると、照らす行が替わって違反になる（表に無い名なら表に無いの違反、表に在る別の名なら値の違いの違反）。欄の決まりの写しも名の行と食い違うので、写しの違反も同時に立つ（(d) の 1）。ADR-16 の帰結のとおり、表が証するのは列の根の中身で、どの repo の列かは証さない。
8. **N-3.1 との整合。** 表は道具の側の定数で、検査される側の data（憲法の名・根の file）は行を選べるだけで、選び直しは表に無いの違反か値の違いの違反にしかならない（ADR-16 決定 (4)）。

### (c) 直す先 — 始まりの凍結の命令（FR24・ADR-16 決定 (2)(イ)）

1. **旗の名と口。** `folio check` に旗 **--freeze-start** を足す（既存の 2 つの旗 --freeze-anchor・--freeze-ids の隣）。ほかの 3 つの旗（--emit-amends・--freeze-anchor・--freeze-ids）とは同時に撃てない（clap の conflicts_with_all で引数の断り）。`crates/folio/src/phase.rs` の旗の列挙に 1 値を足し、`crates/folio/src/main.rs` は命令 check の旗の欄 1 つと、旗から列挙の値への振り分けの 1 枝だけを足す（命令の一覧・ほかの命令は変えない）。
2. **数えから外すもの（2 つだけ）。** この旗のときは、憲法の列が 0 本の「まだ分からない」（check_anchor の (i)）と、id の一覧が 0 本の「まだ分からない」（check_ids）の 2 つだけを積まない。前者は今 `--freeze-anchor` のときにも外しており、後者は今 `--freeze-ids` のときにも外している（**今の 2 つの旗の前提は変えない**）。ほかのどの検査も数える（列の根の照らしも外さない）。
3. **断りの条件と順（どれも何も書かない）。**
   1. **どちらか 1 本在る** → 他の出力をせず終了 1（今の --freeze-anchor の版の断りと同じ口）。標準エラーの字は **--freeze-start**・在る方の名（**憲法の列** は anchors/ の索引か constitution- で始まる anchor file が 1 本でも在る・**id の一覧** は anchors/ の ids- で始まる file が 1 本でも在る・両方在れば両方）・**在る**。在る方の判定は、`crates/folio/src/phase.rs` の列の結果に足す欄（憲法の列の file が在るか）と `crates/folio/src/ids.rs` の現行の id の一覧の型に足す欄（id の一覧の file が在るか）で持つ。
   2. **列の始め直し** → 今の `--freeze-anchor` と同じ 2 つの違反（憲法の版が first_version でない・版管理の HEAD か履歴か記録の上で anchor が在った）。
   3. **表に無い名・値の違い** → (b) の 5 と同じ照らし（旗の綴りは --freeze-start）。表に無い名は組んだ木の digest の全桁を出す。
   4. **ほかの検査に違反か「まだ分からない」** → 0 違反で「まだ分からない」も無いときだけ書く。そうでなければ 凍結しない の 1 行（今の 2 つの旗と同じ字の形・旗の綴り --freeze-start）を標準エラーに出し、判定の終了コード（違反が在れば 1・無くて「まだ分からない」なら 2）で終える。
4. **書くもの（2 つを同時に）。** 断りが無ければ、anchors/ に (1) 憲法の最初の版の anchor（`--freeze-anchor` が列の根で書く木と同じ・previous は空・承認一覧は憲法 meta.approval の写し 1 項）と索引（1 項）、(2) 要件書の版の id の一覧（`--freeze-ids` が書く木と同じ）を書く。木と本文は既存の 2 つの旗と **同じ関数** で組む（`crates/folio/src/freeze.rs` の凍結の木の組み立てを、組む関数と書く関数に分け、`crates/folio/src/ids.rs` の `--freeze-ids` の本文の組み立てを、組む関数と書く関数に分ける＝既存の 2 つの旗の出力の字と byte は変えない）。**組めるものを全部組んで断りを全部見てから書く**（書く前に 3 つの path がどれも無いことを確かめ、在れば上書きせずに終了 1）。書く途中で書けなければ「まだ分からない」（書けた file は消さない・N-1.1）。標準エラーの 1 行は **始まりの凍結をした** で始まり、2 つの path と条の数と id の数と、版管理に commit せよの断りを持つ。終了 0。
5. **書いた後。** 書いた 3 file を commit した後は、ふだんの床（旗なしの `folio check`）が全部の検査を回し、列の根の照らしと id の一覧の照らしも回る。床の凍結の土台の写しから anchors/ を消して撃つと、書いた id の一覧は土台の凍結済みの file と byte 一致し、書いた憲法の anchor と索引の digest は folio2 の列の根と一致し、commit の後の床は合格（違反 0・まだ分からない 0）になる（歯 4・起草役の実測）。
6. **N-3.1 との整合。** この旗はどの検査も数えから外さず、自分が作る 2 つの基準の不在だけを前提から除き、書いた後はふだんの床が全部を数える（ADR-16 決定 (4)）。どちらか 1 本在る置き場では何もしないので、今の 2 つの旗の代わりに使って既存の列を書き換える道は無い。

### (d) 写しの形と写しの全数

1. **生成区間の形。** `crates/folio/src/floor.rs` の導出に、置き場の名を受ける口（名つきの導出）を足し、床の突き合わせ（floor_diff）にも名つきの口を足す（名を持たない今の 2 つの口は、名が無い＝表は空の表として残し、ほかの 8 本の正本の導出と各床の単体の歯はそのまま使う）。表の変種の導出は、名の行が無ければ `root_digests: {}` の 1 行、在れば flow の 1 行が幅 100 字に収まるならその 1 行、収まらなければ block（欄名の行と、2 つ深い字下げの `<名>: <digest>` の行）。folio2 の行は幅を超えるので block の 2 行になる。値の引用の規則（規則 5）は今のまま。
2. **folio schema の口。** `crates/folio/src/schema.rs` は、置き場の憲法の名（adr.rs の place_name）を読み、判断の記録の欄の決まり（adr/schema.yaml）だけを名つきで導出する（ほかの 8 本は名を使わない）。名が読めなければ（constitution.yaml が無い・symlink・meta.id が無い）、その file の手前で **2（まだ分からない）** で止め、標準エラーに理由と字 **meta.id** を出す（黙って空の表を書かない・P-4.1）。これで写しは表と置き場の名から決定的に決まる（ADR-16 決定 (2)(ア) の括弧）。
3. **床の突き合わせ。** `crates/folio/src/adr.rs` の check_adr は、置き場の名で欄の決まりの写しと突き合わせる（名つきの floor_diff）。写しの表の行が名の行と違う（余分な行・欠けた行・値の違い・表でない）なら、今と同じ字の違反（adr/schema.yaml schema.anchor.root_digests が床の定数と違う）を 1 件積む。**`crates/folio/src/check.rs` は変えない**（名は check_adr が自分で place_name から読む＝便 ③ の write-set と重ならない）。meta.id が読めない置き場では、folio schema は導出できないので まだ分からない（2）を返し、床の写しの突き合わせは違反にする。この非対称は、条 P-4.2（判定できないものは まだ分からない）と、憲法の正本が読めない置き場は床の別の検査でも落ちること、から受ける。
4. **注。** FLOOR の説明の注（`_note` の欄・床は読まず、生成区間には出る）を直す: anchor_note の列の根の項（root_digest → 列の根の表 root_digests の置き場の名の行・表に無い名は落とす・行を足すのは folio2 の便）と、列の始め直しの項（根の digest が表の置き場の名の行と違う・表に無い名は凍結の命令が digest の全桁を出して凍結しない）、始まりの凍結の項を 1 つ足す（--freeze-start の前提と断りと書いた後）、limits_note の末項（列の根は表・名は digest の外に在る・表が証するのは中身・行を足すのと根を作り直す移行では床の定数を直す・行 D-11）。
5. **写し 19 か所の直し方（全数・(a) の 2 の手順で数えた）。**
   - `design-intent/adr/schema.yaml` の生成区間: `folio schema --write` の導出（区間の外は 1 byte も変えない）。書いた後の `folio schema --check` は 0・`folio check` は合格。
   - 凍結 anchor `tests/fixtures/schema/adr-region.txt`: **実装から独立に**、base の anchor に字面の置き換え 4 か所（列の根の欄の 1 行を block の 2 行に・注 anchor_note の 2 項の書き換えと 1 項の挿入・注 limits_note の末項）を script で当てて作り、`folio schema --write` の導出と byte 一致することを確かめる（起草役の script は §1 (l)・生成物の写しで anchor を作らない・P-10.2）。
   - 床の凍結の土台 `tests/fixtures/floor_base/design-intent/adr/schema.yaml`（名 folio2-constitution）: 列の根の欄の 1 行を、表の block の 2 行（folio2 の行）に。注は前の版の字のまま（床は注を読まない）。
   - 名 fixture-constitution の写し 16 本: 列の根の欄の 1 行を `root_digests: {}` の 1 行に（表に無い名の置き場の写しは空の表）。直さないと、その 16 の置き場の床の歯が写しの違反で落ちる。
   - 節点の要約値の anchor `tests/fixtures/schema/node-digest-anchor.txt`: 土台が 1 行増えるので、独立の script `tests/fixtures/schema/node-digest.py` の出力のうち **残差の 2 行だけ** が動く（残差の要約値と合計の byte・節点の行は 1 つも動かない）。script の出力で置き換え、`crates/folio/tests/graph.rs` の anchor の要約値の定数を直す（行数と byte 数は同じ・`wc -l -c` で測り直せる）。
6. **写しに出ない変更。** 判断の記録の欄の決まりのほかの欄（欄の集合・値域・閾値）と、ほかの 8 本の正本の生成区間は 1 byte も変えない。配信の組み立て（`folio build`）の出力は本便の前後で変わらない（起草役の実測: base の binary と本便の後の binary で実の置き場を別の配信先へ書き、`diff -r` の差 0・参考値 23 file）。

### (e) 採らなかった形

1. **表の全行を全利用者の写しに載せる。** ADR-16 の帰結のとおり、表に行を 1 つ足した組み立てで他の全利用者の床が落ちる結合が生まれる。決定 (2)(ア) が写しを置き場の名の行だけに決めた。採らない。
2. **表を写さない（写しから列の根の欄を消す）。** 行 D-11 が写しの導出を求める判定の定数（閉じた一覧）なので採らない（ADR-16 決定 (2)(ア)）。
3. **最初の版も表の行ごとに持つ。** 決定 (2)(ア) が最初の版は表に持たせず全行で共通の今の値のままと決めた。採らない。
4. **表に無い名の凍結を「まだ分からない」にする。** 決定 (2)(ア) と要件 FR23 は凍結の命令では digest の全桁を出して断り、既に在る列では違反で落とすと決めた（どちらも合格にしない）。今の値の違いと同じ違反の種別で積み、終了 1 にする。採らない。
5. **名を `crates/folio/src/check.rs` の読んだ憲法から渡す。** 便 ③ の write-set（`crates/folio/src/check.rs`）と重なり、受付の重なりの断りで 2 便が直列に縛られる。check_adr が adr.rs の place_name で自分で読む形にした（同じ関数を `folio schema` も使う）。採らない。
6. **始まりの凍結で、書ける方だけを書く。** 決定 (2)(イ) は 2 つを同時に書くと決めた。片方だけを書くと id の一覧だけが在る段が生じ、(a) の 5 の障害（id の一覧だけが在る置き場では憲法の列の根を凍結できない）に落ちる。採らない。
7. **始まりの凍結を旗でなく新しい命令にする。** 検査の式と材料（列の結果・id の一覧）は `folio check` の中に在り、既存の 2 つの凍結も `folio check` の旗である。命令の一覧（`crates/folio/tests/check.rs` の閉じた一覧）を増やさず、旗 1 つで足りる。採らない。
8. **表の行を歯のための行で増やす（test 用の行・環境変数で行を足す口）。** 表は folio2 の便で持ち主の承認の裁定 id を名指して足す閉じた一覧で、実行時に足す口は例外の口（N-3.1）である。始まりの凍結の成り立つ歯は、folio2 の行の中身をそのまま持つ床の凍結の土台（第 1.0 版の憲法）で書く。採らない。

### (f) 歯（新しく 8 本）

関数名は f121_ で始める（verify の絞り込みの語・base で `git grep -n 'f121_'` は 0 件）。新しい歯の file は 1 本（`crates/folio/tests/` の下の freeze_root.rs・歯 7 本）と、`crates/folio/src/floor.rs` の単体の歯 1 本。

**凍結 anchor（P-10.1）。** 生成器からも検査の実装からも独立した物差しは 3 つ: (1) 歯の中の定数（folio2 の列の根の digest の 64 字・`crates/folio/tests/anchor.rs` の定数と同じ値・day-1 の床の値）で、歯は crate の中の表を読まない、(2) 床の凍結の土台の凍結済みの anchors/（`tests/fixtures/floor_base/design-intent/anchors/` の constitution-v1.0.yaml の digest の欄と ids-v1.8.yaml の byte 列・前の便で作って凍結した file）、(3) `tests/fixtures/anchor/root-digest-drift/` の索引の digest の値（歯の中の定数に写す）。

**歯の土台。** 一時 dir に design-intent/ として土台を写し、置き場の親の contracts/ に repo の `contracts/schema.toml` を写し、git init と 1 commit を行う（床の版管理の照合のため・`crates/folio/tests/freeze.rs` と同じ作り方・一時 dir は落ちても消す）。

1. **folio2 の床と 1 行の写し。** 床の凍結の土台の写しに旗なしの `folio check` → 0（合格・違反 0・まだ分からない 0）。実の `design-intent/adr/schema.yaml` と土台の写しの、列の根の欄の行とその下の行が、どちらもちょうど「欄名の行」と「folio2-constitution と歯の中の定数の行」の 2 行。実の生成区間に前の欄名 root_digest の行が無い。**base では欄名が違って落ちる＝RED。**
2. **名の書き換えは床で落ちる。** 土台の写しの憲法の meta.id を別の名にして commit → 1・違反はちょうど 2 件 = 種別 anchor で字 列の根・その名・表に無い・歯の中の定数の全桁を持つもの 1 件と、種別 adr で字 schema.anchor.root_digests を持つもの 1 件。**base では 0（合格）で落ちる＝RED。**
3. **別の中身の根は、名でも行でも落ちる。** `tests/fixtures/anchor/root-digest-drift/` の写し → 1・違反はちょうど 1 件で、字 fixture-constitution・表に無い・その置き場の索引の digest の全桁を持つ。憲法の名を folio2-constitution に書き換えて commit → 1・違反の中に字 列の根・列の根の表の folio2-constitution の行・と違う を持つものが在り、表に無い を持つものは無い。**base では字が違って落ちる＝RED。**
4. **始まりの凍結が 2 つを書き、書いた後の床が合格。** 土台の写しから anchors/ を消して commit → 旗なしの床は 2（違反 0）。`--freeze-anchor` と `--freeze-ids` はどちらも 2 で 凍結しない を出し、anchors/ を作らない（今の 2 つの旗の前提は変わらない）。`--freeze-start` → 0・標準エラーに 始まりの凍結をした・anchors/ の file はちょうど constitution-v1.0.yaml と ids-v1.8.yaml と index.yaml の 3 本。書いた anchor と索引の digest の欄は歯の中の定数と一致し、土台の凍結済みの anchor の digest の欄も同じ定数。書いた id の一覧は土台の凍結済みの ids-v1.8.yaml と byte 一致。commit の後の旗なしの床は 0（合格・違反 0・まだ分からない 0）。**base では旗が無く引数の断りで落ちる＝RED。**
5. **どちらか 1 本在れば断る。** 土台の写しから id の一覧だけを消した置き場（憲法の列だけが在る）と、憲法の anchor と索引だけを消した置き場（id の一覧だけが在る）で `--freeze-start` → 1・標準エラーに字 --freeze-start と在る方の名（憲法の列 か id の一覧）と 在る・anchors/ の名と byte 列が撃つ前と同じ。**base では落ちる＝RED。**
6. **ほかの検査に違反か「まだ分からない」、表に無い名。** anchors/ を消した土台の写しで、(1) 欄の決まりの写しの options_rule の min を 3 に（違反 1 つ）→ 1・違反 1 件・凍結しない・anchors/ を作らない、(2) 器の導出 file を置かない（設計ノートが在るのに読めない＝まだ分からない 1 つ）→ 2・違反 0・標準エラーに contracts/schema.toml と 凍結しない・anchors/ を作らない、(3) 憲法の名を別の名にし、欄の決まりの写しの列の根の表を空の表に揃えた（写しの違反を立てない）→ 1・違反はちょうど 1 件で、字 その名・表に無い・歯の中の定数の全桁（中身が folio2 の第 1.0 版なので組んだ digest は folio2 の値）を持つ・anchors/ を作らない。続けて、(4) id の一覧だけが在る置き場で同じ名の書き換えをして `--freeze-anchor` → 1・違反の中に字 --freeze-anchor・表に無い・歯の中の定数の全桁を持つものが在る・anchors/ は撃つ前と同じ（この置き場では (a) の 5 の違反も同時に立つ＝今の前提のまま）。**base では落ちる＝RED。**
7. **写しは置き場の名の行だけ。** 実の design-intent の写し（git の中）に `folio schema --check` → 0。憲法の名を別の名にして `--check` → 1（標準エラーに adr/schema.yaml）。`--write` → 0 で、列の根の欄の行はちょうど `root_digests: {}` の 1 行・file に歯の中の定数の字が無い。`--check` → 0。meta.id の行を消して `--check` → 2（標準エラーに meta.id）。**base では落ちる＝RED。**
8. **（単体の歯・`crates/folio/src/floor.rs`）置き場の名で行を選ぶ表の導出と突き合わせ。** 行 2 つ（短い値と幅を超える値）を持つ表で、名 a → flow の 1 行・名 b → block の 2 行・表に無い名と名なし → 空の表の 1 行（名なしの導出は今の導出の口と同じ字）。突き合わせは、名の行と一致すれば 0 件、表に無い名に名の行・値の違い・余分な行・表でない値はどれも道 1 つ。**base では型の変種が無く組めない＝RED。**

歯の効き（起草役が src に変異を 1 つずつ当てて測った・base a70477e と同じ src の木に本便の patch・検証役が main a70477e に patch を当てた写しで撃ち直して落ちる歯の組が行ごとに一致・撃つのは歯の file freeze_root・freeze・ids・anchor・schema・script は d121-draft-mut.py・逐語の記録は d121-draft-mut.out。組めない変異は compile-error と出して数えない形にしてあり、今の 16 通りに組めないものは無い）:

| 当てた形 | 落ちる歯 |
| --- | --- |
| 本便の形 | なし（8 本と既存の全部が緑） |
| M1 床が列の根を名で引かない（folio2 の行に固定） | 2・3 |
| M2 凍結の命令が列の根を名で引かない | 6 |
| M3 写しに表の全行を出す | 2・3・6・7 と既存の anchor の歯 2 本 |
| M4 id の一覧が在っても始まりの凍結が断らない | 5 |
| M5 憲法の列が在っても始まりの凍結が断らない | 5 |
| M6 ほかの検査の違反で断らない | 6 |
| M7 id の一覧の不在を始まりの凍結でも数える | 4 |
| M8 憲法の列の不在を始まりの凍結でも数える | 4 |
| M9 始まりの凍結が id の一覧を書かない | 4 |
| M10 folio schema が置き場の名を渡さない | 7 と既存の schema の歯 8 本 |
| M11 床の写しの突き合わせが表を見ない | 2 |
| M12 始まりの凍結が列の根を照らさない | 6 |
| M13 床の断りに digest の全桁を出さない | 2・3 |
| M14 凍結の断りに digest の全桁を出さない | 6 |
| M15 --freeze-anchor の前提を変える（憲法の列の不在を数える） | 既存の freeze の歯 2 本 |
| M16 名の読めない置き場でも folio schema が書く | 7 |

### (g) 既存の歯のうち落ちるもの・凍結 anchor が動くか・直し方

起草役が、本便の src と写しと凍結 anchor だけを当てて歯の file を base のままにした木で workspace の nextest を撃つと、**落ちる既存の歯は 7 本**（参考値・base a70477e の 796 本 + src の単体の歯 1 本のうち）。どれも本便の write-set の歯の file で直す。

| 歯（file・名） | 落ちる理由 | 直し方 |
| --- | --- | --- |
| `crates/folio/tests/schema.rs` の r9_population_anchor_holds・schema_adr_region_matches_the_frozen_anchor_and_lists_the_orchestrator_seat・schema_check_matches_the_real_file_and_its_frozen_digest・schema_check_fails_on_one_byte_drift_inside_the_region・schema_write_restores_the_region_and_is_idempotent・schema_write_leaves_bytes_outside_the_region_alone（6 本） | 判断の記録の欄の決まりの生成区間の行数・byte 数・要約値が変わる | 定数 REGION_LINES・REGION_BYTES・REGION_SHA256 を、凍結 anchor を `wc -l -c` と sha256sum で測り直した値に（D-13 のとおり契約には写さない）。定数の上の注と頭の注に便 121 の 1 行ずつ |
| `crates/folio/tests/graph.rs` の f99_the_independent_script_matches_the_anchor | 土台の 1 行で残差の 2 行が動く | anchor を script の出力で置き換え、定数 F99_ANCHOR_SHA256 を直す（行数と byte 数の assert は同じ値のまま通る）。定数の上に便 121 の注 1 行 |

- `crates/folio/src/adr.rs` の単体の歯 2 本（floor_values_and_numbers_are_read_through_the_floor・adr_floor_derives_the_frozen_anchor_byte_for_byte）は、同じ src の file の中で直すので緑のまま: 前者は床の定数の値の読み口で root_digest を読む assert を、表の引き（folio2-constitution の値・fixture-constitution と名なしは None・値の読み口で root_digests は None）に替える。後者は名つきの導出（folio2-constitution）を凍結 anchor と byte で比べる形に替える。
- `crates/folio/tests/anchor.rs` の anchor_root_digest_drift_fails は本文を変えずに緑のまま（違反は 1 件のまま・字 列の根 と 床の定数 を持つ）。ただし違反の理由は、値の違いから表に無い名に替わる（その置き場の名は fixture-constitution）。値の違いの側は新しい歯 3 が当てる。
- `crates/folio/tests/freeze.rs` と `crates/folio/tests/ids.rs`（今の 2 つの旗の歯）は本文を変えずに緑のまま＝今の 2 つの旗の前提と出力は変わらない（変異 M15 が当てる）。
- **凍結 anchor が動くのは 2 本**（`tests/fixtures/schema/adr-region.txt` と `tests/fixtures/schema/node-digest-anchor.txt`）と、凍結の土台の 1 行（`tests/fixtures/floor_base/design-intent/adr/schema.yaml`）と歯の置き場の写し 16 本の 1 行ずつ。どれも (d) の 5 のとおり、生成器の出力の写しではなく独立の手順（字面の置き換えの script・独立の Python の script・行の置き換え）で作り直し、実装の出力と byte 一致を歯が見る。ほかの凍結 anchor（ほかの 8 本の生成区間の anchor・天井の束・所見・面の凍結 fixture・節点の行・凍結の土台の anchors/）は 1 byte も動かない。
- 本便の後の木で workspace の nextest は全部緑（参考値 804 本 = base a70477e の 796 + 新しい歯 8）・clippy 0 警告・`folio check` 合格・`folio schema --check` 9 file 一致・`folio derive --dir design-intent --out ../contracts --check` 0。起草役の 1 回目の全体の実行で、本便と関わらない歯（`crates/folio/tests/entrance.rs` の entrance_unknown_section_fails）が 1 回だけ落ち、単独でも全体の撃ち直しでも緑だった（同じ機械で別の起草役が歯を並べて撃っていた・再現しない）。

### (h) 門（規則の表の開発規律行 D-12）

本便は判断の記録の欄の決まり（`design-intent/adr/schema.yaml`）の生成区間を書き換えるので天井の門の対象である。門は受付の時点の main で patch の前の木に write-set 32 本を渡して撃つ（main a70477e = 28 周目の印で 0 = 通す・参考値）。0 なら D-12 の裁定は要らない。本便が着地すると印が古くなり、後続の便 123 / 125 の門は 2 になる見込み（席が D-12 の前例で受ける）。行 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。受付の時点で 0 でなければ（天井の印が main より古くなっていれば）、器へ出す前に、席が持ち主の裁定を名指すか、天井の周を回して印を新しくする。先例は便 101・103（2026-09-22）と便 117・119（2026-09-24）で、持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes）を行 D-12 の裁定として受け、印が古いまま出した（起草役は決めない）。本便が書き換える正本の中身は生成区間の 6 行（列の根の欄 1 行が 2 行に・注 3 項の書き換えと 1 項の挿入）だけで、人が書く字は 1 字も変わらない。

### (i) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 1 本に `+`（`crates/folio/tests/` の下の freeze_root.rs）。書き換える 31 本は印なし（src 9・歯 2・生成区間 1・凍結 anchor 2・凍結の土台 1・歯の置き場の写し 16）。`-`（行が減る file）・`~`（着地で消える file）は当たらない（写し 16 本は行数が同じで、幅 120 の正規化行数も同じ）。新しい dir は作らない。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない・script は §1 (l)）。size M の見積は 1 file あたり 300。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 本便の後（参考値） |
| --- | --- | --- | --- |
| `crates/folio/src/anchor.rs` | 1,004 | 496 | 1,026 |
| `crates/folio/src/adr.rs` | 902 | 598 | 935 |
| `crates/folio/src/main.rs` | 632 | 868 | 638 |
| `crates/folio/src/floor_adr.rs` | 501 | 999 | 510 |
| `crates/folio/src/floor.rs` | 333 | 1,167 | 432 |
| `crates/folio/src/ids.rs` | 316 | 1,184 | 352 |
| `crates/folio/src/freeze.rs` | 303 | 1,197 | 444 |
| `crates/folio/src/schema.rs` | 156 | 1,344 | 178 |
| `crates/folio/src/phase.rs` | 65 | 1,435 | 71 |

   歯の file は src の外なので余地を測らない（参考に、新しい歯の file は同じ式で 521 行・参考値）。

3. **size は M。** 変える src は 9 本（最も増えるのは freeze.rs の約 140 行と floor.rs の約 100 行）。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test freeze_root f121_` = (f) の新しい歯 1〜7。
   2. `cargo nextest run -p folio --bin folio f121_` = (f) の歯 8（`crates/folio/src/floor.rs` の単体の歯・folio は実行 file だけの crate なので `--bin folio`）。
   3. `cargo nextest run -p folio --bin folio adr_floor` = `crates/folio/src/adr.rs` の単体の歯（名つきの導出と凍結 anchor の byte 一致・承認者の値域）。
   4. `cargo nextest run -p folio --test schema` = 判断の記録の欄の決まりの生成区間と凍結 anchor の歯（直した 6 本を含む全部）。
   5. `cargo nextest run -p folio --test graph f99_` = 節点の要約値の独立の script の anchor の歯（直した 1 本を含む f99_ の全部）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file 3 本（freeze_root・schema・graph）と、`--bin folio` の絞り込みの語を関数名に持つ src の file（語 f121_ は `crates/folio/src/floor.rs` の 1 本だけ・語 adr_floor は `crates/folio/src/adr.rs` の 1 本だけ・本便の後の木で `git grep -n 'fn [a-z0-9_]*f121_\|fn [a-z0-9_]*adr_floor' -- crates/folio/src` が floor.rs 1 件と adr.rs 2 件）は、すべて write-set に在る。anchor・freeze・ids・floor_cases の歯は本文を変えずに緑のままなので write-set に入れず、verify でも名指さない＝共通の検証（`.vessel.toml` の common-verify の workspace の nextest）が走らせる。

### (j) 境界 — 便 ③・便 ④ と触る file

1. **便 ③（`docs/design/delivery-122.md`・行 du・値域の部分集合・FR25）。** write-set は `crates/folio/src/check.rs`・新しい歯の file・凍結の対・`tests/floor_cases.yaml` で、本便の 32 本とは **1 本も重ならない**。本便は憲法の値域の床（`crates/folio/src/check.rs` の check_constitution ほか）にも、組み立てた値域（`crates/folio/src/constitution_enums.rs`）にも触らない。便 ③ は列の根と凍結の file（`crates/folio/src/freeze.rs`・`crates/folio/src/anchor.rs`・判断の記録の欄の決まりの写し）に触らない（③ の契約の §0 と §2）。
2. **便 ④（`docs/design/delivery-123.md`・器の導出 file を版管理の根で解く）。** 重なるのは `crates/folio/src/main.rs` の 1 本で、本便が足すのは命令 check の旗の欄 1 つ（--freeze-start）と旗の振り分けの 1 枝だけ、④ が直すのは命令 derive の --dir の説明の 1 行だけで、hunk は重ならない。④ の生成区間の直し（要件書・語彙・規則の表・相談窓口・索引の欄の決まり）と本便の生成区間（判断の記録の欄の決まり）は別の file である。
3. **便 ④-2（`docs/design/delivery-125.md`・骨格の命令 folio init）。** 重なるのは `crates/folio/src/main.rs` と `crates/folio/src/schema.rs` の 2 本。**骨格が判断の記録の欄の決まりの生成区間を書くときは、本便の名つきの導出（利用者の憲法の名で導く）を使う**＝骨格を書いた直後の利用者の置き場は表に行が無いので、写しの列の根の欄は `root_digests: {}` の 1 行になる（利用者の行は便 ⑦ で足し、`folio schema --write` で写しが 1 行になる）。
4. **順序。** ADR-16 決定 (7) の順（② → ③ → ④）で本便が先に着地する。④ の 2 本は本便の着地の sha に base を取り直してから受付ける（④ の契約 §0 の受付の順序のとおり）。

### (k) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 列の根の表に利用者の行を足すこと（便 ⑦）。骨格の命令と案内の 1 行（便 ④-2）と、床の案内の字（憲法の列が 0 本の「まだ分からない」の 1 行は今のまま --freeze-anchor を案内する＝始まりの凍結の案内は FR3 の案内の便で扱う）。値域の部分集合（便 ③）。要件書・判断の記録・憲法・規則の表・語彙の字（要件 FR23・FR24 の注の「この要件の口は未実装である」と AC20・AC21 の固定の材料の注を含む＝着地の後の要件書の版で席が直す）。版管理の照合が id の一覧の file を anchor の在った印に数えること（(a) の 5・今の前提のまま）。外部 crate。
2. **言えないこと。** 表が証するのは列の根の中身であり、どの repo の列かは証さない（ADR-16 の帰結）。歯 4 の始まりの凍結の成り立つ場合は folio2 の第 1.0 版の中身（表の唯一の行）でしか組めない＝利用者の行での成り立ちは便 ⑦ の歯が確かめる。書く途中の I/O の失敗（3 file のうち途中まで書けた）は歯で当てていない（書けた file は消さず「まだ分からない」で終える形だけを §1 (c) の 4 に書いた）。表に行が 2 つ以上在るときの写しは単体の歯 8 の小さな表でだけ当たる（実の表は 1 行）。(c) の 3 の 2（列の始め直しの 2 つの違反）は床の照合と列の根の照らしが重ねて落とすので、始まりの凍結の側に固有の歯は無い（検証役の変異 X1 / X2 は全部緑・実害なし）。(c) の 3 の 1 の「索引だけが在る」置き場は歯 5 の場合分けに無い（変異 X5 は全部緑・FR24 / AC21 の場合分けの外）。
3. **解けた — 要件の id。** FR23・FR24・AC20・AC21 は版 B の新設で、版 B は main 2d90e8d で着地（FR23・FR24・AC20・AC21 は枝と 1 字も違わない・撤退条件 (5) は発動しない）。本便の §1 と歯の期待は着地の字のままでよい。
4. **まだ分からない — 旗の名の最終の字。** 旗の名は ADR-16 決定 (2)(イ) と要件 FR24 が便に委ねた（本便の案 = --freeze-start）。着地の後の要件書の版で FR24 の注に旗の名を書く（席の起草・持ち主の承認）。
5. **撤退条件。** (1) 本便の後に (g) の 7 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(2) folio2 自身の置き場の床の結果（合否・違反と「まだ分からない」の件数）か、列の根の表の folio2 の行か、`folio build` の出力が本便の前後で違ったら、止めて席へ返す（ADR-16 の撤退条件 (2)・1 回で発動）。(3) 受付の時点の main で `crates/folio/src/freeze.rs`・`crates/folio/src/anchor.rs` の列の根の周り・`crates/folio/src/floor.rs` の導出・`crates/folio/src/ids.rs` の凍結の周りが書き換わっていたら、変更の逐語と写しの 19 か所を測り直してから運ぶ。(4) 受付の時点で `tests/fixtures` の下の adr/schema.yaml の本数か、その置き場の憲法の名の内訳が (a) の 2 と違ったら、write-set の写しの行を実物に合わせてから運ぶ。(5) 版 B の FR23・FR24 の規範文が本便の §1 の前提と違ったら、§1 と歯の期待を着地の字に合わせてから受付ける（版 B は main 2d90e8d で着地（FR23・FR24・AC20・AC21 は枝と 1 字も違わない・撤退条件 (5) は発動しない））。

### (l) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は `~/.local/share/folio2/handoff-2026-09-24/d121-measured.patch`（base afabdf8 と c33ee44 に起草役が、main 2d90e8d と a70477e に検証役が当てて、どれにも `git apply --check` が通る・repo には入れない）。起草の記録は同じ dir の d121-draft.md、変異の script は d121-draft-mut.py（逐語の記録 d121-draft-mut.out）、凍結 anchor の置き換えの script は d121-draft-anchor.py、行数の script は d121-draft-lines.py。

1. base の写し: `git worktree add --detach <写し> a70477e`。
2. RED: 歯の file（`crates/folio/tests/` の下の freeze_root.rs）だけを当てて `cargo nextest run -p folio --test freeze_root f121_ --no-fail-fast` → 7 本とも落ちる。floor.rs の単体の歯は src の中なので、base では組めない。
3. 落ちる既存の歯: patch のうち `crates/folio/src`・`design-intent`・`tests/fixtures` の差分だけを当てて workspace の nextest → (g) の 7 本だけが落ちる。
4. 全部: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）・`folio check --dir design-intent`（合格）・`folio schema --dir design-intent --check`（0）。
5. anchor の独立: base の `tests/fixtures/schema/adr-region.txt` に `python3 d121-draft-anchor.py <base の anchor> <出力>` を当て、本便の後の `design-intent/adr/schema.yaml` の生成区間と byte 一致を見る。`python3 tests/fixtures/schema/node-digest.py tests/fixtures/floor_base/design-intent` の出力と `tests/fixtures/schema/node-digest-anchor.txt` の差が 0。
6. 変異: 本便の後の木の根で `python3 d121-draft-mut.py <scratchpad>`（(f) の表）。
7. 出力の差: base と後の binary で `folio build --dir design-intent --out <別の配信先> --write` を撃ち `diff -r`。
8. 門: 受付の時点の main で patch の前の木に write-set 32 本を渡して撃つ（main a70477e = 28 周目の印で 0 = 通す・参考値）。0 なら D-12 の裁定は要らない。本便が着地すると印が古くなり、後続の便 123 / 125 の門は 2 になる見込み（席が D-12 の前例で受ける）。撃ち方は `folio ceiling --dir design-intent --gate --write-set <write-set の 32 本（接頭辞を剥がす）>`。
9. 余地: `python3 d121-draft-lines.py <file…>`。
10. 輪の実測（(a) の 4 と 5）: 床の凍結の土台を一時 dir に写して git の中に置き、anchors/ を消す・id の一覧だけを消す・憲法の anchor と索引だけを消すの 3 通りで base の binary に `--freeze-anchor` と `--freeze-ids` を撃つ。

## 2. 範囲

- 入れる: `crates/folio/src/floor_adr.rs` の列の根の表の定数 1 つと FLOOR の anchor の節の欄 1 つの置き換えと注 3 項の書き換えと 1 項の挿入。`crates/folio/src/floor.rs` の床の木の型の変種 1 つと、名つきの導出と名つきの突き合わせの口と、導出の内部の関数への名の受け渡しと単体の歯 1 本。`crates/folio/src/adr.rs` の表を引く関数と置き場の名を読む関数と、check_adr の名つきの突き合わせと単体の歯 2 本の直し。`crates/folio/src/anchor.rs` の列の根の照らしの 3 分けと、列の結果への名と憲法の列の file の有無の 2 欄と、始まりの凍結の旗でも (i) を積まない 1 行。`crates/folio/src/phase.rs` の旗の列挙の 1 値と列の結果の 2 欄。`crates/folio/src/ids.rs` の id の一覧の file の有無の欄と、始まりの凍結の旗でも 0 本の「まだ分からない」を積まない 1 行と、`--freeze-ids` の本文の組み立ての組む関数と書く関数への分割（字は不変）。`crates/folio/src/freeze.rs` の凍結の木の組み立ての組む関数と書く関数への分割（字は不変）と、列の根の照らしの関数と、始まりの凍結の本体と振り分けの 2 枝。`crates/folio/src/schema.rs` の名つきの導出の口（判断の記録の欄の決まりだけ）。`crates/folio/src/main.rs` の命令 check の旗 1 つと振り分けの 1 枝。`design-intent/adr/schema.yaml` の生成区間（`folio schema --write`）。凍結 anchor 2 本・凍結の土台の 1 行・歯の置き場の写し 16 本の 1 行ずつ。新しい歯の file（f121_ の 7 本）。`crates/folio/tests/schema.rs` の定数 3 つと注 2 か所・`crates/folio/tests/graph.rs` の定数 1 つと注 1 行。
- 入れない: `crates/folio/src/check.rs`（便 ③ の write-set）・憲法の値域の床と組み立てた値域・面の生成器と配信の組み立て・版管理の照合（`crates/folio/src/gitcheck.rs`）・床の案内の字・ほかの 8 本の正本の生成区間とその anchor・凍結の土台の anchors/・`contracts/` の下の file・要件書と判断の記録と憲法と規則の表と語彙・天井の正本・列の根の表の利用者の行・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| table | 列の根の表 | `crates/folio/src/floor_adr.rs` の定数（鍵 = 憲法の名・値 = 列の根の digest・閉じた一覧・初版は folio2 の 1 行） |
| pick | 名で行を選ぶ表の型 | `crates/folio/src/floor.rs` の床の木の型の変種と、名つきの導出・名つきの突き合わせの口 |
| lookup | 引く口 | `crates/folio/src/adr.rs` の表を引く関数 root_digest（床の照らしと凍結の命令が共有）と置き場の名を読む関数 place_name（床の欄の決まりの突き合わせ check_adr と `folio schema` が共有・床の照らしと凍結の命令は読んだ憲法の meta から名を取る） |
| floor | 床の照らし | `crates/folio/src/anchor.rs` の索引の最初の項の 3 分け（表に無い・値の違い・一致） |
| freeze | 凍結の照らし | `crates/folio/src/freeze.rs` の列の根の照らしの関数（--freeze-anchor と --freeze-start が共有） |
| start | 始まりの凍結 | 旗 --freeze-start（`crates/folio/src/main.rs`・`crates/folio/src/phase.rs`）と本体（`crates/folio/src/freeze.rs`）と、2 つの不在を積まない 2 行（`crates/folio/src/anchor.rs`・`crates/folio/src/ids.rs`） |
| region | 生成区間と写し | `design-intent/adr/schema.yaml` の生成区間・凍結 anchor 2 本・凍結の土台の 1 行・歯の置き場の写し 16 本 |
| teeth | 歯 | 新しい歯の file の f121_ の 7 本と `crates/folio/src/floor.rs` の単体の歯 1 本と、直す既存の歯 7 本 |

## 4. 検査（歯）

§1 (f)(g) と (i) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。
- 前提の着地: ADR-15 の列・一括 14・要件書の版 A（第 1.31 版）・便 ①・便 119・便 120・版 B（要件書 第 1.34 版）。版 B は main 2d90e8d で着地（FR23・FR24・AC20・AC21 は枝と 1 字も違わない・撤退条件 (5) は発動しない）。
- 並行の便: 便 ③（delivery-122）とは write-set が重ならない。便 ④（delivery-123）とは `crates/folio/src/main.rs`、④-2（delivery-125）とは `crates/folio/src/main.rs` と `crates/folio/src/schema.rs` が重なる＝ADR-16 決定 (7) の順で本便が先に着地し、④ の側が base を取り直す。
- 便 ④-2 の起草への申し送り: 骨格の命令が判断の記録の欄の決まりの生成区間を書くときは、本便の名つきの導出を利用者の憲法の名で呼ぶ（書いた直後の写しの列の根の欄は空の表の 1 行）。骨格を書いた直後の置き場は 2 つの基準がどちらも無いので始まりの凍結（--freeze-start）の対象になるが、表に利用者の行が無いので、凍結は digest の全桁を出して断る（その値で便 ⑦ を起こす・要件 FR24 の注のとおり）。
- 便 ④-2 が呼ぶ関数の形（本便が足す・名は散文で書く）: `crates/folio/src/floor.rs` に足す名つきの導出の関数 derive_for は、床の木（Floor の参照）と置き場の憲法の名（字の参照か無し）を受けて生成区間の本文（「schema:」の行から末尾の改行まで・今の導出の関数 derive と同じ体裁）を返す pub の関数で、名が無しなら derive と同じ字を返す。`crates/folio/src/adr.rs` に足す place_name は置き場の dir を受けて constitution.yaml の meta.id を字で返し、読めなければ理由の字を返す pub(crate) の関数。`crates/folio/src/schema.rs` は判断の記録の欄の決まり（adr/schema.yaml）だけを名つきで導き、ほかの 8 本は名を使わない。骨格の命令は、書いた骨格の憲法の名で derive_for を呼べば、`folio schema --write` と同じ写し（表に行の無い名なら root_digests の空の表）を書ける。振り分け（判断の記録の欄の決まりだけを名つきで導く）は `crates/folio/src/schema.rs` の非公開の定数 NAMED と run の中の match に在る。便 125 は同じ比べを自分で書かず（P-6.3）、schema.rs の write-set の中で公開の範囲を広げるか、空の生成区間で正本を書いてから schema::run(Write) を呼ぶ形（便 125 の契約が採る形）で共有する。
- 便 ⑦（列の根の表に利用者の行を足す）の起草への申し送り: 足すのは `crates/folio/src/floor_adr.rs` の表の 1 行と、利用者の置き場の写し（利用者の repo の側で `folio schema --write`）。folio2 の側の写し（`design-intent/adr/schema.yaml` の生成区間と凍結 anchor と歯の置き場の写し 17 本）は、置き場の名の行だけなので 1 byte も動かない（歯 1 と歯 7 が当てる）。
- 本便の着地の後に席が見ること: 要件 FR23・FR24 の注の「この要件の口は未実装である」と旗の名・AC20・AC21 の固定の材料の注（着地の後の要件書の版）。門は受付の時点の main で patch の前の木に write-set 32 本を渡して撃つ（main a70477e = 28 周目の印で 0 = 通す・参考値）。0 なら D-12 の裁定は要らない。本便が着地すると印が古くなり、後続の便 123 / 125 の門は 2 になる見込み（席が D-12 の前例で受ける）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dt"
title = "判断の記録 ADR-16 決定 (2)(ア)(イ) と (7) の ②（裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の 1 便: 列の根を憲法の名ごとの表で照らし、憲法の列と id の一覧がどちらも無い置き場で 2 つを同時に書く凍結の旗を足す。(ア) 判断の記録の欄の決まりの床の定数 crates/folio/src/floor_adr.rs の anchor の節の root_digest（値 1 つ）を、憲法の meta.id から列の根の digest を引く閉じた一覧の表 root_digests に替える（初版の行は folio2-constitution と今の値の 1 行だけで 1 字も変えない・最初の版は表に持たせない）。床の木の型 crates/folio/src/floor.rs に置き場の名で行を選ぶ表の変種と名つきの導出と突き合わせの口を足し、写しは置き場の名の行だけ（表に無い名は空の表）とする。床（crates/folio/src/anchor.rs）は索引の最初の項の digest を名で引いた値と照らし、表に無い名は digest の全桁を出す違反、値の違いは違反で落とす。凍結の命令（crates/folio/src/freeze.rs）も同じ照らしで、表に無い名は組んだ digest の全桁を出して凍結しない。folio schema（crates/folio/src/schema.rs）は置き場の憲法の名で判断の記録の欄の決まりの生成区間を導き、名が読めなければ 2。名を読む関数は crates/folio/src/adr.rs に置き、床の突き合わせと folio schema が共有する（crates/folio/src/check.rs は変えない）。folio2 の床の結果は変わらない。(イ) folio check に旗 --freeze-start を足す（ほかの 3 つの旗と同時に撃てない）。憲法の列と id の一覧のどちらか 1 本でも在れば何も書かずに 1 で断り、2 つの基準の不在の まだ分からない だけを積まずにほかの検査を全部数え、列の根の表の照らしも外さず、0 違反で まだ分からない も無いときだけ最初の版の anchor と索引と id の一覧を既存の 2 つの旗と同じ関数で組んで同時に書く。既存の 2 つの旗の前提と出力は変えない。写し 19 か所（判断の記録の欄の決まりの生成区間・凍結 anchor tests/fixtures/schema/adr-region.txt を独立の script で・床の凍結の土台の 1 行・歯の置き場の写し 16 本を空の表の 1 行に・節点の要約値の anchor の残差の 2 行）を揃える。歯は新しい crates/folio/tests/freeze_root.rs に 7 本と floor.rs の単体の歯 1 本で、落ちる既存の歯 7 本（生成区間の定数の歯 6 本・節点の要約値の anchor の歯）を直す。要件は要件書 第 1.34 版（版 B・main 2d90e8d で着地）の FR23・FR24"
req = ["FR23", "FR24"]
section = "1"
write-set = ["crates/folio/src/floor_adr.rs", "crates/folio/src/floor.rs", "crates/folio/src/adr.rs", "crates/folio/src/anchor.rs", "crates/folio/src/phase.rs", "crates/folio/src/ids.rs", "crates/folio/src/freeze.rs", "crates/folio/src/schema.rs", "crates/folio/src/main.rs", "+crates/folio/tests/freeze_root.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/graph.rs", "design-intent/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test freeze_root f121_", "cargo nextest run -p folio --bin folio f121_", "cargo nextest run -p folio --bin folio adr_floor", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test graph f99_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f121_ の歯 7 本（folio2 の床の凍結の土台が合格のままで実の生成区間と土台の写しの列の根の表が folio2 の 1 行／憲法の名を書き換えると床が表に無いの違反と写しの違反の 2 件で落ち digest の全桁を出す／別の中身の根は表に無い名で落ち folio2 の名に書き換えると表の行と違うで落ちる／2 つの基準が無い置き場で既存の 2 つの旗は互いに断り freeze-start が 3 file を書き digest が列の根と一致し id の一覧が土台の凍結済みの file と byte 一致し commit の後の床が合格／どちらか 1 本在る置き場で 1 で断り anchors は不変／違反 1 つで 1・まだ分からない 1 つで 2・表に無い名で 1 と digest の全桁・どれも書かず freeze-anchor も表に無い名で digest の全桁を出す／folio schema が置き場の名の行だけを写し名を書き換えると check 1・write の後は空の表で check 0・meta.id が無ければ 2）が緑、floor.rs の置き場の名で行を選ぶ表の導出と突き合わせの単体の歯 1 本が緑、adr.rs の名つきの導出と凍結 anchor の byte 一致の単体の歯が緑、判断の記録の欄の決まりの生成区間と凍結 anchor の歯が緑、節点の要約値の独立の script の anchor の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑（anchor・freeze・ids・floor_cases の歯を本文を変えずに含む）で CI が通り、着地の後の main で folio check が合格・folio schema --check が 9 file 一致"
<!-- contracts:end -->
