# 設計: 便 37 — 天井の正本 ceiling.yaml を folio check の床に入れる（正本 7 file・観点 4 つと所見の欄の決まりの形・FR18 / FR5・ADR-8 決定 (1)(3)(4)(6)）

- 要件: FR18（所見の形と天井の 3 値を床で数える。本便はその前段 = 天井の正本の形）/ FR5（検査結果を必ず返す・3 値）
- 条: P-3（床と天井の二層）/ P-4（実行できなかった検査を合格にしない）/ P-5.1（規則・型の一覧は型付きデータに）/ N-5（型ごとの係と観点を増やさない = 観点は 4 つの閉じた一覧）
- 判断の記録: ADR-8 決定 (1)（観点 4 つ・型に依らない）・(3)（所見の欄の決まり・起動の記録）・(4)（床が数えるもの）・(6)（順序 = 天井の正本を床に入れる便が最初）・帰結（床が見る正本を 1 file 広げる・判断の記録 ADR-6 の intake.yaml と同じ運び方・凍結 fixture の期待は変えない）。便 36（f2-648.51）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効と要件書 v1.6・f2-648.52 notes）。天井の正本 v0.1 は planner が PR で main に置く（本便の write-set の外・作業者は 1 字も変えない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 al が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

天井（AI による意味の検査）の観点・所見の欄の決まり・起動の記録の欄・重さと 3 値の値域・文書の一覧・材料の束の要約値の規則は、判断の記録 ADR-8 決定 (1)(3)(4) により新しい正本 design-intent/ceiling.yaml（v0.1・planner が main に置いた・本便の write-set の外）が持つ。本便は folio check がこの 7 本目の正本を読み、形を数えるようにする。便 18（相談窓口の正本 intake.yaml を 6 本目として床に入れた便・docs/design/delivery-18.md）と同じ作りで運ぶ。材料の束を組む命令・所見 file の検査・面の名札は後続の便で、本便は正本の形の床だけを持つ。

planner の実測（2026-09-19・main に置いた ceiling.yaml v0.1 を手で当てた）: 最上位の節は 8 つ（meta・verdicts・weights・documents・viewpoints・finding・record・bundle）。meta は id・title・version・status・generated・approval。verdicts.values は 3 つ（合格・不合格・まだ分からない の順）。weights.values は 3 つ（止める・直す・参考）で weights.refute は 1 つ（止める）。documents は 9 行（id は constitution・rules・vocabulary・srs・index・intake・adr・design-note・ceiling の順・各行 file あり・note は 9 行全部にある）。viewpoints は 4 行（id は fidelity・readability・coherence・reality の順・各行 name・reader・question・reads あり・reads の各行は doc と fields・doc は documents の id に全部解ける・fields は空でない一覧）。finding は required 5 つ（id・viewpoint・place・weight・evidence）・optional 2 つ・place.required 2 つ（doc・at）・refute.values 3 つ（支持・退けた・まだ分からない）。record.required は 5 つ（model・effort・at・read・bundle）。bundle は contents 5 つと digest 1 つ。行 id の重複は documents・viewpoints とも 0。文を持つ欄（meta.title・viewpoints の name・reader・question・documents の note）の英字の語は code・AI・id の類で、便 4 の語彙の検査（語彙 52 語 = 便 36 の後に planner が足した 観点・所見・反証・材料の束・天井の名札 を含む・識別子の一覧）に全部解ける。床の実装側: crates/folio/src/check.rs（正規化 536・余地 964）の正本の一覧 FILES は 6 本の定数（行 27〜34）で、関数 load_all（行 129〜）が同じ読み手で読み roots.pop で取り出す。crates/folio/src/intake.rs（正規化 239）が便 18 の型（節の閉じた一覧の定数・非空・行 id の重複・行き先の解決・語彙・型違いは「まだ分からない」）。crates/folio/src/main.rs（正規化 435・余地 1,065）の check の説明は「正本 5 file」（行 51・古い字）。tests/fixtures/ の下に intake.yaml を持つ組は 18（adr 3・anchor 2・check 4・face 1・link 3・refs 3・vocab 2）。crates/folio/tests/intake.rs（正規化 356）が便 18 の歯の型（写しの Work・canonical・missing・unknown-section・empty・行き先・上限・重複・語彙・型違い・重複キー）。

(a) 読み（crates/folio/src/check.rs）。正本の一覧 FILES を 6 本から 7 本（末尾に ceiling）にし、ceiling.yaml を他の 6 本と同じ読み手（load）で読む。したがって ceiling.yaml が symlink・無い・file でない・読めない・parse できない・最上位が欄の表でない はどれも他と同じ文言の形で「まだ分からない」（終了 2）になり、無いときの文言は「ceiling.yaml: 正本が無い」。重複キーは他と同じ種別「重複キー」の違反。Sources に ceiling を足し、check_dir の検査の並びで相談窓口の検査の直後に (b) を呼ぶ（引数は天井の正本・語彙の正本・報告）。

(b) 天井の正本の検査（新しい module crates/folio/src/ceiling.rs・関数 1 つ check_ceiling）。便 18 の intake.rs と同じ部品（check.rs の non_empty・duplicate_ids・unknown_sections・rows・row_id・便 4 の vocab.rs の既知の語の集合と免除されない語を並べる関数）を使い、式・文言・種別の形を揃える。数えるもの:
- 節の閉じた一覧（床の定数 CEILING_TOP_LEVEL）= meta・verdicts・weights・documents・viewpoints・finding・record・bundle の 8 つ。これに無い最上位の節は種別「未知の節」の違反（文言「ceiling.yaml: 未知の節「名」」）。
- 欄の非空（既存の非空の関数・同じ種別と文言の形）: meta の id・title・version・status／verdicts の values／weights の values・refute／documents の各行の id・file／viewpoints の各行の id・name・reader・question・reads／viewpoints.reads の各行の doc・fields／finding の required・place・refute／finding.place の required／finding.refute の values／record の required／bundle の contents・digest。documents の note・finding の optional・meta の generated・approval は数えない。
- 行 id の重複（既存の重複の関数・種別「重複キー」）: documents の中・viewpoints の中。
- 固定の一覧（種別「ceiling」の違反・文言「ceiling.yaml: 場所: 一覧「値」が床の定数と違う」）: verdicts.values は床の定数 3 つ（合格・不合格・まだ分からない）と順まで同じ（要件書 FR5 の 3 値）／viewpoints の id の列は床の定数 4 つ（fidelity・readability・coherence・reality）と順まで同じ（ADR-8 決定 (1)・増減はどちらも違反）／documents の id の集合は床の定数 9 つ（FILES の 7 本 + adr + design-note）と同じ（増減はどちらも違反）／finding.required は床の定数 5 つ（id・viewpoint・place・weight・evidence）を全部含む／finding.place.required は doc・at を全部含む／record.required は床の定数 5 つ（model・effort・at・read・bundle）を全部含む／finding.refute.values は 3 つ（支持・退けた・まだ分からない）と同じ／bundle.contents は sources・faces・question・finding・reads を全部含む。
- 行き先の解決（種別「ceiling」の違反・文言「ceiling.yaml: 場所: 行き先「値」が一覧に無い」）: viewpoints.reads の各行の doc は documents の id のどれか／weights.refute の各要素は weights.values のどれか。
- 語彙（天井の正本の文の中の英字の語）: meta の title／viewpoints の name・reader・question／documents の note を、便 4 の語彙の検査と同じ関数に通し、免除されない語 1 つごとに種別「ceiling」の違反（文言「ceiling.yaml 場所: 語彙に無い英字の語「語」」）。rules 行 R-9 の母集団（憲法・rules・要件書）は変えない（入口・相談窓口・判断の記録と同じ持ち方）。fields の値と finding / record の欄の名は機械の層なので語彙に通さない。
- 型が違う（節が欄の表でない・行の一覧が表の一覧でない・values や refute や required や contents や fields が一覧でない）は「まだ分からない」（他の正本と同じ文言の形）。
正本が新しい検査を 0 件で通ることの実測は上の planner の実測のとおり（8 節・非空 0・重複 0・固定の一覧 8 つとも一致・行き先 全部解ける・語彙の免除されない語 0）。

(c) 既存の歯の fixture。tests/fixtures/ の下の 18 組（adr/effective-no-approval・adr/schema-drift・adr/two-adopted・anchor/no-anchor・anchor/root-digest-drift・check/dup-key・check/empty-field・check/missing-file・check/unknown-section・face・link/adr-id-missing・link/amended-by-orphan・link/retreat-kind-drift・refs/bad-counts・refs/dangling-id・refs/orphan-rule・vocab/exemptions・vocab/unknown-word）に、同じ中身の最小の ceiling.yaml を 1 本ずつ足す（新しい dir は作らない）。足さないと (a) により全部に「まだ分からない」が 1 件増えて期待が変わる。足した後、既存の歯の期待（終了コード・件数・文言）は 1 つも変えない＝既存の歯の file は 1 字も変えない。check/missing-file にも足す（その組が「まだ分からない」になる理由を憲法が無いことだけに保つ）。最小の ceiling.yaml の中身（(b) の固定の一覧を全部満たす最小 = 8 節・観点 4 行・文書 9 行・欄の名は実の正本と同じ・文は 1 語ずつ）:

```
meta: {id: fixture-ceiling, title: 天井, version: v0.1, status: draft}
verdicts: {values: [合格, 不合格, まだ分からない]}
weights: {values: [止める, 直す, 参考], refute: [止める]}
documents:
  - {id: constitution, file: constitution.yaml}
  - {id: rules, file: rules.yaml}
  - {id: vocabulary, file: vocabulary.yaml}
  - {id: srs, file: srs.yaml}
  - {id: index, file: index.yaml}
  - {id: intake, file: intake.yaml}
  - {id: adr, file: adr/}
  - {id: design-note, file: design-note/}
  - {id: ceiling, file: ceiling.yaml}
viewpoints:
  - {id: fidelity, name: 忠実さ, reader: 編集者, question: 意味を保っているか。, reads: [{doc: srs, fields: [requirements.plain]}]}
  - {id: readability, name: 読みやすさ, reader: 非エンジニア, question: 読めるか。, reads: [{doc: index, fields: [shelf]}]}
  - {id: coherence, name: 文書どうしの整合, reader: 審査役, question: 矛盾は無いか。, reads: [{doc: constitution, fields: [articles]}]}
  - {id: reality, name: 実態との整合, reader: 実装者, question: 食い違いは無いか。, reads: [{doc: srs, fields: [requirements]}]}
finding: {required: [id, viewpoint, place, weight, evidence], optional: [refute, note], place: {required: [doc, at]}, refute: {values: [支持, 退けた, まだ分からない]}}
record: {required: [model, effort, at, read, bundle]}
bundle: {contents: [sources, faces, question, finding, reads], digest: sha256-files-1}
```

(d) 説明の字（crates/folio/src/main.rs）。check の説明「正本 5 file」を「正本 7 file」に直す（1 行）。

(e) 歯（新しい file crates/folio/tests/ceiling.rs・関数名は ceiling を含める・--test ceiling の scope・便 18 の tests/intake.rs と同じ形 = design-intent の写し〔copy_tree + git init・contracts/schema.toml の写し〕を作り folio check を撃つ）:
1. 実の正本の写しそのまま → 合格（違反 0・まだ分からない 0）。
2. 写しの ceiling.yaml を消す → 2 ∧ 標準エラーに「ceiling.yaml: 正本が無い」。
3. 最上位に extra を足す → 不合格 1 ∧「未知の節「extra」」。
4. viewpoints の 1 行の question を空に → 不合格 1 ∧ 非空の文言 ∧「question」。
5. viewpoints の 1 行の id を別の字（mystery）に → 不合格 1 ∧「一覧」∧「mystery」／viewpoints を 3 行に → 不合格 1 ∧「一覧」。
6. verdicts.values の順を入れ替える → 不合格 1 ∧「一覧」∧「verdicts」。
7. documents の 1 行を消す → 不合格 1 ∧「一覧」∧「documents」／reads の doc を documents に無い id に → 不合格 1 ∧「行き先」。
8. weights.refute に values に無い字を足す → 不合格 1 ∧「行き先」∧「refute」。
9. finding.required から evidence を消す → 不合格 1 ∧「一覧」∧「evidence」／record.required から bundle を消す → 不合格 1 ∧「一覧」∧「bundle」。
10. documents の 2 行を同じ id に → 不合格 1 ∧「重複キー」。
11. viewpoints の question に語彙に無い英字の語（zork）を足す → 不合格 1 ∧「語彙に無い英字の語「zork」」／語彙に在る語（yaml）を足す → 合格。
12. viewpoints を表の一覧でなく字にする → 2 ∧ 型違いの文言。
13. ceiling.yaml に同じキーを 2 度書く → 不合格 1 ∧「重複キー」。
既存の歯（crates/folio/tests/check.rs・refs.rs・vocab.rs・adr.rs・link.rs・anchor.rs・intake.rs）は (c) の fixture の追加で期待不変のまま緑（本文は 1 字も変えない・verify の scope に入れる）。

(f) 便 36 までの形との接続: 新規 file = crates/folio/src/ceiling.rs（1 本）・crates/folio/tests/ceiling.rs（1 本）・tests/fixtures/ の 18 組の ceiling.yaml（18 本）。crates/folio/src/check.rs は (a)（+15・余地 964）。crates/folio/src/main.rs は (d)（+0・1 行の字・余地 1,065）。他の module（intake.rs・entrance.rs・note.rs・refs.rs・vocab.rs・adr.rs・link.rs・anchor.rs）と面の生成器・部品目録は触らない。readable.html（render）は要件書の節を名指しで読むので天井の正本を足しても導出物は変わらない。design-intent の写しを copy_tree で作る既存の歯（note.rs・adr.rs の図の歯・face.rs・site.rs 等）は実の ceiling.yaml がそのまま写るので追従が要らない。

## 2. 範囲

- 入れる: 正本の一覧 7 本・天井の正本の形の床（節・非空・重複・固定の一覧・行き先・語彙・型違い）・歯・18 組の最小の ceiling.yaml・説明の字。
- 入れない: 材料の束を組む命令（便 38）・所見 file と起動の記録の検査（便 39）・面の天井の名札（便 40）・語彙の追加（planner PR で済み）・天井の正本の中身の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| read | 読み | FILES 7 本・load_all |
| shape | 形 | ceiling.rs の check_ceiling |
| fixtures | 写し | 18 組の最小の ceiling.yaml |
| teeth | 歯 | tests/ceiling.rs 13 本 + 既存 7 本の期待不変 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "al"
title = "天井の正本 ceiling.yaml を folio check の床に入れる（正本 7 file・節の閉じた一覧・観点 4 つと 3 値と文書 9 つの固定の一覧・所見と記録の必須欄・行き先・語彙）"
req = ["FR18", "FR5"]
section = "1"
write-set = ["+crates/folio/src/ceiling.rs", "+crates/folio/tests/ceiling.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs", "crates/folio/tests/intake.rs", "+tests/fixtures/adr/effective-no-approval/ceiling.yaml", "+tests/fixtures/adr/schema-drift/ceiling.yaml", "+tests/fixtures/adr/two-adopted/ceiling.yaml", "+tests/fixtures/anchor/no-anchor/ceiling.yaml", "+tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "+tests/fixtures/check/dup-key/ceiling.yaml", "+tests/fixtures/check/empty-field/ceiling.yaml", "+tests/fixtures/check/missing-file/ceiling.yaml", "+tests/fixtures/check/unknown-section/ceiling.yaml", "+tests/fixtures/face/ceiling.yaml", "+tests/fixtures/link/adr-id-missing/ceiling.yaml", "+tests/fixtures/link/amended-by-orphan/ceiling.yaml", "+tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "+tests/fixtures/refs/bad-counts/ceiling.yaml", "+tests/fixtures/refs/dangling-id/ceiling.yaml", "+tests/fixtures/refs/orphan-rule/ceiling.yaml", "+tests/fixtures/vocab/exemptions/ceiling.yaml", "+tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio --test ceiling ceiling", "cargo nextest run -p folio --test check check", "cargo nextest run -p folio --test refs refs", "cargo nextest run -p folio --test vocab vocab", "cargo nextest run -p folio --test adr adr", "cargo nextest run -p folio --test link link", "cargo nextest run -p folio --test anchor anchor", "cargo nextest run -p folio --test intake intake", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "ceiling の歯（写しそのまま・無し・未知の節・空の欄・固定の一覧 5 種・行き先 2 種・行 id の重複・語彙 2 つ・型違い・重複キー）が緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、便 7 の歯 anchor が期待不変で緑、便 18 の歯 intake が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
