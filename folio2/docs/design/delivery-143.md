# 設計: 便 143 — 入口の棚のカード全体の当たり判定 sc-hit を効かせる（様式の定義の 1 行・描画は不変）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は様式の定義の正本 `design-intent/preview/folio.css` の 1 行で、カードの中のリンクを上に出す規則から当たり判定 sc-hit を外し、部品目録（`design-intent/preview/parts.json`）の sc-hit の説明「棚カードの全面リンク（実アンカー・aria-hidden・カード内の他リンクは z-index で上）」を実態にする。FR4 の規範文も部品も class も生成器も面の HTML も変えない。契約表の行の req は FR4 の 1 つ（main に在る id・便 139 と同じ）。
- 条: P-2.3（design token と様式の定義は 1 か所＝正本の folio.css だけを直し、写しは作らない）/ P-2.4（部品の閉じた一覧＝部品も class も足さない・sc-hit は目録に在る部品のまま）/ P-6.2（生成物を手で直さない＝面の HTML は 1 byte も変わらない）/ P-10.1（凍結 anchor＝歯は凍結の面 `tests/fixtures/face/expected-index.html` と手書きの小さな様式を読み、1 byte も変えない）/ P-4.1（決められない selector を黙って外さず歯を落とす）。
- 出所: 台帳 **f2-648.213**（便 139 の独立の検証 d139-verify.md の blocking B-1・2026-09-25・base 021b7a4 で 4 枚とも 0×0）。便 139（f2-648.205・着地 main e5b8930）は判断の記録のカードだけ sc-hit を外した先回り。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ep` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 4 本（書き換える 1 本 + 新しい歯の file 1 本〔`+`〕+ 本文が変わらない verify の scope 2 本）。縮む file も消す file も無く、新しい dir も作らない（新しい file は既存の `crates/folio/tests/` の下）。
- 門: 本便が書き換える正本 `design-intent/preview/folio.css` は、門の設計文書の判定（preview の下は外）の対象外である。起草役が作業ツリー planner-d143 の一番上で write-set 4 本を main の binary で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便の全差分を当てた写しの binary でも同じ字で 0 だった。
- 前の便: 前提の着地は無い。**base = main 2fbc1af（便 142 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 一括 22（枝 docs/batch22・起草中）は `design-intent/srs.yaml` と仕分けの文書だけを触り、本便の write-set 4 本と重なり 0 である。契約の枝のうち main に着地していない便の write-set に本便の 4 本は無い。

## 1. 設計

### (a) いま起きていること（実測・base main 2fbc1af・数は参考値）

1. **生成器の字。** 入口の面の棚のカード（部品 shelf-card・`crates/folio/src/face_index.rs` の 348 行と 371 行）は、行 sc-row の最後に、開く → と同じ行き先の中身の無いリンク a.sc-hit（aria-hidden・tabindex -1）を置く。main の design-intent から組んだ面では、憲法・要件書・設計ノートの 3 枚が sc-hit を持つ。判断の記録のカードは便 139 で外した（一覧を覆わないための先回り）。
2. **様式の定義の字（`design-intent/preview/folio.css`）。** 164 行でカード自身が position:relative を持つ。176 行の .sc-hit は position:absolute・inset:0・z-index:0（カードいっぱいに広がり、中身の下に敷く意図）。177 行はカードの中の 開く →・sc-row の中の a・付録の chip の a に position:relative・z-index:1 を与える（付録の chip〔annex-chips〕は今の入口の面ではカードの中に 0 個で、3 つ目の selector は今の面では何にも当たらない）（当たり判定の上に出す意図）。印刷の @media（664 行）は sc-hit を消す。
3. **ブラウザの実測（playwright・loopback で配った base の面・幅 1400 / 1000 / 680 / 390）。** 3 枚とも sc-hit の箱は **0×0**、計算された position は relative、z-index は 1。カードの名札・説明・右下の余白を押すと、当たるのは段落かカードそのもので、行き先は無い。開く → と設計ノートの xref と付録の chip は、それぞれの行き先に当たる。
4. **原因の切り分け。** 親のカードは 164 行で既に position:relative を持つので、親の position は原因でない。sc-hit は p.sc-row の中の a なので、177 行の規則（カードの属性 × sc-row × a ＝ 詳細度 0,2,1）が 176 行の .sc-hit（詳細度 0,1,0）に勝ち、position:relative・z-index:1 の中身の無いインラインの箱になる。台帳の見出しの「折りたたみ規則」は、正しくはこのカードの中のリンクを上に出す規則である。
5. **面の見た目と押した先は利用者に見える不具合ではない。** 当たり判定が 0×0 でも、見える字と色と並びは意図どおりで、開く → は押せる。欠けているのは「カードのどこを押しても行き先へ」という部品目録の意図だけである。
6. **様式の定義の写し 3 本の関係。** 正本は `design-intent/preview/folio.css`（65,790 byte）。`tests/fixtures/face/folio.css`（161 byte・注に byte の写しを測るためだけと書いた最小の手書き）と `tests/fixtures/ceiling/bundle/faces/folio.css`（17 byte・凍結 anchor）は正本の写しではない。`git grep -n folio.css crates/folio/tests crates/folio/src` の読みでは、fixture の 161 byte は歯が一時 dir の preview/ に写して面の組み立てと配信の byte を測るだけ（badge・serve・site）、束の 17 byte は天井の束が様式を写さないことを見る歯（bundle）の置き場の中身で、どちらも正本と突き合わせない。正本を読む歯は、本物の design-intent から面を組み部品目録で数える `crates/folio/tests/parts.rs` の parts_check_passes_on_the_three_generated_faces と `crates/folio/tests/site.rs` の site_on_the_real_sources_passes_parts_check_and_face_check の 2 本である。
7. **base の歯（参考値）。** workspace の nextest 909 / 909・clippy 0 警告・床 4 本（check・inject --check・schema --check・derive --check）rc 0・`folio build` の出力 30 file・`folio parts --check` 合格。歯の file の本数は tests/face_index.rs 26・tests/parts.rs 25・tests/face_index_shelf.rs 11。`git grep -n f143_ -- crates` は 0 件。行 id `ep` は repo の docs/design と作業ツリーの枝に 0 件。

### (b) 直す先 — カードの中のリンクを上に出す規則から sc-hit を外す（案 A）

1. **`design-intent/preview/folio.css` の 177 行だけを変える。** 3 つの selector のうち真ん中（カードの中の sc-row の a）を、sc-row の a のうち sc-hit でないもの（`.sc-row a:not(.sc-hit)`）に替える。宣言（position:relative・z-index:1）と他の 2 つの selector（開く → と付録の chip の a・chip は今の面ではカードの中に 0 個）は変えない。差分は 1 行・13 byte（`:not(.sc-hit)` を足すだけ・組み立てた出力の総 byte も 13 増える）。
2. **効き方。** sc-hit には 177 行が当たらなくなり、176 行の position:absolute・inset:0・z-index:0 が効く。広がる先は位置を持つ一番近い祖先＝164 行のカードである。カードの中の他のリンク（開く →・xref）は今までどおり 177 行で z-index:1 を持ち、当たり判定の上に在る。
3. **描画は変わらない（起草役の実測）。** 便を当てた写しの `folio build` の出力は、base と比べて 30 file のうち folio.css だけが違い、HTML の 29 file は byte で同じ。playwright で 4 幅の全面の画像と、要件書のカードの説明に指を置いた画像を撮ると、base と便の後で 8 枚とも画像の byte が一致した。sc-hit は背景も枠も字も持たないので、フレックスの行から外れても見える並びは動かない（base の 0×0 の箱は行の最後の 開く → の後ろに在り、その前の行の隙間は何も描かない＝画像の byte 一致がそれを示す）。
4. **押した先だけが変わる（起草役の実測）。** 便の後の sc-hit の箱は、カードの枠の内側いっぱい（幅 1400 で 318×269・幅 390 の憲法のカードで 318×274）。名札・説明・右下の余白を押すと sc-hit に当たり、行き先はそのカードの 開く → と同じ（constitution.html・srs.html・note-figures.html）。開く →・xref・付録の chip の押した先は base と同じ。判断の記録のカードは sc-hit を持たないので、押した先は base と同じ。
5. **変えないもの。** 生成器（`crates/folio/src/`）・面の HTML と凍結 anchor・部品目録と class の一覧・design token・176 行と 164 行・印刷の @media・写しの folio.css 2 本・判断の記録のカード（sc-hit なしのまま）。
6. **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は描画を 1 画素も変えない当たり判定の直しで、見た目の直しではない（(b) の 3 の画像の byte 一致）。仮に数えても、shelf-card の規則は day-1 から一度も変わっていない（folio.css を変えた 4 つの着地 88d3d48・29a6a85・686b431・100774d の差分に shelf-card と sc- の行は 0）ので 1 本目で、上限に届かない。

### (c) 歯（関数名 f143_・base で 0 件・新しい file `crates/folio/tests/face_style.rs`）

歯は binary を起動しない（cargo だけで撃てる・器の allowed-commands の外の node や playwright を使わない）。ブラウザの代わりに、CSS の決まりの小さな写し（!important・詳細度・同じなら後に書いた方・印刷の @media は読まない・状態の擬似 class は当たると読む〔通さない側〕・右端が当たるのに兄弟の結合子や構造の擬似 class で決められない selector は黙って外さず歯を落とす）で、面の要素に効く宣言を決める。

1. **f143_the_card_hit_area_covers_the_card_under_its_links。** 凍結の面 `tests/fixtures/face/expected-index.html`（生成器の出力と byte で一致することは tests/face_index.rs の face_index_write_matches_the_frozen_fixture が見る）から sc-hit を 3 つ拾い、各々について面の tag を読んで祖先の列を作り、正本の folio.css で次を確かめる。
   1. 効く position が absolute・inset が 0・display が none でない。
   2. 位置を持つ一番近い祖先（包含する箱）が、属性 data-component が shelf-card のカードである。
   3. 同じカードの中の他の a（開く →・xref）が 1 つ以上在り、どれも位置を持ち、z-index が sc-hit より大きい。
   **base では 1 の position が relative ＝ RED。**
2. **f143_the_cascade_reader_follows_the_css_order。** 1 が使う CSS の決まりの写しそのものを、手書きの小さな面（カード・sc-row・開く →・sc-hit）と手書きの小さな様式 11 通りで確かめる（base の形で relative・:not で外すと absolute・同じ詳細度は後が勝つ・!important が勝つ・印刷の @media は読まない・画面の @media は読む・注と擬似要素は読まない・hover は当たると読む・:not の中の id も詳細度に数える・宣言が無い）。兄弟の結合子の selector は決められずに落ちること（黙って外さない）も確かめる。folio.css を読まないので base でも緑である（写しの決まりの独立の anchor・P-10.1）。

fixture は新しい file を足さない（凍結の面と正本の様式を読むだけ）。新しい歯の file は既存の口を持たないので、repo の根の path を返す口だけを置く。

### (d) 採らなかった形

1. **案 B: 部品目録から sc-hit を退役する（カード全体で押せる意図を捨てる）。** 生成器の 2 か所・部品目録・folio.css の 2 行・凍結の面 2 本と sc-hit を数える既存の歯（tests/face_index.rs と tests/face_index_shelf.rs）を同時に変えることになり、差分が大きい。目録の説明どおりの意図を 1 行で実態にできるので、捨てる理由が無い。
2. **案 A′: .sc-hit の規則の詳細度を上げる（例: sc-row の中の .sc-hit）。** 同じく効く（歯 1 は緑・(e) の 4 の A1）。しかし 177 行の「sc-row の a を全部上に出す」字が残り、読み手はどちらが勝つかを詳細度の計算で知るしかない。:not で外す形は、上に出す相手から当たり判定を除くという意図を字で書く。
3. **親に position:relative を足す。** 親のカードは既に持つ（(a) の 4）。当たり判定の直しにならない。
4. **歯をブラウザ（node・playwright）で撃つ。** 当たり判定そのものを測れるが、器の allowed-commands（cargo・git）の外で、受付の検証が撃てない。ブラウザの実測は起草の記録に残し（(e) の 3）、歯は CSS の決まりの写しで持つ。

### (e) 既存の歯のうち落ちるもの・RED・突然変異・面の変化

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest **911 / 911**（base 909 + f143_ 2）・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file（base と diff -r で folio.css だけ違う）・`folio parts --check --dir design-intent --page index=<出力の index.html>` 合格（違反 0・まだ分からない 0）。tests/face_index.rs 26 / 26・tests/parts.rs 25 / 25・tests/face_index_shelf.rs 11 / 11。凍結 anchor（`tests/fixtures/face/` と `tests/fixtures/ceiling/`）は 1 byte も変わらない。
2. **RED（起草役の実測）。** 歯だけを base に当てた写しで、f143_the_card_hit_area_covers_the_card_under_its_links が落ちる（憲法のカードの sc-hit の position が relative・期待 absolute）。f143_the_cascade_reader_follows_the_css_order は緑（様式の定義を読まない写しの決まりの歯）。歯の file の無い base では `--test face_style` の組み立てが無く、verify の 1 行目は落ちる。
3. **ブラウザ（起草役の実測・参考値・歯ではない）。** (a) の 3 と (b) の 3・4 のとおり。base の sc-hit は 3 枚 × 4 幅で 0×0・relative、便の後は枠の内側いっぱい・absolute・z-index 0。全面と指を置いた画像は 4 幅とも byte で一致した。
4. **突然変異（起草役の実測）。** 便を当てた写しの folio.css か歯の側の決まりの写しを 1 通りずつ変える。

| 変異 | 歯 1（当たり判定） | 歯 2（決まりの写し） |
| --- | --- | --- |
| M1 直しを戻す（base の形） | 落ちる（position） | 緑 |
| M2 カードの position:relative を外す | 落ちる（広がる先が figure） | 緑 |
| M3 カードの中の a の z-index を 0 に | 落ちる（他の a が下） | 緑 |
| M4 sc-hit の inset を外す | 落ちる（広がり） | 緑 |
| M5 画面の @media で sc-hit を消す | 落ちる（消えている） | 緑 |
| M6 hover で sc-hit を relative に | 落ちる（position） | 緑 |
| E1 書いた順を逆に数える | 緑 | 落ちる |
| E2 印刷の @media も読む | 落ちる | 落ちる |
| E3 :not の中の詳細度を数えない | 緑 | 落ちる |
| E4 決められない selector を外れと読む | 緑 | 落ちる |
| A1 別の直し（案 A′・詳細度を上げる） | 緑 | 緑 |

   A1 が緑なのは、歯が直しの字でなく効き方を測るため（案 A′ でも意図は満たす）。

5. **面の変化。** 面の HTML は変わらない。配った面の folio.css だけが 1 行変わり、カードの本文を押すとそのカードの行き先へ飛ぶ。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 4 本。書き換える 1 本 = `design-intent/preview/folio.css`。新しい file 1 本 = `+crates/folio/tests/face_style.rs`。本文を変えない 2 本 = `crates/folio/tests/face_index.rs`（verify の `--test face_index` で名指す・歯 1 が読む凍結の面が生成器の出力と byte で一致することを見る）と `crates/folio/tests/parts.rs`（verify の `--test parts` で名指す・本物の design-intent の folio.css で部品目録を数える）。
2. **余地（CapHeadroom）。** 触る src は無い（write-set に `crates/folio/src/` の下の file が無い）。src の外の参考値（各行を ceil(字数 / 120) で数えて足す・python と awk の 2 実装で一致）は、folio.css 869 → 869・face_style.rs 0 → 602。行数の上限の歯（face.rs 1250・face_srs.rs 1100・tests/schema.rs 700）は本便の write-set に当たらない。
3. **size は S。** 様式の定義の 1 行と、歯の file 1 本。
4. **verify は 4 行**で、done の 4 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test face_style f143_` = (c) の 1・2（2 本）。
   2. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（歯 1 が読む凍結の面と生成器の出力の byte 一致・sc-hit の数の歯を含む・参考値 26 本）。
   3. `cargo nextest run -p folio --test parts` = 部品目録の歯の全部（本物の design-intent の folio.css で 3 面を数える歯を含む・参考値 25 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   base 2fbc1af では 1 が歯の file の無い組み立てで落ち、2 は 26 本・3 は 25 本で緑、4 は 0 警告である。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_style・face_index・parts の 3 本で、どれも write-set に在る。`--bin folio` の絞り込みは使わない（src に歯を置かない）。

### (g) 門と受付

1. **門。** 作業ツリー planner-d143 の一番上で `folio ceiling --gate --dir design-intent --write-set design-intent/preview/folio.css +crates/folio/tests/face_style.rs crates/folio/tests/face_index.rs crates/folio/tests/parts.rs` を main の binary で撃つと 0（通す・設計文書の正本を書き換えない便）。本便を当てた写しの binary でも同じ字で 0。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測・(h) の 6）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d143-draft.md`、模擬の差分と script は同じ dir の d143-scripts（repo には入れない）。

1. 再現: main の binary で `folio build --dir design-intent --out <scratch>/base-out --write`、scratch を loopback で配り、browser-143.js（playwright の run_code）で 4 幅の当たり判定・押した先・画像を測る（browser-143.log）。画像を撮る前に指（mouse）を (0,0) へ動かす（画像の byte 一致は指の位置に依る・前の click の指が xref の上に残ると hover の描画だけで base と後が違って見える）。
2. 模擬: base の写しの根で apply-143.py（実装）と apply-143-teeth.py（歯・face_style.rs を置く）を撃つ。その差分が c143.patch。verify-143.sh で verify の 4 行・workspace の nextest・床 4 本・`folio build` と `folio parts --check`。
3. RED: apply-143-teeth.py だけを base の写しに当てる（r143-teeth.patch・red.log）。
4. 突然変異: mut-143.sh（(e) の 4・folio.css の変異は組み直し不要・歯の側の変異は組み直す）。
5. 行数: lines-143.py と lines-143.awk（同じ式の 2 実装）。
6. 受付の先撃ち: 契約を commit した後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-143.md#ep`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 生成器と面の HTML・部品目録・sc-hit の退役（(d) の 1）・判断の記録のカードに sc-hit を戻すこと（便 139 の決定のまま）・写しの folio.css 2 本・設計文書の正本・台帳への記帳（席）・外部 crate。
2. **言えないこと。** 歯が測るのは CSS の決まりの写しの上の効き方で、ブラウザの描画そのものではない（ブラウザの実測は起草の記録の参考値）。写しは position・inset・z-index・display の勝ち負けだけを決め、top や width などの個別の辺と、カードの角丸の継ぎ（border-radius:inherit）は見ない。写しは要素の style 属性（inline）を読まず、字に print を含む @media の塊（not print を含む）を丸ごと飛ばす（今の面と正本ではどちらも効かない）。inset は字だけを見るので、後の規則が sc-hit に top だけを書く変異は拾わない。便の後はカードの名札と説明の字を指でなぞって選べなくなる（当たり判定が上に在る・全面リンクの部品の常の性質）。支援技術には sc-hit は aria-hidden で tabindex -1 のまま見えない。
3. **撤退条件。** (1) 本便の後に、置き換えも足しもしない既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に `folio build` の出力の HTML が 1 byte でも変わるか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す（本便は描画を変えない前提）。(3) 受付の時点の main で folio.css の 164・176・177 行か、生成器の sc-hit の置き場（sc-row の最後）が base と違えば、(a)(b)(e) を (h) の手順で数え直してから運ぶ（当たり判定が既に効いていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `design-intent/preview/folio.css` の 177 行の真ん中の selector に `:not(.sc-hit)` を足す 1 行。新しい歯の file `crates/folio/tests/face_style.rs`（CSS の決まりの小さな写しと歯 2 本）。
- 入れない: 生成器・面の HTML・凍結 anchor・部品目録・design token・写しの folio.css・設計文書の正本・新しい fixture・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| css | 上に出す規則 | `design-intent/preview/folio.css` の 177 行（sc-row の a から sc-hit を外す） |
| cascade | CSS の決まりの写し | `crates/folio/tests/face_style.rs` の規則の読み・selector の当たり・詳細度・勝ち負け |
| teeth | 歯 | 同じ file の f143_ 2 本（当たり判定の効き方・写しの決まり） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 2fbc1af）。
- 並行の便: 無し（一括 22 と重なり 0）。
- 本便の着地の後に席が見ること: 台帳 f2-648.213 を閉じる。次の報告で入口の面を配るときに、カードの本文を押すとそのカードの行き先へ飛ぶことを持ち主に一言添える。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ep"
title = "面の部品の不具合（台帳 f2-648.213・便 139 の独立の検証 B-1）: 入口の棚のカード全体の当たり判定 sc-hit が、様式の定義の正本 design-intent/preview/folio.css の 177 行（カードの中の sc-row の a を position:relative・z-index:1 で上に出す規則・詳細度 0,2,1）に 176 行の .sc-hit（position:absolute・inset:0・z-index:0・詳細度 0,1,0）が負けて 0×0 になり、カードの本文を押しても行き先が無い（親のカードは 164 行で position:relative を持つので親は原因でない・ブラウザの実測）。177 行の真ん中の selector を sc-row の a のうち sc-hit でないもの（:not(.sc-hit)）に替える 1 行で直す。面の HTML・生成器・部品目録・凍結 anchor・写しの folio.css 2 本は変えず、描画は 4 幅の画像が byte で同じ（ADR-5 撤退条件 ② に当たらない）。歯は新しい file crates/folio/tests/face_style.rs の f143_ 2 本（CSS の決まりの小さな写しで凍結の面の sc-hit に効く宣言を決め、absolute・inset 0・広がる先がカード・カードの中の他の a が上であることを見る歯と、写しの決まりそのものを手書きの様式 11 通りで見る歯）。設計文書の判定の外なので門は 0"
req = ["FR4"]
section = "1"
write-set = ["design-intent/preview/folio.css", "+crates/folio/tests/face_style.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/parts.rs"]
verify = ["cargo nextest run -p folio --test face_style f143_", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test parts", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/face_style.rs の f143_ の 2 本（凍結の面 expected-index.html の sc-hit 3 つに正本の folio.css で効く position が absolute・inset が 0・display が none でなく、位置を持つ一番近い祖先が shelf-card のカードで、同じカードの中の他の a がどれも位置を持ち z-index が sc-hit より大きい / CSS の決まりの写しが手書きの様式 11 通りで期待の値を返し、兄弟の結合子の selector で黙って外さず落ちる）が緑、tests/face_index.rs の歯の全部（凍結の面と生成器の出力の byte 一致と sc-hit の数を含む）が緑、tests/parts.rs の歯の全部（本物の design-intent の folio.css で 3 面の部品目録を数える歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで folio.css のほかは byte も変わらない"
<!-- contracts:end -->
