# 設計: 便 79 — 数値の表の値の枡に日本語の小見出しを添え、裁定の列の前の裁定を折りたたむ（天井 16 / 18 周目 読みやすさ・19 周目 読みやすさ F-5・FR4）

- 要件: FR4（人が読むページを 1 つの生成器から出す）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-2.3（design token は 1 か所）/ P-4.1（読めない入力を異常なしにしない）/ P-6.2（生成物を手で直さない）/ P-10.1（凍結 anchor）
- 出所: 一括 10 の仕分け C。天井の 16 周目 読みやすさ F-3（＝18 周目 読みやすさ F-1 の同じ項）「数値の表の R-16 の値の枡に日本語の小見出しを添える（値そのものは床が読む型付きデータなので変えない）」と 19 周目 読みやすさ F-5「数値の表の裁定の列を開閉式にして、既定では最新の 1 件だけを出す」。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cb が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 1 本）。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ と tests/fixtures/face/ だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

憲法の面の章 05（数値と作法の表）は、規則の表 design-intent/rules.yaml の閾値行と開発規律行を 2 つの表で出す。今の面には読み手が詰まる所が 2 つある。① 閾値行 R-16 の値は入れ子の表（marks・prohibition〔word・clause_ends〕・units）で、値の枡は英語の鍵をそのまま太字にして並べるだけなので、何の一覧なのかが面から読めない。② 裁定の欄は正本の逐語をそのまま 1 枡に流し込むので、最新の裁定と、その後ろに連なる過去の裁定（最大 3 件）が同じ大きさで並び、いま効いている裁定がどれか分からない。本便はこの 2 つを面の側だけで直す。規則の表の値・種別・母集団・裁定の逐語は 1 字も変えない（P-5.1 のとおり正本は型付きデータで、面はその読める形）。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- 値の枡を組むのは crates/folio/src/face_constitution.rs の関数 rules_chapter（706〜801 行）で、閾値行の tr を組む format!（757〜767 行）の中で crates/folio/src/face.rs の関数 val（415〜447 行）を val(&x.f(value)?, 0) の形で呼んでいる。val はこの 1 か所からしか呼ばれない（`grep -rn 'val(&x' crates/folio/src` の実の呼び出しは face_constitution.rs 764 行だけ）。
- val は表を受けると、鍵ごとに 「<全角空白 × 深さ><b><鍵></b>: <値>」 を組み、行の間を br で繋ぐ。入れ子の表は値の前に br を 1 つ置いて深さ + 1 で同じ形に落とす。一覧は各項を 「」 で包んで中黒で繋ぐ。null は code の中に null。
- 裁定の枡を組むのは同じ file の関数 ruling（811〜813 行）で、中身は 「{ruling}（{ruled_at}）」 の 1 行だけ。
- 正本 design-intent/rules.yaml の裁定の欄で過去の裁定を連ねる区切りの字面は 「）・前の裁定 = 」 の 1 つに揃っている。区切りを持つ行は 13 行（閾値行 11 本 = R-1・R-2・R-3・R-4・R-7・R-9・R-11・R-12・R-13・R-14・R-16／開発規律行 2 本 = D-3・D-4）、総出現は 19 回、揺れは 0 回。件数の内訳は R-1 と D-3 が 3 件、R-2 と R-14 が 2 件、残りの 9 行が 1 件。区切りを持たない行は 15 行（D-1 を含む）。
- 小窓は face.rs の関数 hint（358〜363 行・名札と組み立て済みの HTML を受ける）。様式 design-intent/preview/folio.css は小窓の本体に折り返しを許す規則（588〜599 行）を既に持ち、表の枡の中の小窓の規則（584 行）も持つので、本便は様式を 1 byte も触らない。
- 部品目録 design-intent/preview/parts.json は本便で触らない（新しい部品も新しい class も作らない。枡の中で使うのは既存の b・br・小窓だけ）。

### (a) 値の枡の日本語の小見出し

face_constitution.rs に閉じた名札の表と、それを使う関数を 1 本ずつ足す。

1. 定数 VALUE_KEY_LABELS: &[(&str, &str)] = 閾値行の値の表に現れてよい鍵と、その日本語の小見出しの対。初版は 5 対で、R-16 の値の鍵の全部を覆う。順は正本の鍵の順ではなく表引きにだけ使う。
   - marks → 規範の印
   - prohibition → 「禁止」の扱い
   - word → 語
   - clause_ends → 直後の字
   - units → 単位
2. 関数 value_cell(x: &X<'_>, d: usize) -> R<String>。値が表のときだけ自分で組み、表でないときは face::val(x, d) にそのまま渡す（値が表でない閾値行の枡の字面は 1 byte も変わらない）。表のときは鍵ごとに次を組み、行の間を br で繋ぐ。
   - 「<全角空白 × d><b><小見出し>（<鍵>）</b>: <値>」
   - 値が表なら、値の前に br を 1 つ置いて value_cell(値, d + 1) を入れる（val の入れ子と同じ運び）。値が表でなければ face::val(値, d + 1) を入れる。
   - 鍵が VALUE_KEY_LABELS に無ければ Err（面は導出できない・終了コード 2・P-4.1）。エラーの字は 「rules 行の値の表に無い鍵「<鍵>」」 と、その鍵の在り処（X の at）を含める。
3. rules_chapter（764 行）の val(&x.f(value)?, 0) を value_cell(&x.f(value)?, 0) に差し替える。ほかの 5 つの枡（id・何の数値か・種別・裁定・状態）と、開発規律行の表は触らない。face::val は (a) 2 の葉から呼び続けるので、公開のまま残る（使われない関数にはならない）。

### (b) 裁定の列の折りたたみ

同じ file の関数 ruling（811〜813 行）を次の形にする。判定は正本の逐語（escape 済み）に対して行う。区切りの字面 「）・前の裁定 = 」 には escape される字が 1 つも無いので、escape の前後で分け方は変わらない。

1. 逐語を区切り 「）・前の裁定 = 」 で分ける。分かれなければ（区切りが 0 回）今までどおり 「{ruling}（{ruled_at}）」 を返す（15 行の枡の字面は 1 byte も変わらない）。
2. 分かれたら、枡に出すのは 「<1 つ目の切れ端>）（{ruled_at}）」 だけにする（閉じ括弧は区切りの頭の 1 字なので、切り落とした分を書き戻す）。
3. その後ろに hint(「前の裁定 <N> 件」, <本体>) を 1 つ置く。N は 2 つ目以降の切れ端の数。本体は切れ端ごとに 「<p>前の裁定 = <切れ端></p>」 を並べたもの。最後の切れ端以外は末尾の閉じ括弧を書き戻す（切れ端の間の区切りも同じ 1 つの字面だから）。
4. 枡に直に見える字に 「前の裁定 = 」 が 1 つも残らないこと。

### (c) 歯（関数名は f79_ で始める。`grep -rn 'fn f79_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

歯は新しい file crates/folio/tests/face_constitution.rs（新規・+）に置く。憲法の面の歯の置き場が今は crates/folio/tests/face.rs（幅 120 で正規化して 1637 行・92 の関数）に相乗りしており、入口の面（tests/face_index.rs）・判断の記録の面（tests/face_adr.rs）・設計ノートの面（tests/face_note.rs）が既に持つ 1 面 1 file の形に揃える。既存の歯は 1 本も移さない（tests/face.rs は 1 byte も変えない）。新しい file は tests/face_adr.rs と同じ作り（binary を CARGO_BIN_EXE_folio で起動し、実の置き場 design-intent/ と図の道具 vendor/archify/ を一時 dir へ写して面を書く）で始める。

1. f79_rules_value_cell_labels_every_key: 実の置き場の写しで憲法の面を書く → 0 ∧ R-16 の行の値の枡に 5 つの小見出し（規範の印・「禁止」の扱い・語・直後の字・単位）がこの順で在る ∧ 同じ枡に 5 つの鍵（marks・prohibition・word・clause_ends・units）が括弧付きで在る。本便の前の main では小見出しが 1 つも無いので赤い歯。
2. f79_rules_value_cell_is_closed: 写しの rules.yaml の R-16 の値に鍵 extras を 1 つ足す → 2（まだ分からない）∧ 標準エラーに extras と 「rules 行の値の表に無い鍵」。
3. f79_rules_value_cell_leaves_scalars_alone: 同じ面で、値が表でない閾値行（R-1）の値の枡に b の開始タグが 1 つも無い（小見出しは表のときだけ付く）。
4. f79_rules_ruling_shows_only_the_latest: R-14 の行の裁定の枡で、小窓より前に見える字に 「前の裁定 = 」 が 1 つも無く、2026-09-21 22:50 JST の裁定と ruled_at の 2026-09-21 が在る。本便の前の main では見える字に 「前の裁定 = 」 が 2 回在るので赤い歯。
5. f79_rules_ruling_folds_the_previous_ones: 同じ面で、R-14 の小窓の名札が 「前の裁定 2 件」 ∧ 小窓の本体に 「前の裁定 = 」 が 2 回在る。あわせて R-1 と D-3 の名札が 「前の裁定 3 件」、R-2 の名札が 「前の裁定 2 件」 である（件数は歯の側で正本 design-intent/rules.yaml を直に読んで数え、生成器の数えを写さない）。
6. f79_rules_ruling_without_previous_is_unchanged: 区切りを持たない行（D-1）の裁定の枡に小窓が無く、字面が 「{ruling}（{ruled_at}）」 のままである（歯の側で正本から組む）。
7. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯すべて。凍結の写し tests/fixtures/face/expected.html を (d) で生成し直すので、この 3 file が持つ byte 一致の歯がその写しを見る。

### (d) 凍結の写し

面の字が変わるのは憲法の面だけなので、tests/fixtures/face/expected.html を同じ着地で生成し直して置き換える。手で直さず、歯と同じ経路（binary に face --face constitution --dir tests/fixtures/face --out <一時 file> --write を当て、その出力を写す）で作る。ほかの 6 本（expected-index.html・expected-index-sheet.html・expected-srs.html・expected-adr.html・expected-note.html・expected-site-adr-2.html）は 1 byte も変わらない。写しの正本 tests/fixtures/face/rules.yaml は触らない（R-16 の形の値を持たない最小の手書きなので、(a) の小見出しはこの写しでは出ない。(a) の歯は実の置き場の写しで見る）。

### (e) 大きさ

src は crates/folio/src/face_constitution.rs 1 本だけ（幅 120 で正規化して 1130 行・余地 370・見積 + 約 55 行）。歯は新規の crates/folio/tests/face_constitution.rs（+ 約 200 行・helper を含む）。size **S**（src の余地は 100 以上）。外部 crate は増やさない。crates/folio/src/face.rs・face_srs.rs・face_adr.rs・face_note.rs・face_index.rs・図の道具・部品目録・様式・design-intent の下の正本・tests/floor_cases.yaml・CI の yml は触らない。

### (f) 本便が運ばないもの

規則の表の値・種別・母集団・裁定の逐語・注（design-intent/rules.yaml）。要件書 FR4 の規範文と版。部品目録と様式。ほかの 4 面の字。開発規律行の表の裁定の枡（(b) は閾値行と開発規律行の両方が通る同じ関数 ruling を直すので、D-3 と D-4 の枡も折りたたみが付く。これは同じ直しの射程の内で、別の判断は要らない）。

## 2. 範囲

- 入れる: 名札の表 1 本・値の枡の関数 1 本・裁定の関数の書き換え・歯 6 本と新しい歯の file・凍結の写し 1 本の生成し直し。
- 入れない: 正本の値と逐語・床の判定・部品目録・様式・新しい class と新しい部品・ほかの 4 面・face.rs の val の字面・tests/face.rs の既存の歯。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| labels | 名札の表 | face_constitution.rs の VALUE_KEY_LABELS（5 対・閉じた表） |
| cell | 値の枡 | face_constitution.rs の value_cell（表は自分で組み、葉は face::val） |
| fold | 折りたたみ | face_constitution.rs の ruling（最新 1 件 + 小窓） |
| teeth | 歯 | 新規 crates/folio/tests/face_constitution.rs の f79_ 6 本 |
| frozen | 凍結 | tests/fixtures/face/expected.html の生成し直し |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cb"
title = "憲法の面の数値の表で、閾値行の値の枡の鍵に閉じた表から日本語の小見出しを添え（表に無い鍵は まだ分からない）、裁定の枡は最新の 1 件だけを出して過去の裁定を小窓へ畳む（正本の値と逐語と床の判定は不変・凍結の写し 1 本を生成し直す・天井 16 / 18 周目 読みやすさと 19 周目 読みやすさ F-5）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_constitution.rs", "+crates/folio/tests/face_constitution.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs", "tests/fixtures/face/expected.html"]
verify = ["cargo nextest run -p folio --test face_constitution f79_", "cargo nextest run -p folio --test face_constitution --test face --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f79_ の歯 6 本（小見出し 5 つと鍵の並び・知らない鍵で まだ分からない・値が表でない行は素のまま・裁定の枡に過去の裁定が見えない・小窓の名札と件数が正本の数と一致・区切りの無い行は字面が不変）が緑、tests/face.rs と tests/site.rs と tests/badge.rs の既存の歯が全部緑（生成し直した凍結の写し expected.html との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
