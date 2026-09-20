# 設計: 便 59 — 規則 R-11（強度と規範文の文末の一致）を床に戻し、命令の説明の古い面の数と対象を直す（約束と実態の総点検の是正・FR5 / FR19）

- 要件: FR5（構造の床の 3 値）/ FR19（生成区間の対象 4 本）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1 / P-5.1
- 出所: 約束と実態の総点検（2026-09-20・main 92adc46）の止める相当 = 規則の表 R-11「強度欄（must / must-not / should）と規範文の文末（〜ない。）の不一致の数 = 0」を数える口が実装に無い（day-1 の Python の床 scripts/retired/check_draft.py に在った検査が、2026-09-18 の退役で Rust の床へ写されなかった）。天井の 9 周目の実態 F-2 と取り込み前の検証 = folio schema --help が対象を adr/schema.yaml だけと言い（実装は 4 本）、folio parts --help の --page の既定が 3 面と言う（実装は 5 面）。持ち主の裁定 2026-09-21 00:00 JST（f2-648 notes・逐語 すべて推奨で進めて・問 3 = (a) R-11 と (g) --help を先に）。
- 根拠の判断: rules 行 D-11 の範囲の内（判定に使う式を足す）= 根拠の裁定 id は上の 1 行。式は憲法の schema 節の one_polarity の宣言と規則の表 R-11 の what のとおり。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bh が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 床に R-11 を足す。crates/folio/src/check.rs の check_constitution（正規化 625 行・余地 875）で、各条の statements の 1 本ずつについて、strength が must-not なら text の末尾が「ない。」で終わり、must か should なら「ない。」で終わらない、を数える。合わない 1 本につき違反 1 件 = 種別は R-11・文言は「{条の id}: {規範文の id}: strength {値} と文末が合わない（must-not ⇔ 〜ない。）」の形（{ } は値）。strength が値域の外のときは便 55 の値域の検査（種別 schema）が既に落とすので、R-11 は数えない（二重に出さない）。憲法の schema 節の one_polarity は宣言（true）で、床はこの値を読まず式を定数として持つ（規則の表 R-11 の what が正本・ADR-11 決定 (3)(エ)）。実の憲法（27 条・規範文 67 本）は 2026-09-20 の実測で不一致 0 なので、本便で床の結果は変わらない。

(b) 凍結の場合を 1 件足す（tests/floor_cases.yaml・土台は tests/fixtures/floor_base のまま）: id strength-polarity・mutate は constitution.yaml の articles の P-1 の statements の P-1.1（strength must・text は「〜する。」で終わる）の strength を must-not に変える 1 手・expect_rc 1・expect_msg「P-1.1: strength must-not と文末が合わない」。既存の 134 件の期待は 1 字も変えない（134 件の中に strength と文末が食い違う mutation は無い = 席が 2026-09-20 に土台と全件の mutation を読んで確かめた。もし落ちる場合が在れば、その場合の期待を直さず席へ問う）。

(c) 規則の表 R-11 の行の note（design-intent/rules.yaml）の末尾の「day-1 の床の退役（2026-09-18）でこれを数える口が無くなり Rust の床へ写していない＝この行の判定は「まだ分からない」（P-4.2）」を、「day-1 の床の退役（2026-09-18）で数える口が一度無くなり、便 59 で Rust の床に戻した（folio check の種別 R-11）」に替える。行の value・kind・status・ruling・ruled_at は触らない（この注の直しの裁定は上の出所の 1 行 = 2026-09-21 00:00 JST の承認が便 59 を名指す）。

(d) 命令の説明を実装に合わせる（crates/folio/src/main.rs・正規化 551 行）: folio schema の --dir の説明「設計文書の置き場（adr/schema.yaml を読む）」を「設計文書の置き場（欄の決まりの file 4 本 = adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml を読む）」に、folio parts の --page の説明の「1 つも無ければ --dir の下の preview/ の index / constitution / srs」を「1 つも無ければ --dir の下の preview/ の index / constitution / srs / adr / note」に。説明の字だけで、旗の型・既定の値・処理は変えない。

(e) 歯。
1. crates/folio/tests/check.rs（正規化 527 行）に 6 本足す（関数名は r11_ で始める・今この語で始まる歯は無い）。式の分岐は 2 つ（must-not なのに「ない。」で終わらない／must か should なのに「ない。」で終わる）で、歯はその両方を別々に測る: ① 分岐 1 = 実の正本の写しで、文末が「する。」の規範文 1 本（P-1.1・strength must）の strength を must-not に変えると、種別 R-11 で落ち、文言に「P-1.1: strength must-not と文末が合わない」を含む ② 分岐 2 = 実の正本の写しで、strength must の規範文 1 本（P-1.1）の text の末尾の「担う。」を「担わない。」に変える（strength は must のまま）と、種別 R-11 で落ち、文言に「P-1.1: strength must と文末が合わない」を含む ③ 写しを変えずに folio check が合格（違反 0・まだ分からない 0）= 実の正本（27 条・規範文 67 本）に不一致が無いことを、式を足した後の床で確かめる。写しの作り方は同じ file の既存の歯と同じ（実の design-intent を一時の置き場へ写して 1 手だけ書き換える）。④ (c) の注 = 実の design-intent/rules.yaml を読み、id が R-11 の行の note が「便 59 で Rust の床に戻した（folio check の種別 R-11）」を含み、「Rust の床へ写していない」を含まない ⑤ (d) の説明 1 = folio schema --help（env! の CARGO_BIN_EXE_folio・既存の歯と同じ呼び方）の標準出力が「adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml」を含む ⑥ (d) の説明 2 = folio parts --help の標準出力が「index / constitution / srs / adr / note」を含む。④⑤⑥は実装の前の main では赤（注の字と説明の字が古い）。
2. tests/floor_cases.rs の歯 floor_cases_all_pass_with_folio（本文は不変）が 135 / 135 で緑 = (b) の場合と、既存 134 件の不変を確かめる。
3. tests/schema.rs の歯（filter schema・本文は不変）が緑 = rules.yaml の注を直しても FR19 の生成区間（folio schema --check の対象 4 本）が正本と一致したまま（注は生成区間の外なので一致は変わらないはず・変わったら席へ問う）。
4. 回帰（期待不変・共通の検証が回す）: tests/check.rs の既存・tests/parts.rs。

(f) 大きさと接続。新規 file は無い。既存 = check.rs（式 約 +15 行）・main.rs（説明 2 行）・rules.yaml（R-11 の注 1 行）・floor_cases.yaml（1 件）・tests/check.rs（約 +90 行）。size S。tests/floor_cases.rs と tests/schema.rs は本文不変（検証の範囲に入るので write-set に置く）。ほかの設計文書・fixture・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: R-11 の式と違反の文言・凍結の場合 1 件・R-11 の注・命令の説明 2 か所・歯。
- 入れない: R-12（言い換えの無い英語）・極性一覧（P-18 / R-13）・P-17 の差分の検査・R-3 の後半・N-1 の歯（別の便）・要件書の側の注の更新（席の PR）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| r11 | 式 | check.rs の check_constitution に strength と文末の一致を足す |
| case | 凍結の場合 | floor_cases.yaml に strength-polarity を 1 件 |
| note | 注 | rules.yaml の R-11 の注を実態に |
| help | 説明 | main.rs の schema --dir と parts --page の説明 |
| teeth | 歯 | tests/check.rs に r11_ の 6 本（分岐 1・分岐 2・変えない写しの合格・R-11 の注・schema --help・parts --help） |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bh"
title = "規則 R-11（強度と規範文の文末の一致・must-not ⇔ 〜ない。）を床 folio check に戻し（種別 R-11・凍結の場合 1 件・R-11 の注を実態に）、folio schema の --dir と folio parts の --page の説明を実装（欄の決まり 4 本・面 5 つ）に合わせる（約束と実態の総点検の是正・持ち主の裁定 2026-09-21 00:00 JST）"
req = ["FR5", "FR19"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "crates/folio/tests/floor_cases.rs", "crates/folio/tests/schema.rs", "tests/floor_cases.yaml", "design-intent/rules.yaml"]
verify = ["cargo nextest run -p folio --test check r11_", "cargo nextest run -p folio --test floor_cases floor_cases", "cargo nextest run -p folio --test schema schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "r11_ の歯 6 本（分岐 1 = must-not なのに文末が〜する。の規範文が種別 R-11 で落ちる・分岐 2 = must なのに文末が〜ない。の規範文が種別 R-11 で落ちる・変えない写しは合格・rules.yaml の R-11 の注が 便 59 で Rust の床に戻した を含む・folio schema --help が欄の決まり 4 本の名を出す・folio parts --help が面 5 つの名を出す）が緑、floor_cases の歯が 135 / 135 で緑（既存 134 件の期待は不変）、schema の歯が緑（生成区間は正本と一致のまま）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
