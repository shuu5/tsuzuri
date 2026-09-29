# 設計: 便 43 — 実の正本を写す歯 2 本を、正本の版と判断の記録の本数に依らない形に（ceiling の重複キーの針・bundle の判断の記録の数）

- 要件: FR18（天井の正本を床が数える・便 37 の歯 ceiling）/ FR17（材料の束を組む・便 38 の歯 bundle）/ FR5（3 値）
- 条: P-10.1（凍結 anchor は生成側からも検査側からも独立）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-6.2（生成物を手で直さない = 歯の期待の意味は変えず、正本の字面への依存だけを外す）
- 判断の記録: ADR-8（天井の形）。判断の記録 ADR-9 / ADR-10 と天井の正本 v0.2 の提案（裁定 2・3・planner の worktree・持ち主の承認待ち）を main に入れる前提の是正。planner の実測（2026-09-19・main ee56e89）: 提案の枝で nextest は 497/499 で、赤の 2 本は本便の 2 本だけ。
- 裁定: 持ち主 2026-09-19「ひとまず全部推奨でよい。」（裁定 2 = 欄の決まりの file を導出物に・裁定 3 = 天井の正本 v0.2 → 2 周目）。本便は裁定の内容には触れず、正本を上げても歯が赤にならない形にするだけ。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ar が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い・縮む file は無い）。

## 1. 目的と中身

実の正本（design-intent/ の写し）を読む歯 2 本が、正本の版の字面と判断の記録の本数を固定の値で持っている。正本を v0.2 に上げ判断の記録を 2 本足す枝（提案・c545e12）では、この 2 本だけが赤になる（planner の実測・497/499）。本便はその 2 本を、正本の版と本数に依らない形に直す。期待の意味（重複キーは違反 1 件・実の正本の束は adr/ の写しを全部持ち面は判断の記録 1 本に 1 枚）は変えない。src・fixture・正本は触らない。

(a) 歯 1（crates/folio/tests/ceiling.rs・385 行・関数 ceiling_duplicate_key_fails・377 行から）: 今は Work の mutate で ceiling.yaml の行「  version: v0.1」（半角の空白 2 つ + version: v0.1 + 改行）を同じ行の 2 度書きに変え、違反 1 件の種別「重複キー」と文「同じ表にキー「version」を 2 度書いている」を期待している。版が v0.2 になると mutate の当て先が 0 か所になり落ちる（mutate は当て先が 1 か所でないと止まる）。直し = 針を版に依らない行「  id: folio2-ceiling」（半角の空白 2 つ + id: folio2-ceiling + 改行）にし、2 度書きも同じ行で、期待の文を「同じ表にキー「id」を 2 度書いている」にする。実測: main と提案の枝のどちらの ceiling.yaml でも「  id: folio2-ceiling」の行は 1 か所（meta の下）。違反の文は src/check.rs（198 行）の「{file} {} 行: 同じ表にキー「{}」を 2 度書いている」で、キー名だけが変わる。assert_single_violation（終了 1・違反 1・種別・語・標準出力の「不合格（違反 1・」）は変えない。

(b) 歯 2（crates/folio/tests/bundle.rs・543 行〔幅 120 正規化 550〕・関数 bundle_on_the_real_source_builds_four_bundles・476 行から）: 今は実の design-intent/ を一時 dir に写し build → ceiling --write の後、fidelity の sources/adr/ の file 数を 9、faces/ の adr- で始まる file 数を 8 と固定で期待している（530 行と 538 行）。判断の記録を 2 本足すと 11 / 10 になり落ちる。直し = 固定の値をやめ、同じ歯の中で写した正本の dir（変数 dir = 一時 dir の design-intent）の adr/ を既存の関数 names（dir 直下の file の名・byte 順）で数え、
- sources/adr/ の一覧は写しの adr/ の一覧と等しい（名の集合と数が同じ・schema.yaml と ADR-8.yaml を含む主張は残す）
- faces/ の adr- で始まる file 数は、写しの adr/ のうち名が ADR- で始まり .yaml で終わる file の数と等しい
の形にする。design-note の 3 と note-example.html / note-figures.html の固定は本便では触らない（提案の枝で design-note/ は変わらない・実測 3 file = example.yaml・figures.yaml・schema.yaml）。他の 10 本（凍結 anchor・byte 一致・決定性・まだ分からない 7 通り）は本文も期待も変えない。

(c) 確かめ方（歯の中で正本を上げる写しは作らない・正本は main のまま）: 直した 2 本が main の正本で緑（ceiling 17 本・bundle 11 本・期待の意味は不変）。加えて、直した歯の本文に「v0.1」の字面と固定の 9 / 8 の数が無いことは、着地後に admin が grep で測って台帳の notes に写す（verify の行では測れないので done には入れない）。提案の枝での緑は planner が着地後に自分の worktree を main に載せ直して測る（本便の verify には入れない）。

(d) 便 42 までの形との接続: 新規 file は無い。src は触らない（findings.rs・bundle.rs・check.rs は不変）。write-set は歯の file 2 本だけ。verify は nextest の scope 旗で tests/ceiling.rs と tests/bundle.rs に閉じる（フィルタ語 ceiling / bundle は src の unit にも当たるが、--test の形で置き場を歯の file 2 本に限る）。size S = 2 本とも増分は 20 行未満（余地 ceiling 1,115・bundle 950）。

## 2. 範囲

- 入れる: tests/ceiling.rs の重複キーの針を版に依らない行に・tests/bundle.rs の判断の記録の数と面の数を写しの adr/ から数える形に。
- 入れない: src の変更・fixture の変更・正本の変更・design-note の数の固定の解除・判断の記録 ADR-9 / ADR-10 と天井 v0.2 の本文。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| needle | 針 | tests/ceiling.rs の mutate の当て先を id の行に |
| count | 数え | tests/bundle.rs の adr/ の写しから数える |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ar"
title = "実の正本を写す歯 2 本を正本の版と判断の記録の本数に依らない形に（ceiling の重複キーの針を id の行に・bundle の adr/ の数と面の数を写しから数える・期待の意味は不変）"
req = ["FR18", "FR17", "FR5"]
section = "1"
write-set = ["crates/folio/tests/ceiling.rs", "crates/folio/tests/bundle.rs"]
verify = ["cargo nextest run -p folio --test ceiling ceiling", "cargo nextest run -p folio --test bundle bundle", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "便 37 の歯 ceiling 17 本と便 38 の歯 bundle 11 本が main の正本で期待の意味不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
