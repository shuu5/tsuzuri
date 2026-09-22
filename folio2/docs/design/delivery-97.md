# 設計: 便 97 / 便 98 — 天井の束の凍結 anchor を folio に依らない script へ移し、束を読む欄まで絞る（FR17）

- 要件: FR17（天井の材料の束を観点ごとに組む）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1（検査・生成が実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは まだ分からない として表に出す）/ P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、写しを設計文書の置き場へ導出する）/ P-6.2（生成物を手で直さない）/ P-10.1（独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-10.3（anchor が維持できなくなった検査の結果は まだ分からない に落とす）/ N-1.1（管理下の対象を回復不能に削除する操作を拒む）/ N-3.1（規則の例外機構を足さない）
- 出所: 判断の記録 ADR-13 決定 (5)（天井の束を影響集合で切らない。束は観点が宣言する読む欄まで絞ったうえで丸ごと渡す）・決定 (10)（絞るとき、束の凍結 anchor を folio の実装に依らずに作り直す。欄を切り出す独立の script を anchor と一緒に凍結し、維持できなくなったらその検査の結果を まだ分からない に落とす）・決定 (16) の 4 本目（束を読む欄まで絞る・凍結 anchor の作り直しを含む）。設計ノート docs/design/graph-and-incremental-ceiling.md §3.1・§8 の便の列の G3 と、その下の節「G3 の凍結 anchor」。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ct が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は 2 本で先頭に + を付ける・縮む file は無い・新しい dir は作らない）。
- 門: 2 便とも design-intent の下の正本を 1 file も書き換えない（読むだけ）ので、天井の門の対象外である（要件書 FR20 の規範文の 1 つ目・判断の記録 ADR-13 決定 (16) の末尾も ④ を門の対象外と書く）。**起草役が本便の write-set をそのまま渡して folio ceiling --gate を実測し、0（通す・断りの字は 設計文書の正本を書き換えない便）を確かめた**（2026-09-22・main 0806433）。
- 前の便: 便 94（行 cq）・便 95（行 cr）・便 96（行 cs）は **3 本とも着地済み**で、本便の base は main 0806433 である。便 96 の write-set（graph.rs・hello.rs・main.rs・tests/graph.rs・tests/hello.rs・tests/fixtures/schema/graph-digest-anchor.txt）と本便の write-set は **1 file も重ならない**。
- 改訂 b（2026-09-22 16:4x JST・検証役の report `~/.local/share/folio2/handoff-2026-09-22/d97-verify.md` への応答・落ちの数 13 → 17）: §1 (a) の精度の表と落ちる先の表を、最上位の節の始まりを 列 0 の 名: の行だけとする数え方で取り直した。§1 (d) にその規則の 1 文を足し、便 98 の設計に実の正本へ当てる不変条件の歯を 1 本足すことを書き添えた。**行 ct・凍結 anchor の 4 対・write-set・行 ct の歯 3 本は 1 つも変えていない**（変えたのは §1 の数と 1 文と、便 98 の歯 6 本 → 7 本の見込みだけ）。
- **G3 を 2 便に割った**。理由は §1 (b)。**契約表に置くのは行 ct（便 97）だけ**で、便 98 の行はこの便の着地後に docs/design/delivery-98.md で起こす。便 98 の write-set は行 ct が作る 2 本（bundle-anchor.py と .txt）を含み、その 2 本が base に無いあいだは write-set が解けないためである（起草役が 2 行の契約表で scribe2 contracts check を実測し、行 cu に write-set-item-unresolved が 2 件出ることを確かめた）。**便 98 の中身は §1 (d)(e)(f)(g)(h)(i) に全部書いてあるので、そのまま delivery-98.md へ移せる。**

## 1. 目的と中身

天井の束はいま、観点が読む文書の正本を **byte のまま丸ごと**写す。判断の記録 ADR-13 決定 (5) は、これを観点が宣言する読む欄まで絞る（無関係な欄だけ落とす）と決めた。同じ判断の決定 (10) は、絞ると写しが folio の産物になるので、凍結 anchor を folio の実装に依らない形へ作り直せと決めた。

**この 2 つは依存の向きが逆である。** anchor の作り直しは絞る前でもできる（今の束に対して作れる）。絞る方は anchor の作り直しが先に要る。そこで **anchor を先に出し（便 97）、絞るのを後に出す（便 98）**。

### (a) 実測（2026-09-22・base main 0806433）

**src と歯の余地**（幅 120 で正規化＝全行 + 各行の ceil(len/120) − 1・上限 1,500）。

| file | 正規化の行 | 余地 | 本便での役 |
| --- | --- | --- | --- |
| crates/folio/src/bundle.rs | 539 | 961 | 便 98 が絞る口を置く（便 97 は触らない） |
| crates/folio/tests/bundle.rs | 644 | 856 | 2 便とも歯を置く |
| crates/folio/tests/findings.rs | 1,264 | **236** | 便 98 が凍結の要約値 2 つと reads.yaml の行の形の読み手を直す |

**crates/folio/tests/findings.rs の余地は 236 しか無い。** 便 98 がそこへ置くのは 8 行以内（凍結の要約値 2 つ・注釈の行を読み飛ばす枝 2 か所・歯 1 本）に収める。src で M の見積 300 を要るのは bundle.rs 1 本だけで、その余地は 961 ある。

**束の byte**（実の正本・23 周目〔round23〕の束を土台に、絞ったときの大きさを測り直した）。

| 観点 | 束 前 | 束 後 | 割合 | うち sources 前 | sources 後 |
| --- | --- | --- | --- | --- | --- |
| 忠実さ | 1,450,560 | 1,311,214 | 90.4% | 518,973 | 379,627（73.1%） |
| 読みやすさ | 1,475,886 | 1,272,880 | 86.2% | 528,706 | 325,700（61.6%） |
| 整合 | 1,495,448 | 1,386,704 | 92.7% | 548,200 | 439,456（80.2%） |
| 実態 | 1,418,739 | 1,238,949 | 87.3% | 487,486 | 307,696（63.1%） |
| 合計 | 5,840,633 | 5,209,747 | **89.2%** | 2,083,365 | 1,452,479（69.7%） |

**設計ノート §0.1 の表が書く「100% → 39.6%」は、束ではなく sources だけを、行の中の欄まで降りて測った数である。** 束は面（faces/）の HTML が大半（観点あたりおよそ 93 万 byte）を占めるので、束の全体では **10.8% しか減らない**。字数は目的ではない（持ち主の裁定・ADR-13 決定 (0)）ので、これは本便を止める理由にはならないが、**本便を字数のために出すのではない**ことははっきりさせておく。本便の効き目は雑音を落とすこと（精度）にある。

**精度の実測（いちばん重い数）。** 過去の周の所見の根拠（evidence）は、床（crates/folio/src/findings.rs の verbatim）が **束の sources/ の下の 1 行の中に byte 列として在ること**を照合する。絞ると、落とした節に在った根拠は照合に落ちる＝観点が正しい所見を立てても床が 根拠が正本に無い と言う。過去の周の所見 file を全部読み、それぞれの周の reads の宣言で絞ったときに根拠が束の外に出る件数を数えた。

| 絞る粒度 | 直近 10 周（14〜23 周）183 件のうち束の外 | 全 25 周 600 件のうち束の外 |
| --- | --- | --- |
| 行の中の欄まで降りる・骨格なし | 38（20.8%） | 135（22.5%） |
| 行の中の欄まで降りる・骨格あり | 25（13.7%） | 93（15.5%） |
| **最上位の節まで・骨格あり（採る案）** | **17（9.3%）** | 67（11.2%） |
| 最上位の節まで・骨格あり・宣言を実測に合わせて広げた後 | **0** | — |

（全 25 周の数はその周の時点の宣言で測ったもので、宣言は周ごとに違う。直近 10 周が今の宣言に当たる。）

採る案でも今の宣言のままでは **17 件が落ちる**。落ちる先は 8 つ（doc と節の対）に集まっている。

| 観点 | 落ちた節 | 件 |
| --- | --- | --- |
| 読みやすさ | srs の figures / index の intake / srs の acceptance / adr の context / adr の note | 5 / 2 / 1 / 1 / 1 |
| 忠実さ | srs の nonfunctional / srs の scope_m1 / adr の note | 1 / 1 / 1 |
| 整合 | srs の scope / srs の scope_m1 / adr の note | 1 / 1 / 1 |
| 実態 | adr の note | 1 |

**この 17 件は「見えなくなる」だけでは済まない。** 8 つの節はどれも面（srs.html・adr-n.html・index.html）には出ているので、観点は面でそれを読み、正しい所見を立て、床が 根拠が正本に無い で落とす。**したがって絞る前に、この 8 つを reads の宣言へ足す必要がある**（§1 (f) の 🔴 1）。足したうえで測り直すと、直近 10 周の落ちは **0 件**になり、sources は 78.6%（束では 92.3%）に収まる。

### (b) なぜ 2 便に割るか

| | 便 97（行 ct・この文書の契約表） | 便 98（delivery-98.md で起こす） |
| --- | --- | --- |
| 中身 | 凍結 anchor を独立の script とその出力へ作り直す | 束を読む欄まで絞る |
| 触る src | **無し** | crates/folio/src/bundle.rs |
| 先に要る承認 | **無し** | 一括 1 回（§1 (f)） |
| anchor の値 | **今の値と 1 字も違わない** | script が組み直した新しい値 |
| 見積の行 | 212 | 223 |

割る理由は 3 つある。

1. **script が正しいことを、既知の正しい値で先に示せる。** 便 97 の anchor の 4 対（束の file の数・連結の byte 数・要約値）は、いま crates/folio/tests/bundle.rs の expected() に手で置いてある値と 1 字も違わない。script がその 4 対を byte まで再現したなら、script は folio と同じ束を組めている。1 便にまとめると、値が合わないときに script の誤りか絞る口の誤りかを分けられない。
2. **前提が違う。** 便 97 は design-intent も要件書も触らず、先に要る承認が無い。便 98 は §1 (f) の一括が先に要る。割れば便 97 は今すぐ運べる。
3. **1 便に入れると M（300 行の見積）を超える。** 見積は合わせて 450 行で、この repo の契約表は S と M しか使ったことが無い（過去 97 行のうち S 68・M 29）。割れば 2 便とも M に収まる。

### (c) 便 97 — 独立の script と凍結 anchor（P-10.1 / P-10.2 / P-10.3）

いまの凍結 anchor は 2 面に分かれている。中身（凍結の土台 tests/fixtures/ceiling/bundle/ の正本と面）は file で、期待の値（観点ごとの file の一覧・連結の byte 数・要約値）は crates/folio/tests/bundle.rs の定数である。値の出所は **席が cp と sha256sum で手で組んだ**ことだけで、手順が版管理に残っていない。手順が残っていないと、絞ったあとに誰も同じ値を組み直せない（P-10.2 が禁じる形＝生成物どうしの突き合わせだけが残る）。

便 97 は、その手順を **file の 1 本**にする。

- 置き場と名: `tests/fixtures/ceiling/bundle-anchor.py`（凍結の土台 bundle/ の**外**・同じ dir の直下。新しい dir は作らない）。crates/folio/tests/stamp.rs は tests/fixtures/ceiling/bundle を丸ごと写すので、script と出力をその下に置くと束の中身が変わる。だから bundle/ の外に置く。
- 言語: **python3 の標準 library だけ**（PyYAML を使わない）。理由は 2 つ。① CI の runner（.github/workflows/ci.yml・ubuntu-latest）は rust と node だけを支度しており、PyYAML が居る保証が無い。居ないときに黙って通ると P-4.1 に反する。② 正本の YAML は 2 字下げの決まった形なので、行と字下げだけを見る読み口（およそ 40 行）で足りる。**folio の code は 1 行も呼ばない**（別の言語・別の実装）。
- 中身: 凍結の土台の ceiling.yaml から documents と viewpoints と weights を読み、観点ごとに ① sources/（正本の写し）② faces/（面の名の形で選ぶ）③ question.yaml ④ finding.yaml ⑤ reads.yaml を組み、観点の dir からの相対 path の byte 順に中身を連結して hashlib で sha256 を取る。出力は次の anchor file と同じ字を標準出力へ書く。
- 出力（凍結 anchor）: `tests/fixtures/ceiling/bundle-anchor.txt`。1 行 = 種別 + タブ区切り。種別は 観点 / file数 / byte / 要約値 の 4 つ（便 98 で 落とす と 無い の 2 種別が足される）。**便 97 の時点の値は次のとおりで、これは今 crates/folio/tests/bundle.rs に在る 4 対と 1 字も違わない。**

```
# 凍結 anchor: 天井の材料の束（tests/fixtures/ceiling/bundle-anchor.py が組んだ・folio の code を 1 行も呼ばない）
観点	fidelity
file数	13
byte	15055
要約値	2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32
観点	readability
file数	15
byte	17317
要約値	ded1548e1b6c5185b5b2ea0083d6ee68ed66dd5ca998bdd7d5d9a52ed2065576
観点	coherence
file数	14
byte	15770
要約値	91ca924dceef12d5b2df3946e6b56de69b1feeef13ea66f78d950af54c28476c
観点	reality
file数	11
byte	11385
要約値	24ce1b872fa08dd128b39ac42e4ba72096bfe263df3bb463c9616899c96786d7
```

- **維持できなくなったときの形（P-10.3）**: script を走らせる歯は、python3 が起動できないときに標準エラーへ `# まだ分からない: ` で始まる 1 行を出し、**歯を落とさない**。これは crates/folio/tests/bundle.rs の assert_digest が sha256sum を起動できないときに取っている形と同じで、便 97 はその形を写すだけである（新しい例外の口ではない・N-3.1）。folio の束と anchor file を突き合わせる歯は python3 に依らないので、道具が無い環境でも凍結の値との照合は残る。
- **突き合わせは唯一の合格判定ではない（P-10.2）。** anchor の値の出所は独立の実装であり、folio の出力との一致は歯が見る事実の側である。

### (d) 便 98 — 絞る粒度（採る案と、退けた案）

**採る案: 最上位の節（欄）の単位で切る。行の中の欄には降りない。** 切る単位は行で、残す行は正本の byte のまま・正本の順のまま写す。

| 何を残すか | 理由 |
| --- | --- |
| その観点の reads が挙げた節（`articles.plain` なら `articles` の節ぜんぶ） | 観点が読むと宣言した所 |
| 骨格の 6 つの節（meta・id・title・status・date・schema） | どれも本文ではなく、どの版の何を読んでいるかを決める欄。落とすと所見が場所を名指せない。判断の記録は id と title と status が最上位の欄なので、落とすとどの判断かが分からなくなる。schema（生成区間・欄の決まり）は過去に 止める を 4 件生んだ場所（設計ノート §3.4） |
| file の頭（最初の節より前の行）と、各節の直前の連続した注釈 | 節の見出しの注釈は節の一部。落とすと節の意味が取れない |

**最上位の節の始まりは 列 0 の `<名>:` の行だけとする。列 0 の `- ` で始まる行は直前の節の値の続きであって、新しい節ではない。** 束が写す正本でこの形を持つのは 4 本（`srs.yaml` の figures と `adr/ADR-4.yaml`・`ADR-5.yaml`・`ADR-7.yaml` の figures・起草役が design-intent の下を全数走査して確かめた。ほかに `anchors/constitution-v1.0.yaml` が同じ形だが、束の読む文書の一覧に無いので束に入らない）。**この 1 文が無いと、列 0 の連なりの先頭の欄の名（`id`）を節の名と読んで骨格として残し、figures の節が丸ごと残る**（起草役の数えの script が最初にその誤りをして、直近 10 周の落ちを 17 件でなく 13 件と出した・改訂 b の出所）。**凍結の土台 `tests/fixtures/ceiling/bundle/source/` の 8 本には列 0 の `- ` が 1 行も無いので、凍結の土台だけを見る歯ではこの誤りを捕まえられない。** だから便 98 の歯は、実の正本に当てる不変条件（歯 7・歯 8）に加えて、**実の正本の `srs.yaml` と `adr/ADR-4.yaml`・`ADR-5.yaml`・`ADR-7.yaml` について、宣言に figures を持たない観点の写しに figures の節の行が 1 行も残らないことを見る歯を 1 本足す**（不変条件で書き、節の byte も行数も固定しない）。

**退けた案 1: 行の中の欄まで降りる**（`articles.plain` なら articles の各行の id と plain だけ残す）。sources は 69.1% から 61.1% へ縮む（束では 3.2 point ぶん）が、直近 10 周の根拠の落ちが 17 件から 25 件へ増える（§1 (a)）。**字数は目的でなく精度が目的（ADR-13 決定 (0)）なので、8 件の落ちを sources の 8 point と引き換えにはしない。** 加えて、行の中の欄の多くは 1 行の flow の表（`- {id: P-1.1, pattern: ubiquitous, strength: must, text: …}`）で、その一部だけを落とすには行を組み直すしかない。組み直すと byte が変わり、床の根拠の逐語の照合（1 行の中の byte 列）が総崩れになる。**逐語の照合を守る限り、切る単位は行より細かくできない。**

**退けた案 2: どの観点も宣言していない節だけを落とす（4 観点で同じ束）**。落ちは 8 件に減るが sources は 88.7% までしか縮まず、観点ごとに雑音を落とすという ADR-13 決定 (5) の形にならない。**落ちが 0 にならないので、🔴 1 の宣言を広げる手はどのみち要る。**

**退けた案 3: 束を切らない（今のまま）**。ADR-13 決定 (5) が既に決めている。

**骨格の一覧は実装の型付きの定数に置く**（crates/folio/src/bundle.rs の 6 語の閉じた一覧・P-5.1）。その写しを設計文書の置き場へ導出するか（P-5.6）は 🔴 3 に上げる。当面は束の reads.yaml の注釈が読み手に 6 語をそのまま見せる。

### (e) 便 98 — 落とした欄の知らせ方（P-4.1 / P-4.2）

落としたことを黙らない。置き場は **reads.yaml の末尾の注釈の行**にする。

```
- {doc: constitution, fields: [articles.plain, articles.statements.text]}
- {doc: srs, fields: [requirements.plain, requirements.shall, acceptance.plain, acceptance.title]}
# 落とした節（正本には在るが、この観点が読むと宣言していない最上位の節）constitution.yaml: north_star, precedence, rules_pointer, amendment, glossary_pointer, sources
# 落とした節 srs.yaml: goals, scope, scope_m1, actors, outputs, rail, verdicts, nonfunctional, not_frozen, constraints, glossary_pointer, figures
# 常に残す節: meta, id, title, status, date, schema
# 宣言に在るが正本に無い節 rules.yaml: rows
```

置き場を reads.yaml にする理由。

- **束の中身の閉じた一覧を動かさない。** 束の中身は床の定数 BUNDLE_CONTENTS の 5 つ（sources・faces・question・finding・reads）で、天井の正本 ceiling.yaml の生成区間にその写しが在る。file を 1 本足すと、床の定数・生成区間・凍結 anchor tests/fixtures/schema/ceiling-region.txt・crates/folio/src/findings.rs の読み直し・歯が同時に動き、design-intent を書き換える便になる（＝承認が要る）。**reads.yaml に入れれば 1 つも動かない。**
- **sources/ の中へ書かない。** 床の根拠の照合は sources/ の下の行だけを見る。折り込みの注釈を sources/ の写しに足すと、**folio が書いた字が根拠として通ってしまう**（正本に無い字が逐語の照合を抜ける穴）。reads.yaml は照合の母集団の外なので、この穴が開かない。
- 注釈の行なので reads.yaml は YAML として今までどおり読める。crates/folio/tests/findings.rs の 2 か所に在る reads.yaml の行の形の読み手（`- {doc: …}` 以外の行を形が違うと言って落とす）に、`#` で始まる行を読み飛ばす枝を足す（2 行）。

**宣言に在るが正本に無い節**（その doc のどの file にも無い節）は別立てで出す。file ごとの欠け（判断の記録の置き場の中で、ある file にだけ figures の節が無い等）は普通のことなので数えない。**実の正本ではこれが 0 件**（4 観点の宣言 29 対・欄の名 74 個を全部当てて確かめた）で、凍結の土台では 2 件（読みやすさ の index.yaml の sections・整合 の rules.yaml の rows）出る。**束を組むのは止めない**（P-4.2 の 表に出す であって、P-4.1 の 実行できなかった ではない。宣言の古さは正本の側の問題で、束は完全に組めている）。標準出力の 1 行にも、落とした節の数と 宣言に在るが正本に無い節 の数を足す。

### (f) 便 98 の前に要る一括と、次の一括の 🔴

**🔴 1（便 98 の前提・天井の正本 ceiling.yaml の版上げ）**: reads の宣言を実測に合わせて広げる。広げないまま便 98 を着地させると、直近 10 周で 17 件（9.3%）の根拠が束の外に出て、**観点が面を見て立てた正しい所見を床が 根拠が正本に無い で落とす**。足す先は 8 対だけである。

| 観点 | 足す doc と節 |
| --- | --- |
| 忠実さ | srs に nonfunctional と scope_m1・adr に note |
| 読みやすさ | srs に acceptance と figures・adr に context と note・index に intake |
| 整合 | srs に scope と scope_m1・adr に note |
| 実態 | adr に note |

**🔴 2（便 98 の前提・要件書の版上げ）**: 要件 FR17 の規範文は束の中身を 正本の写し（基準） と書く。節を落とした写しはその字より狭い。設計ノート §8.1 は FR17 の改訂を G8 の前に 1 回まとめると書くが、**便 98 が先に着地すると、次の周の 実態 の観点が 止める を立てる見込みが高い**（同じ形の前例＝便 94 の 🔴 4 点目・便 96 の 🔴 3 点目）。🔴 1 と同じ一括に載せる。

**🔴 3（P-5.6 の写し）**: 骨格の 6 語は実装の型付きの定数だが、生成区間への導出の対象（crates/folio/src/schema.rs の 9 file）に入らない。導出の対象へ足すか、天井の正本の生成区間に持たせるか、写しを持たないかを決める。導出の対象を増やすと天井の正本の版上げを伴うので、🔴 1 と同じ承認にまとめられる。

**🔴 4（設計ノートの数の直し）**: §0.1 の表は 読む欄まで絞る の効き目を 100% → 39.6% と書く。これは sources だけを行の中の欄まで降りて測った数で、**束では 90.0%**（採る案・宣言を広げた後は 92.4%）である。値も判定も動かない字の直し。

**🔴 5（判断の記録の帰結の字）**: ADR-13 決定 (5) は 精度は同じか上がる と書くが、**今の宣言のままでは偽**（実測 17/183）。宣言を広げれば真になる。決定の字を変えるのではなく、帰結（consequences）に実測と前提（宣言を広げること）を 1 文添える形を勧める。

**🔴 6（粒度を下げるか）**: 宣言を広げたあと、行の中の欄まで降りる案をもう一度測り直すかどうか。本便では降りない。

### (g) 歯

歯は実行 file 経由で測る（env!(CARGO_BIN_EXE_folio) を起動する形）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、**verify の行に --bin は書かない**。

**便 97（f97_ で始まる 3 本・置き場は crates/folio/tests/bundle.rs）**

1. **f97_the_script_rebuilds_the_frozen_anchor** — tests/fixtures/ceiling/bundle-anchor.py を python3 で走らせ、標準出力が tests/fixtures/ceiling/bundle-anchor.txt と **byte 一致**すること。python3 を起動できないときは標準エラーへ `# まだ分からない: ` の 1 行を出して落とさない（P-10.3・assert_digest の形の写し）。**赤い歯**（本便の前に script も anchor file も無い）。
2. **f97_the_anchor_file_agrees_with_the_teeth** — anchor file の 4 観点の file数・byte・要約値が、同じ file に在る expected() の 4 対と一致すること。**これは 便 97 が値を 1 つも動かしていないことの歯である。赤い歯**（anchor file が無い）。
3. **f97_the_bundle_matches_the_anchor_file** — 凍結の土台から folio ceiling --write で 4 観点の束を組み、各観点の file の数・連結の byte 数・digest.txt の要約値が anchor file の値と一致すること（要約値は sha256sum でも測り直す）。**赤い歯**（anchor file が無い）。

**便 98（f98_ で始まる 7 本・置き場は crates/folio/tests/bundle.rs 6 本と crates/folio/tests/findings.rs 1 本）**

4. **f98_the_cut_bundle_matches_the_rebuilt_anchor** — 絞る規則を写した script の出力が作り直した anchor file と byte 一致し、folio ceiling --write の 4 観点の束の file の数・連結の byte 数・要約値が同じ値になること。凍結の土台での絞りの実測は 忠実さ 13,041 → 9,274 byte（71.1%）・読みやすさ 15,333 → 9,284（60.5%）・整合 13,880 → 9,076（65.4%）・実態 9,690 → 5,521（57.0%）で、**file の一覧は 1 本も増減しない**（落とすのは行であって file ではない）。確定の値は anchor file が持ち、歯はそれと突き合わせる。**赤い歯**。
5. **f98_the_dropped_sections_are_named_in_reads** — 凍結の土台の 忠実さ の reads.yaml に、constitution.yaml の落とした 6 節（north_star・precedence・rules_pointer・amendment・glossary_pointer・sources）と srs.yaml の落とした 12 節（goals・scope・scope_m1・actors・outputs・rail・verdicts・nonfunctional・not_frozen・constraints・glossary_pointer・figures）と、常に残す 6 語が注釈の行に出ること。**数を固定してよいのは凍結の土台だからである**（実の正本には当てない）。**赤い歯**。
6. **f98_a_section_absent_from_every_source_is_named** — 凍結の土台の 読みやすさ の reads.yaml に index.yaml の sections が、整合 の reads.yaml に rules.yaml の rows が 宣言に在るが正本に無い節 として出ること。どちらの場合も束は組めて終了コードは 0 であること。**赤い歯**。
7. **f98_the_skeleton_and_the_head_survive_every_cut** — 凍結の土台と**実の正本**の両方で、4 観点の sources/ の下の全 file について、骨格の 6 つの節の見出しの行と file の頭の行が 1 行も落ちていないこと。**不変条件で書き、節の数も byte も固定しない**（実の正本は便のたびに動くため）。**赤い歯**。
8. **f98_every_kept_line_is_verbatim_and_in_order** — 凍結の土台と実の正本の両方で、絞った写しの各行が正本の同じ file の行として byte のまま在り、写しの中の順序が正本の中の順序と同じであること（＝折り返しも組み直しもしていない＝床の逐語の照合が成り立つ）。**不変条件**。**赤い歯**。
9. **f98_the_frozen_findings_still_pass_the_check**（crates/folio/tests/findings.rs） — 絞った束に凍結の所見 fixture を当て、3 値が今と同じであること（pass-fidelity / pass-readability / pass-coherence / pass-reality が 合格・stop-upheld と stop-refuted と stop-unrefuted と fail-no-findings と missing-field と digest-mismatch と fabricated-evidence が今と同じ結果）。**起草役が 11 本の根拠を絞った束に当てて確かめた**（pass と stop の根拠は絞った束の中に残り、fabricated-evidence の作り話の根拠は絞る前も後も束の外）。**赤い歯**（凍結の要約値が動くため）。

10. **f98_a_column_zero_sequence_is_not_a_new_section**（便 98・(d) の 1 文の歯） — 実の正本の `srs.yaml` と `adr/ADR-4.yaml`・`ADR-5.yaml`・`ADR-7.yaml` について、`figures` を宣言に持たない観点の写しに `figures` の節の行が 1 行も残らず、`figures` を宣言に持つ観点の写しには残ること。**不変条件で書き、節の byte も行数も固定しない。凍結の土台にはこの形が無いので、この歯だけが列 0 の連なりの読み違いを捕まえる。** **赤い歯**。

10 本とも期待値を起草役が独立に出した（1・2・3・4 は (c)(d) の独立の実装の出力、5・6 は凍結の土台の実測、7・8・10 は不変条件、9 は根拠 11 本の実測）。回帰は verify の 2 行目（tests/bundle.rs と tests/findings.rs の全部）と .vessel.toml の common-verify（workspace 全体の nextest と clippy）が見る。

### (h) 大きさ

**便 97（行 ct・M）**

| 何 | 行 |
| --- | --- |
| +tests/fixtures/ceiling/bundle-anchor.py（新しい file・独立の実装） | 140 |
| +tests/fixtures/ceiling/bundle-anchor.txt（新しい file・凍結 anchor） | 12 |
| crates/folio/tests/bundle.rs（歯 3 本と anchor の読み口・余地 856） | 60 |
| 合計 | **212** |

src は 1 行も触らない。新しい dir は作らない。縮む file は無い。外部 crate は増やさない。

**便 98（delivery-98.md の行・M の見込み）**

| 何 | 行 |
| --- | --- |
| crates/folio/src/bundle.rs（節の範囲を取る読み口・切る口・落とした節の一覧・reads.yaml の注釈・余地 961） | 125 |
| crates/folio/tests/bundle.rs（歯 6 本・余地 796〔便 97 の後〕） | 75 |
| crates/folio/tests/findings.rs（凍結の要約値 2 つ・注釈を読み飛ばす枝 2 か所・歯 1 本・**余地 236**） | 8 |
| tests/fixtures/ceiling/bundle-anchor.py と .txt（絞る規則を写し、値を組み直す） | 20 |
| tests/fixtures/ceiling/findings/ の 10 本（record の束の要約値を 1 行ずつ） | 10 |
| 合計 | **238** |

**触る src は bundle.rs 1 本で、余地 961 は M の見積 300 を上回る。** crates/folio/tests/findings.rs の余地は 236 しか無いので、そこへ置くのは 8 行に収める（M の見積を要る file ではない）。新しい dir は作らない。縮む file は無い。外部 crate は増やさない。

**凍結の要約値を持つ file の全数**。tests/fixtures/ceiling/findings/ の下で束の要約値を持つのは 10 本（fabricated-evidence・fail-no-findings・missing-field・pass-coherence・pass-fidelity・pass-readability・pass-reality・stop-refuted・stop-unrefuted・stop-upheld）で、便 98 の write-set に 10 本とも入れた。digest-mismatch.yaml は 0 だけの値を持つ歯なので動かない。stamp-pass / stamp-fail / stamp-unknown / stamp-expected の 4 本も 0 だけの値なので動かない。crates/folio/tests/gate.rs と crates/folio/tests/stamp.rs は同じ fixture を使うが、要約値を字として持たず実行のたびに測るので **1 字も触らない**（起草役が 4 つの要約値を repo 全体で grep して確かめた＝当たるのは crates/folio/tests/bundle.rs・crates/folio/tests/findings.rs・上の 10 本・既に着地した便の設計ノート 4 本だけ）。

### (i) 2 便が運ばないもの・撤退条件

- 天井の正本 ceiling.yaml の reads の宣言を広げること（🔴 1・一括の PR が運ぶ）。要件 FR17 の改訂（🔴 2）。設計ノートと判断の記録の字の直し（🔴 4・🔴 5）。
- 注意の先の一覧と、束の先頭への差し込みと、観点の問いの文の 1 文（設計ノート §8 の G5）。
- 節点ごとの要約値の印と周の引き金（同 G4）・門の単位の変更（同 G9）・要件書の面への実装の状態（同 G8）。
- 束の中身の閉じた一覧（BUNDLE_CONTENTS）を動かすこと・反証の束の中身を動かすこと・所見の欄の決まりを動かすこと。
- crates/folio/src/ の bundle.rs 以外（check.rs・refs.rs・link.rs・mentions.rs・adr.rs・schema.rs・rules.rs・entrance.rs・intake.rs・ceiling.rs・findings.rs・stamp.rs・gate.rs・graph.rs・hello.rs・main.rs・面の生成器）。**床の判定も天井の印も門も 1 つも変わらない。**
- crates/folio/tests/check.rs（p1_commands_closed_list と r11 の助けの字の歯）・tests/floor_cases.yaml・id の一覧の凍結 anchor・欄の決まりの写し 17 本・tests/fixtures/schema/ の下（新しい命令も旗も足さないため）。
- 撤退条件（便 97）: script が凍結の値を再現できないとき（＝ python3 の読み口で束を byte まで組み直せないとき）は、**便 97 を出さずに G3 全体を止め、設計ノート §8 の G3 の大きさを上げる**（設計ノートの G3 の節が既に書く逃げ道）。着地後に script が維持できなくなったら、その歯の結果を まだ分からない に落とす（P-10.3）。
- 撤退条件（便 98）: 絞った束で観点の所見が落ちるようになったとき（次の周で 根拠が正本に無い が 1 件でも出たとき）は、crates/folio/src/bundle.rs の切る口を外して丸ごと写す今の形へ戻し、anchor の値を script で組み直す。**便 1 本で戻せる**（ほかの src を 1 字も触らないため）。戻したときも便 97 の script と anchor は残る＝2 つは別々に捨てられる。

## 2. 範囲

- 便 97 に入れる: 独立の script `tests/fixtures/ceiling/bundle-anchor.py` 1 本・凍結 anchor `tests/fixtures/ceiling/bundle-anchor.txt` 1 本・`crates/folio/tests/bundle.rs` の f97_ 3 本と anchor file の読み口。
- 便 98 に入れる: `crates/folio/src/bundle.rs` の切る口（最上位の節の範囲を取る読み口・骨格の閉じた一覧・落とした節と 正本に無い節 の一覧・reads.yaml の注釈・標準出力の 1 行）・script と anchor file の作り直し・`crates/folio/tests/bundle.rs` の f98_ 6 本・`crates/folio/tests/findings.rs` の f98_ 1 本と凍結の要約値 2 つと注釈の読み飛ばし・凍結の所見 fixture 10 本の束の要約値。
- 2 便とも入れない: design-intent の下の file の書き換え（読むだけ）・天井の正本の reads の宣言・要件書 FR17 の字・束の中身の閉じた一覧・観点の問いの文・注意の先・周の引き金・門の単位・新しい命令と旗・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| script | 独立の実装 | `tests/fixtures/ceiling/bundle-anchor.py`（python3 の標準 library だけ・folio の code を 1 行も呼ばない） |
| anchor | 凍結 anchor | `tests/fixtures/ceiling/bundle-anchor.txt`（観点ごとの file数・byte・要約値。便 98 で 落とす と 無い の 2 種別が足される） |
| cut | 切る口 | `crates/folio/src/bundle.rs`（最上位の節の範囲・骨格の閉じた一覧 6 語・行の byte と順を変えない） |
| notice | 落とした節の知らせ | `crates/folio/src/bundle.rs`（reads.yaml の末尾の注釈と標準出力の 1 行） |
| teeth97 | 歯 | `crates/folio/tests/bundle.rs` の f97_ 3 本 |
| teeth98 | 歯 | `crates/folio/tests/bundle.rs` の f98_ 6 本と `crates/folio/tests/findings.rs` の f98_ 1 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。便 94（行 cq）・便 95（行 cr）・便 96（行 cs）は 3 本とも着地済み（main 0806433）で、本便の write-set はその 3 便と 1 file も重ならない。
- **行 ct（便 97）は先に要るものが無い。** 今の main でそのまま運べる。
- **便 98 は行 ct の着地が前提**（anchor file と script が無いと値を組み直せない）。加えて §1 (f) の 🔴 1 と 🔴 2 の一括（天井の正本の reads を広げる版上げと、要件 FR17 の改訂）が **先に main へ入っていること**が前提である。入る前に着地させると、直近 10 周の実測で 17 件（9.3%）の根拠が束の外に出て、床が正しい所見を落とす。
- python3 は host（/usr/bin/python3）に在る。CI の runner には支度の段が無いので、歯は python3 が無くても落ちない形にする（§1 (c)）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ct"
title = "天井の材料の束の凍結 anchor を、folio の code を 1 行も呼ばない独立の script（python3 の標準 library だけ・置き場は tests/fixtures/ceiling/ の直下で凍結の土台 bundle/ の外）と、その出力の file に作り直す。script は凍結の土台の正本と面から 4 観点の束（sources・faces・question.yaml・finding.yaml・reads.yaml）を組み直し、観点ごとの file の数・連結の byte 数・要約値を anchor file と同じ字で標準出力へ出す。この便は束を 1 byte も変えないので、anchor の 4 対は今 crates/folio/tests/bundle.rs に在る値と 1 字も違わない（fidelity 13 本 15055 byte・readability 15 本 17317・coherence 14 本 15770・reality 11 本 11385）。歯は script の出力と anchor file の byte 一致・anchor file と歯の中の凍結の値の一致・folio ceiling --write の束と anchor file の一致の 3 つを見る。python3 を起動できない環境では、要約値を測れないときと同じ形で まだ分からない の 1 行を標準エラーへ出して歯を落とさない（P-10.3）。src は 1 行も触らず、design-intent の下も 1 file も書き換えない"
req = ["FR17"]
section = "1"
write-set = ["+tests/fixtures/ceiling/bundle-anchor.py", "+tests/fixtures/ceiling/bundle-anchor.txt", "crates/folio/tests/bundle.rs"]
verify = ["cargo nextest run -p folio --test bundle f97_", "cargo nextest run -p folio --test bundle", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f97_ の歯 3 本（独立の script の出力が凍結 anchor tests/fixtures/ceiling/bundle-anchor.txt と byte 一致し、python3 が無い環境では まだ分からない の 1 行を出して落ちない／anchor file の 4 観点の file の数と byte 数と要約値が crates/folio/tests/bundle.rs の凍結の 4 対と一致する／凍結の土台から folio ceiling --write で組んだ 4 観点の束の file の数と連結の byte 数と digest.txt の要約値が anchor file の値と一致し sha256sum で測り直しても同じ）が全部緑、crates/folio/tests/bundle.rs の歯が全部緑、clippy が 0 警告で CI が通る"

<!-- contracts:end -->
