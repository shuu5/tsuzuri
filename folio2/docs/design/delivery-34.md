# 設計: 便 34 — 要件書の正本に図の節を持てるようにし、要件書の面に図の章 09 を足す。図の枠の生成は 3 つの面で共有する（FR15 / FR4 / NFR2・ADR-4 決定 (1)(3)）

- 要件: FR15（文書を生成するとき、正本の図の節に型付き記述があれば図の道具で検査と描画に掛け、図の本体だけを埋める・通らない図は生成せず前の生成物も上書きしない）/ FR4（3 枚を 1 つの生成器で出す・要件書の面）/ NFR2（部品目録に無い class は 0）
- 条: P-2.1（生成器は 1 つ・面ごとに図の枠の生成を持たない）/ P-2.4（閉じた一覧）/ P-5.1（節の一覧は型付きデータ・床の定数）/ P-4.1・P-10.1
- 判断の記録: ADR-4 決定 (1)（図の正本は、その図を載せる文書の正本 YAML の図の節に）・決定 (3)（図の本体だけを埋める）。ADR-5（見本 3 面の生成器・要件書の面は手書き）。便 33（f2-648.48）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「では図の追加の方向で進めて。」（判断の記録の次に要件書）。要件書の図 1〜3（folio が自前で描く帯の型）は据え置き・本便で足すのは図の道具の図の章。walk 承認は実の要件書と判断の記録に図を入れた後（planner の別 PR・契約の外）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ai が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

判断の記録（便 33）と同じ規律で、要件書の正本 `design-intent/srs.yaml` にも任意の図の節（figures・最上位の節）を持てるようにし、要件書の面（`folio face --face srs`）に図の章 09「図」（用語集の後・承認欄の前）を足す。加えて、図の枠（figure-panel・fig-title・図の本体・figcaption）の生成を面ごとに 3 度書かず、共有の口（face.rs）に 1 つ持って設計ノート・判断の記録・要件書の 3 面から呼ぶ（P-2.1・出力は byte 不変）。本便は (1) 床（節の閉じた一覧と行の欄）、(2) 共有の図の枠、(3) 要件書の面の章 09、(4) 凍結 fixture と歯、の 4 つ。実の要件書に図を書くのは本便の外。

planner の実測（2026-09-19・main d98fba2）: 要件書の正本には schema 節が無く、節の閉じた一覧は床 check.rs（正規化 473）の定数 SRS_TOP_LEVEL（meta・goals・scope・scope_m1・actors・outputs・rail・verdicts・requirements・nonfunctional・acceptance・not_frozen・constraints・sources・glossary_pointer の 15）が持ち、未知の節は「未知の節」で落ちる（歯の fixture tests/fixtures/check/unknown-section は extra_section で落ちる形）。check_srs は goals・verdicts・requirements・nonfunctional・acceptance・constraints の行の欄の非空と id の重複を数える。要件の行の figures 欄（例 [図2-1, 図1]）は folio が自前で描く図 1〜3 の参照で、本便の図の節とは別（据え置き）。要件書の面 face_srs.rs（正規化 1,370・余地 130）は FRAME を const で持ち、CHAPTERS と BANDS が 8 章固定・toc は CHAPTERS から・関数 figure_open / figure_close が図 1〜3 の枠を出す・PARTS に figure-panel を含む・関数 xref が同じ面の id へ、article_link が憲法の条へのリンクを出す・Ctx に fr・nfr・acs・cons・goals の項を持つ。判断の記録の面 face_adr.rs（正規化 789）の関数 figures_chapter と設計ノートの面 face_note.rs（正規化 1,049）の関数 figures_chapter は同じ字面（帯・figure-panel・fig-title・図の本体・figcaption の ver「図 <i> · <型の名札> · <図の id> · 根拠: …」）で、型の名札 FIGURE_LABELS は face_note.rs の pub(crate)・図の本体は figure.rs の関数 render（pub・dir・図 id・型の字・spec の X）。face.rs（正規化 884・余地 616）は Frame（band・toc・approval_band・foot・dc）と esc・anchor・hint・card を持つ。tests/face.rs（正規化 839）の fixture_copy は正本 4 file だけを写し（図の道具は写さない）、要件書の面の凍結は tests/fixtures/face/expected-srs.html（正規化 329・fixture の srs.yaml は正規化 106・図の節なし）。tests/site.rs の srs.html の期待は同じ expected-srs.html。tests/check.rs（89 行）は fixture dir 4 組を撃つだけで design-intent の写しを作る補助を持たない（tests/adr.rs が便 33 で copy_tree + git init の補助を持つ）。readable.html の生成（render.rs）は要件書の節を名指しで読むので図の節を足しても導出物は変わらない。入口の面の要件書の card は counts の 4 値だけ読む。実の要件書と凍結 fixture に図は無い＝本便の後も実の面は byte 不変。図の型の一覧 FigureType とその関数 from_name は parts.rs の pub mod catalog（build.rs が OUT_DIR に生成する parts_catalog.rs）の pub fn で、adr.rs（便 33）と note.rs が既に呼んでいる＝check.rs から呼ぶのに figure.rs と parts.rs は触らない。verify の scope に入る既存の歯は関数名に filter 語を含む（tests/face_adr.rs の 25 本・tests/face_note.rs の 24 本・tests/site.rs の 7 本・本文不変・admin の実測 2026-09-19）。

(a) 床（`crates/folio/src/check.rs`）。SRS_TOP_LEVEL の末尾に figures を足す（16）。check_srs に figures の行の検査を足す: 節が無いか null なら 0 行・一覧でなければ「まだ分からない」（rows と同じ扱い）・各行は id・type・caption・spec が必須で refs・note が任意（未知の欄は「未知の欄」の違反・欠落は非空の違反と同じ形）・id と caption は非空・type は部品目録の図の型（FigureType の関数 from_name・一覧に無ければ「図の型「<型>」が部品目録の一覧に無い」）・spec は表でなければ違反・refs が在れば各項は要件書の id の形（GOAL・FR・NFR・AC・CON + 数字）か条・rules 行・判断の記録の id の形（adr.rs の is_basis_id と同じ判定を check.rs に写さず adr.rs の関数を pub(crate) にして呼ぶ）・図の id は他の節の id と重複しない（duplicate_ids の all に足す）。違反の種類は schema（他の要件書の欄と同じ）。

(b) 共有の図の枠（`crates/folio/src/face.rs`）。関数 figure_panel（pub・引数は o・Frame・図の番号 i・図の id・caption の escape 済みの字・図の本体・型の名札・根拠のリンクの一覧）を足し、face_adr.rs と face_note.rs の figures_chapter の figure-panel の区間（開始タグ〜figcaption〜終了タグ）と同じ字面を出す。型の名札の表 FIGURE_LABELS を face_note.rs から face.rs へ移し pub(crate) のまま（表は 1 つ）。図の本体と型の名札を得る関数 figure_body（pub・dir・図の X・戻り値は（図の id・本体・名札））も face.rs に置き、型が表に無ければ「図の型「<型>」は図の道具の型でない」・本体は figure.rs の関数 render。face_adr.rs と face_note.rs の figures_chapter はこの 2 つを呼ぶ形に縮める（出力は byte 不変・凍結 expected-adr.html と expected-note.html と expected-site-adr-2.html は変えない・根拠のリンクの解き方は各面のまま）。

(c) 要件書の面（`crates/folio/src/face_srs.rs`）。FRAME を const から関数 derive の中で組む形にし、BANDS を 9 本にする（9 本目は band-3 の class と BANDS の 3 番目の絵記号の字面）。正本の最上位の figures（任意・無ければ今の形と byte 不変）が 1 つ以上のとき: toc に 09「図」（k「図」・t「<数> 枚」）を承認の前に足し、章 09 を用語集の後・承認欄の前に置く。章 09 の帯は h2「図 <数> 枚」・kicker「図」。中身は (b) の figure_body と figure_panel で図ごとに出す。根拠（refs）のリンク: 要件書の id（GOAL・FR・NFR・AC・CON）は関数 xref（同じ面の anchor）・条（P・A・N）は関数 article_link・rules 行（R・D）は憲法の面の該当の節（article_link と同じ href の形で anchor は rules 行の id）・判断の記録は `adr-<数>.html`・どれでもなければ id の直後に「（まだ分からない）」（判断の記録の面の resolve と同じ 4 形 + 判断の記録）。foot の機械の面の dl に figures（数）を足すのは 1 枚以上のときだけ。cover と他の章は不変。図が 1 枚でも導出できなければ面全体が 2（全部か無しか・前の面は残る）。

(d) 凍結 fixture。`tests/fixtures/face/srs.yaml` に最上位の figures を 1 枚足す（id fig-1・type archify-architecture・caption「見本の図」・refs [FR1]・spec は 2 箱の構成図で layout grid〔tests/fixtures/face/design-note/full.yaml の fig-1 と同じ形〕）。`tests/fixtures/face/expected-srs.html` を (c) の形で再凍結する（差分は toc の 09・章 09・foot の figures だけ＝他の章は byte 不変を歯で見る）。tests/site.rs の srs.html の期待は同じ file なので追従。expected-index.html は変わらない（入口は counts しか読まない・歯で見る）。

(e) 写し。tests/face.rs の fixture_copy に repo の `vendor/archify/` を一時 dir（src/ の親）の `vendor/archify/` へ再帰で写す（tests/figure.rs と同じ形・憲法の面の歯にも同じ写しが渡るが害は無い）。tests/site.rs・serve.rs・face_index.rs・face_note.rs・face_adr.rs の写しは変えない（face_index は要件書の figures を読まない・site と serve は便 31 で写している）。

(f) 歯。`crates/folio/tests/face.rs`（関数名は face を含める・`--test face` の scope）: 既存の歯は再凍結に追従（要件書の census は章の帯を 8 でなく「8 + 図の有無」で数える）。次を足す。
1. 写し（srs.yaml・図 1 枚）で toc に 09「図」∧ figure-panel が図 1〜3 の分 + 1 ∧ svg 1 ∧ fig-title に caption ∧ figcaption の ver に「構成図（architecture）」と「根拠: 」と FR1 の同じ面へのリンク（href が # で始まる）。
2. 写しの figures を消した面で 章 09 と toc の 09 と foot の figures が無い ∧ その面が便 33 までの形と同じ（図ありの面から章 09・toc の 09・foot の figures を切り出して除いた残りと byte で同じ）。
3. 写しの spec から layout を消す → --write = 2 ∧「図の道具の検査を通らない」∧ 先に書いた面の byte が不変。
4. 写しの図の type を pipeline-rail に → 2 ∧「図の道具の型でない」／ 親 dir の道具を消す → 2 ∧「図の道具が無い」。
5. 実の正本で figure-panel の数が 図 1〜3 の分（実測 2 枚の枠）+ 正本の figures の数（実測 0）∧ parts --check 合格（既存の歯に足す）。
`crates/folio/tests/check.rs`（関数名は check を含める・`--test check` の scope）: design-intent の写し（copy_tree + git init・tests/adr.rs の補助と同じ形・親 dir に contracts/schema.toml）に変異を当てて `folio check` を撃つ補助を足し、次を見る。
6. srs.yaml に最上位の figures を 1 枚（型 archify-architecture・欄そろい・refs [FR1]）足す → 合格（違反 0・まだ分からない 0）。
7. 型を一覧の外の字（mystery）に → 不合格 1 ∧「図の型」／ caption を消す → 不合格 1 ∧「caption」／ refs に「FR」（数字なし）→ 不合格 1 ∧「refs」／ 図の id を FR1 に → 不合格 1 ∧「重複」／ 行に未知の欄 extra を足す → 不合格 1 ∧「未知の欄」。
8. 既存の fixture 4 組の歯は不変。
`crates/folio/tests/face_adr.rs`・`crates/folio/tests/face_note.rs`・`crates/folio/tests/site.rs`: 本文も期待も変えない（(b) の共有で出力が byte 不変なことをこれらの凍結の歯で見る・verify の scope として write-set に載せる）。

(g) 便 33 までの形との接続: 新規 file は無い。`crates/folio/src/check.rs` は (a)（+60 行・余地 1,027）・`crates/folio/src/face.rs` は (b)（+60・余地 616）・`crates/folio/src/face_srs.rs` は (c)（+45・余地 130・共有の口を使うので図の枠の字面は持たない）・`crates/folio/src/face_adr.rs` と `crates/folio/src/face_note.rs` は (b) で縮む（-25 ずつ・write-set では中身を変える既存 file）・`crates/folio/src/adr.rs` は is_basis_id を pub(crate) にする 1 行・`tests/fixtures/face/srs.yaml` と `tests/fixtures/face/expected-srs.html` は (d)・`crates/folio/tests/face.rs` は (e)(f)（+150・余地 661）・`crates/folio/tests/check.rs` は (f)（+110）。`figure.rs`・`note.rs`・`site.rs`・`render.rs`・`parts.json`・`folio.css`・`vendor/`・実の要件書・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の src の file 1 本あたりの増分は check.rs +60・face.rs +60・face_srs.rs +45・adr.rs +1 で、face_adr.rs と face_note.rs は縮む（器の受付は src の余地を size の見積と比べる: face_srs.rs の余地 130 は S の 100 以上・tests/ の増分は器の測りの外・admin の事前読み 2026-09-19）。

## 2. 範囲

- 入れる: 床の節と行・共有の図の枠・要件書の面の章 09・fixture と歯。
- 入れない: 実の要件書と判断の記録への図の記入（planner の PR・往復は台帳）・図 1〜3（帯の型）の道具化・要件書の版上げ（要件は変わらない・節の追加は床の定数）・図の雛形・walk 承認（着地後）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床 | figures の節と行 |
| shared | 共有の枠 | figure_panel・figure_body・FIGURE_LABELS |
| chapter | 章 09 | 要件書の面の図 |
| copies | 写し | tests/face.rs に図の道具 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない。図の道具の写しと Node.js は受け皿（便 30）のまま。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ai"
title = "要件書の正本に任意の図の節（figures）を持てるようにし（床）、要件書の面に図の章 09 を足す。図の枠の生成は face.rs に 1 つ持ち 3 面で共有する（出力は byte 不変）"
req = ["FR15", "FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/src/face.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/adr.rs", "tests/fixtures/face/srs.yaml", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face.rs", "crates/folio/tests/check.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --test face face", "cargo nextest run -p folio --test check check", "cargo nextest run -p folio --test face_adr face_adr", "cargo nextest run -p folio --test face_note face_note", "cargo nextest run -p folio --test site site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（再凍結・章 09 と toc・図なしは不変・通らない図で 2 と前の面・型外と道具の不在・実の正本）が緑、check の歯（写しの figures の合格と違反 5 種・既存 4 組は不変）が緑、face_adr の歯が期待不変で緑、face_note の歯が期待不変で緑、site の歯が再凍結に追従して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
