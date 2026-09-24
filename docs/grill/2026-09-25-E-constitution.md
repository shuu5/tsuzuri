# grill 論点 E — 憲法の合流（討論の記録・正本は design-intent/constitution.yaml）

- 出自: docs/handoff/2026-09-24-v3-kickoff.md §2 論点 E・ADR-4 決定 (4)（憲法の最小版 v1.0 が凍結と M3 の判定点の前提）
- 状態: **発効**（裁定 id = user 2026-09-24T22:39Z・記帳先 s2-07l.214・承認欄記入済み）。凍結は folio2 の列の根の表に行が足されるのを待つ（digest 35eb6b36…）
- 逐語は台帳へ

## 1. 事実（verified・2026-09-25）
| 事実 | 出所 |
|---|---|
| scribe2 憲法 = 25 条（C1〜C17・A1〜A4・N1〜N4）・HTML が唯一の層・順位 = 場当たりの修正の禁止 → 増殖の防止 → compile 速度 | scribe2/design-intent/spec/constitution.html |
| folio2 憲法 v1.4 = 27 条（P1〜P18〔P9 欠番〕・A1〜A4・N1〜N6）・YAML 正本・順位 = 網羅より揃い ／ 速さより黙らない ／ 賢さより戻せる・段 = never ＞ ask-first ＞ always | folio2/design-intent/constitution.yaml |
| scribe3 の床（folio 68196cf）は条 id を P-n / A-n / N-n の形でしか受けない。規範文は 1 文・強度と文末の一致・語彙に無い英字語 0・rules 行は条と双方向 | folio check の実測 |
| 草案は 38 条（always 27・ask-first 4・never 7）・rules 行 13（R-1〜R-12・R-16）+ 作法 4（D-1〜D-4）・folio check 違反 0 | design-intent/constitution.yaml・rules.yaml |

## 2. 合流の方針（deduced）
- 折れない線は落とさない。重なる条は 1 条にまとめ、出所（rationale）に両方の番号を残す（機械が消し込めるのは folio2 の P・A・N を「P5」のようにハイフン無しで書いた記法）。
- 例外 1 つ: folio2 P13 の枝 4「self-host は外部の利用者で受入が通った後」は folio2 の M3 の条件で、v3 は scribe2 A4（自己開発は既定で許可）を採る。folio2 側は判断の記録 21 番が扱う。
- v3 で足した線（前の版に無い・持ち主の要望と v2 の残りから）: P-7.3 断るときは次の 1 手を名指す／P-11.2 便の終端は event で持つ／P-20.3 席の状態は描画でなく構造から読む／N-5.2 app ごとの表示の仕組みを増設しない／P-1.6 規則の種類は閉じた型の枝（scribe2 C2 から）。
- 機構の live: now = 今日 scribe3 で数えられる（folio の床）・M1 = code の持ち込み（ADR-4 決定 (4) の 3 段目）後に数えられる（scribe2 の CI）。

## 3. 対応表（前の版 → v3）
| v3 | 出所 |
|---|---|
| P-1 規則は型付きデータ | folio2 P5・scribe2 C1 / C2 / C14 / C15 |
| P-2 正本は 1 つ | folio2 P6・scribe2 C1.2 |
| P-3 番号は増やすだけ | folio2 P7 |
| P-4 撤退条件 | folio2 P8 |
| P-5 拘束は設計文書・前の版は道具 | folio2 P13（枝 1〜3）・scribe2 C8 |
| P-6 床と天井 | folio2 P3・scribe2 lens |
| P-7 黙って飛ばさない | folio2 P4・doctor の名指し |
| P-8 値の出所を型で | scribe2 C10 |
| P-9 独立の物差し | folio2 P10・scribe2 C12.5 |
| P-10 同じ穴は止めて人へ | folio2 P11 |
| P-11 席の停止と終端 | scribe2 C9・v2 の残り 7 |
| P-12 承認は 1 面から逐語 | folio2 P12・scribe2 C7 |
| P-13 注入 | folio2 P14・scribe2 ADR-0022 |
| P-14 編集時に止める | folio2 P18・scribe2 C16 |
| P-15 仕掛けに寄りかからない | folio2 P15 |
| P-16 最上位モデル | folio2 P16 |
| P-17 数値の表は裁定 id | folio2 P17・scribe2 C5 |
| P-18 提示層は単一の生成器 | folio2 P2 |
| P-19 判断を代行しない | folio2 P1・scribe2 C7 |
| P-20 状態は 1 つの database・描画を読まない | scribe2 C3・v2 の残り 1 |
| P-21 1 関数 1 列挙 | scribe2 C2 |
| P-22 大きさの上限 | scribe2 C4 |
| P-23 予算と起動口 | scribe2 C6 |
| P-24 lint | scribe2 C11 |
| P-25 歯は 1 枠組み・snapshot | scribe2 C12 |
| P-26 依存と compile の予算 | scribe2 C13・folio2 A3 |
| P-27 決定はしご | scribe2 C17 |
| A-1〜A-4 | folio2 A1〜A4・scribe2 A1〜A4 |
| N-1 削除 / N-2 散文 / N-3 例外 / N-4 改憲 / N-5 AI 係と表示 / N-6 公開先 / N-7 host 分岐 | folio2 N1〜N6・scribe2 N1〜N4（N3 = N-7） |

## 4. 持ち主に決めてもらう点（1 問に束ねる）
1. **前文の順位**: 段 = 絶対にやらない ＞ 確認してから ＞ いつも守る。条 = 場当たりの修正の禁止 ／ 黙らないこと ／ 戻せること ／ 増殖の防止 ／ compile の速さ（scribe2 の 3 つと folio2 の 3 つを 1 列に）。
2. **例外 1 つ**（§2・self-host の条件を持ち込まない）。
3. **足した線 5 本**（§2）。

## 5. 問い（1 問）
この草案を v3 の憲法 v1.0 として発効してよいか（前文の順位・例外 1 つ・足した線 5 本を含めて）。発効の承認は逐語・日付・裁定 id で憲法の承認欄と台帳に記帳し、その後に始まりの凍結（folio check --freeze-start）を撃つ。直したい条があれば条の番号で名指してほしい。

## 6. 経緯
- 2026-09-25: s3 席が 2 憲法を読み、草案 v1.0 を書いて床を通し（違反 0）、読む面を tailnet で出して持ち主へ提示（答え待ち）
- 2026-09-24T22:40Z: 持ち主「これでよい」（草案 v1.0 の発効・前文の順位・例外 1 つ・足した線 5 本を含む）。次: 裁定 id → meta.approval / status effective / binding true / rules 行の ruling と ruled_at を全行に（P-17.3）→ folio check --freeze-start（列の根の表に scribe3 の行が無いので要約値の全桁を出して断る見込み → folio2 席へ送って行を足す便を起こしてもらう → 凍結）
