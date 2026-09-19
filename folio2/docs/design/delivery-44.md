# 設計: 便 44 — 天井の正本の字面に依る歯 2 本を、版に依らない形に（読む文書の針・文書の一覧から消す歯の期待）

- 要件: FR18（天井の正本を床が数える・便 37 の歯 ceiling）/ FR5（3 値）
- 条: P-10.1（凍結 anchor は生成側からも検査側からも独立）/ P-6.2（歯の期待の意味は変えず、正本の字面への依存だけを外す）
- 判断の記録: ADR-8（天井の形）。判断の記録 ADR-9 / ADR-10 と天井の正本 v0.2 の提案（裁定 2・3・planner の worktree・持ち主の承認待ち）を main に入れる前提の是正の 2 本目。便 43（f2-648.59・着地 aaaf716）は正本の版と判断の記録の本数への依存を外したが、天井の正本 v0.2 の読む欄の変更に依る歯が 2 本残っていた（planner の実測 2026-09-19・提案の枝を main aaaf716 に載せて nextest 497/499・赤はこの 2 本だけ）。
- 裁定: 持ち主 2026-09-19「ひとまず全部推奨でよい。」（裁定 3 = 天井の正本 v0.2 → 2 周目）。本便は裁定の内容には触れない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 as が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い・縮む file は無い）。

## 1. 目的と中身

crates/folio/tests/ceiling.rs（389 行・幅 120 正規化も 389・17 本・関数名は全部 ceiling で始まる）の歯 2 本が、天井の正本の読む欄の字面に依っている。天井の正本 v0.2（提案）は読みやすさの入口の読む欄を 読み手・棚・読む順番 に直し、整合の観点に天井の正本そのものを読む行を足す。すると (a) 入口の読む欄の行を針にする歯は当て先が 0 か所になって落ち、(b) 文書の一覧から天井の正本の行を消す歯は「一覧が床の定数と違う」の違反に加えて「整合の reads[6] の doc: 行き先「ceiling」が一覧に無い」の違反が 1 件増えて「違反は 1 件だけ」の期待が落ちる（実測の違反の逐語は本節の末尾）。本便はその 2 本を、読む欄の字面と読む観点の数に依らない形に直す。期待の意味（一覧に無い文書を読む行は違反 1 件・文書の一覧の欠けは違反）は変えない。src・fixture・正本は触らない。

(a) 歯 1（関数 ceiling_read_doc_not_a_document_fails・270 行から）: 今は Work の mutate で行「      - {doc: index, fields: [shelf, sections]}」（半角の空白 6 つ + 行の全部）を doc が nowhere の行に替え、違反 1 件の種別「ceiling」と語「viewpoints の行 readability の reads[0] の doc」「行き先「nowhere」が一覧に無い」を期待している。直し = 針を行の先頭「      - {doc: index, fields: [」（半角の空白 6 つ + 「- {doc: index, fields: [」・欄の一覧は含めない）にし、替え先を「      - {doc: nowhere, fields: [」にする（欄の一覧はそのまま残る）。実測: main と提案の枝のどちらの ceiling.yaml でも「      - {doc: index, fields: [」で始まる行は 1 か所（読みやすさの reads[0]・入口を読む観点は読みやすさだけ）。mutate は当て先が 1 か所でないと止まるので、この針も 1 か所の条件を満たす。期待の語は変えない（readability の reads[0] のまま）。

(b) 歯 2（関数 ceiling_missing_document_fails・256 行から）: 今は mutate で文書の一覧の行「  - {id: ceiling, file: ceiling.yaml, note: この正本}」（+ 改行）を消し、assert_single_violation（終了 1・違反ちょうど 1・種別「ceiling」・語「documents の id」「一覧」「（無い: ceiling）」）を期待している。直し = 針は同じ行のまま（main と提案の枝のどちらにも 1 か所）。期待を次の形にする: 終了 1・標準出力に「folio check: 不合格（違反 」を含む・違反の一覧（既存の関数 violations = 標準出力の [ で始まる行）の件数 = 1 + n（n = 写しの ceiling.yaml の中で「      - {doc: ceiling,」で始まる行の数を歯の中で数える・main では 0・提案の枝では 1）・違反のうちちょうど 1 件が「documents の id」「一覧」「（無い: ceiling）」を全部含む・残りの n 件は全部「行き先「ceiling」が一覧に無い」を含む・全件が「[ceiling] ceiling.yaml」で始まる。assert_single_violation は他の 13 か所（歯 1 を含む）が使うので変えず、本歯だけ本文に書く（新しい補助関数は足さない）。

(c) 確かめ方（正本は main のまま）: 直した 2 本が main の正本で緑（ceiling 17 本・期待の意味は不変・他の 15 本は本文も期待も変えない）。提案の枝での緑は planner が着地後に自分の worktree を main に載せ直して測る（verify には入れない）。

(d) 便 43 までの形との接続: 新規 file は無い。src は触らない（check.rs・ceiling.rs は不変）。write-set は歯の file 1 本だけ。verify は nextest の scope 旗で tests/ceiling.rs に閉じる（フィルタ語 ceiling は src の unit にも当たるが --test の形で置き場を歯の file 1 本に限る）。size S = 増分は 20 行未満（余地 1,111）。

実測の違反の逐語（提案の枝・歯 2 の現行の期待が落ちる出力・2 件）: 「[ceiling] ceiling.yaml: documents の id: 一覧「constitution, rules, vocabulary, srs, index, intake, adr, design-note」が床の定数と違う（無い: ceiling）」と「[ceiling] ceiling.yaml: viewpoints の行 coherence の reads[6] の doc: 行き先「ceiling」が一覧に無い」。歯 1 の現行の落ち方は mutate の「変異の当て先が 1 か所でない」（0 か所）。

## 2. 範囲

- 入れる: tests/ceiling.rs の歯 2 本（読む文書の針を行の先頭に・文書の一覧から消す歯の期待を 1 + n 件の形に）。
- 入れない: src の変更・fixture の変更・正本の変更・補助関数の追加・他の歯 15 本の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| needle | 針 | 読む文書の歯の当て先を行の先頭だけに |
| count | 数え | 文書の一覧の歯の違反の数を写しから数える |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "as"
title = "天井の正本の字面に依る歯 2 本を版に依らない形に（読む文書の針を行の先頭 doc: index だけに・文書の一覧から消す歯の期待を一覧の違反 1 件 + その文書を読む観点の行の違反 n 件に・期待の意味は不変）"
req = ["FR18", "FR5"]
section = "1"
write-set = ["crates/folio/tests/ceiling.rs"]
verify = ["cargo nextest run -p folio --test ceiling ceiling", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "便 37 の歯 ceiling 17 本が main の正本で期待の意味不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
