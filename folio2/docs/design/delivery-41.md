# 設計: 便 41 — 反証で「止める」が全部退けられた観点を「再判定待ち（まだ分からない）」に落とす（folio ceiling --check の規則 9・FR18・ADR-8 決定 (3)(4)）

- 要件: FR18（所見の形と天井の 3 値を床で数える・所見の意味の当否は判定しない）/ FR5（3 値・判定できないものは「まだ分からない」）
- 条: P-1（最終判断を代行しない = 床が verdict を書き換えない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-3.3（床の合格を天井の合格として扱わない）
- 判断の記録: ADR-8 決定 (3)（「止める」だけ反証の 2 段目を通す・反証が「まだ分からない」なら所見は残す）・(4)（床が数える = 欄の決まりと 3 値・「まだ分からない」の観点があれば合格にしない）・帰結（folio2 自身で 1 周回した結果を撤退条件の数えの起点にする）。便 40（f2-648.56）の後・1 周目（f2-648 notes 2026-09-19）の是正。
- 裁定: 持ち主 2026-09-19「承認する」（ADR-8 の発効）。本便は天井の正本を変えず、床の規則の場合分けを 1 つ足すだけ（規則の値域は天井の正本のまま）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ap が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + の接頭辞・縮む file は無い）。

## 1. 目的と中身

1 周目（2026-09-19・folio2 自身の設計文書）で、読みやすさの審査役が「止める」4 件を出して verdict を 不合格 と書き、反証役が 4 件とも 退けた と判定した後、床（便 39 の規則 9 = verdict が 不合格 で所見が 1 件以上なら 不合格）は「readability: 不合格（所見 13・止める 0）」を返した。止める所見が 1 つも残らないのに不合格が残るのは、審査役の verdict が反証の前の判断だからで、床がそれを 合格 に読み替えるのは最終判断の代行（P-1）になる。本便は、その状態を「止める所見が全部退けられた（再判定待ち）」として まだ分からない に落とし、席か器が審査役に反証の根拠を渡して verdict を書き直す（1 周目でそうした）手順を床の側から促す。

planner の実測（2026-09-19・main 100774d・1 回目の run f2-648.57-20260919T073057Z は gate で連結の字〔全角の読点〕を指摘され退役 = 同じ種類の失敗 1 回目・本節はその字を明示した版）: crates/folio/src/findings.rs（正規化 759）の観点ごとの 3 値は便 39 の設計文書 §1 (c) の規則 1〜10 の順で、規則 9 の文言「不合格なのに所見が無い」・規則 10 の文言「止める所見が残っている（<id>）」・規則 8 の文言「AI が判定できなかった」を持つ。crates/folio/tests/findings.rs（歯 25 本・正規化 592）。凍結 fixture tests/fixtures/ceiling/findings/ は 10 本（pass-<観点> 4・missing-field・digest-mismatch・fabricated-evidence・stop-unrefuted・stop-upheld・fail-no-findings）で、stop-upheld.yaml は verdict 合格 + 止める F-1 + refute 支持。1 周目の実データ: readability の findings.yaml（verdict 不合格・止める 4 件に refute 退けた）で床が 不合格 を返した。

(a) 規則 9 の場合分け（便 39 §1 (c) の 9 を次に置き換える・他の規則と文言は変えない）: verdict = 不合格 のとき、
- 所見が 0 件 → まだ分からない「不合格なのに所見が無い」（従来どおり）。
- 所見が 1 件以上で、重さが 止める の所見が 1 件以上あり、その全部が refute = 退けた（= 残る止めるが 0）→ まだ分からない「止める所見が全部退けられた（再判定待ち・<id の列>）」。id の列は、複数のときは半角の読点と半角の空白（, の 2 字・便 38 の bundle.rs の一覧と同じ）で連結し、全角の読点（、）は使わない。id が 1 件なら「F-1」の 4 字だけ。
- それ以外（残る止めるが 1 件以上、または 止める の所見が元々 0 件で審査役が 直す・参考 だけで不合格と判定した）→ 不合格（従来どおり・審査役の判定を代行しない）。
規則 7（止める に refute が無い → まだ分からない「反証が未」）は先に当たるので、退けた と未の混在は 7 で止まる（変えない）。規則 10（verdict = 合格 で残る止めるが在れば 不合格）は変えない。標準出力の観点の行「<id>: <3 値>（所見 <数>・止める <数>）」の「止める <数>」は残る止めるの数（従来どおり = 退けた は数えない）。

(b) 凍結 fixture（新しい file 1 本・tests/fixtures/ceiling/findings/stop-refuted.yaml・本便の write-set の + の file）: stop-upheld.yaml の verdict を 不合格 に、F-1 の refute を 退けた に変えたもの（他は 1 byte も変えない）。逐語（字下げは半角 2 つ・末尾は改行 1 つ）:
  verdict: 不合格
  record:
    model: opus
    effort: high
    at: 2026-09-19T05:00:00Z
    read: [constitution, srs, adr, design-note]
    bundle: sha256-files-1 2bd676377b9d6e75d752e645b11cd50088de5288b1de897bf12aa6f5e7cb5a32
  findings:
    - id: F-1
      viewpoint: fidelity
      place: {doc: srs, at: requirements.FR2.plain}
      weight: 止める
      evidence: 合格か不合格のどちらかを出します。
      refute: 退けた
      note: 規範文の 3 値のうち「まだ分からない」を平易文が落としている。

(c) 歯（既存の file crates/folio/tests/findings.rs に 2 本足す・関数名は findings を含める・既存 25 本の期待は不変）:
1. fidelity に stop-refuted.yaml・他 3 観点に pass → 2 ∧「fidelity: まだ分からない（所見 1・止める 0）」∧ 標準エラーに「止める所見が全部退けられた（再判定待ち・F-1）」∧ 最後の行「folio ceiling: まだ分からない（観点 4・合格 3・不合格 0・まだ分からない 1）」。
2. stop-refuted.yaml の verdict を 合格 に変えて写す → 0（残る止めるが 0 で verdict 合格 = 規則 10 の従来どおり）／pass-fidelity.yaml の verdict を 不合格 に変えて写す（止める 0・直す 1）→ 1（従来どおり・審査役の不合格を代行しない）。
既存の歯（stop-upheld → 1・stop-unrefuted → 2・fail-no-findings → 2 等）は期待不変。

(d) 接続: findings.rs の規則 9 の分岐と文言だけ（余地 741）。main.rs・bundle.rs・face の名札（便 40 の (b) は findings.rs の観点ごとの結果の関数を呼ぶので、名札も同じ 3 値を出す = 「再判定待ち」の観点は まだ分からない と出る・字面は変えない）・天井の正本・語彙は触らない。手順の側: 反証の後に席か器が審査役へ反証の根拠を渡し、verdict の 1 行だけを書き直させる（1 周目の手順・台帳 f2-648 notes 2026-09-19）。反証の材料の束を組む口は別の便。

## 2. 範囲

- 入れる: 規則 9 の場合分けと文言・凍結 fixture 1 本・歯 2 本。
- 入れない: 反証の材料の束を組む口・天井の正本の変更・面の名札の字面の変更・審査役の verdict の書き換え（床は代行しない・P-1）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| rule | 規則 | findings.rs の規則 9 の場合分け |
| anchor | 凍結 | tests/fixtures/ceiling/findings/stop-refuted.yaml |
| teeth | 歯 | tests/findings.rs +2（既存 25 本は期待不変） |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ap"
title = "反証で止めるが全部退けられた観点を「止める所見が全部退けられた（再判定待ち）」として まだ分からない に落とす（folio ceiling --check の規則 9 の場合分け・床は verdict を代行しない）"
req = ["FR18", "FR5"]
section = "1"
write-set = ["crates/folio/src/findings.rs", "crates/folio/tests/findings.rs", "+tests/fixtures/ceiling/findings/stop-refuted.yaml"]
verify = ["cargo nextest run -p folio --test findings findings", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "findings の歯（再判定待ち 1 本・従来どおり 2 通り 1 本・既存 25 本は期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
