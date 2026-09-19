# 設計: 便 40 — 面に天井の名札を出す（部品 ceiling-stamp・folio face / build の --ceiling・観点 4 つの 3 値と日付・ADR-8 決定 (4)）

- 要件: FR18（天井の 3 値を床で数える・その結果を面に出す）/ FR4（全面を 1 つの生成器と 1 つの design token で出す）/ FR16（面の部品の名札が部品目録に在る・AC14）
- 条: P-3.3（床の合格を天井の合格として扱わない = 別の欄に出す）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-2.3・P-2.4（design token は 1 か所・部品は閉じた一覧で追加には判断の記録）/ P-6.3（数と字は正本から導出）
- 判断の記録: ADR-8 決定 (4)（生成した面には天井の名札〔観点ごとの 3 値と日付〕を床の合格の名札と別の欄に出す）・帰結（部品目録に天井の名札の部品を足す便は、生成器の閉じた一覧と目録の一致の検査を通す・面の型は増えない）・(6)（順序 = 便 39〔所見の検査〕→ 本便 → folio2 自身で 1 周）。語彙 ceiling-badge（天井の名札）。判断の記録 ADR-5（部品目録・見た目は見本・手書きの生成器）。便 39（f2-648.55）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効・f2-648.52 notes）。部品を 1 つ足す判断の記録は ADR-8 帰結（P-2.4）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ao が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

天井の結果（観点ごとの 合格・不合格・まだ分からない と日付）を、生成した 5 つの面（入口・憲法・要件書・判断の記録・設計ノート）の site-bar に、床の名札（生成日・版・状態 = 部品 freshness-stamp）と別の部品 ceiling-stamp として出す（ADR-8 決定 (4)・P-3.3）。面の生成器は束の置き場を任意の旗 --ceiling で受け、無ければ 4 観点とも「まだ分からない」を出す（P-4.2・天井を回していない面が合格に見えない）。3 値の数え方は便 39 の規則を使い、面の生成の中では測れない 1 つ（束が古いか）だけを除く（理由は (b)）。

planner の実測（2026-09-19・main d9f3d64）: 部品目録 design-intent/preview/parts.json の components は 29（各行は role と faces）・crates/folio/build.rs が組み立て時に目録から閉じた一覧 Component を導出し crates/folio/src/parts.rs が取り込む（人は enum を書かない・parts.rs の unit の歯が件数 29 を固定）。凍結目録 tests/fixtures/floor/parts-catalog.json（parts --print の byte 一致）。床の名札 = crates/folio/src/face.rs の Frame の関数 head が site-bar に出す部品 freshness-stamp（字面「生成 <b>生成日</b> · <b>版</b>（状態）」・値は各面の生成器が渡す）で、5 つの面の生成器（face_index・face_constitution・face_srs・face_adr・face_note）が Frame の parts の一覧に FreshnessStamp を持つ。様式 design-intent/preview/folio.css（正規化 678）の freshness-stamp の規則は 93・94 行目と、中幅と狭幅で site-bar の名札を隠す 508・542 行目。面の生成器の口は derive(dir) か derive(dir, id)（crates/folio/src/site.rs の build_all が呼ぶ）・命令の口は main.rs の Face と Build（--dir・--out・--write・--check）。面の凍結 fixture は tests/fixtures/face/ の expected.html・expected-srs.html・expected-index.html・expected-index-sheet.html・expected-adr.html・expected-note.html・expected-site-adr-2.html の 7 本（site-bar に freshness-stamp を含む）。便 39 の module crates/folio/src/findings.rs が観点ごとの 3 値の規則（束が無い・壊れている・古い・所見 file の欄の決まり・要約値・止める の反証・AI の判定）を持つ。天井の正本 design-intent/ceiling.yaml の viewpoints の各行が id と name（忠実さ・読みやすさ・文書どうしの整合・実態との整合）を持ち、fixture の写し（tests/fixtures/ の 18 組 + 便 38 の source）にも同じ欄の ceiling.yaml が在る。

(a) 命令の口（main.rs の Face と Build に旗を 1 つ足す）: folio face … --ceiling <束の置き場> と folio build … --ceiling <束の置き場>。任意（省略可）・相対なら --dir からの相対・絶対ならそのまま（--out と同じ読み）。--write でも --check でも同じ値を渡す（--check は同じ入力から導出して配信先と byte 比較するので、名札の中身も一致の対象）。旗が無いときは名札を「未実施」の形で出す（(c)）。他の旗・終了コード・文言は変えない。

(b) 3 値の読み（便 39 の module findings.rs に、観点ごとの結果を返す関数を 1 つ足し、面の生成器がそれを呼ぶ）: 束の置き場の <観点の id>/ ごとに便 39 の規則 (c) の 1（束が無い）・2（束が壊れている）・4〜10（所見 file の欄の決まり・要約値の一致・止める の反証・AI の判定）をそのまま当て、3（束が古い = 現在の正本と面から組み直した要約値との比較）だけを当てない。理由: 名札を載せた面そのものが次の束の入力（faces/）になるので、面の生成の中で「現在の面から組んだ要約値」を求めると固定点が無い（名札の字が要約値を変え、要約値が名札の字を変える）。束が古いかは folio ceiling --check（--faces = AI が読んだ配信先）が数える側の領分で、名札はその代わりに束の要約値の先頭 8 字を出して読み手が突き合わせられるようにする。返す値 = 観点ごとに（id・3 値・起動の記録の at〔読めたときだけ〕・digest.txt の 16 進の先頭 8 字〔読めたときだけ〕）。理由の文言は名札に出さない（面は数えた結果だけ・理由は --check の標準エラー）。
- 束の置き場が無い・観点の dir が無い → その観点は まだ分からない（at と要約値は無し）。天井の正本 ceiling.yaml の viewpoints の順に 4 つ。

(c) 部品 ceiling-stamp（部品目録に 1 行足す: role は chrome・faces は index・constitution・srs・adr・note の 5 つ = freshness-stamp と同じ）。site-bar の中で freshness-stamp の直後に置く。字面は次の 2 形（値は escape 済み・観点の名は天井の正本 <dir>/ceiling.yaml の viewpoints の name を逐語・順も正本のとおり）:
- 束の置き場が在るとき: 「<span data-component=ceiling-stamp>天井 <b>忠実さ 合格</b> · <b>読みやすさ 合格</b> · <b>文書どうしの整合 合格</b> · <b>実態との整合 まだ分からない</b>（<日付>・束 <要約値の先頭 8 字>）</span>」の形。<日付> = 4 観点の at のうち読めたものの byte 順で最大の 1 つ（1 つも読めなければ「日付なし」）。<要約値の先頭 8 字> = 4 観点の digest.txt の先頭 8 字を観点の順に「/」で繋いだもの（読めない観点は「--------」= 半角の横線 8 つ）。
- 束の置き場が無いとき（--ceiling なし）: 「<span data-component=ceiling-stamp>天井 <b>忠実さ まだ分からない</b> · <b>読みやすさ まだ分からない</b> · <b>文書どうしの整合 まだ分からない</b> · <b>実態との整合 まだ分からない</b>（未実施）</span>」の形。
- 属性の字面は data-component の値を二重引用符で囲む（他の部品と同じ・上の形では引用符を省いて書いた）。
- 様式（design-intent/preview/folio.css）: ceiling-stamp の規則は freshness-stamp の 93・94 行目と同じ値で（色・枠・角・余白・nowrap）、中幅・狭幅で隠す 508・542 行目の選択子に ceiling-stamp を並べて足す（bar を 1 行に保つ）。design token は足さない（既存の変数だけ）。印刷は名札を紙に残す（目録の rules の print と同じ）。

(d) 面の生成器の接続: Frame の parts の一覧（5 面）に CeilingStamp を足し、Frame の関数 head に名札の字（組み立て済み）を渡す引数を 1 つ足す。5 つの面の生成器の口を derive(dir, ceiling: Option<&Path>)（判断の記録・設計ノートは id も）にし、site.rs の run と build_all が --ceiling を通す。face.rs の run も同じ。名札の字は面ごとに組み直さず 1 つの関数（findings.rs の結果 → (c) の字面）で作り、5 面で同じ字になる。天井の正本が読めない（ceiling.yaml が無い・viewpoints が読めない）ときは面を導出できない = 2「まだ分からない」（面は 1 byte も書かない・P-4.1・他の正本が読めないときと同じ）。

(e) 部品目録と凍結物の更新: design-intent/preview/parts.json に ceiling-stamp の行（components は 30）・parts.rs の unit の歯の件数 29 → 30・tests/fixtures/floor/parts-catalog.json を parts --print の出力で更新（byte 一致の歯）・面の凍結 fixture 7 本を --ceiling なしの形（(c) の「未実施」の名札）で更新（各 file は site-bar の 1 行だけが変わる = ceiling-stamp の span が freshness-stamp の直後に入る・他の行は 1 byte も変えない）。凍結 fixture の更新は生成器の出力を写す形になるので、独立の物差しとして (c) の字面の逐語と (f) の歯 1 を置く（P-10.2）。

(f) 歯（新しい file crates/folio/tests/badge.rs・関数名は badge を含める・--test badge の scope・組み立てた binary を子の処理で撃つ）:
1. 逐語: 便 38 の凍結 fixture（tests/fixtures/ceiling/bundle/ の source と faces）から folio ceiling --write で束を組み、便 39 の凍結 fixture の pass-<観点>.yaml 4 本を写して folio face --face index --ceiling <束> --write → 0 ∧ 出力に (c) の 1 形目の字面（観点 4 つの名と 合格 ×4・日付 2026-09-19T05:00:00Z・束 2bd67637/ded1548e/91ca924d/24ce1b87）が逐語で 1 度在る。fidelity を stop-upheld.yaml に差し替える → 「忠実さ 不合格」。fidelity を missing-field.yaml に → 「忠実さ まだ分からない」∧ 日付は残り 3 観点の at。fidelity の dir を消す → 「忠実さ まだ分からない」∧ 束の先頭が「--------」。
2. 未実施: --ceiling なし → (c) の 2 形目の字面が逐語で在る ∧ 「未実施」。
3. 5 面: build --ceiling <束> --write（便 38 の fixture の source を写した dir・面の凍結 fixture の dir ではなく実の正本の写し）→ 0 ∧ index.html・constitution.html・srs.html・adr-1.html・note-example.html の全部に ceiling-stamp の span が 1 つずつ ∧ 5 面で字面が同じ ∧ freshness-stamp の直後に在る。
4. 一致: build --ceiling <束> --check → 0（--write と同じ入力で byte 一致）。--ceiling を変えて --check → 1（DRIFT）。
5. 目録: parts --print が凍結目録と byte 一致 ∧ parts --check が 5 面で合格 0/0（ceiling-stamp が目録に在る）。
6. 凍結: face --face index / constitution / srs / adr / note を fixture で --write して 7 本の期待と byte 一致（既存の歯の期待 = 更新した期待）。
既存の歯 crates/folio/tests/site.rs（配信先の凍結・3 値・全部か無しか）と crates/folio/tests/findings.rs（便 39・3 値の規則）は本文不変で期待不変（verify に回帰を 1 行ずつ）。既存の歯 crates/folio/tests/face.rs と crates/folio/tests/face_index.rs は部品の名札の census（面の data-component が閉じた一覧 ALLOWED に在る）を持つので、一覧に ceiling-stamp を足す: face.rs の憲法の面 14 → 15・要件書の面 17 → 18・face_index.rs の入口の面 12 → 13（断りの文言の数も同じく）・他は 1 字も変えない。face_adr.rs・face_note.rs・face_srs.rs の凍結の歯は (e) の期待の更新で緑（本文は変えない・verify の common-verify が回す）。

(g) 便 39 までの形との接続: 新規 file = crates/folio/tests/badge.rs（1 本）。既存 = findings.rs（結果を返す関数 1 つ）・face.rs（Frame の parts と head）・5 つの面の生成器・site.rs・main.rs・parts.rs（件数）・parts.json（行 1 つ・note に「便 40 で ceiling-stamp を足した（ADR-8）」の 1 行を添える）・folio.css・parts-catalog.json・面の凍結 fixture 7 本・census の歯 2 本（tests/face.rs・tests/face_index.rs）。src の増分は各 file 数行〜数十行で、余地の小さい face_srs.rs（117）・face_index.rs（176）でも収まる = size S。天井の正本・rules・語彙は変えない。folio2 自身で 4 観点を 1 周回す手順 = build → ceiling --write → 席が AI を回して findings.yaml → ceiling --check → build --ceiling → serve。

## 2. 範囲

- 入れる: 部品 ceiling-stamp・--ceiling の旗（face / build）・findings.rs の結果の関数・5 面の site-bar・様式・目録と凍結物の更新・歯。
- 入れない: 反証の材料の束を組む口（後続の便）・AI の起動・面の型の追加・天井の正本の変更・脚注や承認欄への名札（site-bar だけ）・束が古いかの判定（--check の領分）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 口 | main.rs の Face / Build に --ceiling（任意） |
| badge | 名札 | findings.rs の結果の関数 + face.rs の字面 (c) |
| faces | 面 | 5 つの生成器の parts と head の呼び・site.rs の通し |
| catalog | 目録 | parts.json +1・parts.rs 30・parts-catalog.json・folio.css |
| anchor | 凍結 | 面の凍結 fixture 7 本の更新 + (c) の逐語 + 歯 1 |
| teeth | 歯 | tests/badge.rs 6 群 + census の一覧 +1（face / face_index）+ 既存 site / findings の期待不変 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ao"
title = "面に天井の名札を出す（部品 ceiling-stamp・folio face / build の --ceiling・観点 4 つの 3 値と日付と束の要約値の先頭・無ければ未実施・5 面の site-bar・目録 +1 と凍結物の更新）"
req = ["FR18", "FR4", "FR16"]
section = "1"
write-set = ["+crates/folio/tests/badge.rs", "crates/folio/src/findings.rs", "crates/folio/src/face.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/site.rs", "crates/folio/src/main.rs", "crates/folio/src/parts.rs", "crates/folio/tests/site.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/face.rs", "crates/folio/tests/face_index.rs", "design-intent/preview/parts.json", "design-intent/preview/folio.css", "tests/fixtures/floor/parts-catalog.json", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test badge badge", "cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test findings findings", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "badge の歯（逐語 4 通り・未実施・5 面・一致と DRIFT・目録・凍結 7 本）が緑、便 17 以降の歯 site が期待不変で緑、便 39 の歯 findings が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
