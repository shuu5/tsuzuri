# 設計: 便 51 — 規則の表の欄の決まりの節の床の木 FLOOR を実装の定数として持ち（凍結 anchor と byte 一致）、床は最上位の節の閉じた一覧を file からではなく定数から読む（ADR-11 決定 (4)② の 3 本目の 1 段目・FR5 / FR19）

- 要件: FR5（床）/ FR19（決まりの部分を床の定数から導出する・対象に足すのは 3 段目）
- 条: P-5.1・P-5.6 / P-6.3・P-6.4 / P-10.1 / N-3.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ) と (4)②「規則の表の欄の決まりの節は門が無いので (イ) とする」。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)②。向きの決定は docs/design/adr-11-step2-enums.md §3 の 6。値は 1 つも変えない。
- 位置（3 段の 1 段目）: 本便 = 床の木と、床が定数から読む形 → 設計判断の席の PR = 実の rules.yaml の schema の節を生成区間に替える・要件書の次の版（FR19 と AC17 の対象に規則の表を足す・持ち主の承認）→ 便 53 = 命令 folio schema の 4 本目の対象に足す。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 az が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + を付けた 1 本）。

## 1. 目的と中身

規則の表の正本 design-intent/rules.yaml は、先頭の節 schema に欄の決まり（最上位の節の閉じた一覧 top_level・閾値の行と作法の行の欄の集合・値域 kind と status と stage・種別の意味・種別と憲法の機構の対応・凍結から外したものとその理由・逆向きの参照の説明）を人が書いて持つ。床（crates/folio/src/check.rs の関数 check_rules）は、この file 自身の schema.top_level を読んで「未知の節」を落としている = 検査される file の側で top_level に名を足せば、どんな節でも通せる。憲法と違って規則の表の schema の節には変更の門が無い。ADR-11 決定 (3)(イ) により、こういう形の決まりは実装の型付きの定数が正本で、file には生成区間として写しを導出する。本便はその 1 段目 = 床の木を実装に持ち、床が定数から読む形にする。実の rules.yaml・fixture・命令 folio schema の対象は本便では変えない。

設計判断の席の実測（2026-09-20・main 6e91b63）: check.rs は正規化 542 行。関数 schema_top_level が file の schema.top_level を読み（読めなければ「まだ分からない」で文言は「schema.top_level（節の閉じた一覧）が読めない」）、check_constitution と check_rules の 2 か所が使う。憲法の側は ADR-11 決定 (3) により file が正本のままなので変えない。規則の表の写しの fixture は 25 本で、schema.top_level を持つ 18 本は全部 schema・thresholds・discipline の 3 値、持たない 7 本（注入の fixture 5 本・天井の束の fixture 2 本）は床を通さない歯の材料である。凍結の場合 134 件（tests/floor_cases.yaml）に規則の表を書き換える場合は無い。床の木を生成区間の字面にする関数は crates/folio/src/schema.rs の derive（便 45・46・47 と同じ）。憲法の値域から導出した型は crates/folio/src/constitution_enums.rs に在る（便 49・Stage と MechanismKind を本便が使う・name は const fn）。

(a) 新しい file crates/folio/src/rules.rs に床の木 FLOOR を置く（pub(crate) の定数・型は schema.rs の Floor）。木の中身と順は凍結 anchor tests/fixtures/schema/rules-region.txt（設計判断の席が独立の実装で組んだ byte・27 行・1,764 byte・sha256 = dcf207ced150b5ebd3d6ae03bcdb5bc42ef9a06d6f74ff3f830ff96729dafad2）のとおり = 最上位の欄は順に version（数の 1）・top_level・top_level_note・threshold_row（required と optional）・discipline_row（required と optional）・enums（kind・status・stage）・enums_note・kind_meaning（4 つ）・kind_map_to_constitution（3 つ）・kind_map_to_constitution_note・excluded（what と why）・reverse_reference。今の実の rules.yaml の schema の節と値は同じで、足すのは説明の注 3 つ（_note で終わる欄 = 今は行末の注釈として書かれている文を欄に移したもの）だけ。値域 stage は手書きにせず憲法から導出した Stage の NAMES を使い、kind_map_to_constitution の右辺の値（reject・build-check・none）は MechanismKind の各値の name で書く = 憲法の値と規則の表の値が 1 か所から出る。値域 kind と status は規則の表だけの値域なので、この file の定数が正本である（status の順は今の file の順 = 仮・凍結・未定）。src/main.rs に mod の 1 行を足す。

(b) 床が定数から読む。check_rules は「未知の節」の許す一覧を、file の schema.top_level ではなく FLOOR の top_level から取る。これで、rules.yaml の側で top_level に名を足しても未知の節は落ちる（N-3.1）。file の schema.top_level が無い・読めないことは、もう「まだ分からない」にしない（写しは生成区間の検査 folio schema が見る = 3 段目）。check_constitution と関数 schema_top_level はそのまま残す。文言「未知の節」は変えない。床は本便では file の schema の節の中身を FLOOR と突き合わせない（写しの fixture 25 本を直さずに済ませるため。実の file の写しの一致は 3 段目で folio schema --check と歯が数える）。

(c) 歯。新しい歯の関数名は、src/rules.rs の unit test は rules_floor で、tests/check.rs は check_rules で始める。
1. src/rules.rs の unit test: schema.rs の derive に FLOOR を渡した結果が、凍結 anchor tests/fixtures/schema/rules-region.txt と byte 一致（anchor を書き換えて合わせてはいけない・変えるなら設計判断の席へ問う）。
2. 同じ file: FLOOR の enums.stage が憲法から導出した Stage の NAMES と同じ・top_level が schema・thresholds・discipline の 3 値（字面を歯に直に書く凍結の針）。
3. crates/folio/tests/check.rs: 実の設計文書の置き場の写しで、rules.yaml の schema.top_level に extras を足し、最上位に節 extras を足す → 不合格 ∧ 種別「未知の節」の違反がちょうど 1 件で extras を含む（本便の前の main では合格してしまう = 落ちる歯）。
4. 同じ file: 写しの rules.yaml から schema.top_level の行を消す → 規則の表についての「まだ分からない」が出ない（床の結果は消す前と同じ）。
5. 回帰（期待不変・共通の検証が回す）: tests/check.rs の既存の歯・tests/floor_cases.rs（凍結の場合 134 件）・tests/link.rs・tests/schema.rs（3 行のまま）・面と束の歯の全部。

(d) 大きさと接続。新規 file は 1 本（src/rules.rs・約 120 行）。既存 = check.rs（数行）・main.rs（+1 行）・tests/check.rs（正規化 310・約 +60 行）。size S。実の rules.yaml・設計文書・fixture（凍結 anchor は本便の前に設計判断の席が置く）・schema.rs・面の生成器の表 RULE_KIND と RULE_STATUS（3 段目の後の便）・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 床の木 FLOOR（rules.rs）・check_rules が定数から読む形・歯。
- 入れない: 実の rules.yaml の生成区間と要件書の次の版（設計判断の席の PR・持ち主の承認）・folio schema の対象に足すこと（便 53）・床が file の schema の節を突き合わせること・面の生成器の規則の表の側の表（後続）・行の required の全欄を床が数えること（別の話）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の木 | src/rules.rs の FLOOR（凍結 anchor と byte 一致・stage と機構の名は憲法から導出した型） |
| read | 読み | check_rules が top_level を定数から読む |
| teeth | 歯 | anchor の一致・凍結の針・file の側で緩められない・top_level が無くても黙る |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "az"
title = "規則の表の欄の決まりの節の床の木 FLOOR を実装の定数として持ち（新しい file rules.rs・凍結 anchor と byte 一致・stage と機構の名は憲法から導出した型）、床は規則の表の最上位の節の閉じた一覧を file からではなく定数から読む（file の側で節を足して通す口を塞ぐ・実の rules.yaml と fixture は不変・ADR-11 決定 (4)② の 3 本目の 1 段目）"
req = ["FR5", "FR19"]
section = "1"
write-set = ["+crates/folio/src/rules.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio rules_floor", "cargo nextest run -p folio --test check check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "rules_floor の歯（床の木の導出が凍結 anchor と byte 一致・stage と top_level の凍結の針）が緑、check の歯（file の側で top_level に足した節が未知の節で落ちる・top_level の行が無くても規則の表の「まだ分からない」が出ない・既存は期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
