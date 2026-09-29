# 設計: 便 39 — 所見 file と起動の記録を数える folio ceiling --check（観点ごとの 3 値・FR18 / AC16・ADR-8 決定 (3)(4)(5)）

- 要件: FR18（所見の形と天井の 3 値を床で数える・所見の意味の当否は判定しない）/ FR5（検査結果を必ず返す・3 値）
- 受入基準: AC16（所見 file の必須欄が欠けるか束の要約値が合わないと「まだ分からない」になり、その観点があると天井は合格にならない）
- 条: P-1（最終判断を代行しない）/ P-3.3（床の合格を天井の合格として扱わない）/ P-4（実行できなかった検査を合格にしない・判定できないものは「まだ分からない」）/ P-5.1（値域は型付きデータに = 天井の正本）/ P-10.1（凍結 anchor）/ P-11（同じ失敗が続いたら止めて持ち主へ = 本便は数えるだけで止める仕組みは持たない）
- 判断の記録: ADR-8 決定 (3)（所見 = 型付き file・観点の 3 値・合格の観点にも必須の欄〔読んだ欄・束の要約値・起動の記録〕・欠けか要約値の不一致は床が「まだ分からない」に落とす・「止める」だけ反証の 2 段目を通し、反証が「まだ分からない」なら所見は残す）・(4)（床が数える = 欄の決まりと 3 値・観点の一覧の一致・根拠の逐語の実在・「まだ分からない」の観点が 1 つでもあれば合格にしない・--check は 4 観点が全部合格のときだけ 0）・(6)（順序 = 便 38〔着地 main bad1c9f〕→ 本便 → 面の名札〔便 40〕）。便 38（f2-648.54）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効と要件書 v1.6・f2-648.52 notes）。所見 file の名と外形（本文の最上位の欄）は本便で床の定数として決める（天井の正本は変えない・理由は §1 の冒頭）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 an が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

便 38 で folio は観点ごとの材料の束を <out>/<観点の id>/ に組むようになった（sources/・faces/・question.yaml・finding.yaml・reads.yaml・digest.txt）。席か器が束を読んで AI を回し、所見 file を同じ dir へ書く。本便は folio ceiling --check がその所見 file を天井の正本の欄の決まりで数え、観点ごとに 合格・不合格・まだ分からない を返すようにする（FR18）。数えるのは形・実在・一致だけで、所見の中身が正しいかは判定しない（P-1）。所見 file の名（findings.yaml）と外形（最上位の欄 verdict・record・findings）は床の定数で持ち、天井の正本 ceiling.yaml は変えない。理由: 欄ごとの決まり（必須欄・値域）は天井の正本が既に持ち、外形は命令の側の約束であること・便 37 の床が最上位の節を 8 つに閉じているので正本に節を足すと床と凍結 fixture（便 38 の source/ceiling.yaml は v0.1 の写し）の両方を動かすことになること。

planner の実測（2026-09-19・main bad1c9f）: crates/folio/src/bundle.rs（正規化 503・便 38）が天井の正本の読み load・束の組み立て build_all・要約値 digest_text・書き write_all を持つ（どれも module の内側の関数）。束の要約値の規則 = 5 つの下の file を観点の dir からの相対 path の byte 順に連結した sha256（sha256-files-1）で、digest.txt の 1 行は「sha256-files-1 <16 進 64 字>」。crates/folio/src/ceiling.rs（便 37）は床の定数 VERDICT_VALUES（合格・不合格・まだ分からない）・VIEWPOINT_IDS（4 つ）・DOCUMENT_IDS（9 つ）・FINDING_REQUIRED（id・viewpoint・place・weight・evidence）・PLACE_REQUIRED（doc・at）・REFUTE_VALUES（支持・退けた・まだ分からない）・RECORD_REQUIRED（model・effort・at・read・bundle）を pub で持つ。天井の正本 v0.1 の値: finding.optional = refute・note／weights.values = 止める・直す・参考／weights.refute = 止める。crates/folio/src/verdict.rs の Verdict と Report（読めない → まだ分からない／違反 → 不合格／測れない → まだ分からない の並び）。crates/folio/src/main.rs（正規化 468）の Command Ceiling は --dir・--faces・--out・--write。実の正本で束を組むと 観点 4・file 121・3,735,167 byte で、凍結 fixture（tests/fixtures/ceiling/bundle/）から組むと 4 観点の digest.txt は便 38 の設計文書 §1 (e) の値（fidelity 2bd676…／readability ded154…／coherence 91ca92…／reality 24ce1b…）。歯 crates/folio/tests/bundle.rs（11 本）。

(a) 命令の口（main.rs の Command Ceiling に旗を 1 つ足す）: folio ceiling --dir <正本の置き場> --faces <配信先> --out <置き場> --check。--write と --check はどちらか 1 つが要る（両方・どちらも無し は命令の使い方の誤り = clap の断り・終了 2）。--faces は --check にも要る（束が古くないかを現在の正本から組み直して測るため）。終了は 4 観点が全部 合格 のときだけ 0、まだ分からない の観点が 1 つでも在れば 2、無くて 不合格 の観点が在れば 1（Report の並びと同じ = 読めない・測れない が違反より先に立つ・ADR-8 決定 (4)・P-4）。標準出力は観点ごとに 1 行「<観点の id>: <3 値>（所見 <数>・止める <数>）」を天井の正本の viewpoints の順に出し、最後に 1 行「folio ceiling: <3 値>（観点 4・合格 <数>・不合格 <数>・まだ分からない <数>）」。標準エラーは理由ごとに 1 行「folio ceiling: <観点の id>: <理由>」（1 観点に理由が複数あれば全部出す・最初の 1 件で止めない・folio check の違反の並びと同じ形）。天井の正本が読めない・束の置き場の親が無い・--faces が無い は観点に入る前の「まだ分からない」で、bundle.rs の --write と同じ文言を 1 行出して 2。

(b) 所見 file（床の定数 FINDINGS_FILE = findings.yaml・置き場は <out>/<観点の id>/findings.yaml・席か器が書く・folio は書かない）。外形は最上位の欄 3 つに閉じる（床の定数・verdict・record・findings・順は問わない・他の欄は違反）:
- verdict = その観点の 3 値（天井の正本 verdicts.values のどれか・床の定数 VERDICT_VALUES と同じ）。
- record = 起動の記録（欄の表）。必須の欄は天井の正本 record.required（model・effort・at・read・bundle・床の定数 RECORD_REQUIRED）で、他の欄は違反。model・effort・at は空でない文（値の意味は数えない = 時計は持たない・P-1）。read = 読んだ文書の id の一覧（空でない・各要素はその観点の reads.yaml に在る doc・reads.yaml の doc が全部在る = 集合として一致）。bundle = 読んだ束の要約値の 1 行で、その観点の digest.txt の中身（末尾の改行を除く）と字まで同じ。
- findings = 所見の一覧（空でよい）。所見 1 件の欄は天井の正本 finding.required（id・viewpoint・place・weight・evidence）と finding.optional（refute・note）に閉じる（他の欄は違反・必須は全部在って空でない）。id は file の中で一意。viewpoint はその dir の観点の id と同じ。place は欄の表で place.required（doc・at）を持ち、doc はその観点の reads.yaml に在る doc（読んでいない文書への所見は違反）、at は空でない文（欄か節か行の id・解決はしない）。weight は天井の正本 weights.values のどれか。evidence は空でない文で、その観点の束の sources/ の下のどれかの file の中身に逐語で（byte 列の部分一致・改行を跨がない）在ること（根拠の実在・ADR-8 決定 (4)・無ければ「根拠が正本に無い」）。refute は在れば天井の正本 finding.refute.values のどれか（床の定数 REFUTE_VALUES）。note は自由。
- 型が違う（最上位が欄の表でない・record が表でない・findings が一覧でない・read が一覧でない・所見の行が表でない）・重複キー・parse できない は「まだ分からない」で、文言は他の正本の読み手と同じ形（file 名: 理由）。

(c) 観点ごとの 3 値（天井の正本 viewpoints の順に 1 観点ずつ・理由は全部集めてから決める）:
1. 束が無い: <out>/<観点の id>/ が無い、または 5 つの中身のどれか（sources/・faces/・question.yaml・finding.yaml・reads.yaml）か digest.txt が無い → まだ分からない「束が無い」。
2. 束が壊れている: 置き場の file から測り直した要約値（--write と同じ規則・digest.txt 自身は数えない）が digest.txt と違う → まだ分からない「束が壊れている（要約値が digest.txt と違う）」。
3. 束が古い: --dir と --faces から --write と同じ規則で memory の上に組み直した束の要約値が digest.txt と違う → まだ分からない「束が古い（現在の正本から組んだ要約値と違う）」。組み直せない（正本が無い・面が無い）ときは --write と同じ文言で まだ分からない。
4. 所見 file が無い・読めない・型が違う → まだ分からない「所見 file が無い」／「所見 file: <理由>」。
5. (b) の欄の決まりの違反（未知の欄・必須の欠け・値域の外・viewpoint の不一致・read の集合の不一致・doc が reads に無い・id の重複・根拠が正本に無い）→ まだ分からない（理由を全部列挙・形を守らない返事は数えられない = AC16「必須欄が欠けると まだ分からない」）。
6. record.bundle が digest.txt と違う → まだ分からない「要約値が束と合わない（読んだ束が違う）」（AC16）。
7. 止める の反証: weight が天井の正本 weights.refute に在る所見（止める）に refute が無い → まだ分からない「反証が未（<id>）」（2 段目を通っていない・ADR-8 決定 (3)）。refute = 退けた の所見は「残らない」。refute = 支持 か まだ分からない の所見は「残る」（反証が「まだ分からない」なら所見は残す）。
8. verdict = まだ分からない → まだ分からない「AI が判定できなかった」。
9. verdict = 不合格 → 所見が 1 件以上在れば 不合格。所見が 0 件なら まだ分からない「不合格なのに所見が無い」。
10. verdict = 合格 → 残る 止める が 1 件でも在れば 不合格「止める所見が残っている（<id>）」（file の verdict より所見が先に立つ）。無ければ 合格（直す・参考 の所見は残っていてよい・数は標準出力に出す）。
1〜6 のどれかに当たれば その観点は まだ分からない で、7〜10 は見ない（理由は 1〜6 の分を全部出す）。観点の一覧の一致 = 天井の正本の viewpoints にある観点だけを見る（置き場に余分の dir が在っても数えない）。

(d) 接続: 新しい module crates/folio/src/findings.rs に --check の本体（所見 file の読み・(b) の欄の決まり・(c) の 3 値・出力の組み立て）を置く。bundle.rs は load・build_all・要約値の計算・digest.txt の名を module の外（crate の中）から呼べるようにするだけで、--write の振る舞いと文言は変えない（便 38 の歯が期待不変で緑）。main.rs は Command Ceiling に --check を足し、--write と --check を clap の ArgGroup で「どちらか 1 つ・必須」にする。ceiling.rs（床の定数）・check.rs・site.rs・面の生成器・部品目録・design-intent（天井の正本を含む）は触らない。外部 crate は増やさない。lib.rs は無い（module の宣言は main.rs）。

(e) 凍結 fixture（新しい dir tests/fixtures/ceiling/findings/・所見 file 10 本・本便の write-set の + の file で置く・AC16 の red_test の fixture）。歯は便 38 の凍結 fixture（tests/fixtures/ceiling/bundle/ の source と faces）から --write で束を一時 dir に組み、下の file を <観点>/findings.yaml の名で写してから --check を撃つ。record.bundle の値は便 38 の設計文書 §1 (e) が凍結した要約値（束が凍結なので値も凍結）。中身は次の逐語（YAML・引用符は使わない・末尾は改行 1 つ・字下げは半角 2 つ）:

pass-fidelity.yaml（合格・所見 1 件 = 仕込んだ欠陥を拾った形・evidence は source/srs.yaml の 78 行目の逐語）:
  verdict: 合格
  record:
    model: opus
    effort: high
    at: 2026-09-19T05:00:00Z
    read: [constitution, srs, adr, design-note]
    bundle: sha256-files-1 2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32
  findings:
    - id: F-1
      viewpoint: fidelity
      place: {doc: srs, at: requirements.FR2.plain}
      weight: 直す
      evidence: 合格か不合格のどちらかを出します。
      note: 規範文の 3 値のうち「まだ分からない」を平易文が落としている。

pass-readability.yaml（合格・所見 0）:
  verdict: 合格
  record:
    model: opus
    effort: high
    at: 2026-09-19T05:00:00Z
    read: [index, constitution, srs, adr, design-note]
    bundle: sha256-files-1 ded1548e1b6c5185b5b2ea0083d6ee68ed66dd5ca998bdd7d5d9a52ed2065576
  findings: []

pass-coherence.yaml（合格・所見 0）:
  verdict: 合格
  record:
    model: opus
    effort: high
    at: 2026-09-19T05:00:00Z
    read: [constitution, rules, srs, adr, design-note]
    bundle: sha256-files-1 91ca924dceef12d5b2df3946e6b56de69b1feeef13ea66f78d950af54c28476c
  findings: []

pass-reality.yaml（合格・所見 0）:
  verdict: 合格
  record:
    model: opus
    effort: high
    at: 2026-09-19T05:00:00Z
    read: [srs, adr, design-note]
    bundle: sha256-files-1 24ce1b872fa08dd128b39ac42e4ba72096bfe263df3bb463c9616899c96786d7
  findings: []

missing-field.yaml（fidelity 用・record に at が無い → まだ分からない）: pass-fidelity.yaml から「    at: 2026-09-19T05:00:00Z」の 1 行を除いたもの。
digest-mismatch.yaml（fidelity 用・要約値が束と違う → まだ分からない）: pass-fidelity.yaml の bundle の 16 進 64 字を 0 が 64 個の字に変えたもの。
fabricated-evidence.yaml（fidelity 用・根拠が正本に無い → まだ分からない）: pass-fidelity.yaml の evidence を「合格・不合格・まだ分からない のどれかを出します。」（欠陥を仕込む前の元の文 = 束の sources/ には無い）に変えたもの。
stop-unrefuted.yaml（fidelity 用・止める に反証が無い → まだ分からない）: pass-fidelity.yaml の weight を 止める に変えたもの（refute は無し）。
stop-upheld.yaml（fidelity 用・verdict は 合格 だが 止める が反証で支持されて残る → 不合格）: pass-fidelity.yaml の weight を 止める に変え、note の前に「      refute: 支持」の 1 行を足したもの。
fail-no-findings.yaml（fidelity 用・不合格なのに所見が無い → まだ分からない）: pass-reality.yaml の verdict を 不合格 に変え、read と bundle を pass-fidelity.yaml の値にしたもの。

(f) 歯（新しい file crates/folio/tests/findings.rs・関数名は findings を含める・--test findings の scope・組み立てた binary を子の処理で撃つ・束は歯の中で便 38 の凍結 fixture から --write で組む）:
1. 合格: 4 観点に pass-<観点>.yaml を写して --check → 0 ∧ 標準出力の 1 行目「fidelity: 合格（所見 1・止める 0）」∧ 最後の行「folio ceiling: 合格（観点 4・合格 4・不合格 0・まだ分からない 0）」。
2. まだ分からない（fidelity だけ差し替え・他 3 観点は pass）: missing-field → 2 ∧「fidelity: 起動の記録に at が無い」の趣旨の行（record と at の字を含む）／digest-mismatch → 2 ∧「要約値が束と合わない」／fabricated-evidence → 2 ∧「根拠が正本に無い」∧「F-1」／stop-unrefuted → 2 ∧「反証が未」∧「F-1」／fail-no-findings → 2 ∧「不合格なのに所見が無い」／pass-fidelity の verdict を まだ分からない に → 2 ∧「AI が判定できなかった」／最上位に extra の欄を足す → 2 ∧「未知の欄」∧「extra」／viewpoint を readability に → 2 ∧「viewpoint」／read から srs を除く → 2 ∧「read」∧「srs」／weight を 大変 に → 2 ∧「weight」∧「大変」／findings.yaml を置かない → 2 ∧「所見 file が無い」。どの周も最後の行は「folio ceiling: まだ分からない（観点 4・合格 3・不合格 0・まだ分からない 1）」。
3. 不合格: stop-upheld → 1 ∧「fidelity: 不合格（所見 1・止める 1）」∧「止める所見が残っている」∧「F-1」∧ 最後の行「folio ceiling: 不合格（観点 4・合格 3・不合格 1・まだ分からない 0）」／pass-fidelity の verdict を 不合格 に → 1（所見 1 件在る）／stop-upheld の refute を 退けた に → 0（残らない・所見 1・止める 0）／refute を まだ分からない に → 1（残る）。
4. 束の側: fidelity の束の dir を消す → 2 ∧「束が無い」／fidelity の sources/srs.yaml の 1 byte を書き換える → 2 ∧「束が壊れている」／束を組んだ後に fixture の写しの source/srs.yaml の 78 行目を元の文に戻して --check → 2 ∧「束が古い」／--faces に無い dir → 2 ∧「配信先が無い」。
5. 混ざり: 3 観点 pass + reality を 不合格（verdict 不合格 + 所見 1 件 = evidence は sources/srs.yaml の逐語）→ 1／不合格の観点と まだ分からない の観点が同時に在る → 2（まだ分からない が先に立つ）。
6. 旗: --write と --check の両方 → 終了 2 ∧ 標準エラーに使い方／どちらも無し → 終了 2。
7. 実の正本: design-intent の写し（copy_tree・folio build --out で配信先 → folio ceiling --write）から束を組み、4 観点の digest.txt から record.bundle を写した合格の所見 file（所見 0・read は reads.yaml の doc 全部）を歯の中で作って --check → 0。
既存の歯 crates/folio/tests/bundle.rs（便 38・11 本）は bundle.rs の内側の公開の変更で期待不変のまま緑（本文は 1 字も変えない・verify に bundle の回帰を 1 行）。

(g) 便 38 までの形との接続: 新規 file = crates/folio/src/findings.rs（1 本）・crates/folio/tests/findings.rs（1 本）・tests/fixtures/ceiling/findings/ の 10 本。crates/folio/src/bundle.rs は (d) の公開だけ・crates/folio/src/main.rs は (a)。便 40（面の天井の名札）は本便の 3 値（観点ごと・日付は record.at）を読んで面に出す。反証の材料の束（止める の所見を別の文脈の AI が反証するための束・ADR-8 決定 (3)）を組む口は後続の便で、本便は refute の欄の値を数えるだけ。

## 2. 範囲

- 入れる: --check の旗・所見 file の外形と欄の決まりの床・観点ごとの 3 値と終了コード・凍結 fixture の所見 file 10 本・歯・bundle.rs の公開。
- 入れない: 反証の材料の束を組む口（後続の便）・面の天井の名札（便 40）・AI の起動（席か器の手番）・天井の正本の変更・止める仕組み（P-11 の回数の数え = 台帳）・所見の意味の当否の判定（P-1）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cli | 口 | main.rs の Command Ceiling に --check（--write と排他・必須） |
| findings | 数え | findings.rs（所見 file の読み・欄の決まり・3 値・出力） |
| bundle | 束 | bundle.rs の load / build_all / 要約値 を crate の中へ公開（振る舞い不変） |
| anchor | 凍結 fixture | tests/fixtures/ceiling/findings/ の 10 本 + 便 38 の束の fixture |
| teeth | 歯 | tests/findings.rs 7 群 + 既存 bundle の期待不変 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "an"
title = "所見 file と起動の記録を数える folio ceiling --check（所見 file の外形と欄の決まり・根拠の逐語の実在・束の要約値の一致 3 方向・止める の反証・観点ごとの 3 値・4 観点が全部合格のときだけ 0）"
req = ["FR18", "FR5"]
section = "1"
write-set = ["+crates/folio/src/findings.rs", "crates/folio/src/bundle.rs", "crates/folio/src/main.rs", "+crates/folio/tests/findings.rs", "crates/folio/tests/bundle.rs", "+tests/fixtures/ceiling/findings/pass-fidelity.yaml", "+tests/fixtures/ceiling/findings/pass-readability.yaml", "+tests/fixtures/ceiling/findings/pass-coherence.yaml", "+tests/fixtures/ceiling/findings/pass-reality.yaml", "+tests/fixtures/ceiling/findings/missing-field.yaml", "+tests/fixtures/ceiling/findings/digest-mismatch.yaml", "+tests/fixtures/ceiling/findings/fabricated-evidence.yaml", "+tests/fixtures/ceiling/findings/stop-unrefuted.yaml", "+tests/fixtures/ceiling/findings/stop-upheld.yaml", "+tests/fixtures/ceiling/findings/fail-no-findings.yaml"]
verify = ["cargo nextest run -p folio --test findings findings", "cargo nextest run -p folio --test bundle bundle", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "findings の歯（合格 4 本・まだ分からない 11 種・不合格 4 種・束の側 4 種・混ざり 2 種・旗 2 種・実の正本）が緑、便 38 の歯 bundle が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
