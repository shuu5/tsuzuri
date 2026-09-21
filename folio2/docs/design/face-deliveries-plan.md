# 面の便の列 — 一括 10 の仕分け C（15 件）と台帳 f2-648.113 / .115

起草 2026-09-22・orchestrator 席の代理の起草役（AI・opus）・作業場所 .worktrees/planner-faces（branch docs/face-deliveries・main 81bc02c から切った）。
材料 = docs/design/batch10-triage.md の仕分け C の 15 行・台帳 f2-648.113 と f2-648.115・契約の見本 docs/design/delivery-76.md / 77.md / 78.md。
書いたもの = 便 79 から 86 の契約 8 本（docs/design/delivery-79.md から delivery-86.md）と本 file。契約表の id は便 78（ca）の次から cb・cc・cd・ce・cf・cg・ch・ci。

## 1. 便の列

| 便 | 契約 | 中身 | 候補（周・観点・id） | 触る src | size | 門 |
|---|---|---|---|---|---|---|
| 79 | cb | 数値の表の値の枡に日本語の小見出し・裁定の列の前の裁定を折りたたむ | 16 読みやすさ F-3（= 18 読みやすさ F-1）／19 読みやすさ F-5 | face_constitution.rs | S | 通す |
| 80 | cc | 条の欠番の行（P-9）・改訂来歴の行に前の文と理由 | 17 読みやすさ F-4／19 読みやすさ F-3 | face_constitution.rs | S | 通す |
| 81 | cd | 図の段の名札・受入の英字の欄名・承認欄の来歴の折りたたみ | 17 読みやすさ F-1／18 読みやすさ F-4／18 読みやすさ F-6 | face_srs.rs | S | 通す |
| 82 | ce | 判断の記録の面の注の折りたたみ・図の英語の日本語化 | 17 読みやすさ F-3／18 読みやすさ F-5 | face_adr.rs・figure.rs | S | 通す |
| 83 | cf | 天井の名札を印から読む・旗を外す・名札に説明の小窓 | 台帳 f2-648.113（16 実態 F-5）／17 読みやすさ F-5 | face.rs・stamp.rs・findings.rs・main.rs・site.rs・5 面 | S | 通す |
| 84 | cg | 機構の札にいつから動くかの意味・用語集に欄の名前の節 | 18 読みやすさ F-3／20 読みやすさ F-2 | face.rs・face_constitution.rs・face_srs_rtm.rs | S | 通す |
| 85 | ch | 規則の表の種別 deny の意味を下限と固定の値にも当たる字に | 16 整合 F-4 | rules.rs | S | **印を読む** |
| 86 | ci | 要件の行の欄の閉じた一覧を要件書の生成区間へ | 台帳 f2-648.115 | check.rs | S | **印を読む** |

合わせて 15 件のうち 14 件と、台帳 2 件を運ぶ。外した 1 件は §4。

## 2. 順と、順が要る理由

79 → 80 → 81 → 82 → 83 → 84 → 85 → 86。順を入れ替えてよい所と、入れ替えられない所は次のとおり。

- **80 は 79 の後**（79 が歯の新しい file crates/folio/tests/face_constitution.rs を置き、80 がそこに足す）。
- **84 は 79 と 81 の後**（84 の歯は tests/face_constitution.rs と tests/face_srs.rs の両方に入る）。
- **85 と 86 は互いに独立**で、79 から 84 とも独立。ただしどちらも design-intent の正本を書き換えるので、天井の門が 通す を返す時（印が新しく 4 観点とも合格の時）にまとめて出すのが得である。周を 1 回挟む余裕が無ければ、規則の表 D-12 のとおり持ち主の裁定を取ってから出す。
- **83 は 81 の後が望ましい**（83 が 5 面の呼びを 1 面 2 行ずつ直すので、face_srs.rs の直しが先に着地していた方が衝突が少ない）。必須ではない。
- 82 はどの順でもよい（face_adr.rs と figure.rs はほかの便が触らない）。

## 3. 受付の cap（src の余地・幅 120 で正規化・上限 1500）

実測（2026-09-22・main 81bc02c）と、便を順に当てたときの見通し。size S は余地 100 以上、M は 300 以上が要る。どの便も受付の時点で 100 を下回らないので、**関数を別の module へ移すだけの切り出しの便は要らない**。

| file | 実測の余地 | 79 の後 | 80 の後 | 81 の後 | 83 の後 | 84 の後 |
|---|---|---|---|---|---|---|
| face.rs | 194 | 194 | 194 | 194 | 約 174 | 約 159 |
| face_srs.rs | 195 | 195 | 195 | 約 165 | 約 163 | 約 163 |
| face_constitution.rs | 370 | 約 315 | 約 270 | 約 270 | 約 268 | 約 253 |
| face_index.rs | 209 | 209 | 209 | 209 | 約 207 | 約 207 |
| face_adr.rs | 699 | — | — | — | 約 694 | 約 694 |
| figure.rs | 944 | — | — | — | 約 899 | 約 899 |
| check.rs | 786 | — | — | — | — | — |
| rules.rs | 1241 | — | — | — | — | — |

8 本とも size は **S**。M にした便は 1 本も無い。いちばん大きいのは 79（face_constitution.rs に + 約 55 行）と 83（src を合わせて + 約 55 行・減る所もある）で、どちらも 100 行の見積に収まる。

歯の側の実測: crates/folio/tests/face.rs は 1637 行（正規化）で既に 1500 を超えている。受付の cap は src の各 file だけを測るので受付は通るが、これ以上太らせないために、便 79 と 81 は憲法の面と要件書の面の歯を新しい file（tests/face_constitution.rs と tests/face_srs.rs）へ分ける。入口・判断の記録・設計ノートの 3 面は既に 1 面 1 file の形を持つので、同じ形に揃えるだけで新しい作法は増えない。既存の歯は 1 本も移さない。

## 4. 外した候補（1 件）

| 周・観点・id | 一言 | 外した理由 |
|---|---|---|
| 18 実態 F-2 | day-1 の生成 script 3 本を scripts/retired/ へ可逆に移す | file の移動（= 元の path の削除）は便では運べない。器は write-set に在る file を書き換える形しか運ばないので、削除と移動は席の PR で行う。退役は可逆な移動としてのみ行う（N-1.2）ので、席が git mv で移し、要件書の側は直さなくてよい（仕分けの表の一言のとおり）。 |

重複の 1 件（18 周目 読みやすさ F-1）は 16 周目 読みやすさ F-3 と同じ項なので、便 79 の 1 件として数えた。

## 5. 門が掛かる便と、席の手番

- 便 85 と 86 は design-intent の下の正本（rules.yaml と srs.yaml）の生成区間を書き換えるので、受付の事前読みで `folio ceiling --gate` が印 design-intent/preview/ceiling-stamp.yaml を読む。規則の表 D-12 のとおり、**通す** でなければ器へ出さない。席は受付の直前に門を撃って 3 値を実測し、印を付け直す（周を回して `folio ceiling --stamp`）か、持ち主の裁定を取って台帳の notes に逐語と時刻を記帳する。
- 便 79 から 84 は crates/folio/ と tests/fixtures/face/ しか触らないので門は 通す を返す。
- 便 83 の着地の後、席が別の PR で直すもの: 要件書 FR18 の注（旗 folio face --ceiling と folio build --ceiling を名指している）・docs/design/delivery-40.md と delivery-42.md と docs/design/ceiling-gate.md の旗の言及。規範文と平易文は不変なので持ち主の手番は要らない（便 77 の §1 (i) と同じ運び）。
- 便 83 の着地の後、天井の周の手順が変わる: 「周を回す → `folio ceiling --stamp` で印を書く → `folio build` で面を出す」の順になる（面が印を読むので、印より先に面を出すと 1 周前の名札が出る）。旗 `--ceiling` は無くなる。
- 便 86 の着地の後、席が判断するもの: 判断の記録 ADR-11 の決定 (4) の列に本便の項が無いので、決定 (4) に足す注が要るか、判断の記録の注記で足りるか（台帳 f2-648.115 の断り）。発効済みの欄なので便では触らない。

## 6. 歯の接頭辞と置き場

`grep -rn 'fn f7[0-9]_\|fn f8[0-9]_' crates/folio/tests` の実測（2026-09-22）: 使われている最大は f78_（tests/schema.rs）。f79_ 以降は空いている。便の番号と歯の接頭辞を揃えた。

| 便 | 歯の接頭辞 | 置き場 |
|---|---|---|
| 79 | f79_ | 新規 crates/folio/tests/face_constitution.rs（6 本） |
| 80 | f80_ | crates/folio/tests/face_constitution.rs（5 本） |
| 81 | f81_ | 新規 crates/folio/tests/face_srs.rs（6 本） |
| 82 | f82_ | crates/folio/tests/face_adr.rs（2 本）・tests/figure.rs（3 本） |
| 83 | f83_ | crates/folio/tests/badge.rs（6 本） |
| 84 | f84_ | tests/face_constitution.rs（3 本）・tests/face_srs.rs（2 本） |
| 85 | f85_ | crates/folio/tests/schema.rs（2 本）・tests/face.rs（1 本） |
| 86 | f86_ | crates/folio/tests/schema.rs（2 本）・tests/check.rs（2 本） |

## 7. 凍結の写しの触れ方

面の字が変わる便は、凍結の写し tests/fixtures/face/expected*.html を同じ着地で生成し直して置き換える（手で直さない・便 70 と 74 と同じ運び）。どの便がどの写しを動かすか。

| 便 | 生成し直す写し |
|---|---|
| 79 | expected.html |
| 80 | expected.html |
| 81 | expected-srs.html |
| 82 | expected-srs.html・expected-adr.html・expected-site-adr-2.html・expected-note.html |
| 83 | 7 本すべて |
| 84 | expected.html・expected-srs.html |
| 85・86 | 動かさない（実の置き場の正本だけを触り、写しの正本は別の最小の手書きのため） |

凍結 anchor のうち手で置く写しは 2 本だけで、どちらも起草役が独立に組んで §1 に逐語と行数と byte 数と sha256 を書いた。

- 便 85: tests/fixtures/schema/rules-region.txt（27 行・1833 byte・sha256 54580596905e2d70d834c34553d3ad9000dd356473a0df0f60e8afe8fafe652c）
- 便 86: tests/fixtures/schema/srs-region.txt（29 行・1168 byte・sha256 3937340f77713b087b33ca43b8fe866ca310b97cd04e5e1f4ce9967e19b724e1）

図の凍結 anchor tests/fixtures/figure/anchor/body.svg は便 82 でも 1 byte も変えない（照合は日本語への置き換えの前の出力で行うため）。

## 8. 部品目録と様式

8 本とも design-intent/preview/parts.json と design-intent/preview/folio.css を 1 byte も触らない。新しい部品も新しい class も作らず、既存の部品（item-row・principle-amendment-history・glossary-term-table・approval-block・ceiling-stamp・figure-panel）と既存の class（hint・hint-btn・hint-body・vh・legend-line・note・am-row・am-meta・am-kick・rq-where）だけで組む。`folio parts --check` は面の class が folio.css に在ることを見る（crates/folio/src/parts.rs 233〜242 行）ので、この線を守るかぎり様式の追加は要らない。
