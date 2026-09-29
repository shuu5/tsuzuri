# 設計: 便 56 — folio build の --write が最初に構造の床を回す（不合格なら書かない・まだ分からないなら書いて 2・要件書 v1.11 の FR5）

- 要件: FR5（第 1.11 版 = 配信先を組み立てるたびに床を実行する）/ FR7（組み立てて見せる）
- 条: P-3.1・P-3.3 / P-4.1・P-4.2 / P-1.1
- 出所: 天井の 3 周目（2026-09-20）の止める所見 実態 F-1（反証 支持）= 要件 FR5 は「生成のたびに構造の床を実行し」と約束していたが、folio build も folio face も床を一度も呼ばず、違反を入れた正本からでも終了 0 で面を書いた。持ち主の裁定 f2-648 notes 2026-09-20 13:30 JST「どちらも推奨で」（問 2 = 案 A）= 床を回す口を folio build に定め、要件書 v1.11 で FR5 の文を直した。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 be が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

要件書 第 1.11 版の FR5 は「folio は配信先を組み立てるたびに（folio build）構造の床を実行し、結果を合格・不合格・まだ分からない の 3 値で返す。実行できなかった検査を合格と表示しない。」である。いまの folio build（crates/folio/src/site.rs の関数 run・呼び手は crates/folio/src/main.rs の Build の枝）は床（crates/folio/src/check.rs の関数 check_dir）を呼ばない。実測（main c279bce）= 設計ノートの散文に規範の印を持ち参照 id の無い文を 1 つ入れた写しで、folio check は不合格（終了 1）、同じ置き場の folio build --write は「書いた（18 file）」で終了 0。本便は folio build の --write の最初に床を回し、床の 3 値を命令の結果に結ぶ。

設計判断の席の実測（2026-09-20・main c279bce）: site.rs は正規化 211 行・main.rs は 551 行。check_dir は置き場と旗（改訂の下書きを出す・anchor を凍結する）を受けて Report と後始末を返し、folio check の枝だけが呼んでいる（旗なしの形で呼べば副作用は無い）。Report は違反の数と「まだ分からない」の数と 3 値を持つ。folio build を呼ぶ歯の file は 6 本 = tests/site.rs（7 本）・tests/badge.rs・tests/bundle.rs・tests/findings.rs・tests/serve.rs・tests/parts.rs。このうち実の設計文書の置き場をそのまま --dir に渡す歯（tests/parts.rs の 1 本と tests/site.rs の一部）は、置き場が版管理の中に在り床が合格するので、終了 0 のままである。実の置き場を一時 dir へ写してから組む歯（tests/bundle.rs・tests/findings.rs・tests/badge.rs）は、写しに版管理が無いので床が「まだ分からない」になる。最小の手書きの fixture tests/fixtures/face/ から組む歯（tests/serve.rs・tests/site.rs の一部）は、正本が揃っていないので床が「まだ分からない」になる。

(a) folio build の --write。最初に check_dir を旗なしで回す（副作用を起こさない = 下書きも凍結もしない）。
1. 床が 不合格 = 配信先へ 1 file も書かない（配信先の dir も作らない・既に在る配信先は 1 byte も変えない）。標準出力は 1 行「folio build: 床 = 不合格（違反 n・まだ分からない m）・書かない（folio check で中身を見る）」。終了 1。
2. 床が まだ分からない = 今までどおり全部を書く。標準出力は 2 行 = 1 行目「folio build: 床 = まだ分からない（違反 0・まだ分からない m）」・2 行目は今の「folio build: 書いた（…）」の行。終了 2。
3. 床が 合格 = 今までどおり全部を書く。標準出力は 2 行 = 1 行目「folio build: 床 = 合格（違反 0・まだ分からない 0）」・2 行目は今の行。終了 0。
4. 床は合格か「まだ分からない」だが、面の用意そのものが出来ない（今の Err の道）ときは、今までどおり「まだ分からない」で終了 2・何も書かない（1 行目に床の行を足す）。
--check の口（配信先と正本の byte 一致）は変えない（床を回さない・出力も終了コードも今のまま）。folio face・folio figure・folio render・folio ceiling は変えない。

(b) 歯。新しい歯の関数名は site_floor で始める（crates/folio/tests/site.rs）。写しは、実の設計文書の置き場 design-intent と contracts/schema.toml を一時 dir へ写し、git の 1 commit にする（tests/schema.rs の Work と同じ形）= 床が合格する写し。
1. 床が合格の写し → --write は終了 0 ∧ 標準出力の 1 行目が「folio build: 床 = 合格」で始まる ∧ 2 行目が「folio build: 書いた」で始まる。
2. その写しの設計ノートの散文に、規範の印を持ち参照 id の無い文を 1 つ足して commit → folio check は終了 1 ∧ folio build --write は終了 1 ∧ 標準出力が 1 行で「床 = 不合格」と「書かない」を含む ∧ 配信先の dir が出来ていない。
3. 歯 2 の前に合格の写しで 1 度組んだ配信先が在るとき、違反を足してからの --write は配信先の file を 1 byte も変えない（全 file の中身を前後で比べる）。
4. 版管理の無い写し（commit しない）→ --write は終了 2 ∧ 1 行目が「床 = まだ分からない」で始まる ∧ 面は書かれている（index.html が在る）。
5. --check は床を回さない = 歯 2 の違反の入った写しでも、合格のときに組んだ配信先に対する --check の終了コードと出力は、床と無関係に今の規則のまま（正本が変わったので不一致の 1）。
6. 既存の歯の期待の直し（根拠 = (a) の 2 と 3）: 実の置き場の写しから組む歯（tests/bundle.rs・tests/findings.rs・tests/badge.rs）は、写しを git の 1 commit にして contracts/schema.toml も写し、終了 0 の期待を保つ（その歯が確かめたい事は束・所見・名札で、床の 3 値ではない）。最小の fixture から組む歯（tests/serve.rs・tests/site.rs の fixture の歯）は、終了コードの期待を 2 に直し、面が書かれていることの期待は保つ。標準出力の全文を比べている歯は、1 行目の床の行を足した形に直す。凍結の fixture との byte 一致の期待は 1 つも変えない。
7. 回帰（期待不変・共通の検証が回す）: tests/parts.rs（実の置き場を直に渡す = 終了 0 のまま）・tests/check.rs・tests/floor_cases.rs（凍結の場合 134 件）・面の歯の全部。

(c) 大きさと接続。新規 file は無い。既存 = site.rs か main.rs（床を回して結ぶ = 約 +50 行）・tests/site.rs（約 +100 行）・tests/bundle.rs と tests/findings.rs と tests/badge.rs と tests/serve.rs（写しの commit か期待の数字 = 各 +20 行以内）。size S。設計文書・fixture・src/check.rs・src/face.rs・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: folio build --write が床を先に回すこと・3 値と終了コードと出力の結び・歯・既存の歯の期待の直し。
- 入れない: folio build --check・face・figure・render・ceiling に床を回すこと・床の検査の中身・CI に folio check を足すこと。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| gate | 結び | build --write の最初に check_dir（旗なし）・不合格は書かない 1・まだ分からないは書いて 2 |
| teeth | 歯 | site_floor の 5 本・既存の歯の期待の直し |

## 4. 検査（歯）

§1 (b) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "be"
title = "folio build の --write が最初に構造の床を回す（不合格なら配信先へ 1 file も書かず終了 1・まだ分からないなら書いて終了 2・合格なら 0・標準出力の 1 行目に床の 3 値）= 要件書 v1.11 の FR5・--check と face と figure は変えない・既存の歯は写しを commit するか期待を 2 に直す"
req = ["FR5", "FR7"]
section = "1"
write-set = ["crates/folio/src/site.rs", "crates/folio/src/main.rs", "crates/folio/tests/site.rs", "crates/folio/tests/bundle.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/serve.rs"]
verify = ["cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test bundle bundle", "cargo nextest run -p folio --test findings findings", "cargo nextest run -p folio --test badge badge", "cargo nextest run -p folio --test serve serve", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "site の歯（site_floor の 5 本 = 合格 0・不合格は書かず 1・既に在る配信先は不変・版管理なしは書いて 2・--check は床と無関係）が緑、bundle・findings・badge・serve の歯が期待の直しの後で緑（凍結の fixture との byte 一致は不変）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
