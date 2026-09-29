# 設計: 便 38 — 天井の材料の束を観点ごとに組む命令 folio ceiling --write（FR17 / AC15・凍結 anchor・ADR-8 決定 (2)(6)）

- 要件: FR17（天井の材料の束を観点ごとに組む・AI は起動しない）/ FR5（検査結果を必ず返す・3 値）
- 受入基準: AC15（観点 4 つの材料の束が凍結 fixture〔欠陥を 1 つ仕込んだ最小の正本〕から決定的に組め、束の要約値が期待と一致する）
- 条: P-3（床と天井の二層）/ P-4（実行できなかった検査を合格にしない）/ P-5.1（型の一覧は型付きデータに）/ P-10.1・P-10.2（生成側からも検査側からも独立した凍結 anchor・生成物どうしの突き合わせを唯一の合格判定にしない）/ N-5（型ごとの係を増やさない）/ A-1（鍵・課金を持つ操作をしない = AI を起動しない）
- 判断の記録: ADR-8 決定 (2)（folio は AI を起動せず、命令 1 つが観点ごとに材料の束を 1 つの置き場へ組む・束の中身 = 正本の写し・面の写し・問いの文・所見の欄の決まり・読む欄の一覧）・(6)（順序 = 天井の正本を床に〔便 37・着地 main d5e87c1〕→ 本便 → 所見の検査〔便 39〕→ 面の名札〔便 40〕）・帰結（凍結 anchor = 欠陥を 1 つ仕込んだ最小の正本と、そこから決定的に組める束の写し）。便 37（f2-648.53）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効と要件書 v1.6・f2-648.52 notes）。命令の名は本便で決める（ADR-8 決定 (2)「名は便で決める」）= folio ceiling。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 am が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

天井（AI による意味の検査）は、folio が AI を起動せず、観点ごとに「材料の束」を 1 つの置き場へ組むところまでを担う（ADR-8 決定 (2)・FR17）。束を読んで AI を回すのは席（planner 席）か器（scribe2）で、所見と起動の記録を同じ置き場へ書く（便 39 がそれを数える）。本便は命令 folio ceiling の --write（束を組む）だけを運ぶ。所見 file と起動の記録の検査（--check・3 値）は便 39、面の天井の名札は便 40。凍結 anchor（P-10.1）は、欠陥を 1 つ仕込んだ最小の正本の fixture と、そこから組める束の file の一覧と要約値で、planner が folio の code を使わずに手（shell）で組んで測った値を本節 (e) が凍結して持つ。

planner の実測（2026-09-19・main d5e87c1）: 天井の正本 design-intent/ceiling.yaml v0.1 は便 37 で床（folio check・正本 7 file）に入った。最上位の節は 8 つ（meta・verdicts・weights・documents・viewpoints・finding・record・bundle）。documents は 9 行（id と file・file は設計文書の置き場からの相対・adr と design-note は末尾が / の dir）。viewpoints は 4 行（id・name・reader・question・reads の各行は doc と fields）。bundle は contents（sources・faces・question・finding・reads の 5 つ）と digest（sha256-files-1）。床の実装側: crates/folio/src/ceiling.rs（正規化 438・天井の正本の形の検査・本便は触らない）・crates/folio/src/check.rs（正規化 540・正本の読み手 load〔symlink・無い・file でない・読めない・parse できない は「まだ分からない」の文言つき〕・本便は触らない）・crates/folio/src/site.rs（配信先の file の名 = index.html・constitution.html・srs.html・folio.css・folio-ui.js・adr-<数>.html・note-<文書 id>.html・本便は触らない）・crates/folio/src/sha256.rs（要約値 digest と 16 進 hex の関数・正規化 123・本便は触らない）・crates/folio/src/main.rs（正規化 436・命令の一覧 Command・余地 1,000 超）。rules 行 R-15 の要約値の規則（vendor/README.md）= 全 file を path の byte 順に並べ、その中身を区切りなしに連結した byte 列の sha256。要約値の名 sha256-files-1 は天井の正本 bundle.digest の逐語。

(a) 命令の口（crates/folio/src/main.rs の Command に 1 つ足す）: folio ceiling --dir <正本の置き場> --faces <配信先> --out <置き場> --write。--dir の既定は他の命令と同じ。--faces は folio build の --out（相対なら --dir からの相対・絶対ならそのまま）。--out は既定なし（相対なら --dir からの相対・絶対ならそのまま・build と同じ読み）。本便の旗は --write だけ（--check は便 39 が足す = 所見 file と起動の記録と束の要約値の検査）。終了は 組めた 0 / まだ分からない 2 で、本便の命令は 1（不合格）を返さない（束を組むだけで判定を持たない・FR5 の 3 値のうち 2 つ）。標準出力は 1 行「folio ceiling: 束を組んだ（観点 4・file <数>・<byte> byte）」（数は 4 観点の合計・digest.txt を含む）。標準エラーは 1 行「folio ceiling: まだ分からない: <理由>」（build と同じ形）。説明文（--help）は「天井の材料の束を観点ごとに置き場へ組む（--write）。AI は起動しない」の趣旨で、rules 行 R-1 の母集団（AI の係の説明文）には数えない。

(b) 読み（新しい module crates/folio/src/bundle.rs・命令の本体）: 天井の正本 <dir>/ceiling.yaml を check.rs の正本の読み手と同じ文言で読む（symlink・無い・file でない・読めない・parse できない・最上位が欄の表でない はどれも 2 で、無いときの文言は「ceiling.yaml: 正本が無い」）。読む欄は documents（各行の id と file）・viewpoints（各行の id・name・reader・question と reads の各行の doc・fields）・finding（required・optional・place の required・refute の values）・weights（values・refute）・verdicts（values）・record（required）・bundle（contents・digest）。欄が無い・型が違う（一覧でない・表でない・文でない）は 2「ceiling.yaml: <場所>: 読めない」（形の検査そのものは便 37 の床の領分なので本便は数えず、組めないことだけを返す）。bundle.contents が床の定数（sources・faces・question・finding・reads の 5 つ・順も同じ）と違う、または bundle.digest が sha256-files-1 でなければ 2「ceiling.yaml: bundle: 床が組める形でない」（束の形の正本は天井の正本だが、組む規則は本便の code が持つ = 食い違いは組まない・P-4.1）。reads の doc が documents の id に無ければ 2「ceiling.yaml: viewpoints[<id>].reads: 行き先「<doc>」が一覧に無い」。

(c) 束の中身（観点ごとに <out>/<観点の id>/・viewpoints の順に 4 つ）。中身は bundle.contents の 5 つと要約値の file で、以下の形に固定する（手で組んだ期待〔(e)〕と byte で一致させる）:
- sources/ = 正本の写し（基準）。その観点の reads の各行の doc を documents で file に解き、file 形（末尾が / でない）なら <dir>/<file> を sources/<file> へ byte のまま写す。dir 形（末尾が /）なら <dir>/<file> の直下の .yaml を名の byte 順に全部 sources/<file><名> へ写す（下の dir は見ない・schema.yaml も figures.yaml も除外しない = 直下の .yaml 全部）。同じ doc が reads に 2 度出ても写しは 1 度。写す元が無い（file が無い・dir に .yaml が 0 本）なら 2「<file>: 正本が無い」・symlink なら 2「<file>: symlink は認めない」。
- faces/ = 生成した面の写し。doc の id から面の file の名の形を床の定数の表で引く: index → index.html／constitution → constitution.html／srs → srs.html／adr → adr-<数>.html（adr- で始まり .html で終わる直下の file 全部）／design-note → note-<文書 id>.html（note- で始まり .html で終わる直下の file 全部）／rules・vocabulary・intake・ceiling → 面は無い（何も写さない）。当たる file を --faces の直下から名の byte 順に faces/<名> へ byte のまま写す。folio.css・folio-ui.js は写さない。形を持つ doc なのに当たる file が 0 本なら 2「<doc>: 面が無い（<faces>）」。--faces が dir でなければ 2「<faces>: 配信先が無い」。
- question.yaml = その観点の問いの文。6 行に固定: 1 行目「id: <id>」・2 行目「name: <name>」・3 行目「reader: |」・4 行目 2 字下げで reader の文・5 行目「question: |」・6 行目 2 字下げで question の文（文に改行が含まれれば各行を 2 字下げで並べる・末尾は改行 1 つ）。
- finding.yaml = 所見の欄の決まり。天井の正本の finding・weights・verdicts・record を次の型に流し込む（一覧は「[a, b, c]」= 半角の読点と空白で連結・行の順と字は下のとおり・末尾は改行 1 つ）:
  1 行目「# 所見の欄の決まり（天井の正本 ceiling.yaml の finding・weights・verdicts・record の写し・folio ceiling が組んだ）」
  「finding:」／「  required: [<finding.required>]」／「  optional: [<finding.optional>]」／「  place: {required: [<finding.place.required>]}」／「  refute: {values: [<finding.refute.values>]}」
  「weights:」／「  values: [<weights.values>]」／「  refute: [<weights.refute>]」
  「verdicts:」／「  values: [<verdicts.values>]」
  「record:」／「  required: [<record.required>]」
- reads.yaml = 読む欄の一覧。reads の各行を天井の正本の順に 1 行ずつ「- {doc: <doc>, fields: [<fields>]}」（fields は半角の読点と空白で連結・末尾は改行 1 つ）。
- digest.txt = 束の要約値 1 行「sha256-files-1 <16 進 64 字の小文字>」+ 改行。規則 = bundle.contents の 5 つの下の file（sources/ と faces/ の下の全 file・question.yaml・finding.yaml・reads.yaml）を観点の dir からの相対 path の byte 順（LC_ALL=C sort と同じ = path の文字列の byte 比較）に並べ、中身を区切りなしに連結した byte 列の sha256（rules 行 R-15 の写しの要約値と同じ規則・crates/folio/src/sha256.rs の関数で計算する）。digest.txt 自身は数えない。
- 決定性: 同じ入力から byte まで同じ束が組める（時刻・絶対 path・環境の値をどの file にも書かない）。
- 全部か無しか: 4 観点の全 file を memory の上で先に用意し（1 本でも用意できなければ何も書かず 2）、それから書く。書くときは各観点の dir の下の sources/ と faces/ を消してから作り直し、question.yaml・finding.yaml・reads.yaml・digest.txt を上書きする（古い写しが要約値に混ざらないため）。それ以外の file（席や器が書く所見 file・起動の記録・便 39 が読む）は触らない。<out> が無ければ作る（親 dir が無ければ 2「<out>: 置き場の親 dir が無い」・build と同じ）。

(d) 接続: check.rs・ceiling.rs（床）・site.rs・面の生成器・部品目録・readable.html（render）は触らない（天井の正本は変えないので導出物は変わらない）。要約値は crates/folio/src/sha256.rs を使い外部 crate を増やさない。main.rs は Command に Ceiling を 1 つ足し module bundle を宣言する（+30 行の見積・lib.rs は無い）。語彙・rules 行・天井の正本は変えない。

(e) 凍結 anchor（新しい dir tests/fixtures/ceiling/bundle/・P-10.1）。本便で新規 file として置く（新しい dir は本便の write-set の + の file で作る）。中身は次のとおりで、期待の値は planner が folio を使わず shell で組んで測った（生成器から独立・P-10.2）:
- source/ = 欠陥を 1 つ仕込んだ最小の正本。source/ceiling.yaml = design-intent/ceiling.yaml（main d5e87c1・v0.1）の byte の写し（以後 ceiling.yaml が変わっても fixture は変えない = 凍結）。source/constitution.yaml・source/rules.yaml・source/index.yaml = tests/fixtures/face/ の同じ名の file の byte の写し。source/adr/ADR-1.yaml・source/adr/ADR-2.yaml = tests/fixtures/face/adr/ の同じ名の写し。source/design-note/full.yaml = tests/fixtures/face/design-note/full.yaml の写し。source/srs.yaml = tests/fixtures/face/srs.yaml の写しに欠陥を 1 つ仕込む = 78 行目の FR2 の plain「    plain: 合格・不合格・まだ分からない のどれかを出します。」を「    plain: 合格か不合格のどちらかを出します。」に変える（他は 1 byte も変えない・shall の 3 値のうち「まだ分からない」を平易文が落としている = 忠実さの観点で拾われるべき欠陥・床は数えない・folio2 自身で回す 1 周の「見つける力」の物差し）。vocabulary.yaml・intake.yaml・intake-sheet.yaml は置かない（どの観点も読まない・本便の命令は ceiling.yaml と読む文書だけを読む）。
- faces/ = 面の写しの見本 7 本（中身は読まない・名で写す）。index.html・constitution.html・srs.html・adr-1.html・adr-2.html・note-full.html の 6 本は各 1 行で、中身は「<!doctype html><title>見本の面 <名>.html の <名> の部分</title><p>凍結 fixture の面の写し（便 38・中身は読まない）</p>」+ 改行 1 つ。ここで <名> は file の名から .html を除いた字（index・constitution・srs・adr-1・adr-2・note-full）で、たとえば index.html の中身は「<!doctype html><title>見本の面 index</title><p>凍結 fixture の面の写し（便 38・中身は読まない）</p>」+ 改行。faces/folio.css の中身は「body{color:#000}」+ 改行（写されないことの物差し）。
- 期待の束（fixture の source と faces から (c) の規則で組んだもの・planner の手組みの実測）。file の一覧（観点の dir からの相対・byte 順・digest.txt を含む）と要約値:
  fidelity（13 file + digest.txt）: faces/adr-1.html・faces/adr-2.html・faces/constitution.html・faces/note-full.html・faces/srs.html・finding.yaml・question.yaml・reads.yaml・sources/adr/ADR-1.yaml・sources/adr/ADR-2.yaml・sources/constitution.yaml・sources/design-note/full.yaml・sources/srs.yaml。連結 15,055 byte。digest.txt = sha256-files-1 2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32
  readability（15 file + digest.txt）: fidelity の一覧に faces/index.html と sources/index.yaml を足したもの。連結 17,317 byte。digest.txt = sha256-files-1 ded1548e1b6c5185b5b2ea0083d6ee68ed66dd5ca998bdd7d5d9a52ed2065576
  coherence（14 file + digest.txt）: fidelity の一覧に sources/rules.yaml を足したもの。連結 15,770 byte。digest.txt = sha256-files-1 91ca924dceef12d5b2df3946e6b56de69b1feeef13ea66f78d950af54c28476c
  reality（11 file + digest.txt）: faces/adr-1.html・faces/adr-2.html・faces/note-full.html・faces/srs.html・finding.yaml・question.yaml・reads.yaml・sources/adr/ADR-1.yaml・sources/adr/ADR-2.yaml・sources/design-note/full.yaml・sources/srs.yaml。連結 11,385 byte。digest.txt = sha256-files-1 24ce1b872fa08dd128b39ac42e4ba72096bfe263df3bb463c9616899c96786d7
  fidelity の question.yaml の逐語（6 行）: 「id: fidelity」「name: 忠実さ」「reader: |」「  元の文と平易文の両方を読み、意味の差だけを拾う編集者」「question: |」「  人が書いた自由文（やさしく言うと・平易文・平易な説明）は、元の文（規範文・要件の文・決定の文）の意味を保っているか。義務を足していないか、落としていないか。専門語の日本語の言い換えは元の語と同じものを指しているか。図の根拠が指す先は、図の中身と合っているか。判定できない箇所は「まだ分からない」と書く。」。fidelity の reads.yaml の逐語（4 行）: 「- {doc: constitution, fields: [articles.plain, articles.statements.text]}」「- {doc: srs, fields: [requirements.plain, requirements.shall, acceptance.plain, acceptance.title]}」「- {doc: adr, fields: [plain, decision, options.text, figures.refs]}」「- {doc: design-note, fields: [sections, figures.refs]}」。finding.yaml は 4 観点とも同じ（(c) の型に天井の正本 v0.1 の値: required は id, viewpoint, place, weight, evidence／optional は refute, note／place の required は doc, at／refute の values は 支持, 退けた, まだ分からない／weights の values は 止める, 直す, 参考・refute は 止める／verdicts の values は 合格, 不合格, まだ分からない／record の required は model, effort, at, read, bundle）。
  測り方（歯が同じ規則で測り直す・folio の code に依らない）: 観点の dir で sources と faces の下の全 file と question.yaml・finding.yaml・reads.yaml を LC_ALL=C sort の順に並べ、中身を連結して sha256sum に流す（便 30 の歯 tests/figure.rs が R-15 の要約値を sha256sum の子の処理で測るのと同じ形）。
- 要約値が本節の値と食い違ったら、それは (c) の規則か fixture の写しのどちらかが本節と違うということで、作業者は本節の逐語に合わせる（期待の値の側を変えない・変えるなら planner へ問う）。

(f) 歯（新しい file crates/folio/tests/bundle.rs・関数名は bundle を含める・--test bundle の scope・組み立てた binary を子の処理で撃つ）:
1. 凍結 anchor: fixture の source と faces から --out を一時 dir にして --write → 0 ∧ 標準出力に「観点 4」。4 観点それぞれで、file の一覧が (e) の一覧と一致し、digest.txt の中身が (e) の値と一致し、sha256sum で測り直した要約値も同じ。
2. 写しの byte: fidelity の sources/srs.yaml が fixture の source/srs.yaml と byte 一致（欠陥の行を含む）・faces/srs.html が fixture の faces/srs.html と一致・faces/ に folio.css が無い・readability の faces/ に index.html が在り reality の faces/ に無い・fidelity の question.yaml と reads.yaml が (e) の逐語と一致・finding.yaml が (e) の型と一致。
3. 決定性と全部か無しか: 同じ入力で 2 度組んで全 file が byte 一致。fidelity の dir に findings.yaml と record.yaml（中身は任意の 1 行）と sources/zzz.yaml を先に置いてから組み直すと、findings.yaml と record.yaml は残り sources/zzz.yaml は消えて digest.txt は 1 の値のまま。
4. まだ分からない: 写しの source から ceiling.yaml を消す → 2 ∧「ceiling.yaml: 正本が無い」／srs.yaml を消す → 2 ∧「srs.yaml: 正本が無い」∧ --out の下に file が 1 つも無い／faces から srs.html を消す → 2 ∧「srs: 面が無い」／--faces に無い dir → 2 ∧「配信先が無い」／--out の親 dir が無い → 2 ∧「親 dir が無い」／写しの ceiling.yaml の bundle.contents から reads を消す → 2 ∧「床が組める形でない」／reads の doc を documents に無い id（mystery）に → 2 ∧「行き先」∧「mystery」。
5. 実の正本: design-intent の写し（copy_tree・本便の前段 = folio build --out で配信先を作る）から組む → 0 ∧ 4 dir ∧ 各 digest.txt を sha256sum で測り直して一致 ∧ fidelity の sources/adr/ に .yaml が 9 本（ADR-1〜8 と schema.yaml）・sources/design-note/ に 3 本・faces/ に adr- の面 8 本と note- の面 2 本（note-example.html と note-figures.html）。
既存の歯 crates/folio/tests/ceiling.rs（便 37・天井の正本の形）は本便の write-set に載せるが本文は変えない（受付の歯の置き場の解決のため・verify に ceiling の回帰を 1 行）。

(g) 便 37 までの形との接続: 新規 file = crates/folio/src/bundle.rs（1 本）・crates/folio/tests/bundle.rs（1 本）・tests/fixtures/ceiling/bundle/ の 15 本（source 8・faces 7）。crates/folio/src/main.rs は (a)(d)。他の module と面の生成器・部品目録・design-intent は触らない。便 39 は本便の置き場の形（<out>/<観点>/ の 5 つ + digest.txt）を読み手の前提にし、席か器が書く所見 file と起動の記録の名を決める。

## 2. 範囲

- 入れる: 命令 folio ceiling --write・module bundle・凍結 anchor の fixture 15 本・歯・main.rs の口。
- 入れない: 所見 file と起動の記録の検査（--check・便 39）・面の天井の名札（便 40）・AI の起動（席か器の手番・A-1）・天井の正本の変更・語彙の追加・器（scribe2）への結線。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 口 | main.rs の Command Ceiling（--dir・--faces・--out・--write） |
| bundle | 束 | bundle.rs（読み・写し・問い・欄の決まり・読む欄・要約値・全部か無しか） |
| anchor | 凍結 anchor | tests/fixtures/ceiling/bundle/（source 8・faces 7）+ §1 (e) の一覧と要約値 |
| teeth | 歯 | tests/bundle.rs 5 群 + 既存 ceiling の期待不変 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない（要約値は crates/folio/src/sha256.rs）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "am"
title = "天井の材料の束を観点ごとに組む命令 folio ceiling --write（正本の写し・面の写し・問いの文・所見の欄の決まり・読む欄・要約値 sha256-files-1・全部か無しか・凍結 anchor）"
req = ["FR17", "FR5"]
section = "1"
write-set = ["+crates/folio/src/bundle.rs", "crates/folio/src/main.rs", "+crates/folio/tests/bundle.rs", "crates/folio/tests/ceiling.rs", "+tests/fixtures/ceiling/bundle/source/ceiling.yaml", "+tests/fixtures/ceiling/bundle/source/constitution.yaml", "+tests/fixtures/ceiling/bundle/source/rules.yaml", "+tests/fixtures/ceiling/bundle/source/srs.yaml", "+tests/fixtures/ceiling/bundle/source/index.yaml", "+tests/fixtures/ceiling/bundle/source/adr/ADR-1.yaml", "+tests/fixtures/ceiling/bundle/source/adr/ADR-2.yaml", "+tests/fixtures/ceiling/bundle/source/design-note/full.yaml", "+tests/fixtures/ceiling/bundle/faces/index.html", "+tests/fixtures/ceiling/bundle/faces/constitution.html", "+tests/fixtures/ceiling/bundle/faces/srs.html", "+tests/fixtures/ceiling/bundle/faces/adr-1.html", "+tests/fixtures/ceiling/bundle/faces/adr-2.html", "+tests/fixtures/ceiling/bundle/faces/note-full.html", "+tests/fixtures/ceiling/bundle/faces/folio.css"]
verify = ["cargo nextest run -p folio --test bundle bundle", "cargo nextest run -p folio --test ceiling ceiling", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "bundle の歯（凍結 anchor の 4 観点の一覧と要約値・写しの byte・決定性と全部か無しか・まだ分からない 7 種・実の正本）が緑、便 37 の歯 ceiling が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
