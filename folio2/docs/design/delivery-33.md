# 設計: 便 33 — 判断の記録の正本に図の節を持てるようにし、判断の記録の面に図の章を足す（FR15 / FR16 / NFR2・ADR-4 決定 (1)(3)）

- 要件: FR15（文書を生成するとき、正本の図の節に型付き記述があれば図の道具で検査と描画に掛け、図の本体だけを埋める・通らない図は生成せず前の生成物も上書きしない）/ FR16（判断の記録の面を正本から逐語で生成する）/ NFR2（部品目録に無い class は 0）
- 条: P-2.1（生成器は 1 つ）/ P-2.4（閉じた一覧）/ P-5.1・P-6.3（欄の決まりは型付きデータ・床の定数と 1 字も違わない写し）/ P-4.1（読めなかった正本を「無い」と扱わない）/ P-10.1（凍結 anchor）
- 判断の記録: ADR-4 決定 (1)（図の正本は、その図を載せる文書の正本 YAML の図の節に型付き記述で置く＝文書を限定しない）・決定 (3)（図の本体だけを埋める・意味の属性は残す）。ADR-7（判断の記録の面・1 本 1 枚・逐語）。便 32（f2-648.47）の後に直列で置く。要件書の面への図（便 34）は別の行。
- 裁定: 持ち主 2026-09-19「では図の追加の方向で進めて。」（判断の記録から・要件書は後・planner の推奨に対して）。新しい判断の記録は要らない（ADR-4 決定 (1) の範囲）。walk 承認は図を実の判断の記録に入れた後（planner の別 PR・契約の外）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ah が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

設計ノート（便 28〜32）と同じ規律で、判断の記録（`design-intent/adr/ADR-<数>.yaml`）にも任意の図の節（figures）を持てるようにし、判断の記録の面（`folio face --face adr`）に図の章（章 06「図」・承認欄の前）を足す。本便は (1) 欄の決まりと床（任意の欄 figures・行の欄・型の一覧）、(2) 面の図の章（設計ノートの面と同じ枠・同じ生成器）、(3) 部品目録の figure-panel を判断の記録の面にも、(4) 凍結 fixture と歯、の 4 つ。実の判断の記録に図を書くのは本便の外（planner の PR・往復は台帳に記帳）。

planner の実測（2026-09-19・main 0c2c1bd）: 判断の記録の欄の決まり `design-intent/adr/schema.yaml`（正規化 166 行）の schema 節は床 adr.rs（正規化 959）の定数の写しで、required 10 欄・optional 7 欄（amends・grill・approval・consequences・supersedes・superseded_by・note）・未知の欄は check_keys が「未知の欄（N-3）」で落とす・*_note で終わる欄は strip_notes が外す。同じ optional の一覧を持つ写しが歯の fixture に 5 本（tests/fixtures/adr/schema-drift・two-adopted・effective-no-approval・tests/fixtures/anchor/no-anchor・tests/fixtures/link/retreat-kind-drift の各 adr/schema.yaml・42 行）在り、床の定数を変えれば全部が「欄の決まりの写しが床と違う」で落ちる（schema-drift は options_rule.min だけで落ちる形を保つ）。判断の記録の面 face_adr.rs（正規化 692）は FRAME を const で持ち（bands 5 本・CHAPTERS と H2 が 5 章固定・toc が固定）、章は 問題・決定・案・根拠と撤退条件・改訂と帰結 の 5 つ + 承認欄・PARTS は 7 部品（figure-panel 無し）・関数 id_link（dir・ctx・X）が根拠の id を 4 形 + 判断の記録に解く。設計ノートの面 face_note.rs（正規化 1,049）の関数 figures_chapter は figure.rs の関数 render（pub・引数は dir・図 id・型の字・spec の X）で図の本体を得て figure-panel に埋め、型の名札は非公開の定数 FIGURE_LABELS（5 組）・帯は BANDS の 12 本の切り出し。face_srs.rs の BANDS は band-1〜6 の 6 本（band-6 の絵記号あり）。部品目録の figure-panel の faces は index・constitution・srs・note（adr 無し）・凍結目録 tests/fixtures/floor/parts-catalog.json も同じ。tests/face_adr.rs（正規化 671・歯 19）の fixture_copy は正本 4 file + adr/ADR-1.yaml（2 行の stub）+ adr/ADR-2.yaml（19 行・最小の手書き・図なし）を写し、図の道具は写さない。tests/site.rs と tests/serve.rs の写しは便 31 で図の道具を写している。凍結 expected-adr.html（章の帯 5）と expected-site-adr-2.html は ADR-2 の面。tests/adr.rs（65 行）は fixture dir 3 組で違反 1 件ずつを見る（design-intent の写しを作る補助は無い・tests/note.rs の Work が copy_tree と git init で持つ）。readable.html の生成（render.rs）は判断の記録の欄を名指しで読む（plain・context・decision・options・retreat）ので figures を足しても導出物は変わらない。入口の面（face_index.rs の records）は id・title・status・date だけ読む。実の判断の記録 7 本はいずれも図を持たない。

(a) 欄の決まり（`design-intent/adr/schema.yaml`）。schema 節の optional の末尾に figures を足す（[amends, grill, approval, consequences, supersedes, superseded_by, note, figures]）。schema 節に figures の節を足す: `figures: {entry: {required: [id, type, caption, spec], optional: [refs, note]}, type_enum_ref: design-intent/preview/parts.json figure_type_enum}`（値は床の定数の写し・設計ノートの欄の決まりの figures.entry と同じ形）と、figures_note（*_note・人が読む説明: 図の正本は判断の記録の図の節に型付き記述で置く（ADR-4 決定 (1)）・型は部品目録の図の道具の 5 型・refs は basis と同じ id の形で行き先の解決は面が「まだ分からない」で表す・設計ノートと同じく手順図（非エンジニア向け）と順序図（エンジニア向け）の対を指針とする）。歯の fixture の写し 5 本（上の実測）の optional の一覧にも同じ figures を足し、figures の節（entry・type_enum_ref）も同じ字面で足す（*_note は写しに持たない・schema-drift は options_rule.min の変異を保つ）。

(b) 床（`crates/folio/src/adr.rs`）。RECORD.optional の末尾に figures。FLOOR の写像に figures の節（entry の required / optional・type_enum_ref の値）を足す（(a) と 1 字も違わない）。check_fields で figures が在れば: 一覧でなければ「figures が一覧でない」・各行は entry の欄の集合を check_keys で（未知の欄・欠落）・id と caption は非空・type は部品目録の FigureType（型の一覧に無ければ「図の型「<型>」が部品目録の一覧に無い」）・spec は表・refs が在れば各項は is_basis_id の形（違えば「refs の id の形」）・図の id は 1 本の記録の中で一意。行き先の解決は床では数えない（面が「まだ分からない」で表す・便 25 の根拠と同じ）。違反の種類は他の判断の記録の欄と同じ adr。

(c) 面（`crates/folio/src/face_adr.rs`）。FRAME を const から関数 derive の中で組む形にし（face_note.rs と同じ）、BANDS を 6 本にする（6 本目は band-6 と face_srs.rs の BANDS の band-6 の絵記号の字面）。figures（任意・無ければ今の形と byte 不変）が 1 つ以上のとき: toc に 06「図」（k「図」・t「<数> 枚」）を承認の前に足し、章 06 を「改訂と帰結」の後・承認欄の前に置く。章 06 の帯は h2「図 <数> 枚」・kicker「図」。中身は設計ノートの面の関数 figures_chapter と同じ字面（図ごとに figure-panel: data-role diagram・id は図の id・fig-title の fn「図 <i>」と caption・fig-tools の zoom-btn・zoom-close・図の本体は figure.rs の関数 render の戻り値をそのまま・figcaption の ver「図 <i> · <型の名札> · <図の id> · 根拠: <refs の各 id を関数 id_link で>」・refs が無いか空なら「根拠:」以降を出さない）。型の名札は face_note.rs の FIGURE_LABELS を pub(crate) にして使う（表を二重に持たない・face_note.rs の変更はこの 1 行）。図が 1 枚でも導出できなければ面全体が 2（全部か無しか・前の面は残る）。PARTS に figure-panel を足す（7 → 8）。cover-meta に「図 <数> 枚」を足す（既存の項の末尾・0 枚なら出さない＝図なしの面は byte 不変）。foot の機械の面の dl に figures（数）を足すのも 1 枚以上のときだけ。他の章は不変。

(d) 部品目録（`design-intent/preview/parts.json`・`tests/fixtures/floor/parts-catalog.json`）。figure-panel の faces に adr を足す（index・constitution・srs・adr・note の順）。凍結目録も同じ。

(e) 凍結 fixture。`tests/fixtures/face/adr/ADR-2.yaml` に図を 1 枚足す（figures: id fig-1・type archify-architecture・caption「見本の図」・refs [FR1]・spec は 2 箱の構成図で layout grid〔tests/fixtures/face/design-note/full.yaml の fig-1 と同じ形〕）。`tests/fixtures/face/expected-adr.html` と `tests/fixtures/face/expected-site-adr-2.html` を (c) の形で再凍結する（差分は toc の 06・cover-meta の図・章 06・foot の figures だけ＝他の章は byte 不変を歯で見る）。ADR-1.yaml（stub）は触らない。

(f) 写し。tests/face_adr.rs の fixture_copy に repo の `vendor/archify/` を一時 dir（src/ の親）の `vendor/archify/` へ再帰で写す（tests/figure.rs と同じ形）。tests/site.rs と tests/serve.rs は便 31 で写しているので変えない。

(g) 歯。`crates/folio/tests/face_adr.rs`（関数名は face_adr を含める・`--test face_adr` の scope）: 既存 19 本は再凍結に追従（census の歯は章の帯を 5 でなく「5 + 図の有無」で数える）。次を足す。
1. 写し（ADR-2・図 1 枚）で toc に 06「図」∧ figure-panel 1（data-role diagram・id fig-1）∧ svg 1 ∧ fig-title に caption ∧ figcaption の ver に「構成図（architecture）」と「根拠: 」と FR1 のリンク ∧ cover-meta に「図 1 枚」。
2. 写しの figures を消した面で 章 06・figure-panel・cover-meta の図が 0 ∧ その面が便 32 までの形と同じ（歯の中で図ありの面から章 06 と toc の 06 と cover-meta の図と foot の figures を切り出して除いた残りと byte で同じ）。
3. 写しの spec から layout を消す → --write = 2 ∧「図の道具の検査を通らない」∧ 先に書いた面の byte が不変。
4. 写しの図の type を pipeline-rail に → 2 ∧「図の道具の型でない」／ 親 dir の道具を消す → 2 ∧「図の道具が無い」。
5. 実の正本 7 本で figure-panel の数が各正本の figures の数（実測 0）と一致 ∧ parts --check 合格（既存の歯に足す）。
`crates/folio/tests/adr.rs`（関数名は adr を含める・`--test adr` の scope）: design-intent の写し（copy_tree + git init・tests/note.rs の Work と同じ形・親 dir に contracts/schema.toml）に変異を当てて `folio check` を撃つ補助を足し、次を見る。
6. ADR-4.yaml に figures を 1 枚（型 archify-architecture・欄そろい）足す → 合格（違反 0・まだ分からない 0）。
7. 型を pipeline-rail に → 不合格 1 ∧「図の型」／ caption を消す → 不合格 1 ∧「caption」／ refs に「FR」（数字なし）→ 不合格 1 ∧「refs」／ 図の id を 2 つ同じに → 不合格 1 ∧「図の id」。
8. 既存の fixture dir 3 組は写しの optional の追従で今までの 1 件だけが出る（既存 3 本の歯・不変）。
`crates/folio/tests/site.rs`（関数名は site を含める）: 本文不変・凍結 expected-site-adr-2.html の再凍結に追従（write-set に載せるが変えない）。`crates/folio/tests/parts.rs`: 本文不変・凍結目録に adr（載せるが変えない）。`crates/folio/tests/face_note.rs`: 本文不変（pub(crate) の 1 行・verify の scope）。`crates/folio/tests/anchor.rs`・`tests/link.rs`・`tests/note.rs`: 本文不変（fixture の写しの追従だけ・verify は common-verify が回す）。

(h) 便 32 までの形との接続: 新規 file は無い。`crates/folio/src/adr.rs` は (b)（+70 行・余地 541）・`crates/folio/src/face_adr.rs` は (c)（+90・余地 808）・`crates/folio/src/face_note.rs` は 1 行・`design-intent/adr/schema.yaml` と fixture の写し 5 本は (a)・`design-intent/preview/parts.json` と凍結目録は (d)・`tests/fixtures/face/adr/ADR-2.yaml` と凍結 2 本は (e)・`crates/folio/tests/face_adr.rs` は (f)(g)（+150・余地 829）・`crates/folio/tests/adr.rs` は (g)（+120）。`figure.rs`・`note.rs`・`face.rs`・`site.rs`・`render.rs`・`folio.css`・`vendor/`・実の判断の記録 7 本・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は M。

## 2. 範囲

- 入れる: 欄の決まりと床の figures・面の図の章・目録の adr・fixture と歯。
- 入れない: 実の判断の記録への図の記入（planner の PR・往復は台帳）・要件書の面の図（便 34）・図の雛形（別の便）・walk 承認（着地後）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| schema | 欄の決まり | optional に figures・entry の欄 |
| floor | 床 | figures の行の検査 |
| chapter | 図の章 | 章 06・figure-panel |
| copies | 写し | face_adr に図の道具・fixture の schema 5 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（fixture の写しの追従で anchor / link / note の歯も不変で回る）。

## 5. 依存

外部 crate は増やさない。図の道具の写しと Node.js は受け皿（便 30）のまま。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ah"
title = "判断の記録の正本に任意の図の節（figures）を持てるようにし（欄の決まり + 床）、判断の記録の面に図の章 06 を足す（設計ノートと同じ枠と生成器）"
req = ["FR15", "FR16", "NFR2"]
section = "1"
write-set = ["design-intent/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "crates/folio/src/adr.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "design-intent/preview/parts.json", "tests/fixtures/floor/parts-catalog.json", "tests/fixtures/face/adr/ADR-2.yaml", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-site-adr-2.html", "crates/folio/tests/face_adr.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/site.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/face_note.rs"]
verify = ["cargo nextest run -p folio --test face_adr face_adr", "cargo nextest run -p folio --test adr adr", "cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test parts parts", "cargo nextest run -p folio --test face_note face_note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "face_adr の歯（再凍結・図の章と toc と cover-meta・図なしは不変・通らない図で 2 と前の面・型外と道具の不在・実の正本）が緑、adr の歯（写しの figures の合格と違反 4 種・既存 3 組は不変）が緑、site の歯が再凍結に追従して緑、parts の歯が凍結目録の adr で緑、face_note の歯が本文不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
