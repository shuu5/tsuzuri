# 設計: 便 117 — 要件書の最上位の節の閉じた一覧（床の定数）に M3 の範囲の節 scope_m3 を scope_m1 の次に足し、写しの生成区間と凍結 anchor と歯を揃える（FR19 / FR5・ADR-16 決定 (7) の ①）

- 要件: FR19（決まりの部分を機械の中の決まりから各 file の生成区間へ写す・対象に要件書の最上位の節の閉じた一覧を含む・第 1.32 版）/ FR5（検査結果を必ず返す＝床が未知の節を落とす 3 値・第 1.32 版）。本便は FR19 の写しの中身を 1 語広げ、FR5 の床が受ける最上位の節を 1 語広げる。規範文はどちらも変えない。
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ N-3.1（規則の例外機構を足さない）/ P-13.3（設計文書の変更は判断の記録か版付きの文書として残す）
- 出所: 判断の記録 **ADR-16**（M3 = 契約表を持つ設計ノートを外部の利用者 1 つが自分の repo で床に掛けられる形・発効 2026-09-23 17:39 JST・持ち主の承認・対話面 R-8・逐語「裁定は両方承認する」・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (1) の末尾「M3 の範囲の節を足すには、要件書の最上位の節の閉じた一覧（床の定数）を改訂する便が、要件書の版上げより先に要る」と、決定 (7) の便の列の **①「要件書の最上位の節の閉じた一覧に M3 の範囲の節を足す（小・1 便）→ 版 B」**。同じ判断の文脈 (1) が「要件書の最上位の節は床の閉じた一覧（17 節）なので、M3 の範囲の節（scope_m3）を足すには床の定数の改訂が要る」と書く。規則の表の開発規律行 D-11 が言う「P-5.6 が掛かる定数の値を変える便は、根拠の判断の記録か裁定 id を名指す」は、この行と契約表の行の title が ADR-16 を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dp` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 5 本、**新しい file は 1 本も無く、縮む file も無く、消す file も無く、新しい dir も作らない**（接頭辞 `+` / `-` / `~` は 1 つも使わない。§1 (f) の 1）。
- 門: 本便は `design-intent/srs.yaml`（要件書の正本）の生成区間を書き換えるので天井の門の対象である。起草役が write-set 5 本をそのまま `folio ceiling --gate --write-set …` に渡して実測すると **2（まだ分からない・断りの字は 印が古い）**（2026-09-24・base の binary と本便の後の binary の両方で同じ）。設計文書の正本を外した 4 本だけを渡すと 0（通す・設計文書の正本を書き換えない便）なので、門の対象になる理由は要件書の 1 本だけである。規則の表の開発規律行 D-12 の扱いは §1 (e)。
- 前の便: 判断の記録 ADR-16 決定 (7) の前提 = ADR-15 の列（便 106〜116）と一括 14 の着地。**便 116 は着地済み（main 87c9d48・行 `do`）・一括 14 は着地済み（main 2c3a631・PR #285・要件書 第 1.32 版）**。**この契約の数はすべて base main 2c3a631 の実測（参考値）である**（規則の表の行 D-13）。
- 並行の便との重なり: 一括 15（枝 docs/batch15・aa528f6・承認待ち）が `design-intent/srs.yaml` の人が書く欄（meta と要件の行と図の節）を直す。**本便が触るのは同じ file の生成区間の 1 行だけで、一括 15 の差分の塊とは 1 行も重ならない**（起草役が実測 patch を docs/batch15 の先頭に `git apply --check` で当てて通ることを確かめた）。どちらが先に着地しても、後の側は字面の衝突なく当たる。ADR-16 決定 (7) の ⑤（導出物の独立の命令）は未起草で、重なりはその起草のときに確かめる。
- 改訂: 改訂 b（2026-09-24・席）= 検証役（d117-verifier・写しで patch を当て 782/782・定数だけの木で落ちる 16 本の名前と残る 4 本が契約と一致・`~` の意味〔着地で消える file・refuse.rs〕を確かめ 印なしが正・blocking 0）の文面の所見 2 つを当てた（§1 (a) 5 の「20 か所ほど」を実測 30 行に・§1 (b) ④ の注が指す節）。区間・write-set・verify・size・done は変えていない。

## 1. 設計

### (a) いま起きていること（実測・base main 2c3a631）

1. 要件書の最上位の節の閉じた一覧の正本は、`crates/folio/src/check.rs` の定数 SRS_TOP_LEVEL（17 語・固定長の配列）である。床（`folio check`）はこの定数で要件書の最上位の鍵を数え、一覧に無い鍵を種別 未知の節 の違反にする（同じ file の `check_srs` が unknown_sections に定数を渡す 1 か所）。file の側の schema.top_level は写しで、床はそれを読まない（便 77 の f77_ の歯が、file の側に語を足しても床が落とすことを固定している）。
2. 写しは `folio schema --write` が要件書の末尾の生成区間（`# folio:schema:begin` と `# folio:schema:end` の間）へ書く。写しを作る床の木は同じ file の SRS_FLOOR で、その top_level の葉が SRS_TOP_LEVEL そのものを指す（`crates/folio/src/schema.rs` が要件書を SRS_FLOOR に結ぶ 1 行）。**写しは SRS_TOP_LEVEL から自動で導出されるので、写しの側に手で書く定数は無い。**
3. 床の定数 SRS_TOP_LEVEL を名指すのは `crates/folio/src/check.rs` の 3 か所（定義・SRS_FLOOR の葉・unknown_sections への受け渡し）だけである。組み直す手順:

   ```
   git grep -n 'SRS_TOP_LEVEL\|SRS_FLOOR' 2c3a631 -- crates
   ```

4. 生成区間を持つ要件書は `design-intent/srs.yaml` の 1 本だけで、凍結の土台（tests/fixtures の下）の要件書 20 本はどれも生成区間を持たない（便 102 の天井の正本のような写しの全数の列挙は要らない）。凍結 anchor は `tests/fixtures/schema/srs-region.txt` の 1 本で、その自己検査の値（行数・byte 数・要約値）は `crates/folio/tests/schema_docs.rs` の F77_REGIONS の srs.yaml の行と、同じ file の F86_SRS_LINES / F86_SRS_BYTES / F86_SRS_SHA256 の 2 か所に同じ値で pin されている。組み直す手順:

   ```
   git grep -l 'folio:schema:begin' 2c3a631 -- '*srs.yaml'
   git grep -n 'srs-region.txt' 2c3a631 -- crates tests
   ```

5. 語 scope_m1 の字面は src と tests と design-intent に 30 行（参考値・base 2c3a631）在るが、**閉じた一覧の写しは上の 2 か所（生成区間と凍結 anchor）だけ**である。ほかは写しでない＝要件書の面の生成器が scope と scope_m1 の 2 節を描く表（`crates/folio/src/face_srs.rs`）・憲法の面の名札の文（M1 の段の説明）・設計ノートの欄の決まりの注の字（要件書 scope_m1.not_build を指す）・天井の正本の観点の読む欄（scope_m1 を読む 2 観点）・束の歯の期待の字（落とした節の列）・面の fixture の要件書。これらは本便で 1 字も触らない（§1 (h) の 2 と 3 で、要件書が scope_m3 の節を持った後の効きを書く）。組み直す手順:

   ```
   git grep -n 'scope_m1' 2c3a631 -- crates tests design-intent
   ```

6. base で要件書に scope_m3 の節を足すと、床は **不合格 1（未知の節 scope_m3 の 1 件）** を返す（起草役が base の写しの要件書の actors の直前に scope_m1 と同じ形の最小の節を足して撃った）。これが ADR-16 決定 (1) の言う「版上げより先に床の定数の改訂が要る」の実測である。
7. base の workspace の nextest は全部緑（参考値 779 本・`cargo nextest run --workspace`）。

### (b) 直す先 — 定数に 1 語を足し、写しを導出で書き、凍結 anchor を独立に組み直す

**① 定数（逐語・`crates/folio/src/check.rs`・base 2c3a631 の 44〜64 行目）。** 前:

```
/// 要件書の節の閉じた一覧（正本は床の定数・file の schema 節はその写し）。figures は任意の図の節（便 34・FR15）。
/// 末尾の schema は生成区間（便 77・ADR-11 決定 (4)⑤）。
pub const SRS_TOP_LEVEL: [&str; 17] = [
    「meta」,
    「goals」,
    「scope」,
    「scope_m1」,
    「actors」,
    …（以下 base のまま）
```

後（配列の長さを 17 から 18 へ・scope_m1 の直後に 1 語・頭の注に 1 文）:

```
/// 要件書の節の閉じた一覧（正本は床の定数・file の schema 節はその写し）。figures は任意の図の節（便 34・FR15）。
/// 末尾の schema は生成区間（便 77・ADR-11 決定 (4)⑤）。scope_m3 は M3 の範囲の節（便 117・判断の記録 ADR-16 決定 (1)(7)①・
/// 節の中身は要件書の版上げが書く）。
pub const SRS_TOP_LEVEL: [&str; 18] = [
    「meta」,
    「goals」,
    「scope」,
    「scope_m1」,
    「scope_m3」,
    「actors」,
    …（以下 base のまま）
```

（この節は引用符を置けないので、Rust の字の literal の二重引用符を「」で書いた。実物は base の字の literal と同じ二重引用符である。）

置く場所は **scope_m1 の直後**。理由は 2 つで、要件書の人が書く節の並び（scope → scope_m1 → actors）の段の順に揃えること、写しを読む人が段の節を 1 か所で見られることである。床は鍵の順を数えないので、順は写しの読みやすさだけに効く。M2 の範囲の節は要件書に無く（M2 の範囲は判断の記録 ADR-8 と要件の欄 milestone が持つ）、本便は scope_m2 を足さない（§1 (c) の 2）。SRS_FLOOR・unknown_sections・ほかの定数は 1 字も触らない（葉が定数そのものを指すので写しは自動で 1 語増える）。

**② 写し（生成区間）。** 定数を変えた後に次の命令で書く。人は生成区間を手で書かない（P-6.2）。

```
cargo run -q -p folio -- schema --write --dir design-intent
```

起草役の実測では、9 file のうち `design-intent/srs.yaml` だけが「書いた」になり、残り 8 file は「変わらない」だった。差分は生成区間の top_level の一覧の `    - scope_m1` の次に `    - scope_m3` の 1 行が入るだけで、生成区間の外の byte は 1 つも変わらない（参考値: 生成区間が 29 行から 30 行へ・1,174 byte から 1,189 byte へ・要件書の全体が 15 byte 増える・base 2c3a631）。**要件書の版・承認欄・人が書く節（meta・scope・scope_m1 ほか）は 1 字も変えない＝本便は要件書の版を上げない**（便 86 と同じ扱い。生成区間は機械が書く写しで、人が書いた中身は変わらないため）。要件書に scope_m3 の節そのものを書くのは版 B（持ち主の承認）で、本便ではない。

**③ 凍結 anchor（独立に組み直す・P-10.1 / P-10.2）。** `tests/fixtures/schema/srs-region.txt` は生成器の出力を写して作らない。手順は次の 3 つで、生成器を通らない。

1. base の anchor の `    - scope_m1` の行の直後に、`    - scope_m3` の 1 行を手で足す（ほかの行は 1 字も触らない）。
2. ② で書いた要件書の生成区間（印 2 本の間）と、1 の anchor が byte 一致することを `cmp` で確かめる（独立の 2 つの作り方の一致）。
3. anchor の行数と byte 数と要約値を `wc -l`・`wc -c`・`sha256sum` で測り、`crates/folio/tests/schema_docs.rs` の F77_REGIONS の srs.yaml の行の 3 つの値へ写す。

**④ 同じ anchor の値を 2 か所に持たない。** 同じ file の F86_SRS_LINES / F86_SRS_BYTES / F86_SRS_SHA256 は、数を書き直さず F77_REGIONS の srs.yaml の行（先頭の組）の 3 つの値を指す形に替える（Rust の定数の文脈で配列の添字と組の欄を引く）。寄せた行に添える注（便 117 の 2 行）は、この設計ノートの §1 (b)(d) を指す（§1 (c) は採らなかった形なので指さない）。anchor の値を pin する場所は F77_REGIONS の 1 か所になり、次に要件書の閉じた一覧を動かす便は 1 か所だけ直せばよい。F86_SRS_ANCHOR（置き場の path）は変えない。歯の関数名と本文と断りの字は 1 字も変えない。あわせて file の頭の注の便ごとの一覧の末尾に、便 117 の 2 行（何を足し、値をどう測り直したか）を足す。

### (c) 採らなかった形

1. **定数を字の頭（scope_m で始まる鍵）で受ける。** 閉じた一覧を型の外の規則で開くことになり、段の名の綴りの誤り（scope_m と scope_m4 など）を黙って通す。P-2.4 と N-3.1 の向きの逆で、ADR-16 決定 (1) も「最上位の節の閉じた一覧（床の定数）を改訂する」と書く。採らない。f117_ の歯の 2 本目がこの形を落とす（§1 (d)）。
2. **scope_m2 も同時に足す。** 判断の記録は M3 の節だけを求め、M2 の範囲は判断の記録 ADR-8 と要件の欄 milestone が持つ。使う予定の無い鍵を開けると、閉じた一覧の意味が「在ってよい」から「在るかもしれない」へ緩む。採らない。
3. **要件書の file の側の schema.top_level から床が読む。** 検査される側の data で閉じた一覧を広げられる形になる（便 77 の f77_ の歯が塞いでいる向き）。採らない。
4. **凍結 anchor の自己検査の値（行数・byte 数・要約値）を外し、生成区間と anchor の byte 一致だけにする。** anchor が生成器の出力の写しになっても歯が気づかず、生成物どうしの突き合わせだけが合格判定になる（P-10.2）。値の pin は anchor の同一性で、製品の数を固定する歯ではない。採らない。代わりに ④ で pin を 1 か所に寄せた。
5. **要件書の面の生成器（`crates/folio/src/face_srs.rs`）に scope_m3 の描画を足す。** 面の出力に効き、要件 FR4 の面の歯と凍結 fixture の範囲へ広がる。ADR-16 決定 (7) の ① は床の定数の便で、面は範囲に無い。本便では採らず、§1 (h) の 2 に「まだ分からない」として残す。

### (d) 歯（新しく 3 本・置き場は `crates/folio/tests/check.rs`）

新しい歯は `crates/folio/tests/check.rs` の末尾に、便 117 の見出しの注の下で 3 本足す。関数名は f117_ で始める（verify の絞り込みの語）。**どれも数を pin しない**＝一覧の長さも要件書の節の数も見ず、性質だけを見る。見本の節は歯の file の中の最小の手書きの字（scope_m1 と同じ形＝build と not_build の 2 つの一覧に 1 行ずつ）で、**新しい fixture の file も dir も作らない**。足し方は既存の歯と同じく、実の design-intent の写し（同じ file の Work・git init 済み）の要件書の最上位の節 actors の直前へ、字面の変異 1 か所（同じ file の mutate_file）で入れる。

1. **scope_m3 の節を持つ要件書は床を通る。** 写しの要件書に scope_m3 の節を足して `folio check` → 終了コード 0・違反 0 件・合格の 1 行（違反 0・まだ分からない 0）。**実の要件書が既に scope_m3 の節を持つとき（版 B の後）は足さずにそのまま撃つ**＝版 B の後も歯が重複鍵で落ちない。足した後の最上位の scope_m3 の鍵がちょうど 1 つであることも見る。**base では未知の節の違反 1 件で不合格 1 になる＝RED**（起草役が base の写しに歯だけを当てて実測）。
2. **ほかの段の名の節は未知の節のまま落ちる。** scope_m2・scope_m4・scope_m の 3 つを 1 つずつ別の写しに足して `folio check` → 終了コード 1・未知の節の違反がちょうど 1 件・その行が要件書を名指し、足した名を含む。base でも緑の歯で、閉じた一覧が 1 語だけ広がったこと（字の頭で受ける形や、余分な語を足す形を採っていないこと）を確かめる。
3. **実の要件書の生成区間の一覧は scope_m3 をちょうど 1 回、scope_m1 の直後に持ち、実の要件書の最上位の節はどれもその一覧に在る。** 実の `design-intent/srs.yaml` を読み、生成区間の top_level の一覧の項を拾って見る。**base では scope_m3 が一覧に無いので落ちる＝RED**。後半（実の最上位の節 ⊆ 一覧）は版 B が要件書に scope_m3 の節を書いた後も、写しが追随していることを見る。

歯の効き（起草役が変異を当てて測った・base 2c3a631 の写し）:

| 当てた形 | 歯 1 | 歯 2 | 歯 3 |
| --- | --- | --- | --- |
| base（定数も写しも base のまま） | 落ちる | 通る | 落ちる |
| 本便の形（scope_m1 の直後に 1 語・写しを導出） | 通る | 通る | 通る |
| scope_m3 を一覧の末尾に足す | 通る | 通る | 落ちる |
| scope_m2 と scope_m4 も一緒に足す | 通る | 落ちる | 落ちる |

定数を変えて写しを書き忘れると、既存の生成区間の歯（下の (e) の 1）が落ちる。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くもの・門

**1. 定数だけを変えた木で落ちる既存の歯**（参考値 16 本・起草役の実測・base 2c3a631 に定数の 1 語だけを当てて `cargo nextest run --workspace --no-fail-fast`）。原因は 1 つで、実の要件書の生成区間が導出とずれ、`folio schema --check` が非 0 を返すことである。内訳は `crates/folio/tests/schema_docs.rs` の 12 本（実の正本の置き場に命令を撃つ歯・f76_ / f77_ / f78_ / f85_ / f86_ / f95_ の各 1〜4 本と、天井の正本と規則の表の実の file の歯）、`crates/folio/tests/schema.rs` の 3 本（判断の記録と設計ノートの欄の決まりの実の file の歯と R-9 の母集団の歯）、`crates/folio/tests/ceiling.rs` の f102_ の 1 本。**どれも ② の導出（`folio schema --write`）で緑に戻り、歯の本文は直さない。** tests/schema.rs と tests/ceiling.rs は本文を変えないので write-set に入れない（verify もこの 2 本を `--test` で名指さない。workspace の nextest が走らせる）。

**2. 写しと anchor の file を直した後も残る歯**（参考値 4 本）。`crates/folio/tests/schema_docs.rs` の f77_ の 3 本（8〜9 行の一致と byte 数・3 本の凍結 anchor との byte 一致と自己検査・3 file のずれ）と f86_ の 1 本（要件書の生成区間が anchor と byte 一致し自己検査が通る）で、**原因はどれも anchor の自己検査の値の pin**（F77_REGIONS の srs.yaml の行と F86_SRS_*）である。(b) の ③ の 3 と ④ で直す。f77_ の自己検査の歯は定数だけの木では緑で（生成区間も anchor も base のまま一致するため）、写しと anchor を直した後にだけ落ちる。

**3. 凍結 anchor が動くもの・動かないもの。** 動くのは `tests/fixtures/schema/srs-region.txt` の 1 本だけ（1 行増える）。節点の要約値の anchor（`tests/fixtures/schema/node-digest-anchor.txt`）と索引の歯（`crates/folio/tests/graph.rs`）・束の凍結 anchor と所見 fixture・面の凍結 fixture は動かない（起草役の実測で、本便の後の workspace の nextest が全部緑で、これらの file は 1 byte も変えていない）。ほかの 8 file の生成区間も動かない（② の実測）。組み直す手順は (b) の ③ と、次の 1 行:

```
cargo nextest run --workspace --no-tests=fail --no-fail-fast
```

**4. 門（規則の表の開発規律行 D-12）。** 本便は要件書の正本を書き換えるので門の対象で、起草役の実測は **2（まだ分からない・印が古い）**（§0 の 門）。印は天井の周の結果から書かれ、本便の中身では変わらない。D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定めるので、**器へ出す前に、席が持ち主の裁定を名指すか、天井の周を回して印を新しくする**。先例は便 101 と便 103（2026-09-22）で、持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes）を D-12 の裁定として受け、印が古いまま出した。本便にその裁定を当てるかどうかは席が決め、当てるなら裁定 id を台帳の notes に記帳する（起草役は決めない）。本便が書き換える正本の中身は生成区間の 1 行だけで、人が書く字は 1 字も変わらない。

### (f) 大きさ（器の行数の式・幅 120 で正規化）

1. **write-set の印。** 5 本とも base に在る file を書き換えるので印なし。`+`（新しい file）・`-`（行が減る file）・`~`（着地で消える file）はどれも当たらない（器の受付の宣言の文法で、`~` は消える file の印である＝要件書に付けると着地で要件書を消す宣言になる。付けない）。
2. **余地（CapHeadroom）。** 測るのは write-set のうち `crates/folio/src/` の下の `+` と印なしの .rs、つまり `crates/folio/src/check.rs` の 1 本だけである。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | size S の見積 | 本便の後（参考値） |
| --- | --- | --- | --- | --- |
| `crates/folio/src/check.rs` | 887 | 613 | 100 | 889（2 行増える） |

   測り方: 各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない）。tests の 2 本（`crates/folio/tests/check.rs` と `crates/folio/tests/schema_docs.rs`）は src の外なので余地を測らないが、参考に書くと、tests/check.rs は 1,177 から 1,285、tests/schema_docs.rs は 1,137 から 1,141 で、後者は便 89 の歯（その file が器の式で 1,200 以下）の内に収まる。

3. **size は S。** 変える src は 1 本で 2 行、歯の file の増分も小さい。

### (g) verify と done の対応

verify は 3 行で、done の 3 つの塊と 1 対 1 に揃える。

1. `cargo nextest run -p folio --test check f117_` = (d) の新しい歯 3 本。
2. `cargo nextest run -p folio --test schema_docs --test check` = 書き換える歯の file 2 本の歯の全部（f77_・f78_・f86_ と、実の置き場の要件書に `folio check` を当てる歯と、便 77 の最上位の節の閉じた一覧の歯を含む）。
3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

verify が `--test` で名指す歯の file（tests/check.rs と tests/schema_docs.rs）は 2 本とも write-set に在る。絞り込みの語 f117_ は `--test check` の中だけで効き、src の単体の歯に同じ語で始まる関数は無い（base で `git grep -n 'fn f117_' 2c3a631` が 0 件）。共通の検証（.vessel.toml の common-verify）が workspace の nextest の全部と clippy を撃つ。

### (h) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 要件書の scope_m3 の節の中身と要件書の版上げ（版 B・持ち主の承認）。M3 の骨格の命令・列の根の表・始まりの凍結（ADR-16 決定 (7) の ②〜④）。導出物の独立の命令（⑤）。語彙・憲法・規則の表・判断の記録・天井の正本・入口の正本の変更。面の生成器と部品目録と様式。
2. **まだ分からない — 要件書の面が scope_m3 を描かない。** 要件書の面の生成器は scope と scope_m1 の 2 節を名指しの表で描き、ほかの最上位の節を読まない。起草役が本便の後の木で要件書に scope_m3 の節を足して `folio build --write` を撃つと、床は合格・組み立ても成功し、**scope_m3 の節の字は配信先のどの面にも 1 字も出なかった**（面の検査もそれを数えない）。版 B が scope_m3 の節を書くと、人が読む要件書の面からその節が黙って落ちる（P-6.1 の逐語の生成と、P-4.2 の向きに当たるおそれ）。**版 B の発効の前に、面が scope_m3 を描く便（要件 FR4 の面の側）を起こすか、描かないことを持ち主が裁定するかが要る。** 本便ではどちらも決めない。
3. **まだ分からない — 天井の観点が scope_m3 を読まない。** 同じ木で天井の束（`folio ceiling --write`）を組むと、4 観点すべての読む欄の宣言の外として scope_m3 が「落とした節」の行に記録された（黙っては落ちない）。scope_m1 は 2 観点が読んでいる。版 B の後に天井の観点が M3 の範囲を読むかは、天井の正本の読む欄（人が書く節）の判断で、本便では触らない。
4. **言えないこと。** 本便の歯は「床が scope_m3 の節を受ける」ことだけを確かめ、節の中身の形（build と not_build の 2 つの一覧か、ほかの欄を持つか）は数えない。今の床は scope と scope_m1 の中身の形（欄の名と型）も数えていない（base の `crates/folio/src/check.rs` の要件書の検査が欄を読む節の一覧に scope 系の節が無い）。中身の形を床に置くかは版 B の設計で決める。
5. **撤退条件。** (1) 定数を 1 語足して写しを導出したとき、`design-intent/srs.yaml` 以外の file の生成区間が 1 byte でも変わったら、本便の前提（写しは要件書の 1 か所）が崩れているので止めて席へ返す。(2) (e) の 1 と 2 に挙げた歯のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す（歯の性質が本便の想定の外で定数に依っている）。(3) 実測 patch が base に当たらない（受付の時点の main が 2c3a631 から動き、`crates/folio/src/check.rs` の定数の周りか要件書の生成区間か tests/schema_docs.rs の F77_REGIONS の周りが書き換わっていた）ときは、数を測り直してから運ぶ。

### (i) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は `~/.local/share/folio2/handoff-2026-09-24/d117-measured.patch`（base 2c3a631 に `git apply --check` が通る・repo には入れない）。起草の記録は同じ dir の `d117-draft.md`。

1. base の写しを作る: `git clone --shared <folio2> <写し>` の後に `git -C <写し> checkout 2c3a631`。
2. 定数だけの木: (b) の ① だけを当てて `cargo nextest run --workspace --no-fail-fast` → (e) の 1 の FAIL の行。
3. 写し: `cargo run -q -p folio -- schema --write --dir design-intent` → 「書いた」の行が要件書だけ・`git diff --stat` が要件書の 1 行だけ。
4. anchor: (b) の ③ の 1〜3。
5. 歯: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）・`cargo run -q -p folio -- schema --check`（9 行とも 一致）・`cargo run -q -p folio -- check`（合格・違反 0・まだ分からない 0）。
6. RED: base の写しに歯の file（tests/check.rs）だけを当てて `cargo nextest run -p folio --test check f117_` → 歯 1 と歯 3 が落ち、歯 2 は通る。
7. 門: `cargo run -q -p folio -- ceiling --gate --write-set <write-set の 5 本>`。
8. 余地: (f) の 2 の式で `crates/folio/src/check.rs` を数える。

## 2. 範囲

- 入れる: `crates/folio/src/check.rs` の SRS_TOP_LEVEL に scope_m3 の 1 語（scope_m1 の直後・配列の長さを 18 へ）と頭の注の 1 文・`design-intent/srs.yaml` の生成区間の 1 行（`folio schema --write` の導出）・`tests/fixtures/schema/srs-region.txt` の 1 行（手で足す）・`crates/folio/tests/schema_docs.rs` の F77_REGIONS の srs.yaml の行の 3 つの値と F86_SRS_* の 3 つを F77_REGIONS の行を指す形に・頭の注の 2 行・`crates/folio/tests/check.rs` の f117_ の歯 3 本と見本の節の字と小さな helper 2 つと頭の注の 2 行。
- 入れない: 要件書の版・承認欄・人が書く節（scope_m3 の節そのものを含む）・ほかの 8 file の生成区間・SRS_FLOOR のほかの葉・床の判定と断りの字面・面の生成器・天井の正本の読む欄・判断の記録・語彙・規則の表・新しい命令と旗・新しい file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| const | 要件書の最上位の節の閉じた一覧 | `crates/folio/src/check.rs` の SRS_TOP_LEVEL（scope_m1 の直後に scope_m3） |
| region | 生成区間 | `design-intent/srs.yaml` の schema の節の top_level（`folio schema --write` が書く） |
| anchor | 凍結 anchor | `tests/fixtures/schema/srs-region.txt`（手で 1 行）と、その自己検査の値（`crates/folio/tests/schema_docs.rs` の F77_REGIONS の srs.yaml の行） |
| teeth | 歯 | `crates/folio/tests/check.rs` の f117_ の 3 本 |

## 4. 検査（歯）

§1 (d) と (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい file も dir も無い。
- 前提の着地: ADR-15 の列（便 106〜116）と一括 14（ADR-16 決定 (7)）。どちらも main 2c3a631 で済んでいる。
- 一括 15（docs/batch15）とは `design-intent/srs.yaml` の同じ file を触るが、字面の塊は重ならない（§0 の 並行の便との重なり）。どちらが先でもよい。
- 本便の着地の後に、席が要件書の版 B（scope_m3 の節の中身・骨格の命令と列の根の表と始まりの凍結の要件と受入基準）を起草し、持ち主の承認を得る（ADR-16 決定 (7)）。その前に §1 (h) の 2 の裁定が要る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dp"
title = "要件書の最上位の節の閉じた一覧（床の定数 crates/folio/src/check.rs の SRS_TOP_LEVEL）に M3 の範囲の節 scope_m3 を scope_m1 の直後へ 1 語足し（配列の長さを 17 から 18 へ・頭の注に ADR-16 の 1 文）、写しを folio schema --write で要件書の生成区間へ導出し（要件書の全 file のうち生成区間の top_level の 1 行だけが増え、版・承認欄・人が書く節は 1 字も変えない）、凍結 anchor tests/fixtures/schema/srs-region.txt に同じ 1 行を手で足して生成区間と byte 一致させ、その自己検査の値（行数・byte 数・要約値）を wc と sha256sum で測り直して crates/folio/tests/schema_docs.rs の F77_REGIONS の要件書の行へ写し、同じ file の F86_SRS_LINES・F86_SRS_BYTES・F86_SRS_SHA256 は数を書き直さず F77_REGIONS の要件書の行を指す形に替える。これは判断の記録 ADR-16（発効 2026-09-23 17:39 JST・持ち主の承認・逐語 裁定は両方承認する）の決定 (1) の末尾と決定 (7) の便の列の 1 本目で、要件書の版 B（scope_m3 の節の中身）より先に要る床の定数の改訂である。歯は crates/folio/tests/check.rs に 3 本足し（scope_m3 の節を持つ要件書の写しが床を通る・scope_m2 と scope_m4 と scope_m の節は未知の節のまま落ちる・実の要件書の生成区間の一覧が scope_m3 をちょうど 1 回 scope_m1 の直後に持ち実の最上位の節がどれも一覧に在る）、どれも数を pin せず、見本の節は歯の file の中の最小の手書きの字で新しい fixture の file も dir も作らない。床の判定と断りの字面・SRS_FLOOR のほかの葉・ほかの 8 file の生成区間・面の生成器・天井の正本・判断の記録・語彙・規則の表・ほかの凍結 anchor は 1 字も触らず、file も消さない"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/check.rs", "design-intent/srs.yaml", "tests/fixtures/schema/srs-region.txt", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test check f117_", "cargo nextest run -p folio --test schema_docs --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f117_ の歯 3 本（scope_m3 の節を持つ要件書の写しに folio check が合格〔違反 0・まだ分からない 0〕を返し、実の要件書が既に節を持つときは足さずに撃つ／scope_m2・scope_m4・scope_m の節は 1 つずつ不合格 1 で未知の節がちょうど 1 件でその名を含む／実の要件書の生成区間の top_level が scope_m3 をちょうど 1 回 scope_m1 の直後に持ち、実の要件書の最上位の節がどれもその一覧に在る）が緑、crates/folio/tests/schema_docs.rs と crates/folio/tests/check.rs の既存の歯が全部緑（要件書の生成区間が手で組み直した凍結 anchor と byte 一致し、その行数・byte 数・要約値が wc と sha256sum の測り直しと一致する f77_ と f86_ の歯と、実の置き場に folio schema --check を撃って 9 file とも一致する歯と、実の置き場に folio check を撃って合格する歯を含む）、clippy が 0 警告で、workspace の nextest が全部緑で CI が通る"
<!-- contracts:end -->
