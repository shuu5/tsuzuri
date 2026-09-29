# 設計: 便 74 — 判断の記録と設計ノートの面に用語集への導線を 1 本ずつ、要件書の面の表紙の図の列に章 09 の図を、設計ノートの面の表紙に読み手の名札を置く（天井の 14 周目の読みやすさの是正・FR4 / FR9 / FR16）

- 要件: FR4（3 枚を 1 つの生成器で出す = 要件書の面）/ FR9（設計ノートを 1 つの型で生成する = 設計ノートの面）/ FR16（判断の記録の面を正本から逐語で生成する）/ GOAL2（非エンジニアが読める）
- 条: P-2.1（人が読むページは 1 つの生成器から）/ P-2.3（元の値は 1 か所）/ P-2.4（部品は閉じた一覧）/ P-6.1（正本から逐語で生成）
- 出所: 天井の 14 周目（2026-09-21・main 6de202c）の読みやすさの所見 3 件。① 判断の記録の面と設計ノートの面から用語集へ行く導線が 0 本（入口の面だけが札を持つ）。② 要件書の面の表紙の「図」の行が 図 1〜3 だけで、章 09 の図（正本の最上位の figures から出る図）が表紙から辿れない。③ 設計ノートの面だけ読み手の印が無い（憲法・要件書・判断の記録の面は表紙に平易文の枠を持つ）。
- 根拠の判断: 見た目と導線の直しであり、新しい判断は無い。札の字面・行き先・部品は入口の面が既に出しているものをそのまま使い、図の番号は章 09 の番号の付け方（face_srs.rs の own_figures と figures_chapter）に合わせるだけ。③ の 1 行は生成器の名札で、正本には欄を足さない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bw が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規 file は無い）。
- 門: 本便は design-intent の下の file を 1 つも書き換えないので、天井の門（folio ceiling --gate）は 通す（設計文書の正本を書き換えない便）。

## 1. 目的と中身

(a) 用語集への札の共有の口。crates/folio/src/face.rs（生の行 1371・正規化 1281・余地 219）に 2 つ足す。

1. `pub fn glossary_chip(dir: &Path) -> R<String>`。語彙の正本 vocabulary.yaml の terms の数を数え、入口の面の棚の札（crates/folio/src/face_index.rs の 983〜998 行 shelf_link の chips・札の字面は 988 行）と同じ字面の 1 枚を組む: 外が span（class は annex-chips）・その中が a（href は constitution.html#s7）・a の中身が 「付録 語彙 」 + span（class は cnt）で 「{terms} 語 → 憲法 §7」（属性の値は入口の面と同じく二重引用符で囲む・入口の面の 988 行の字面をそのまま写す）。章の番号 7 と単位「語」は face.rs の 740 行の定数 ANNEXES の vocabulary の行から取る（面の側に数を書かない）。terms が読めなければ Err（その面は「まだ分からない」に落ちる）。数は正本から数えるので固定の数を書かない（今の実の正本 design-intent/vocabulary.yaml は 49 語・凍結 fixture tests/fixtures/face/vocabulary.yaml は 2 語）。

2. `Frame::foot`（face.rs の 972〜994 行・doc-locator の行は 989 行）の隣に `pub fn foot_aside(&self, o: &mut Vec<String>, version: &str, generated: &str, dl: &str, aside: &str)` を置き、所属の行（p・class は doc-locator）の末尾、入口へ戻る の a の閉じタグの直後に `{aside}` を差し込む（行のほかの字は変えない）。`foot` は `aside` に空の字を渡して `foot_aside` を呼ぶだけの薄い口にする（憲法の面と要件書の面の脚は 1 byte も変わらない）。

(b) 判断の記録の面。crates/folio/src/face_adr.rs（生の行 845・正規化 800・余地 700）の中の脚の関数 foot（797〜817 行）に引数 `chip: &str` を足し、815 行の `f.foot(o, id, &date, &dl)` を `f.foot_aside(o, id, &date, &dl, chip)` にする。札そのものは derive の側（225 行の呼び出し）で `face::glossary_chip(dir)?` から組んで渡す（derive は dir を持つので、脚の関数は path を知らないまま）。表紙・章・目次は触らない。

(c) 設計ノートの面。crates/folio/src/face_note.rs（生の行 1009・正規化 946・余地 554）で 2 つ。

1. 脚の関数 foot（997〜1009 行・1007 行の `f.foot(o, …)`）を (b) と同じ形にする（引数 `chip: &str` を足し、札は derive の側の 303 行の呼び出しで組んで渡す）。
2. 表紙（cover・631〜660 行）の副題の直後、任意の「注」の枠より前に、読み手の名札を既存の部品 summary-card で 1 枚置く: 印の字 = 読・名札 = 読み手・本文 = 「作る人（実装する人）。中の組み立て方（口・検査・契約表）を書く面です。何を作るかは要件書へ、なぜそう決めたかは判断の記録へ。」。文は生成器が持つ固定の 1 行で、正本 design-note/*.yaml にも欄の決まり design-intent/design-note/schema.yaml にも欄を足さない（足すと設計文書が変わり天井の門に掛かる）。

(d) 要件書の面の表紙の図の列。crates/folio/src/face_srs.rs（生の行 1364・正規化 1299・余地 201）の cover の figs（531〜537 行）を、自前の図（`own_figures` = 2 + verdicts の節が在れば 1・1174 行）の後に章 09 の図を続ける形にする。章 09 は `(own_figures(ctx) + 1..).zip(&ctx.figures)` で番号を振る（figures_chapter・1189 行）ので、表紙も同じ数え方で `図 {n}` を並べる。行き先は図の行の id（章 09 の枠 `face::figure_panel`・face.rs の 1069 行〜 が `<figure … id={id}>` で付ける anchor）。figs の型は静的な字の列から所有する字の列に変える。数も番号も正本の figures から来るので固定の数を書かない。今の実の正本 design-intent/srs.yaml は最上位の figures が 2 枚（fig-plain・fig-tech・480 行〜）で verdicts の節が在るので表紙は 図 1〜図 5、凍結 fixture は 1 枚（fig-1）で 図 1〜図 4 になる。

(e) 凍結の面の写し。(b)(c) で tests/fixtures/face/expected-adr.html・expected-note.html・expected-site-adr-2.html が、(d) で expected-srs.html が変わるので、同じ着地で凍結の正本から生成し直して置き換える（手で直さない）。expected.html（憲法）・expected-index.html・expected-index-sheet.html は 1 byte も変わらない。

(f) 歯（関数名は f74_ で始める・`grep -rn 'fn f74_' crates/folio/tests` は今 0 本）。

1. f74_adr_foot_links_to_the_glossary（crates/folio/tests/face_adr.rs）: 凍結の正本から出した判断の記録の面の doc-locator の行に annex-chips の span が 1 つ在り、その中の a の行き先が constitution.html#s7 で、字が「付録 語彙 2 語 → 憲法 §7」（凍結 fixture の語彙は 2 語）。同じ正本の写しに語を 1 つ足して出すと「3 語」になる（数が正本から来る・固定でない）。
2. f74_glossary_chip_is_verbatim_the_index_chip（同）: 1 で取り出した a 要素の逐語が、入口の面の凍結 fixture tests/fixtures/face/expected-index.html（本便で変わらない独立の anchor・読むだけ）の中にそのまま在る。入口の札と字面・行き先・数が一致しなければ赤。
3. f74_note_foot_links_to_the_glossary（crates/folio/tests/face_note.rs）: 設計ノートの面でも doc-locator の行・行き先・字が 1 と同じ。
4. f74_note_cover_names_the_reader（同）: 表紙に名札が「読み手」の summary-card が 1 枚在り、「注」の枠より前に在る。かつ凍結の正本 tests/fixtures/face/design-note/full.yaml に「読み手」の字が無い（正本に欄を足していない）。
5. f74_srs_cover_lists_every_figure_panel（crates/folio/tests/face.rs）: 要件書の面の表紙の「図」の欄の a の列（行き先と番号）が、同じ面の中の figure-panel の id の列と順も数も一致し、番号が 1 からの連番。凍結 fixture では 4 つ（#fig-context 図 1・#fig-rail 図 2・#fig-verdicts 図 3・#fig-1 図 4）。今の生成器は表紙に 3 つしか出さないのでこの歯は赤になる。
6. f74_srs_cover_figure_href_comes_from_the_source（同）: 写しの正本の最上位の図の id を fig-1 から fig-x に替えて出すと、表紙の 4 つ目の行き先が #fig-x になる（行き先が正本から来る・固定でない）。
7. 回帰（期待不変・verify の 2 行目）: tests/face_adr.rs・tests/face_note.rs・tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯すべて（(e) で生成し直した凍結の写し 4 本との byte 一致の歯を含む）。

(g) 大きさと接続。新規 file は無い。face.rs（+約 25 行・余地 219 → 約 194）・face_adr.rs（+約 6 行・余地 700 → 約 694）・face_note.rs（+約 18 行・余地 554 → 約 536）・face_srs.rs（+約 12 行・余地 201 → 約 189）・tests/face_adr.rs（+約 70 行）・tests/face_note.rs（+約 70 行）・tests/face.rs（+約 80 行）・凍結の写し 4 本（再生成）。size S（src の各 file の余地はどれも 100 以上）。行数は生の行と正規化（空行を除き幅 120 で折る）の両方を書いたが、受付のときは席が測り直す。部品目録 design-intent/preview/parts.json にも様式 design-intent/preview/folio.css にも足さない: 使う class は annex-chips・cnt・summary-card・ic・lab・txt で、いずれも既に folio.css に在り入口の面と判断の記録の面が使っている（R-3 の判定は crates/folio/src/parts.rs の 213 行 check_face が面の class を CSS の class の集合と突き合わせる形で、目録に無い class は落ちる）。data-component は 1 つも増やさない。外部 crate は増やさない。design-intent は 1 file も触らない。先行の便は無い（便 71 行 bt・便 72 行 bu・便 73 行 bv はいずれも main 6de202c に着地済みで、本便が触る face.rs の脚の口と face_srs.rs の表紙の図の列はそのどれとも重ならない）。

(h) 置く場所を doc-locator の行にした理由（表紙でない理由）。判断の記録の面と設計ノートの面が共有する唯一の口が face.rs の脚（Frame::foot）で、ここに置けば 2 面に同じ 1 本の code path で入る。札の様式 `.annex-chips a` は紙の色（paper-2）と本文の墨（ink）で書かれていて、表紙の帯は濃い背景に別の link の色を当てているため、紙の地の上に出る doc-locator の行が札の設計どおりの見え方になる。出口の導線（入口へ戻る）と用語集への導線が同じ行に並ぶ。

## 2. 範囲

- 入れる: 用語集への札の共有の口・判断の記録と設計ノートの 2 面の脚・設計ノートの表紙の読み手の名札・要件書の表紙の図の列・凍結の写し 4 本・歯 6 本。
- 入れない: 憲法の面と入口の面（札を持っているので触らない）・要件書の面の用語集の章（§8 は既に憲法 §7 を指す）・正本と欄の決まりへの欄の追加・部品目録と様式の追加・章 09 の中身。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| chip | 用語集の札 | face.rs の glossary_chip（語彙の正本の terms を数える） |
| aside | 脚の口 | face.rs の foot_aside（doc-locator の行の末尾） |
| adr | 判断の記録 | face_adr.rs の脚が札を渡す |
| note | 設計ノート | face_note.rs の脚が札を渡す・表紙に読み手の名札 |
| figs | 図の列 | face_srs.rs の表紙が章 09 の図を続けて並べる |
| fixture | 写し | 凍結の面 4 本を再生成 |
| teeth | 歯 | tests/face_adr.rs に 2 本・tests/face_note.rs に 2 本・tests/face.rs に 2 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bw"
title = "判断の記録の面と設計ノートの面の脚に用語集への札（入口の面と同じ字面・行き先 constitution.html#s7・語の数は語彙の正本から）を 1 本ずつ置き、要件書の面の表紙の図の列に章 09 の図を own_figures の続きの番号で並べ、設計ノートの面の表紙に読み手の名札を既存の部品で 1 行置く（天井の 14 周目の読みやすさ 3 件・正本と部品目録と様式は触らない）"
req = ["FR4", "FR9", "FR16"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/face_srs.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test face_adr --test face_note --test face f74_", "cargo nextest run -p folio --test face_adr --test face_note --test face --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f74_ の歯 6 本（判断の記録と設計ノートの 2 面の用語集の札・入口の札との逐語一致・設計ノートの読み手の名札・要件書の表紙の図の列が面の figure-panel と一致・図の行き先が正本から来る）が緑、tests/face_adr.rs・tests/face_note.rs・tests/face.rs・tests/site.rs・tests/badge.rs の既存の歯が全部緑（生成し直した凍結の写し 4 本との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
