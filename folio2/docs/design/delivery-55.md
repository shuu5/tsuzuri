# 設計: 便 55 — 床が憲法の各欄の値を値域で数える（床の穴 f2-648.82・FR5 / NFR3）

- 要件: FR5（検査結果を必ず返す）/ NFR3（参照は必ずつながる）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1・P-4.2 / P-5.1 / P-10.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(ア)（憲法の値域は憲法の file が正本・実装は組み立て時に導出）。rules 行 D-11 の作法 = 本便が判定に使う値域は便 49 が憲法から導出した型で、新しい定数は足さない。
- 出所: 設計ノート docs/design/adr-11-step2-enums.md §1 の 4 と §5 の 4（床の穴）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bd が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

床（crates/folio/src/check.rs の関数 check_constitution）は、憲法の条の id の形・空でない欄・重複だけを数えていて、値域を持つ欄の値が値域の内かどうかを数えていない。値域の外の値（例 = 段 tier に sometimes）を初めて落とすのは面の生成器の表引き（Err =「まだ分からない」）と注入（強度だけ）で、folio check は合格を返してしまう。機械で決定的に確かめられる項目なので床に置く（P-3.1）。値域は、便 49 が憲法の正本 design-intent/constitution.yaml の schema.enums から組み立て時に導出した型（crates/folio/src/constitution_enums.rs・型ごとに from_name）をそのまま使う = 値域の字面を check.rs に書かない。

設計判断の席の実測（2026-09-20・main ecc84e0）: check.rs は正規化 544 行・tests/check.rs は 387 行。値域を持つ欄は条 1 つにつき次の 10 か所 = 条の tier（型 Tier）と binds（Binds）・規範文 statements の各行の pattern（Pattern）と strength（Strength）・根拠 rationale の各行の kind（RationaleKind）・機構 mechanism の kind（MechanismKind）と live（MechanismLive）と stage（Stage）と polarity（Polarity）・撤退条件 retreat の kind（RetreatKind）。実の憲法と、憲法の写しの fixture 24 本（凍結した土台 tests/fixtures/floor_base/ を含む）を全部読んで数えた結果、値域の外の値は 0 件 = 本便で今の歯の期待は変わらない。最小の写しの fixture のうち注入の 5 本は条に tier などの欄そのものを持たない（欄の有無は本便では数えない）。凍結の場合 134 件（tests/floor_cases.yaml）にも、これらの欄へ値域の外の値を入れる場合は無い。

(a) check_constitution に足す。上の 10 か所について、欄が **在って** 値が導出した型の from_name で引けないとき（文字列でない値も同じ）、種別 schema の違反 1 件を出す。文言は 条の id・欄の道（例 = tier・statements の P-8.1 の pattern・mechanism.kind）・値・「が憲法の値域 schema.enums.」に鍵の名（tier・binds・pattern・strength・rationale_kind・mechanism_kind・mechanism_live・stage・polarity・retreat_kind）・「に無い」を含む。欄が無い・null のときは本便では何も出さない（必須の欄の有無は別の話で、最小の写しの fixture を直さずに済ませる）。rationale・mechanism・retreat が一覧や表でないときも本便では何も出さない。ほかの検査と文言は変えない。

(b) 値域の物差しは「この folio を組み立てた版の憲法」の値域である。読んでいる置き場の憲法が値域を改訂していて folio が古いときは、正しい値も違反に見える = その場合は床の撤退条件の種類の突き合わせ（src/link.rs）と面のずれの検査（便 50）が「組み立て直す」を先に知らせる。本便はこの関係を変えない。

(c) 歯。新しい歯の関数名は check_constitution_enum で始める（crates/folio/tests/check.rs・実の設計文書の置き場の写しを一時 dir に取り git の 1 commit にして床を回す既存の形）。
1. 改訂の差分の検査（A-2）の範囲の外の欄 3 つ = mechanism.kind・mechanism.live・rationale の kind を、それぞれ値域の外の値に書き換える → 不合格 ∧ 違反がちょうど 1 件 ∧ 種別 schema ∧ 文言に値と鍵の名（mechanism_kind・mechanism_live・rationale_kind）。
2. 改訂の範囲の内の欄 = 条の tier を sometimes に、規範文 1 つの strength を may に書き換える → 不合格 ∧ 違反の中に、種別 schema で値と鍵の名（tier・strength）を含む 1 件が在る（同じ書き換えで改訂の差分の違反も出るので、件数は固定しない）。
3. 欄が無いときは黙る = 条 1 つから mechanism の stage の欄を消す → 本便の違反は出ない（床の結果に schema.enums.stage を含む行が無い）。
4. 回帰（期待不変・共通の検証が回す）: tests/check.rs の既存の歯・tests/floor_cases.rs（凍結の場合 134 件）・tests/link.rs・tests/adr.rs・tests/inject.rs・面と束の歯の全部。

(d) 大きさと接続。新規 file は無い。既存 = check.rs（約 +70 行）・tests/check.rs（約 +90 行）。size S。設計文書・fixture・src/constitution_enums.rs・build.rs・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: check_constitution の値域の検査（在る欄だけ）・歯。
- 入れない: 必須の欄の有無を床が数えること・規則の表と要件書の値域（別の便）・要件の行の欠けた欄（f2-648.67）・値域や文言の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| range | 値域 | check_constitution が在る欄の値を導出した型の from_name で引く |
| teeth | 歯 | 範囲の外の欄 3 つは違反 1 件・範囲の内の欄 2 つは違反を含む・欄が無ければ黙る |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bd"
title = "床が憲法の条の値域を持つ欄 10 か所の値を、憲法から組み立て時に導出した型で数える（在る欄だけ・値域の外は種別 schema の違反・値域の字面を check.rs に書かない・今の設計文書と fixture の結果は不変・床の穴 f2-648.82）"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test check check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "check の歯（改訂の範囲の外の欄 3 つは違反ちょうど 1 件・範囲の内の欄 2 つは値域の違反を含む・欄が無ければ黙る・既存は期待不変）が緑、共通の検証（floor_cases 134 件ほか）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
