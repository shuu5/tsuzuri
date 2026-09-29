# 設計: 便 42 — 反証の材料の束を組む folio ceiling --refute と、--check が反証の結果 file を読む形（ADR-8 決定 (3)・FR17 / FR18）

- 要件: FR17（天井の材料の束を組む・AI は起動しない）/ FR18（所見の形と天井の 3 値を床で数える・止める所見の反証の 2 段目も同じ欄の決まりで受ける）/ FR5（3 値）
- 条: P-1（最終判断を代行しない = 反証の当否は AI・席は所見 file を書き換えない）/ P-4.1・P-4.2（実行できなかった検査を合格にしない・判定できないものは「まだ分からない」）/ P-10.1（凍結 anchor）/ N-5（観点も係も増やさない = 反証は観点の問いと同じ材料で回す）
- 判断の記録: ADR-8 決定 (3)（「止める」の重さの所見だけは別の文脈の AI が中立に反証する 2 段目を通す・反証の材料も folio が組む・反証の結果が「まだ分からない」なら所見は残す）・(4)（床が数える = 欄の決まりと要約値の一致）。便 41（f2-648.57）の後・1 周目（f2-648 notes 2026-09-19・席が所見と正本を手で反証役に渡し findings.yaml に refute を手で書き足した）の残りの是正。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効）。反証の規則の文は本便で床の定数として置く（天井の正本 v0.2 で正本へ移すかは持ち主の裁定 3〔撤退条件 ①〕の後）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 aq が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

1 周目では「止める」の所見 6 件の反証を、席が所見の逐語と正本の場所を手で反証役に渡し、返ってきた結果を席が findings.yaml の refute 欄に書き足す形で回した。ADR-8 決定 (3) は反証の材料も folio が組むと決めているので、本便は (1) folio ceiling --refute が「止める」の所見ごとに反証の材料の束を観点の束の下に組み、(2) --check が反証役の書く result.yaml を反証の結果として読む形にする。席も器も所見 file（審査役が書いた findings.yaml）を触らずに済む。反証役が読む正本は観点の束の sources/（既に在る）で、反証の束はそれを写さず親の要約値で縛る。

planner の実測（2026-09-19・main 01b3dcd）: crates/folio/src/findings.rs（正規化 759・余地 741）は所見 1 件の refute を欄の値から読み（値域の外は違反・止める で無ければ「反証が未」・退けた は残らない・支持 と まだ分からない は残る）、便 41 の規則 9 の場合分けを持つ。crates/folio/src/bundle.rs は load（天井の正本）・build_one（観点 1 つの束）・digest_text（要約値の 1 行）・read_dir_names を crate の中へ公開し、観点の束の question.yaml（6 行）・reads.yaml・digest.txt を組む。main.rs の Command Ceiling は --dir・--faces（必須）・--out・--write / --check（排他・必須）・--ceiling は face / build の側。凍結 fixture tests/fixtures/ceiling/findings/stop-unrefuted.yaml = 止める F-1 に refute が無い所見 file（verdict 合格・record は便 38 の凍結値）。歯 crates/folio/tests/findings.rs は 26 本。

(a) 命令の口（main.rs）: folio ceiling --dir <正本> --out <束の置き場> --refute。--refute は --write / --check と排他で 3 つのどれか 1 つが要る。--faces は任意（Option）にし、--write と --check で無ければ 2「--faces が要る」（文言は bundle.rs の Outcome の形）・--refute では読まない。終了は 組めた 0 / まだ分からない 2（判定を持たない）。標準出力 1 行「folio ceiling: 反証の束を組んだ（所見 <数>・file <数>・<byte> byte）」（対象の所見が 0 件でも 0 で「所見 0・file 0・0 byte」）。標準エラーは理由ごとに 1 行「folio ceiling: <観点>: <理由>」。

(b) 対象と置き場: 天井の正本の viewpoints の順に、<out>/<観点の id>/findings.yaml を便 39 の読み手で読み（所見 file が無い観点は飛ばす・読めない・欄の決まりの違反が在る観点は 2「<観点>: 所見 file: <理由>」で何も書かない）、重さが天井の正本 weights.refute のどれか（止める）で、findings.yaml に refute の欄が無く、かつ <out>/<観点>/refute/<所見の id>/result.yaml が無い所見を対象にする。対象ごとに dir <out>/<観点の id>/refute/<所見の id>/ を作り（既に在れば中の 5 つと digest.txt を書き直す・result.yaml が在る dir は触らない）、5 つの file と要約値を書く。全部か無しか（1 件でも組めなければ何も書かず 2）。

(c) 反証の束の中身（5 つ + digest.txt・手で組んだ期待〔(e)〕と byte で一致させる）:
- finding.yaml = 所見 1 件の逐語を固定の型に流し込む（所見 file の書き方の差を消す）: 「id: <id>」「viewpoint: <観点>」「place: {doc: <doc>, at: <at>}」「weight: <重さ>」「evidence: |」「  <evidence の文>」、note が在れば「note: |」「  <note の文>」（文に改行が含まれれば各行を 2 字下げ・無ければ note の 2 行を出さない・refute と他の欄は出さない・末尾は改行 1 つ）。
- question.yaml = 観点の束の question.yaml と同じ 6 行の後に 4 行を足す: 「refute:」「  values: [<天井の正本 finding.refute.values を半角の読点と半角の空白 , の 2 字で連結>]」「  rule: |」「    <反証の規則の文>」。反証の規則の文は床の定数 1 つ（逐語）: 所見を出した文脈から独立して中立に検証する。根拠が正本に逐語で在り、主張が正本の文から裏付けられれば 支持。根拠が無い、または主張が正本の文と両立しないと裏付けられれば 退けた。材料だけでは決められなければ まだ分からない（所見は残る）。
- reads.yaml = 観点の束の reads.yaml の byte の写し（反証役は親の sources/ を読む）。
- sources.txt = 観点の束の digest.txt の byte の写し（1 行・反証役が読む正本を親の要約値で縛る）。
- schema.yaml = 結果の欄の決まり 4 行: 「# 反証の結果の欄の決まり（folio ceiling --refute が組んだ・結果は同じ dir の result.yaml に書く）」「result:」「  required: [id, refute, model, effort, at, bundle]」「  values: [<天井の正本 finding.refute.values を同じ連結で>]」。
- digest.txt = 「sha256-files-1 <16 進 64 字>」+ 改行。規則は観点の束と同じ（5 つの file を名の byte 順 = finding.yaml・question.yaml・reads.yaml・schema.yaml・sources.txt に並べ、中身を区切りなしに連結した sha256・digest.txt と result.yaml は数えない）。
- 決定性: 時刻・絶対 path を書かない。同じ所見 file から 2 度組めば byte 一致。

(d) --check の読み（findings.rs・便 39 の規則 7 の前に足す）: 重さが weights.refute の所見ごとに <out>/<観点>/refute/<id>/result.yaml を見る。
- 在れば読む: 最上位は欄の表で、必須の欄は id・refute・model・effort・at・bundle（他の欄は違反・全部空でない文）。id は所見の id と同じ。refute は天井の正本 finding.refute.values のどれか。bundle は同じ dir の digest.txt の中身（末尾の改行を除く）と字まで同じで、digest.txt は dir の 5 つの file から測り直した要約値と同じ（違えば「反証の束が壊れている」）。どれかに当たれば その観点は まだ分からない「反証の結果 file: <id>: <理由>」（理由は全部列挙）。
- findings.yaml の所見に refute の欄も在るとき: 両方の値が同じなら可、違えば まだ分からない「反証の結果が 2 つ（<id>）」。
- result.yaml が正しく読めたら、その refute の値を所見の refute として規則 7〜10 に渡す（退けた は残らない・支持 と まだ分からない は残る・便 41 の再判定待ちも同じ）。result.yaml も refute の欄も無ければ従来どおり「反証が未（<id>）」。
- 標準出力の「止める <数>」は残る止めるの数（従来どおり）。反証の束の dir が在るが result.yaml が無い所見も「反証が未」。

(e) 凍結 anchor（新しい fixture は置かない・便 38 の束と便 39 の所見 file の凍結 fixture から決定的に組める・planner が folio を使わず shell で手組みして測った）: 便 38 の凍結 fixture の source / faces から --write で束を組み、fidelity に stop-unrefuted.yaml を写して --refute → <out>/fidelity/refute/F-1/ に 5 つ + digest.txt。file の一覧（byte 順）= finding.yaml・question.yaml・reads.yaml・schema.yaml・sources.txt。連結 1,888 byte。digest.txt = sha256-files-1 843ad558c1ee8cf7ea0f8b787f710bec0e4f7c58b7b1d2fab997e044b7b5f72d。逐語: finding.yaml は 8 行（「id: F-1」「viewpoint: fidelity」「place: {doc: srs, at: requirements.FR2.plain}」「weight: 止める」「evidence: |」「  合格か不合格のどちらかを出します。」「note: |」「  規範文の 3 値のうち「まだ分からない」を平易文が落としている。」）。question.yaml は 10 行（便 38 の設計文書 §1 (e) の fidelity の question.yaml の 6 行 + 「refute:」「  values: [支持, 退けた, まだ分からない]」「  rule: |」「    所見を出した文脈から独立して中立に検証する。根拠が正本に逐語で在り、主張が正本の文から裏付けられれば 支持。根拠が無い、または主張が正本の文と両立しないと裏付けられれば 退けた。材料だけでは決められなければ まだ分からない（所見は残る）。」）。reads.yaml は便 38 の (e) の fidelity の 4 行。sources.txt は「sha256-files-1 2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32」+ 改行。schema.yaml は (c) の 4 行（values は 支持, 退けた, まだ分からない）。測り方は便 38 と同じ（LC_ALL=C sort・cat・sha256sum）。要約値が食い違えば (c) の規則か写しが本節と違うということで、作業者は本節の逐語に合わせる（期待の値の側を変えない・変えるなら planner へ問う）。

(f) 歯（既存の file crates/folio/tests/findings.rs に足す・関数名は findings を含める・既存 26 本の期待は不変）:
1. 凍結 anchor: (e) のとおり組んで 0 ∧ 標準出力「所見 1・file 6」（5 つ + digest.txt）∧ file の一覧が (e) と一致 ∧ digest.txt が (e) の値 ∧ sha256sum で測り直して同じ ∧ finding.yaml と question.yaml が (e) の逐語と一致。2 度組んで byte 一致。
2. 対象なし: 4 観点 pass（止める 0）→ 0 ∧「所見 0・file 0・0 byte」∧ refute/ の dir を作らない。stop-upheld（refute 在り）→ 対象外で 所見 0。
3. result.yaml の読み: 歯 1 の後に result.yaml（id: F-1・refute: 退けた・model: opus・effort: default・at: 2026-09-19T08:00:00Z・bundle: digest.txt の中身）を置いて --check → 0（stop-unrefuted の verdict は 合格 で、退けた は残らないので規則 10 で残る止めるが 0）∧「fidelity: 合格（所見 1・止める 0）」。refute: 支持 に → 1 ∧「止める所見が残っている（F-1）」。refute: まだ分からない に → 1。findings.yaml の verdict を 不合格 に変えて result.yaml が 退けた → 2 ∧「再判定待ち・F-1」（便 41 の規則）。
4. result.yaml の違反: bundle を別の字に → 2 ∧「反証の結果 file: F-1」∧「bundle」／refute を値域の外（大変）に → 2／id を F-9 に → 2／必須の欄 at を消す → 2／extra の欄を足す → 2／反証の束の finding.yaml の 1 byte を書き換える → 2 ∧「反証の束が壊れている」。
5. 2 つの結果: findings.yaml の F-1 に refute: 支持 を書き足し、result.yaml は 退けた → 2 ∧「反証の結果が 2 つ（F-1）」。両方 支持 → 1（同じ値は可）。
6. 旗: --refute と --write の両方 → 終了 2／--write で --faces なし → 2 ∧「--faces が要る」／--refute で --faces なし → 0。
7. 実の正本: design-intent の写しから build → --write → fidelity に止める 1 件（evidence は sources/srs.yaml の逐語・refute なし）の所見 file を歯の中で作る → --refute → 0 ∧ 所見 1 ∧ digest.txt を sha256sum で測り直して一致 → result.yaml（退けた）を置いて --check → 規則 10 で 0。
既存の歯 crates/folio/tests/bundle.rs（便 38・11 本）は --faces を任意にしても期待不変（全部 --faces 付きで撃つ・verify に回帰を 1 行）。

(g) 便 41 までの形との接続: 新規 file は無い。既存 = findings.rs（--refute の本体と (d) の読み）・main.rs（旗）・tests/findings.rs（+7 群）・tests/bundle.rs（本文不変・回帰）。bundle.rs は question_text / reads の写しを組むために公開する関数が 1 つ増えるだけ（振る舞い不変・write-set に載せる）。面の名札（便 40）は findings.rs の観点ごとの結果の関数を呼ぶので、result.yaml を読む変更がそのまま名札に効く（字面は変えない）。天井の正本・語彙・rules は変えない。手順の側: 1 周 = build → --write → 審査役 → --refute → 反証役（result.yaml を書く）→ --check → 再判定（便 41）→ build --ceiling。

## 2. 範囲

- 入れる: --refute の旗と反証の束（5 つ + 要約値）・--check の result.yaml の読み・--faces を任意に・凍結 anchor の一致の歯・歯 7 群。
- 入れない: AI の起動（席か器の手番）・天井の正本の変更（反証の規則の文は床の定数）・面の名札の字面の変更・所見 file の書き換え（席も床もしない・P-1）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 口 | main.rs の --refute（--write / --check と排他）・--faces を任意に |
| refute | 反証の束 | findings.rs（対象の所見・5 つの file・要約値・全部か無しか） |
| result | 結果の読み | findings.rs の --check（result.yaml の欄の決まり・要約値・2 つの結果） |
| anchor | 凍結 | (e) の逐語と要約値 843ad558…（新規 fixture なし） |
| teeth | 歯 | tests/findings.rs +7 群・既存 26 本と bundle 11 本は期待不変 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "aq"
title = "反証の材料の束を組む folio ceiling --refute（止める の所見ごとに finding / question / reads / sources.txt / schema + 要約値）と、--check が反証役の result.yaml を反証の結果として読む形（欄の決まり・要約値・2 つの結果・--faces を任意に）"
req = ["FR17", "FR18", "FR5"]
section = "1"
write-set = ["crates/folio/src/findings.rs", "crates/folio/src/bundle.rs", "crates/folio/src/main.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/bundle.rs"]
verify = ["cargo nextest run -p folio --test findings findings", "cargo nextest run -p folio --test bundle bundle", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "findings の歯（凍結 anchor・対象なし・result の読み 3 通り・違反 6 種・2 つの結果・旗 3 通り・実の正本・既存 26 本は期待不変）が緑、便 38 の歯 bundle が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
