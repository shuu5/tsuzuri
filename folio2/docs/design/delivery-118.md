# 設計: 便 118 — 要件書の面に M3 の範囲の節 scope_m3 を、在れば scope_m1 と同じ形で描く（FR4・便 117 の §1 (h) の 2 の穴）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す・第 1.32 版）。本便は要件書の面の生成器が正本の最上位の節のうち描く節を 1 つ広げる。規範文は変えない。
- 条: P-6.1（各文書の正本は YAML 1 file で、人が読むページはそこから逐語で生成する）/ P-4.2（判定できないものは「まだ分からない」として表に出す＝正本に在る節を面が黙って落とさない向き）/ P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-2.4（部品は閉じた一覧＝本便は新しい部品を足さない）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）
- 出所: 便 117 の契約（`docs/design/delivery-117.md`・行 `dp`・着地済み main a34c6b2）の §1 (h) の 2「まだ分からない — 要件書の面が scope_m3 を描かない」。便 117 の起草役と検証役が、便 117 の後の木で要件書に scope_m3 の節を足して組み立てを撃ち、床は合格・組み立ても成功するのに、**scope_m3 の節の字は配信先のどの面にも 1 字も出なかった**ことを実測した。便 117 の契約はこの穴を「版 B の発効の前に、面が scope_m3 を描く便を起こすか、描かないことを持ち主が裁定するかが要る」と書いた。本便は前者で、席が起こした（判断の記録 ADR-16 決定 (7) の便の列〔① 〜 ⑦・合計 7〜9 便〕には名が無い追加の 1 便で、設計文書は変えない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dq` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 2 本、**新しい file は 1 本も無く、縮む file も無く、消す file も無く、新しい dir も作らない**（接頭辞 `+` / `-` / `~` は 1 つも使わない。§1 (g) の 1）。
- 門: write-set の 2 本はどちらも設計文書の置き場の外（`crates/` の下）で、起草役の実測（2026-09-24・base a34c6b2 に本便の patch を当てた木の binary）は **0（通す・設計文書の正本を書き換えない便）**。規則の表の開発規律行 D-12 の対象ではない。
- 前の便: 便 117（行 `dp`・**着地済み main a34c6b2**・床の定数 SRS_TOP_LEVEL に scope_m3・要件書の生成区間 +1 行）。**base = main a34c6b2**。この契約の数はすべて base a34c6b2 の実測（参考値）である（規則の表の行 D-13）。
- 改訂: 改訂 c（2026-09-24・席）= 検証役（d118-verifier・a34c6b2 の写しで patch を当て 786/786・変異の表 7 行一致・build 差 0・門 0・blocking 0）の文面の所見 3 つを当てた（code と歯の注の出所を便 117 §1 (h) の 2 に・helper 6 と定数 4 の内訳・scope_m3 の余分な欄は黙って落ちることの明記）。区間・write-set・verify・size・done は変えていない。
- 改訂: 改訂 b（2026-09-24 01:0x JST・起草役）= 便 117 の着地（main a34c6b2）を受け、base を「便 117 の着地の sha（未定）」から a34c6b2 に置き換え、実測を a34c6b2 の写しで全部取り直した（RED 2 本・変異の表・workspace の nextest 782 → 786・clippy 0 警告・組み立ての差 0・門 0・余地・parts の合格はどれも初版と同じ値）。区間・write-set・verify・size・done は変えていない。

## 1. 設計

### (a) いま起きていること（実測・base main a34c6b2）

1. 要件書の面の生成器は `crates/folio/src/face_srs.rs` の 1 本で、章 02「範囲」は同じ file の scope_chapter が書く。その末尾の「作るもの / 作らないもの」の小見出しの下で、**段の範囲の節を名指しの表 2 行（鍵 scope に名札 M0・鍵 scope_m1 に名札 M1）で回し、節ごとに部品 section-lead-callout の塊を 1 つ置く**（base a34c6b2 の 779〜806 行目・参考値）。塊の中は card が 2 枚＝「<名札> で作る」（build の値を「 ／ 」で繋ぐ）と「<名札> では作らない」＋札「対象外」（not_build の値を同じく繋ぐ）。
2. 名札の字（M0・M1）の正本は **その表の字の literal**（面の名札の表・生成器の file の中の β）で、要件書の欄ではない。要件書は名札を持たない。語 M1 の字は憲法の面の名札の文（`crates/folio/src/face.rs` と `crates/folio/src/face_labels.rs` の段の説明）にも在るが、それは要件書の面の表の写しではなく段の説明の文なので、本便は触らない。
3. **節の読み方。** 表の 2 行とも必須の欄の口（cursor の f・無ければ Err「欄 <鍵> が無い」）で読む。要件書に scope か scope_m1 が無いと面は導出されず、命令は 2（まだ分からない）で終わり面を書かない。注（note の欄）は **鍵が scope_m1 のときだけ** 任意の欄の口（cursor の g・無い・null は無し）で読み、「では作らない」の card の本文の後ろに小窓「注」（`crates/folio/src/face.rs` の hint）で置く。scope の note は読まない。
4. **scope_m3 は読まれない。** 表に行が無いので、要件書が最上位に scope_m3 の節を持っても面の字は 1 byte も変わらない。起草役が面の fixture（`tests/fixtures/face/` の正本 5 file）の写しの要件書の actors の直前に scope_m3 の節（build 1 行・not_build 1 行・note）を足して `folio face --face srs --write` を撃つと、終了コード 0 で、出力は凍結 fixture `tests/fixtures/face/expected-srs.html` と **byte 一致**した（＝節が黙って落ちる）。面の検査（`folio parts --check`）も数えない。
5. 実の要件書（`design-intent/srs.yaml`・第 1.32 版）は scope_m3 の節を持たない（版 B の前）。要件書の面の凍結 fixture は `tests/fixtures/face/expected-srs.html` の 1 本で、`crates/folio/tests/face_srs_body.rs` の face_srs_write_matches_the_frozen_fixture が面の fixture の正本から書いた面と byte 一致を見る（ほかに `crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/badge.rs` が同じ file を読む）。面の fixture の要件書も scope_m3 を持たない。組み直す手順:

   ```
   git grep -n scope_m1 -- crates/folio/src
   git grep -n expected-srs.html -- crates/folio/tests
   ```

6. base の workspace の nextest は全部緑（参考値 782 本・base a34c6b2・`cargo nextest run --workspace --no-tests=fail --no-fail-fast`）。

### (b) 直す先 — 名札の表に 3 行目（任意の節）を足し、注を段の節に広げる

**変更の逐語（`crates/folio/src/face_srs.rs` の scope_chapter・base a34c6b2 の 780〜783 行目・参考値）。** この節は二重引用符を置けないので、Rust の字の literal の二重引用符を「」で書いた。実物は base の字の literal と同じ二重引用符である。前:

```
    for (key, label) in [(「scope」, 「M0」), (「scope_m1」, 「M1」)] {
        let sc = s.f(key)?;
        let note = if key == 「scope_m1」
            && let Some(n) = sc.g(「note」)?
```

後（表の各行に「必須か」の真偽を足し、3 行目に scope_m3 を任意で置く・注の条件を「scope でない」に）:

```
    // 段の範囲の節（鍵・名札・必須か）。scope_m3 は任意の節で、無ければ描かない（便 118・便 117 の §1 (h) の 2・ADR-16 決定 (1)）。
    for (key, label, required) in [
        (「scope」, 「M0」, true),
        (「scope_m1」, 「M1」, true),
        (「scope_m3」, 「M3」, false),
    ] {
        let Some(sc) = (if required { Some(s.f(key)?) } else { s.g(key)? }) else {
            continue;
        };
        let note = if key != 「scope」
            && let Some(n) = sc.g(「note」)?
```

塊の本体（section-lead-callout の div・card 2 枚・閉じの div）と、その後ろの行は 1 字も変えない。増える行は 7 行（参考値）。

決めたこと 4 つ:

1. **名札は M3、置き場は同じ表。** 名札の正本は (a) の 2 のとおり生成器の表の字なので、同じ表に 1 行足す。字は scope_m1 の M1 と同じ規則（段の名そのもの）で「M3」。card の字は「M3 で作る」「M3 では作らない」になる。
2. **scope_m3 は任意、scope と scope_m1 は必須のまま。** 実の要件書は版 B まで scope_m3 を持たないので、必須にすると実の要件書の面が導出されなくなる（§1 (c) の 2）。在れば描き、無い・null なら塊を置かない（cursor の g が null を無しに畳む）。scope と scope_m1 の扱いは変えない＝落とせば今までどおり 2（まだ分からない）。
3. **置く位置は scope_m1 の塊の直後。** 表の順＝段の順（M0 → M1 → M3）で、便 117 が閉じた一覧で scope_m3 を scope_m1 の直後に置いたのと揃う。
4. **注は段の節（scope_m1 と scope_m3）で読む。** 「scope_m1 と同じ形」の中身として、scope_m3 の note も「では作らない」の card の後ろに同じ小窓で置く。scope（M0）の note は今までどおり読まない（条件は「鍵が scope でない」）。要件書の scope は note を持たないので、面の字は変わらない。

### (c) 採らなかった形

1. **scope_m3 を表に足さず、要件書の最上位の節を名の頭（scope で始まる鍵）で拾って回す。** 名札を鍵の字から作ることになり（scope_m3 → M3）、閉じた一覧（床の定数）の外の規則で面が開く。床の定数は便 117 で 1 語だけ広がり、面はその 1 語を名指しで受ければ足りる。採らない。
2. **scope_m3 も必須にする。** 版 B の前の実の要件書で面が導出されなくなり（配信の組み立ても全部か無しかで止まる）、便 117 と版 B の間に配信が止まる。採らない。
3. **scope_m1 も任意にそろえる。** 今の要件書と面の fixture はどちらも scope_m1 を持ち、落ちたら「まだ分からない」で止める今の扱いは P-4.2 の向きに合う。本便の範囲の外の緩めで、歯 3 がこの形を落とす。採らない。
4. **面の fixture の要件書（`tests/fixtures/face/srs.yaml`）に scope_m3 を書き足し、凍結 fixture `tests/fixtures/face/expected-srs.html` を取り直す。** 凍結 anchor が生成器の出力の写しで動き、実の要件書に無い節を fixture だけが持つ。歯は歯の file の中の最小の手書きの節を写しの要件書へ字面の変異で入れれば足りる（便 117 の歯と同じ作法）。採らない。
5. **名札の字を要件書の欄（例えば節の中の名札の欄）から読む。** 要件書の節の欄の閉じた形を広げることになり、版 B の設計（節の中身の形）を先取りする。採らない。

### (d) 歯（新しく 4 本・置き場は `crates/folio/tests/face_srs.rs`）

新しい歯は `crates/folio/tests/face_srs.rs` の末尾に、便 118 の見出しの注の下で 4 本足す。関数名は f118_ で始める（verify の絞り込みの語）。同じ file の頭の注の箇条に 2 行を足す。

**歯の土台。** 面の fixture（`tests/fixtures/face/` の constitution・rules・vocabulary・srs・ceiling の 5 file）を一時 dir の下の src へ写し、図の道具（`vendor/archify/`）を親 dir へ写し、**写しの要件書だけを字面の変異 1 か所**で書き換えて `folio face --face srs --write` を撃つ（既存の f84_ の歯と同じ形・一時 dir は消す）。scope_m3 の節は歯の file の中の最小の手書きの字（build 1 行・not_build 1 行・note 1 行。build の値に「&」、note の値に「<」と「>」を含めて escape を見る）で、最上位の節 actors の直前に入れる。**新しい fixture の file も dir も作らない。凍結 fixture は動かさない。** 章 02 の中の段の範囲の塊は、部品 section-lead-callout の開き（style の --band-n:2 付き）から、次の行頭の閉じの div までを 1 つとして切り出す。同じ file に在る helper（chapter・write_face・code・esc・real_srs・temp_dir・copy_dir・vendor・design_intent・repo_root）を使い、足す helper は 6 つ（見本の節を組む口・面を書く口・節を足す口・節を落とす口・塊を切り出す口・終了コード 0 を見る口）と定数 4 つ（見本の字 3 = build / not_build / note と、塊の開きの字 1）である。

1. **scope_m3 は scope_m1 の塊の直後に同じ形で出る。** 面の fixture の scope_m1 の節の字をそのまま写し、鍵の名だけ scope_m3 に替えて足す → 終了コード 0・章 02 の段の範囲の塊がちょうど 3 つ・順に名札 M0・M1・M3 の「で作る」を持つ・**3 つ目の塊は、2 つ目の塊の名札の「M1 」を「M3 」に替えた字と byte 一致**（card の class・並び・注の小窓の位置まで同じ形）・3 つ目に注の小窓が在る。**base では塊が 2 つで落ちる＝RED。**
2. **手書きの scope_m3 は塊 1 つだけを足し、値は escape される。** 手書きの節を足す → 終了コード 0・塊が 3 つ・3 つ目が「M3 で作る」の card に build の値の escape 済みの字・「M3 では作らない」＋札「対象外」の card に not_build の値の escape 済みの字・注の小窓の本文に note の値の escape 済みの字を持つ・面のどこにも note の生の「<」「>」の字面が無い・**面から 3 つ目の塊の字を 1 回だけ除くと、凍結 fixture `tests/fixtures/face/expected-srs.html` と一致する**（塊のほかに面の字が 1 字も変わらない）。escape の期待の字は歯の側の 5 字の escape（同じ file の esc）で作り、生成器の字を写さない。**base では塊が 2 つで落ちる＝RED。**
3. **scope_m3 が null なら面は凍結 fixture と byte 一致し、scope と scope_m1 は必須のまま。** 写しの要件書に値の無い scope_m3 の鍵だけを足す → 終了コード 0・面が凍結 fixture と一致。続けて scope の節と scope_m1 の節を 1 つずつ別の写しから落とす → 終了コード 2・標準エラーに「欄 <鍵> が無い」・面の file を書かない。**base でも緑**（本便の後も「無い・null は描かない」と「必須の節は緩めない」が残ることを見る土台の歯）。
4. **実の要件書の面は、実の要件書が最上位に scope_m3 の節を持つときに限り M3 の塊をちょうど 1 つ持つ。** 実の `design-intent/srs.yaml` の行頭に scope_m3 の鍵が在るかを見て、実の置き場の写しで書いた面の「M3 で作る」の card の数が 1（在る）か 0（無い）であることを見る。**版 B の前の base でも緑**（0 = 0）。版 B が節を書いた後は、本便の直しが外れると落ちる（版 B の後の回帰の歯）。

歯の効き（起草役が変異を当てて測った・base a34c6b2 の写し）:

| 当てた形 | 歯 1 | 歯 2 | 歯 3 | 歯 4 |
| --- | --- | --- | --- | --- |
| base（生成器は表 2 行のまま） | 落ちる | 落ちる | 通る | 通る |
| 本便の形 | 通る | 通る | 通る | 通る |
| 注を scope_m1 のときだけ読む（今の条件のまま） | 落ちる | 落ちる | 通る | 通る |
| scope_m3 の行を scope_m1 の行の前に置く | 落ちる | 落ちる | 通る | 通る |
| scope_m3 の名札を M2 にする | 落ちる | 落ちる | 通る | 通る |
| scope_m3 を必須にする | 通る | 通る | 落ちる | 落ちる |
| scope_m1 を任意にする | 通る | 通る | 落ちる | 通る |

### (e) 既存の歯のうち落ちるもの・凍結 anchor・面の出力の byte

1. **落ちる既存の歯は 0 本**（起草役の実測・本便の後の木で `cargo nextest run --workspace --no-tests=fail --no-fail-fast` が全部緑・参考値 786 本 = base 782 本 + 新しい歯 4 本）。要件書の面の凍結 fixture を読む歯（`crates/folio/tests/face_srs_body.rs` の byte 一致・`crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/badge.rs`）は、面の fixture の要件書が scope_m3 を持たないので字が変わらず緑のまま。write-set に入れない。
2. **凍結 anchor は 1 本も動かない。** `tests/fixtures/face/expected-srs.html` ほか面の凍結 fixture・束の凍結 anchor・所見 fixture・節点の要約値の anchor はどれも 1 byte も変えない。「動かない」ことは歯 3（null → 凍結 fixture と byte 一致）と既存の byte 一致の歯が凍結の土台で確かめる。
3. **面の出力の byte（実の要件書）。** 実の要件書は scope_m3 を持たないので、本便の前後で配信の組み立ての出力は変わらない。起草役が実の置き場を本便の前の binary と後の binary でそれぞれ `folio build --write` へ別の配信先に書き、2 つの dir を `diff -r` で比べて **差 0**（参考値 23 file・床 = 合格）。組み直す手順:

   ```
   folio build --dir <写し>/design-intent --out <配信先 A> --write   # 本便の前の binary
   folio build --dir <写し>/design-intent --out <配信先 B> --write   # 本便の後の binary
   diff -r <配信先 A> <配信先 B>
   ```

4. **面の検査。** 歯 2 の手書きの節を足した面に `folio parts --check --page srs=<面>` を撃つと合格（違反 0・まだ分からない 0）。塊は既存の部品 section-lead-callout と card の class だけで組むので、部品目録と様式は変えない。
5. **門（規則の表の開発規律行 D-12）。** write-set 2 本を `folio ceiling --gate --write-set` に渡すと **0（通す・設計文書の正本を書き換えない便）**（起草役の実測・2026-09-24）。

### (f) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 要件書の scope_m3 の節の中身と版上げ（版 B・持ち主の承認）。床の定数（便 117）。天井の正本の観点の読む欄（便 117 の §1 (h) の 3＝天井の束は scope_m3 を「落とした節」に記録する・本便でも変わらない）。部品目録・様式・面の fixture・凍結 fixture。語彙・憲法・規則の表・判断の記録。
2. **本便の着地の後も、版 B の前は面が変わらない。** 実の要件書に scope_m3 が無いあいだ、配信先の面は 1 byte も変わらない（(e) の 3）。本便の効きが実の面に出るのは版 B の後である。版 B の後に実の面で M3 の塊が出ることは歯 4 が見る。
3. **言えないこと。** 歯は「scope_m3 が在れば scope_m1 と同じ形の塊が出る」ことだけを見る。節の中身の形（build と not_build の 2 つの一覧と任意の note か、ほかの欄を持つか）は床も面も数えない（便 117 の §1 (h) の 4 と同じ）。版 B が build か not_build を持たない形で scope_m3 を書くと、面は必須の欄の Err で 2（まだ分からない）になり黙っては落ちないが、描く形は版 B の設計で決め直す。scope（M0）の note を読まないことは変えないが、面の fixture の scope が note を持たないので、歯はこの条件を scope まで広げる変異を拾えない（起草役の実測・本便の範囲の外）。
4. **まだ分からない — 版 B の節の中身。** 版 B が scope_m3 に scope_m1 と違う欄（例えば判定点や出口の条件）を持たせるなら、本便の塊（card 2 枚＋注）はそれを描かない。**余分な欄は床も面も数えず黙って落ちる**（検証役の実測: scope_m3 に exit と entry を足しても `folio face --face srs --write` は rc 0 で、その字は面に 0 回。床は scope 系の節の中身を数えない〔便 117 §1 (h) 4〕。base の scope / scope_m1 でも同じで、本便が作った穴ではない）。版 B の scope_m3 は build / not_build / note の 3 欄に収めるか、面の描き方を足す便を版 B の前に起こす。
5. **まだ分からない — ADR-16 の便の数。** 判断の記録 ADR-16 決定 (7) の便の列は面の便を名指さず、合計 7〜9 便の数にも入っていない。本便はその列の外の追加で、判断の記録の字は変えない。列の数え直しを判断の記録に記すかどうかは席が決める（起草役は決めない）。
6. **撤退条件。** (1) 本便の直しの後に既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す（本便の前提＝面の fixture と実の要件書が scope_m3 を持たないので面の字は変わらない、が崩れている）。(2) `folio build` の出力が本便の前後で 1 byte でも変わったら止めて席へ返す。(3) 受付の時点の main で `crates/folio/src/face_srs.rs` の scope_chapter の表の周りが書き換わっていたら、変更の逐語を測り直してから運ぶ。

### (g) 大きさ・verify と done の対応

1. **write-set の印。** 2 本とも base に在る file を書き換えるので印なし。`+`（新しい file）・`-`（行が減る file）・`~`（着地で消える file）はどれも当たらない。
2. **余地（CapHeadroom）。** 測るのは write-set のうち `crates/folio/src/` の下の印なしの .rs、つまり `crates/folio/src/face_srs.rs` の 1 本だけである。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない）。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | size S の見積 | 本便の後（参考値） |
| --- | --- | --- | --- | --- |
| `crates/folio/src/face_srs.rs` | 1,024 | 476 | 100 | 1,031（7 行増える） |

   同じ file には便 100 の歯（`crates/folio/tests/face_srs.rs` の f100_・器の式で 1,100 行以下）も掛かり、本便の後の余地は 69（参考値）で内に収まる。歯の file（`crates/folio/tests/face_srs.rs`）は src の外なので余地を測らない（参考に書くと、器の式で 397 行から 587 行）。

3. **size は S。** 変える src は 1 本で 7 行、歯は 1 file に 4 本。
4. **verify は 3 行**で、done の 3 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test face_srs f118_` = (d) の新しい歯 4 本。
   2. `cargo nextest run -p folio --test face_srs` = 書き換える歯の file の歯の全部（既存の f81_・f84_ と、器の式の行数の上限を見る f100_ を含む）。
   3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file は `crates/folio/tests/face_srs.rs` の 1 本だけで、write-set に在る。要件書の面の凍結 fixture との byte 一致の歯（`crates/folio/tests/face_srs_body.rs`）と実の置き場の組み立ての歯（`crates/folio/tests/site.rs`）は本文を変えないので write-set に入れず、verify でも名指さない＝共通の検証（`.vessel.toml` の common-verify の workspace の nextest）が走らせる。絞り込みの語 f118_ は `--test face_srs` の中だけで効き、src の単体の歯に同じ語で始まる関数は無い（base で `git grep -n 'fn f118_'` が 0 件）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は `~/.local/share/folio2/handoff-2026-09-24/d118-measured.patch`（base a34c6b2 に `git apply --check` が通る・repo には入れない）。起草の記録は同じ dir の `d118-draft.md`。

1. base の写し: `git clone --shared <folio2> <写し>` → `git -C <写し> checkout a34c6b2`。
2. RED: 歯の file（`crates/folio/tests/face_srs.rs`）だけを当てて `cargo nextest run -p folio --test face_srs f118_ --no-fail-fast` → 歯 1 と歯 2 が落ち、歯 3 と歯 4 は通る。
3. 全部: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）。
4. 変異: (d) の表の各行を `crates/folio/src/face_srs.rs` に 1 つずつ当て、`cargo nextest run -p folio --test face_srs f118_ --no-fail-fast`。
5. 出力の差: (e) の 3 の手順。
6. 門: `folio ceiling --gate --write-set crates/folio/src/face_srs.rs crates/folio/tests/face_srs.rs`。
7. 余地: (g) の 2 の式で `crates/folio/src/face_srs.rs` を数える。

## 2. 範囲

- 入れる: `crates/folio/src/face_srs.rs` の scope_chapter の段の範囲の表に「必須か」の真偽と scope_m3（名札 M3・任意）の 1 行・任意の節を無ければ飛ばす 3 行・注の条件を「鍵が scope でない」に・注の 1 行。`crates/folio/tests/face_srs.rs` の f118_ の歯 4 本と定数 4 つ（見本の字 3 と塊の開きの字 1）と helper 6 つと頭の注の 2 行。
- 入れない: 要件書の正本（scope_m3 の節そのものを含む）・面の fixture・凍結 fixture・部品目録・様式・ほかの面の生成器（憲法の面の M1 の名札の文を含む）・床の定数と床の判定・天井の正本の読む欄・判断の記録・語彙・規則の表・新しい命令と旗・新しい file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| table | 段の範囲の表 | `crates/folio/src/face_srs.rs` の scope_chapter の中の表（鍵・名札・必須か） |
| callout | 段の範囲の塊 | 部品 section-lead-callout の div と card 2 枚（既存の部品・本便で足さない） |
| teeth | 歯 | `crates/folio/tests/face_srs.rs` の f118_ の 4 本 |
| frozen | 凍結 fixture | `tests/fixtures/face/expected-srs.html`（本便で動かさない・歯 2 と歯 3 の突き合わせの相手） |

## 4. 検査（歯）

§1 (d) と (g) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい file も dir も無い。
- 前提の着地: 便 117（行 `dp`・床の定数に scope_m3・main a34c6b2 で着地済み）。本便の歯は面だけを撃つので床の定数には依らないが、要件書が scope_m3 を持てる順序（床が先・面が後）はこの着地で満ちた。
- 並行の便: 一括 15（枝 docs/batch15・承認待ち）は設計文書の正本と `docs/design/` だけを書き換え、本便の write-set 2 本とは重ならない。便 117 の write-set（床の定数・要件書の生成区間・凍結 anchor・床の歯の 2 file）とも重ならない。
- 本便の着地の後に、席が要件書の版 B（scope_m3 の節の中身）を起草し、持ち主の承認を得る（ADR-16 決定 (7)）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dq"
title = "要件書の面の生成器 crates/folio/src/face_srs.rs の章 02 範囲の段の範囲の表（鍵と名札 M0 の scope・M1 の scope_m1）に、M3 の範囲の節 scope_m3 を名札 M3 の任意の行として scope_m1 の行の直後へ足し（表の各行に必須かの真偽を持たせ、scope と scope_m1 は必須のまま・scope_m3 は無い・null なら塊を置かない）、注の小窓を scope_m1 だけでなく段の節（scope でない鍵）の note に広げる。これは便 117 の契約 docs/design/delivery-117.md の §1 (h) の 2 の穴（要件書に scope_m3 の節を足しても面に 1 字も出ない）を、判断の記録 ADR-16 の版 B より先に塞ぐ面の側の 1 便である。歯は crates/folio/tests/face_srs.rs に 4 本足す（面の fixture の scope_m1 の節を scope_m3 の名で写すと 3 つ目の塊が 2 つ目の塊の名札 M1 を M3 に替えた字と byte 一致／手書きの scope_m3 の節は塊 1 つだけを足し値は escape され、その塊を除いた面は凍結 fixture tests/fixtures/face/expected-srs.html と一致／scope_m3 が null なら面は凍結 fixture と byte 一致し、scope か scope_m1 を落とすと 2 で面を書かない／実の要件書の面の M3 の塊の数は実の要件書が scope_m3 を持つとき 1・持たないとき 0）。見本の節は歯の file の中の最小の手書きの字で、面の fixture・凍結 fixture・部品目録・様式・要件書の正本・床・天井の正本・判断の記録・語彙・規則の表は 1 字も触らず、新しい file も dir も作らず、file も消さない。実の要件書は scope_m3 を持たないので配信の組み立ての出力は本便の前後で変わらない"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_srs.rs", "crates/folio/tests/face_srs.rs"]
verify = ["cargo nextest run -p folio --test face_srs f118_", "cargo nextest run -p folio --test face_srs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f118_ の歯 4 本（scope_m1 の節を scope_m3 の名で写した写しの要件書の面の章 02 に段の範囲の塊が M0・M1・M3 の順に 3 つ在り、3 つ目が 2 つ目の名札 M1 を M3 に替えた字と byte 一致／手書きの scope_m3 の節の値が escape されて M3 の塊に在り、その塊を除いた面が凍結 fixture と一致／scope_m3 が null の面が凍結 fixture と byte 一致し、scope か scope_m1 を落とした写しは終了コード 2 で面を書かない／実の要件書の面の M3 の塊の数が実の要件書の scope_m3 の有無と一致）が緑、crates/folio/tests/face_srs.rs の既存の歯が全部緑、clippy が 0 警告で、workspace の nextest が全部緑（要件書の面の凍結 fixture との byte 一致の歯と実の置き場の組み立ての歯を含む）で CI が通る"
<!-- contracts:end -->
