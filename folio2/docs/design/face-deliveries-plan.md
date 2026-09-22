# 面の便の列 — 一括 10 の仕分け C（15 件）と台帳 f2-648.113 / .115

起草 2026-09-22・orchestrator 席の代理の起草役（AI・opus）。
初版の作業場所 .worktrees/planner-faces（branch docs/face-deliveries・main 81bc02c）。
改訂 b（2026-09-22）の作業場所 .worktrees/planner-faces2（branch docs/face-split・main bb80fef）。
材料 = docs/design/batch10-triage.md の仕分け C の 15 行・台帳 f2-648.113 と f2-648.115・契約の見本 docs/design/delivery-76.md / 77.md / 78.md。
書いたもの = 便 79 から 87 の契約 9 本（docs/design/delivery-79.md から delivery-87.md）と本 file。契約表の id は便 78（ca）の次から cb・cc・cd・ce・cf・cg・ch・ci・cj。

改訂 b で変えたこと。① 受付の cap の測り方を器の式に直した（§3）。初版は空行を除いて数えていたので 100 行ほど甘く、便 83 と 84 が受付で断られた。② 切り出しの便 87 を足した（初版は「切り出しは要らない」と書いたが、器の式では face.rs の余地が 93 行しかない）。③ 便 83 と 84 を改訂 b にし、便 87 の後を前提にした。

## 1. 便の列

| 便 | 契約 | 中身 | 候補（周・観点・id） | 触る src | size | 門 | 状態 |
|---|---|---|---|---|---|---|---|
| 79 | cb | 数値の表の値の枡に日本語の小見出し・裁定の列の前の裁定を折りたたむ | 16 読みやすさ F-3（= 18 読みやすさ F-1）／19 読みやすさ F-5 | face_constitution.rs | S | 通す | 着地 |
| 80 | cc | 条の欠番の行（P-9）・改訂来歴の行に前の文と理由 | 17 読みやすさ F-4／19 読みやすさ F-3 | face_constitution.rs | S | 通す | 着地 |
| 81 | cd | 図の段の名札・受入の英字の欄名・承認欄の来歴の折りたたみ | 17 読みやすさ F-1／18 読みやすさ F-4／18 読みやすさ F-6 | face_srs.rs | S | 通す | 着地（改訂 b で受付） |
| 82 | ce | 判断の記録の面の注の折りたたみ・図の英語の日本語化 | 17 読みやすさ F-3／18 読みやすさ F-5 | face_adr.rs・figure.rs | S | 通す | 走行中 |
| **87** | **cj** | **face.rs の名札と名札の表を face_labels.rs へそのまま移す（振る舞い不変）** | — （受付の cap を空けるための切り出し） | face.rs・+face_labels.rs・main.rs | S | 通す | 未 |
| 83 | cf | 天井の名札を印から読む・旗を外す・名札に説明の小窓 | 台帳 f2-648.113（16 実態 F-5）／17 読みやすさ F-5 | face.rs・stamp.rs・findings.rs・main.rs・site.rs・5 面 | S | 通す | 改訂 b（87 の後） |
| 84 | cg | 機構の札にいつから動くかの意味・用語集に欄の名前の節 | 18 読みやすさ F-3／20 読みやすさ F-2 | face_labels.rs・face.rs・face_constitution.rs・face_srs_rtm.rs | S | 通す | 改訂 b（87 の後） |
| 85 | ch | 規則の表の種別 deny の意味を下限と固定の値にも当たる字に | 16 整合 F-4 | rules.rs | S | **印を読む** | 未 |
| 86 | ci | 要件の行の欄の閉じた一覧を要件書の生成区間へ | 台帳 f2-648.115 | check.rs | S | **印を読む** | 未 |

仕分け C の 15 件のうち 14 件と、台帳 2 件を運ぶ。外した 1 件は §4。便 87 は候補を運ばない（受付を通すための切り出し）。

## 2. 順と、順が要る理由

79 → 80 → 81 → 82 → **87** → 83 → 84。85 と 86 はこの列と独立。

- **80 は 79 の後**（79 が歯の新しい file crates/folio/tests/face_constitution.rs を置き、80 がそこに足す）。着地済み。
- **87 は 83 と 84 の前**（器は face.rs の余地を 93 行と測るので、face.rs に行を足す便は size S の見積 100 行に届かず断られる。87 が約 398 行に空ける）。
- **84 は 79・81・87 の後**（84 の歯は tests/face_constitution.rs と tests/face_srs.rs に入り、84 が足す関数は 87 が移した先の face_labels.rs に入る）。
- **83 は 87 の後**（face.rs に約 20 行足す）。83 と 84 の間の順はどちらでもよいが、83 → 84 が読みやすい（84 の見積が 83 の着地を織り込んである）。
- 便 84 の write-set は便 87 が作る crates/folio/src/face_labels.rs を名指す。便 87 の着地より前に `scribe2 contracts check` を撃つと、その 1 行が write-set-item-unresolved として出る（新規の印 + は付けない。この便では新規でないため）。便 87 が main に着地すれば所見は消えるので、受付はその後に行う。
- **85 と 86 は互いに独立**で、79 から 87 とも独立。どちらも design-intent の正本を書き換えるので、天井の門が 通す を返す時（印が新しく 4 観点とも合格の時）にまとめて出すのが得である。周を 1 回挟む余裕が無ければ、規則の表 D-12 のとおり持ち主の裁定を取ってから出す。
- 82 はどの順でもよい（face_adr.rs と figure.rs はほかの便が触らない）。

## 3. 受付の cap（器の式）

**器の行数の式**: 空行を含む全行を数え、1 行の字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す。余地 = 1500 − その値。size S は余地 100 以上、M は 300 以上が要る。

初版の本 file はこの式を「空行を除いて数え、幅 120 で折る」と誤って書いていた。実測で 100 行ほど甘く出るので、以後はこの節の式で測る。

実測（2026-09-22・main bb80fef・便 79 / 80 / 81 の着地後）と、便を順に当てたときの見通し。

| file | 生 | 折り返し | 計 | 余地 | 87 の後 | 83 の後 | 84 の後 |
|---|---|---|---|---|---|---|---|
| face.rs | 1397 | 10 | 1407 | **93** | 約 398 | 約 378 | 約 368 |
| face_srs.rs | 1383 | 15 | 1398 | **102** | 102 | 102 | 102 |
| face_index.rs | 1363 | 15 | 1378 | **122** | 122 | 122 | 122 |
| face_constitution.rs | 1265 | 24 | 1289 | 211 | 211 | 211 | 約 196 |
| findings.rs | 1144 | 0 | 1144 | 356 | 356 | 約 376 | 約 376 |
| face_note.rs | 1016 | 7 | 1023 | 477 | 477 | 477 | 477 |
| face_adr.rs | 846 | 7 | 853 | 647 | 約 644 | 約 644 | 約 644 |
| check.rs | 748 | 2 | 750 | 750 | 750 | 750 | 750 |
| figure.rs | 600 | 1 | 601 | 899 | 約 854 | 約 854 | 約 854 |
| main.rs | 538 | 4 | 542 | 958 | 約 957 | 約 967 | 約 967 |
| rules.rs | 277 | 0 | 277 | 1223 | 1223 | 1223 | 1223 |
| stamp.rs | 277 | 0 | 277 | 1223 | 1223 | 約 1168 | 約 1168 |
| site.rs | 262 | 0 | 262 | 1238 | 1238 | 約 1241 | 約 1241 |
| face_srs_rtm.rs | 112 | 0 | 112 | 1388 | 1388 | 1388 | 約 1378 |
| face_labels.rs（新） | — | — | — | — | 約 318 / 余地 約 1182 | 同左 | 約 1167 |

9 本とも size は **S**。M にした便は 1 本も無い。

**次の底は face_srs.rs（余地 102）と face_index.rs（余地 122）**。便 83 はこの 2 本の行を 1 行も増やさない（引数を 1 つ外すだけで、長い行が短くなる）ので通る。要件書の面か入口の面に行を足す便を起こすときは、便 87 と同じ形の切り出しを先に 1 本置くこと。

歯の側の実測: crates/folio/tests/face.rs は器の式でも 1500 を超えている。受付の cap は src の各 file だけを測るので受付は通るが、これ以上太らせないために、便 79 と 81 は憲法の面と要件書の面の歯を新しい file（tests/face_constitution.rs と tests/face_srs.rs）へ分けた。入口・判断の記録・設計ノートの 3 面は既に 1 面 1 file の形を持つので、新しい作法は増えていない。

## 4. 外した候補（1 件）

| 周・観点・id | 一言 | 外した理由 |
|---|---|---|
| 18 実態 F-2 | day-1 の生成 script 3 本を scripts/retired/ へ可逆に移す | file の移動（= 元の path の削除）は便では運べない。器は write-set に在る file を書き換える形しか運ばないので、削除と移動は席の PR で行う。退役は可逆な移動としてのみ行う（N-1.2）ので、席が git mv で移し、要件書の側は直さなくてよい（仕分けの表の一言のとおり）。 |

重複の 1 件（18 周目 読みやすさ F-1）は 16 周目 読みやすさ F-3 と同じ項なので、便 79 の 1 件として数えた。

## 5. 門が掛かる便と、席の手番

- 便 85 と 86 は design-intent の下の正本（rules.yaml と srs.yaml）の生成区間を書き換えるので、受付の事前読みで `folio ceiling --gate` が印 design-intent/preview/ceiling-stamp.yaml を読む。規則の表 D-12 のとおり、**通す** でなければ器へ出さない。席は受付の直前に門を撃って 3 値を実測し、印を付け直す（周を回して `folio ceiling --stamp`）か、持ち主の裁定を取って台帳の notes に逐語と時刻を記帳する。
- 便 79 から 84 と 87 は crates/folio/ と tests/fixtures/face/ しか触らないので門は 通す を返す。
- 便 83 の着地の後、席が別の PR で直すもの: 要件書 FR18 の注（旗 folio face --ceiling と folio build --ceiling を名指している）・docs/design/delivery-40.md と delivery-42.md と docs/design/ceiling-gate.md の旗の言及。規範文と平易文は不変なので持ち主の手番は要らない（便 77 の §1 (i) と同じ運び）。
- 便 83 の着地の後、天井の周の手順が変わる: 「周を回す → `folio ceiling --stamp` で印を書く → `folio build` で面を出す」の順になる（面が印を読むので、印より先に面を出すと 1 周前の名札が出る）。旗 `--ceiling` は無くなる。
- 便 86 の着地の後、席が判断するもの: 判断の記録 ADR-11 の決定 (4) の列に本便の項が無いので、決定 (4) に足す注が要るか、判断の記録の注記で足りるか（台帳 f2-648.115 の断り）。発効済みの欄なので便では触らない。

## 6. 歯の接頭辞と置き場

`grep -rn 'fn f7[0-9]_\|fn f8[0-9]_' crates/folio/tests` の実測（2026-09-22）: 使われている最大は f86_（便 86 の契約が予約）。f87_ が空いている。便の番号と歯の接頭辞を揃えた。

| 便 | 歯の接頭辞 | 置き場 |
|---|---|---|
| 79 | f79_ | 新規 crates/folio/tests/face_constitution.rs（6 本） |
| 80 | f80_ | crates/folio/tests/face_constitution.rs（5 本） |
| 81 | f81_ | 新規 crates/folio/tests/face_srs.rs（6 本） |
| 82 | f82_ | crates/folio/tests/face_adr.rs（2 本）・tests/figure.rs（3 本） |
| 87 | f87_ | 新規 crates/folio/tests/face_labels.rs（1 本・器の式で face.rs を測る） |
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
| 87 | **動かさない**（字を 1 字も変えない移動なので出力は byte で同じ。7 本との byte 一致がそのまま振る舞い不変の物差しになる） |
| 83 | 7 本すべて |
| 84 | expected.html・expected-srs.html |
| 85・86 | 動かさない（実の置き場の正本だけを触り、写しの正本は別の最小の手書きのため） |

凍結 anchor のうち手で置く写しは 2 本だけで、どちらも起草役が独立に組んで §1 に逐語と行数と byte 数と sha256 を書いた。

- 便 85: tests/fixtures/schema/rules-region.txt（27 行・1833 byte・sha256 54580596905e2d70d834c34553d3ad9000dd356473a0df0f60e8afe8fafe652c）
- 便 86: tests/fixtures/schema/srs-region.txt（29 行・1168 byte・sha256 3937340f77713b087b33ca43b8fe866ca310b97cd04e5e1f4ce9967e19b724e1）

図の凍結 anchor tests/fixtures/figure/anchor/body.svg は便 82 でも 1 byte も変えない（照合は日本語への置き換えの前の出力で行うため）。

## 8. 部品目録と様式

9 本とも design-intent/preview/parts.json と design-intent/preview/folio.css を 1 byte も触らない。新しい部品も新しい class も作らず、既存の部品（item-row・principle-amendment-history・glossary-term-table・approval-block・ceiling-stamp・figure-panel）と既存の class（hint・hint-btn・hint-body・vh・legend-line・note・am-row・am-meta・am-kick・rq-where）だけで組む。`folio parts --check` は面の class が folio.css に在ることを見る（crates/folio/src/parts.rs 233〜242 行）ので、この線を守るかぎり様式の追加は要らない。

## 9. 一括 11 からの便候補

天井の 21〜23 周目の所見のうち、直す先が正本の file ではなく実装の型付きの定数（生成区間の正本）だったもの。仕分けで C に置いた所見は 2 件（21 周目 整合 F-7・22 周目 実態 F-3）で、下の 3 本目は 21 周目 整合 F-4 の生成区間の側＝人が書く欄の側は A で直したので、仕分けの表ではその行の一言に書いてある。仕分けは docs/design/batch11-triage.md。

- 判断の記録の欄の決まりの限界の注（adr/schema.yaml の limits_note・正本は crates/folio/src/adr.rs）— 床の凍結の場合の file を「M0 で床の入力に取り込む」と書いたまま残る。判断の記録 ADR-2 は同じ文を帰結に持ち、その注（天井の 11 周目・一括 5）で「起きていない・後続に残すか取り下げるかは別の判断で決める」と断ったので、断りの無い側だけが古い（天井の 21 周目 整合 F-7）。
- 同じ欄のもう 1 つの字（adr/schema.yaml の limits_note と anchor_note）— 「P-7.2〔廃止は状態で〕を条に適用する形は M0 で決める」のまま。M0 は 2026-09-18 に着地し、判断の記録 ADR-2 の帰結は同じ事実を今の状態（台帳 f2-648.100 に起票・open）へ直している（天井の 22 周目 実態 F-3）。
- 設計ノートの欄の決まりの生成区間の中の節の型の注（design-note/schema.yaml の section.type_note・正本は crates/folio/src/note.rs）— 条 P-2.4 を根拠に引くが、P-2.4 が縛るのは部品・図の型・密度 profile とその置き場（部品目録）で、設計ノートの節の型は入らない。人が書く側の注（meta.type_enum_ruling_note）には一括 11 で射程の断りを足したので、生成区間の側も同じ射程に揃える（天井の 21 周目 整合 F-4）。

## 10. 一括 12 からの便候補

天井の 24 周目の所見のうち、直す先が正本の file ではなく面の生成器（実装）だったもの。仕分けは `docs/design/batch12-triage.md`。

- **憲法の面と要件書の面から判断の記録の面への行き先が 1 本も無い**（24 周目 読みやすさ F-4）— 両面は判断の記録の番号を本文と出所の欄で多数指しているのに、`adr-<数>.html` へのリンクを 1 つも持たない。逆向き（判断の記録の面 → 憲法・要件書の節）は在る。判断の記録の id を面の行き先へ解く関数は `crates/folio/src/face_srs.rs` に既に在り（`adr-<数>.html` と `adr/<id>.yaml` の存在を確かめる形・便 90 が要件の欄の名札で使っている）、同じ解決器を憲法の面（`face_constitution.rs`）と要件書の出所・注の欄へ回す便でよい。**余地の実測を取り直してから大きさを決める**（`face.rs` と `face_constitution.rs` は余地が細い）。正本（設計文書）は 1 字も動かない。
