# 設計: 便 25 — 判断の記録の面を正本から `folio face --face adr --id ADR-n` で生成する（4 面目・ADR-7 決定 (1)〜(4)）

- 要件: FR16（判断の記録 1 本につき人が読むページ 1 枚を 3 面と同じ単一の生成器と単一の design token で・中身は正本の欄から逐語・採用と却下は同じ形・提案中の印・廃止は後継への導線・根拠の id の行き先が無ければ「まだ分からない」の印）/ NFR2（部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.1・P-2.4（人が読むページは 1 つの生成器から・部品は閉じた一覧）/ P-6.1・P-6.2・P-6.3（正本は YAML・生成物を手で直さない・数は数えて出す）/ P-4.1・P-4.2（黙って飛ばさない・判定できないものは「まだ分からない」）/ P-10.1・P-10.2（独立した凍結 anchor）/ P-12.2（承認欄は逐語と日付）
- 判断の記録: ADR-7（2026-09-18 発効・決定 (1) 1 本 1 枚・(2) 手書きのまま部品は最小・(3) 逐語で組み案は同じ形・(4) 受入 = 名札の一致 + walk）。ADR-5 決定 (3)（手書き・外部 crate なし・部品目録を型で閉じる）。便 24（f2-648.38）の後に直列で置く。入口の棚の導線と build / serve への追加は便 26（別の行）で運ぶ＝本便は面 1 枚の生成器と歯だけ。
- 裁定: 持ち主 2026-09-18「推奨で承認する」（ADR-5 の撤退条件 ① = 手書きのまま・f2-648 notes）・「承認する」（ADR-7 発効・f2-648.39 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 z が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

判断の記録の正本（`design-intent/adr/ADR-n.yaml`・欄の決まりは `design-intent/adr/schema.yaml`）1 本から、人が読むページ 1 枚を便 14〜16 の `folio face` と同じ規律で生成する。生成物の文字列は (α) 正本の欄の値を escape して逐語で差し込んだもの／(β) 生成器が閉じた表として持つ名札／(γ) 正本から数えた数 の 3 種だけ。判断の記録だけの決まり（ADR-7 決定 (3)）: 採用した案と退けた案を同じ形（同じ部品・同じ欄の並び）で出す／状態が proposed の記録は見出しの帯と状態の行に「提案中・拘束力なし」を出す／retired の記録は後継の id（superseded_by）へのリンクを出す／根拠の id は行き先の面の該当の場所へ跳ぶリンクにし、行き先の無い id はリンクにせず id の直後に「（まだ分からない）」を添える。本便は面 1 枚の生成器・部品目録の faces の追加・凍結 fixture・歯だけで、入口の棚の導線と build / serve の出力への追加（便 26）・図の型の追加（無し）・folio.css の変更（無し）は含まない。

planner の実測（2026-09-18・main f3f300c）: 判断の記録の正本は ADR-1〜ADR-7 の 7 本で全部 accepted（承認欄あり）・amends は全部空・grill は無し・supersedes / superseded_by は無し・note は ADR-6 と ADR-7 が持つ。basis の id の種類は 条（P-n・A-n・N-n）・rules 行（R-n・D-n）・要件（FR / NFR / AC / CON）・判断の記録（ADR-n）の 4 種で、7 本の basis は全部 床（folio check・adr.rs の参照の解決）で実在が確かめられている。部品目録 `design-intent/preview/parts.json` の部品は 30（Component の ALL の数は parts.rs の unit の歯が 30 と固定）。`folio.css` に判断の記録だけの class は無い（`.adr` `.option` `.verdict` 等は 0 件）＝本便は css を変えず、既存 3 面が使う class だけで組む。

(a) 命令の口。`crates/folio/src/main.rs` の Face に旗 `--id <ADR-n>`（任意・文字列）を足す。`crates/folio/src/face.rs` の関数 run は引数に id（無いことも許す）を 1 つ増やし、面の名が adr なら id が要る（無ければ 2・「--id が無い」）、adr 以外の面に id が付いていれば 2（「--id は面 adr にだけ付く」）。面の名 adr の生成器は `crates/folio/src/face_adr.rs` の関数 derive（引数は dir と id）。run の「どれでもない」の文言は「index・constitution・srs・adr のどれでもない」に直す（歯 face.rs の既存の期待は「どれでもない」を含むことだけ＝不変）。`--out` の扱い（相対なら --dir からの相対）・--write / --check の 3 値（無い 2・不一致 1・一致 0）は既存の面と同じ。

(b) 正本の読み。face_adr.rs は face.rs の関数 load で `adr/<id>.yaml` を読む（id は `ADR-` + ASCII の数字列 1 つ以上の形でなければ 2・「id の形でない」・file が無ければ 2）。同じ dir の `constitution.yaml`・`rules.yaml`・`srs.yaml` も load で読み、根拠の id の行き先の解決に使う（読めなければ 2・便 15 と同じ）。読む欄は欄の決まりの required（id・title・status・date・context・decision・options・basis・retreat・plain）と optional（amends・grill・approval・consequences・supersedes・superseded_by・note）。file の id と `--id` が違えば 2。無い欄・型違いは face.rs の X の文言つきの Err ＝ 2（便 14 と同じ）。options は 1 つ以上・各案は id・name・text・verdict・reason の 5 欄。本便は欄の決まりの規則（採用は 1 つ・案は 2 以上・承認者の値域）を数えない（床の領分・二重に持たない P-6.3）。

(c) 名札の表（β・表に無い値は導出できない = 2）:
- 状態（status）→ 名札と状態の行: proposed →「提案中・拘束力なし」／「提案中・拘束力なし → 持ち主の承認で発効」・accepted →「発効」／「発効・拘束力あり（承認 <approval.date>）」（approval が無ければ 2・「accepted に承認欄が無い」）・retired →「廃止」／「廃止 → 後継 <superseded_by へのリンク>」（superseded_by が無ければ 2）。
- 案の判定（verdict）→ 名札: adopted →「採用」・rejected →「退けた」。
- 撤退条件の種類（retreat.kind）→ 名札: spike →「小さな試し」・measure →「数えた値」・ruling →「持ち主の裁定」。
- 根拠の id の種類 → 行き先: `P-<数>`・`A-<数>`・`N-<数>` → `constitution.html#<id を小文字に>`（constitution.yaml の articles の id に在ること）／`R-<数>`・`D-<数>` → `constitution.html#<小文字>`（rules.yaml の rows の id に在ること）／`FR<数>`・`NFR<数>`・`AC<数>`・`CON<数>`・`GOAL<数>` → `srs.html#<小文字>`（srs.yaml の requirements / nonfunctional / acceptance / constraints / goals の id に在ること）／`ADR-<数>` → `adr-<数>.html`（同じ dir の `adr/ADR-<数>.yaml` が在ること）。在れば a 要素（class は xref・href は行き先・字は id）、無ければ id の字の直後に「（まだ分からない）」（リンク無し・class 無し）。上の 4 形のどれでもない id は 2（「根拠の id の形でない」）。
- 章の名と帯: 01 問題（context）・02 決定（decision）・03 案（options）・04 根拠と撤退条件（basis・retreat）・05 改訂と帰結（amends・consequences・grill・supersedes・superseded_by・note）・承認欄（approval）。帯の class は band-1・band-2・band-3・band-4・band-5 の順、kicker の絵記号は要件書の面（face_srs.rs の BANDS）の章 01〜05 の svg を写す。

(d) 面の組み立て（face_adr.rs）。骨格は要件書の面と同じ順で、便 15 で face.rs に寄せた共有の口（Frame の関数 head・toc・band・approval_band・foot・dc と関数 card・hint・esc・anchor）を使う。Frame は const で name「判断の記録」・source「adr/ADR-n.yaml」（字面そのまま・foot の 1 行に出る）・favicon は要件書の面の FRAME の favicon の字面を写す・current は NAV の要素数（3）＝どの nav にも aria-current が付かない（NAV は 3 のまま・face.rs は型を変えない）・first 1・bands は (c) の 5 本・prev は（srs.html・要件書）・next は（index.html・入口）・parts は (e) の一覧。
- head: title「folio2 — 判断の記録 <id>（<状態の名札>）」・generated は date・version は id・status は状態の名札。here の行は Frame が出す「判断の記録 ▸ 全 6 章 — 目次へ」（章 5 + 承認欄）。
- cover（doc-cover-band・要件書の面の関数 cover と同じ字面の組 = cover-eyebrow / doc-type / h1 / summary-card / ic / lab / txt / cover-meta / m / k / v / cover-status）: eyebrow は「判断の記録 (ADR)」+「folio2 — <id>」・h1 は title・summary-card の lab は「やさしく言うと」・txt は plain・cover-meta は 状態（名札）・日付（date）・案「<数> 件（採用 1・退けた <数>）」（数は options から数える・採用の数は adopted の数をそのまま出す）・根拠「<数> 件」（basis の数）・改訂「<数> 件」（amends の数・欄が無ければ 0）・撤退条件（種類の名札）・cover-status は (c) の状態の行 +「（承認欄へ）」のリンク。
- toc: 章 01〜05 と承認欄（Frame の toc）。
- 章 01・02: 帯の h2 は「何が問題か」「何を決めたか」・chapbody の中に p 1 つ（class 無し）で context / decision を逐語で。
- 章 03 案: 各案を item-row 1 つで出す（要件書の面の関数 item_row と同じ属性の組 = article の data-component と id・ir-head / rid / rt / badges / pill・norm・plain / pk・details.machine）。id は `opt-<案の id>`・rid は「案 <案の id>」・rt は name・badges は pill 1 つ（判定の名札）・norm は text・plain の pk は「理由」で本文は reason・machine の dl は verdict の正本の値。採用と退けた案で部品も欄の並びも変えない（違いは pill の字だけ）。meta-chips は出さない。
- 章 04: basis は ul（class 無し）の li に (c) の行き先のリンク（無ければ「（まだ分からない）」）を basis の順で。撤退条件は section-lead-callout（style は `--band-n:4`）の中に face.rs の関数 card で 1 枚（class「card」・cid「撤退条件（<種類の名札>）」・本文は p に condition）。
- 章 05: amends が空か無ければ p「条文の改訂なし」、在れば ul の li に「<version> <target>.<field>: 「<previous_text>」→「<new_text>」」（render.rs の判断の記録の節と同じ字面）。consequences が在れば h3「この判断で変わること」+ ul の li に各項。grill が在れば h3「反対側からの確認」+ p「<when>・<who>・<where>」+ p summary。supersedes / superseded_by が在れば p「置き換えた判断 / 後継の判断: <adr-<数>.html へのリンク>」。note が在れば h3「注」+ p。
- 承認欄: Frame の approval_band（h2「承認」・lead は状態の名札）→ chapbody → approval-block（要件書の面の関数 approval と同じ字面の組 = sign / role / who / when / stamp）。approval が在れば sign 1 つ（role「承認」・who・when は date・when に「逐語「<verbatim>」」・stamp は ruling）、無ければ approval-block の中に p「未（提案中・持ち主の逐語と日付が入ると発効）」。
- foot: Frame の foot（version は id・generated は date・機械のための面の dl は id / status / date / basis（「・」で連結）/ amends の数）。字下げ無し・部品ごとに改行 1 つ・末尾に改行 1 つ・escape と id の規則は便 14 と同じ。

(e) 部品目録。`design-intent/preview/parts.json` の 7 部品の faces に adr を足す: freshness-stamp・font-size-control・doc-cover-band・approval-block・chapter-deck-band・section-lead-callout・item-row の 7 つ（+ 便 14〜16 の面が使う lane-chip は足さない）。部品は足さない（30 のまま・parts.rs の unit の歯の 30 は不変）。`crates/folio/src/parts.rs` の FACES を 4 つ（index・constitution・srs・adr）にし、`--page adr=<path>` を受ける（「どれでもない」の文言は 4 つの列挙に）。build.rs は触らない（部品目録から Component を組み立て時に導出する既存の仕組みが faces の追加を拾う）。`folio parts --print` は部品ごとの faces を出し、歯 parts.rs はその出力を凍結目録 `tests/fixtures/floor/parts-catalog.json` と byte 一致で比べるので、凍結目録の同じ 7 部品の faces にも adr を足す（それ以外の byte は変えない・admin の事前読み 2026-09-18）。face_adr.rs の PARTS はこの 7 つで、Frame の dc は一覧に無い部品を debug で止める（既存と同じ）。

(f) 凍結 fixture（P-10.1・最小の手書き・正本の写しにしない）: `tests/fixtures/face/adr/ADR-2.yaml`（proposed・approval 無し・options 2 = a adopted / b rejected・basis = [P-1, A-1, R-1, FR1, AC1, ADR-1]（fixture の憲法・rules・要件書に在る id と、同じ dir の ADR-1）・retreat kind ruling・amends 空・consequences 2 項・本文は日本語だけ）と、その期待 `tests/fixtures/face/expected-adr.html`。既存の `tests/fixtures/face/adr/ADR-1.yaml`（2 行・入口の面の数え用）は触らない（入口の歯 face.rs は adr/ADR-1.yaml だけを写すので、ADR-2.yaml を足しても入口の期待は変わらない）。AC14 の赤の fixture `tests/fixtures/face/adr/extra-class.html`（手書き・数十行）: 判断の記録の面の骨格（site-bar・doc-cover-band・item-row 1 つ）に部品目録に無い class を 1 つ（`not-in-catalog`）持つ。

(g) 歯 `crates/folio/tests/face_adr.rs`（binary 経由・関数名はすべて face_adr を含める＝verify の filter 語・`--test face_adr` の scope）。写しの作り: fixture の 4 file（constitution・rules・vocabulary・srs）+ `adr/ADR-1.yaml` + `adr/ADR-2.yaml` を一時 dir へ写す（歯 face.rs の関数と同じ形を自前に持つ・face.rs は触らない）。
1. 写しで `--face adr --id ADR-2 --write` = 0 ∧ 出力が expected-adr.html と byte 一致。
2. title に `<b>` を入れた写し = 0 ∧ 出力に `&lt;b&gt;` が在り `<b>` が無い。
3. 実の正本 7 本（ADR-1〜ADR-7）をそれぞれ `--write` = 0 ∧ `folio parts --check --dir design-intent --page adr=<出力>` = 0 ∧ stdout に「違反 0」（AC14 の緑側・NFR2）。
4. 実の正本 7 本の census: h1 が title の逐語（escape 後）・item-row の数が options の数・pill の「採用」が 1 つ・cover-status に状態の名札・basis の数だけ xref のリンクが在り「（まだ分からない）」が 0。
5. `--check` の 3 値: 面が無い 2・1 byte 変えた面 1・一致 0。
6. `--face adr` に --id 無し = 2 ∧「--id が無い」／`--id ADR-9`（無い）= 2／`--face srs --id ADR-2` = 2 ∧「--id は面 adr にだけ付く」／`--id foo` = 2 ∧「id の形でない」。
7. 写しの status を「draft」に = 2 ∧「状態」。
8. 写しの案 b の verdict を「maybe」に = 2 ∧「判定」。
9. 写しの basis を文字列に = 2。
10. 写しの basis に FR9（fixture の要件書に無い）を足す = 0 ∧ 出力に「FR9（まだ分からない）」∧ その id のリンクは無い。
11. 写しの status を accepted にし approval（who 持ち主・date・ruling・verbatim・surface R-8）を足す = 0 ∧ cover-status に「発効・拘束力あり（承認 <date>）」∧ approval-block に sign 1 つ／status を retired にし superseded_by ADR-1 を足す = 0 ∧「廃止」∧ `adr-1.html` へのリンク。
12. `folio parts --check --dir design-intent --page adr=tests/fixtures/face/adr/extra-class.html` = 1 ∧ stdout に「部品目録に無い class「not-in-catalog」」（AC14 の赤・要件書 AC14 の red_test）。
13. `--write` と `--check` を両方付ける = 2（clap の group・既存の面と同じ）。
- 既存の歯: `crates/folio/tests/parts.rs` は write-set に在るが本文は変えない（FACES が 4 になっても既存の歯は面 3 つの path しか渡さない）。期待のうち凍結目録の fixture だけ (e) のとおり 7 部品の faces に adr を足す。歯 face.rs（1,597 行・正規化）は write-set に載せず本文も fixture も触らない。unit は置かない。

(h) 便 24 までの形との接続: 新規は `crates/folio/src/face_adr.rs`（見積 600〜800 行）・歯 `crates/folio/tests/face_adr.rs`・fixture 3 本。`crates/folio/src/face.rs` は run の引数 1 つと adr の腕と文言（数行）・`crates/folio/src/main.rs` は旗 --id と Face の doc の 1 行と run の呼び出し・`crates/folio/src/parts.rs` は FACES の 1 語と文言 1 行・`design-intent/preview/parts.json` は 7 部品の faces。`face_srs.rs`・`face_constitution.rs`・`face_index.rs`・`site.rs`（build の出力は 5 のまま）・`render.rs`・`adr.rs`・`build.rs`・`folio.css`・`folio-ui.js`・`Cargo.toml`・`.github/workflows/` は触らない。face_srs.rs の非公開の関数（item_row・cover・approval・article_link・xref・meta_span）は呼ばず、同じ字面を face_adr.rs に自前で持つ。外部 crate は増えない。正規表現は使わない。size は M = 中身を変える既存の file 1 本あたりの増分は小さい（face.rs 十数行・main.rs 数行・parts.rs 2 行・parts.json 7 行）が、新規の face_adr.rs が大きい。

## 2. 範囲

- 入れる: 面 adr の生成器・--id の口・部品目録の faces・凍結 fixture・AC14 の赤 fixture・歯 13 本。
- 入れない: 入口の棚から各ページへの導線と build / serve の出力への追加（便 26）・walk 承認（便 26 の後・AC5 の形）・図の型・css の変更・暫定の読み物（readable.html）の退役（器の s2-07l.467 の後に A-1 で問う）・欄の決まりの規則の二重化。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| gate | 命令の口 | --face adr と --id・3 値 |
| read | 読み | adr/<id>.yaml + 憲法・rules・要件書 |
| label | 名札の表 | 状態・判定・撤退の種類・根拠の行き先 |
| page | 組み立て | cover・5 章・承認欄・foot |
| catalog | 部品目録 | 7 部品の faces に adr |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 24 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "z"
title = "判断の記録の面を正本から folio face --face adr --id ADR-n で生成する（4 面目・部品目録の faces・凍結 fixture）"
req = ["FR16", "NFR2"]
section = "1"
write-set = ["+crates/folio/src/face_adr.rs", "+crates/folio/tests/face_adr.rs", "crates/folio/src/face.rs", "crates/folio/src/main.rs", "crates/folio/src/parts.rs", "crates/folio/tests/parts.rs", "design-intent/preview/parts.json", "tests/fixtures/floor/parts-catalog.json", "+tests/fixtures/face/adr/ADR-2.yaml", "+tests/fixtures/face/adr/extra-class.html", "+tests/fixtures/face/expected-adr.html"]
verify = ["cargo nextest run -p folio --test face_adr face_adr", "cargo nextest run -p folio --test parts parts", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "face_adr の歯 13 本（凍結 fixture との byte 一致・escape・実の正本 7 本で parts 合格・census・check の 3 値・id の口 4 形・状態と判定の表外・basis の型・未解決の根拠の印・accepted と retired の状態・AC14 の赤 fixture・mode）が緑、parts の歯は本文不変・凍結目録の fixture に adr を足して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
