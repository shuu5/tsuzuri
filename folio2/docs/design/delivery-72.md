# 設計: 便 72（便 A）— 天井の印を周の結果から決定的に導出して生成物として書く（folio ceiling --stamp・FR20 / AC18・設計ノート ceiling-gate.md §6 の 2）

- 要件: FR20（天井の門 — 天井の判定の印を生成物として書き、設計文書を書き換える便の受付を印で 3 値に判定する）/ AC18 / GOAL3 / GOAL4
- 条: P-3.3 / P-4.1 / P-4.2 / P-6.1 / P-6.2 / P-10.1
- 出所: 判断の記録 ADR-8 決定 (4)（天井が合格でない間は設計文書の便の着地を止める）・設計ノート docs/design/ceiling-gate.md（持ち主の裁定 2026-09-21 08:59 JST・案 A = 印は repo の生成物・鮮度は厳格・folio の命令 + 席の手順）・要件書 v1.19 の FR20 / AC18（発効 2026-09-21 10:17 JST）。
- 根拠の判断: 判断は設計ノート §3〜§5 で裁定済み。本便は印を書く口だけを持ち、門（--gate）は便 73、面の名札が印を読む形は後続の便。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bu が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + 接頭辞）。

## 1. 目的と中身

(a) 命令。crates/folio/src/main.rs（正規化 約 507 行・余地 約 990）の Command::Ceiling に旗 --stamp を足す（--write / --check / --refute と同じ排他の群 mode の 4 つ目・--out 必須・--faces 不要）。分岐は新しい file crates/folio/src/stamp.rs（新規・+）の `pub fn run(dir: &Path, out: &Path) -> Outcome` へ渡す（main.rs は薄い分岐だけ）。

(b) 印の導出（stamp.rs）。入力 = 天井の正本 `<dir>/ceiling.yaml`（bundle::load）と束の置き場 `<out>/`（--write が組んだ観点ごとの dir・席か器が書いた findings.yaml・止めるの反証 `<out>/<観点>/refute/<id>/result.yaml`）。観点ごとの 3 値は findings.rs の --check と同じ規則で数える（count_viewpoint と同じ口を使う・二重に実装しない = findings.rs の関数を pub(crate) にして呼ぶ）。全観点が数えられたときだけ印を組む。印の中身（YAML・欄は閉じた一覧・この順）:
  - `round`: 束の置き場の dir 名（--out の末尾の要素・例 2026-09-21-round13）
  - `at`: 4 観点の record.at のうち byte 順で最大の 1 つ
  - `verdict`: 全体の 3 値（--check の総合と同じ規則 = 1 つでも 不合格 なら 不合格・1 つでも まだ分からない なら まだ分からない・全部 合格 なら 合格）
  - `sources`: 正本の要約値 = `<out>/*/sources/` の下の file を「観点の sources/ からの相対 path」で和集合にし（同じ相対 path が観点で違う byte なら Err → まだ分からない・何も書かない）、相対 path の byte 順に中身を区切りなしに連結した byte 列の sha256（bundle.rs の digest_text と同じ規則・`sha256::hex`）を「sha256 <64 字の 16 進>」の形で
  - `faces`: 面の要約値 = 同じ規則で `<out>/*/faces/`
  - `viewpoints`: 観点ごとの行 `{id, verdict, findings, stops, bundle, model, effort, at}`（id と順は天井の正本の viewpoints のとおり・bundle = digest.txt の 16 進の先頭 8 字・model / effort / at = findings.yaml の record）
  - `refutes`: 止めるの所見ごとの行 `{viewpoint, finding, refute}`（result.yaml の refute の 3 値・観点の順 → 所見の id の byte 順・無ければ空の列）
  - `reads`: 観点の record.read の和集合（byte 順）
  出力先 = `<dir>/preview/ceiling-stamp.yaml`（先頭に注釈 1 行「# folio2 天井の印 — 生成物（folio ceiling --stamp が書く・手で直さない・P-6.2）」）。既に同じ byte なら書かず標準出力に「folio ceiling: 印は同じ（<path>）」・書いたら「folio ceiling: 印を書いた（<path>・<byte 数> byte）」・組めなければ標準出力に理由 1 行「folio ceiling: まだ分からない（<理由>）」で終了 2・file は触らない（全部か無しか）。

(c) 凍結 anchor（P-10.1・orchestrator 席が独立に組んで本便の前に main に置いた・本便は触らない）。tests/fixtures/ceiling/findings/stamp-expected.yaml = 便 38 の凍結 fixture の束（tests/fixtures/ceiling/bundle/）と所見の凍結 fixture pass-coherence.yaml（verdict 合格・findings 空・record あり）を 4 観点全部に写した周から導出される印の、要約値 3 種（sources / faces / bundle）と at を除いた欄の写し（席が本便の前に §1 (b) の欄の順で手書きし、値 = round: stamp-case・verdict: 合格・viewpoints 4 行の id / verdict 合格 / findings 0 / stops 0 / model opus / effort high・refutes: []・reads: [adr, constitution, design-note, rules, srs]）。anchor の中身 = 先頭の注釈 1 行・round・verdict・viewpoints（各行は id / verdict / findings / stops / model / effort の 6 欄）・refutes・reads。歯は導出した印から鍵が at / sources / faces の行を落とし、viewpoints の各行から bundle と at の欄を落として、anchor と byte 比較する（anchor を生成物に合わせない）。

(d) 歯（新規 file +crates/folio/tests/stamp.rs・関数名は stamp_ で始める・今この語で始まる歯は無い・束の組み方と findings の写し方は tests/findings.rs の helper と同じ形を stamp.rs の中に持つ）。
1. stamp_writes_the_frozen_shape: 凍結 fixture の束を一時 dir に --write で組み、pass-coherence.yaml を 4 観点の findings.yaml に写し、一時 dir の名を stamp-case にして --stamp を撃つと終了 0・`<dir>/preview/ceiling-stamp.yaml` が在り、要約値 3 種と at の行を除いた中身が (c) の anchor と byte 一致。
2. stamp_sources_digest_is_recomputable: 同じ印の sources の 64 字が、歯が独立に `<out>/*/sources/` の和集合を相対 path の byte 順に連結して命令 sha256sum（子の処理・tests/schema.rs と同じ形）で測った値と一致。faces も同じ。
3. stamp_is_idempotent: 続けてもう 1 度 --stamp を撃つと終了 0・標準出力に「印は同じ」・file の byte が不変。
4. stamp_refuses_an_incomplete_round: 4 観点のうち 1 つの findings.yaml を消して --stamp を撃つと終了 2・標準出力に「まだ分からない」と観点の id・印の file は書かれない（在った印は変わらない）。
5. stamp_carries_the_refute_results: 止めるの所見を持つ所見 fixture（fail-no-findings.yaml でなく、止める 1 件を持つ既存の fixture が無ければ歯の中で 1 件足した写しを作る）を 1 観点に置き、`refute/<id>/result.yaml` に refute: 支持 を書いて --stamp を撃つと、印の verdict が 不合格・refutes に {viewpoint, finding, refute: 支持} の 1 行。
6. 回帰（期待不変・verify の 2 行目）: tests/findings.rs・tests/ceiling.rs・tests/bundle.rs・tests/check.rs（p1_commands_closed_list = 命令の一覧は不変・旗が増えるだけ）の既存の歯すべて。

(e) 大きさと接続。新規 file = stamp.rs（約 220 行）・tests/stamp.rs（約 200 行）。anchor は席が置いた（本便は読むだけ）。main.rs（+約 12 行）・findings.rs（pub(crate) 化・+約 5 行・正規化 約 1,070 行・余地 約 430）・bundle.rs（digest の関数の可視性・+約 2 行・余地 約 999）。size M。外部 crate は増やさない。実の置き場 design-intent/preview/ に印を書くのは席が着地の後に周の束で撃つ（本便は書かない）。

## 2. 範囲

- 入れる: --stamp の口・印の導出と書き込み・凍結 anchor・歯 5 本。
- 入れない: 門（--gate・便 73）・面の名札が印を読む形（後続の便）・実の印の 1 本目（席）・器への結線（器の側）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| flag | 旗 | main.rs の --stamp（mode の 4 つ目） |
| derive | 導出 | stamp.rs = 束と所見と反証から印の欄を組む |
| digest | 要約値 | sources / faces の和集合の sha256 |
| write | 書く | preview/ceiling-stamp.yaml（同じなら書かない） |
| anchor | 凍結 | stamp-expected.yaml |
| teeth | 歯 | tests/stamp.rs に stamp_ の 5 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bu"
title = "命令 folio ceiling --stamp が周の結果（観点ごとの所見 file と止めるの反証の結果）から天井の印（周の id・観点ごとの 3 値と反証・正本と面と束の要約値・起動の記録・読んだ文書の id）を決定的に導出し design-intent/preview/ceiling-stamp.yaml に書く（同じなら書かない・組めなければ まだ分からない で何も書かない・凍結 anchor stamp-expected.yaml・FR20 / AC18）"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/main.rs", "+crates/folio/src/stamp.rs", "crates/folio/src/findings.rs", "crates/folio/src/bundle.rs", "+crates/folio/tests/stamp.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/ceiling.rs", "crates/folio/tests/bundle.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test stamp stamp_", "cargo nextest run -p folio --test findings --test ceiling --test bundle --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "stamp_ の歯 5 本（凍結の形・要約値の再計算・冪等・欠けた周は書かない・反証の結果を運ぶ）が緑、tests/findings.rs・tests/ceiling.rs・tests/bundle.rs・tests/check.rs の既存の歯が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
