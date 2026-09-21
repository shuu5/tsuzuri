# 設計: 便 73（便 B）— 天井の門: 印と便の書き換える file の一覧から 3 値を返す（folio ceiling --gate・FR20 / AC18・設計ノート ceiling-gate.md §2 / §6 の 3）

- 要件: FR20 / AC18 / GOAL3 / GOAL4
- 条: P-3.3 / P-4.1 / P-4.2 / N-3.1
- 出所: 判断の記録 ADR-8 決定 (4)・設計ノート docs/design/ceiling-gate.md §2（門が答える問い）・要件書 v1.19 の FR20 / AC18。便 72（--stamp）の後（印の欄の決まりを読む）。
- 根拠の判断: 判断は設計ノートで裁定済み（案 A = folio の命令 + 席の手順）。本便は命令だけを持ち、器への結線と席の手順（mk-run.sh の頭）は着地の後に席が置く。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bv が指す §1 だけ。write-set は手書き（Declared 形・新規は + 接頭辞）。

## 1. 目的と中身

(a) 命令。crates/folio/src/main.rs の Command::Ceiling に旗 --gate（mode の 5 つ目・排他）と、--gate のときだけ要る `--write-set <path>...`（1 つ以上・repo の根からの相対 path・繰り返し可）を足す。--out / --faces は不要。分岐は新しい file crates/folio/src/gate.rs（新規・+）の `pub fn run(dir: &Path, write_set: &[String]) -> Outcome` へ。

(b) 設計文書の判定（gate.rs）。write-set の各 path について「設計文書の正本」= `<dir>`（--dir・通常 design-intent）の下に在り、かつ `<dir>/preview/` の下でなく、path のどの要素も retired でないもの（設計ノート ceiling-gate.md の決め = design-intent/ の直下と adr/・design-note/ の正本 YAML）。接頭辞 + / - / ~ は剥がして読む。設計文書の正本が 1 つも無ければ 通す（終了 0・「folio ceiling: 通す（設計文書の正本を書き換えない便）」）。

(c) 印の判定。設計文書の正本が在るとき `<dir>/preview/ceiling-stamp.yaml`（便 72 の欄の決まり）を読む。印が無い・読めない → まだ分からない（2・「印が無い」）。印の sources の要約値を、今の `<dir>` から同じ相対 path の一覧（印の sources を測った file の一覧 = 印には無いので、束の規則と同じ「天井の正本の documents の一覧が指す file（dir は再帰・YAML だけ）」を `<dir>` で解いた集合）で測り直し、違えば まだ分からない（2・「印が古い」）。印の観点に まだ分からない が 1 つでも在れば まだ分からない（2・観点の id）。不合格 が 1 つでも在れば 止める（1・観点の id）。全部 合格 で要約値が同じなら 通す（0）。標準出力は 1 行「folio ceiling: <3 値>（<理由>）」・終了は 0 / 1 / 2。

(d) 凍結 fixture（席が手書き・最小）。+tests/fixtures/ceiling/findings/stamp-pass.yaml（verdict 合格・観点 4 行とも合格・sources は 64 字の 0 の仮の値）・+stamp-fail.yaml（coherence が 不合格・refutes 1 行）・+stamp-unknown.yaml（reality が まだ分からない）。歯は一時 dir の design-intent の写しに対して要約値を sha256sum（子の処理）で測って仮の値を置き換えてから置く（「同じ要約値」の場合）か、仮の値のまま置く（「古い」の場合）。

(e) 歯（新規 file +crates/folio/tests/gate.rs・関数名は gate_ で始める・今この語で始まる歯は無い・FR20 の 6 場合）。
1. gate_passes_a_delivery_that_touches_no_design_intent: write-set が src と tests と docs/design だけ → 終了 0・「通す」。
2. gate_passes_when_the_stamp_is_all_pass_and_fresh: stamp-pass を要約値を合わせて置き、write-set に design-intent/srs.yaml → 終了 0。
3. gate_stops_on_a_failed_viewpoint: stamp-fail（要約値は合わせる）→ 終了 1・標準出力に「止める」と coherence。
4. gate_is_unknown_when_the_stamp_is_stale: stamp-pass を仮の要約値のまま置く → 終了 2・「印が古い」。
5. gate_is_unknown_on_an_unknown_viewpoint: stamp-unknown（要約値は合わせる）→ 終了 2・reality。
6. gate_is_unknown_without_a_stamp: 印を置かない → 終了 2・「印が無い」。
7. gate_ignores_preview_and_retired_paths: write-set が design-intent/preview/ceiling-stamp.yaml と design-intent/preview/retired/readable.html だけ → 終了 0（設計文書の正本を書き換えない便）。
8. 回帰（期待不変・verify の 2 行目）: tests/stamp.rs・tests/findings.rs・tests/check.rs（命令の一覧は不変）の既存の歯すべて。

(f) 大きさと接続。新規 file = gate.rs（約 150 行）・tests/gate.rs（約 220 行）・fixture 3 本。main.rs（+約 15 行）。size S。外部 crate は増やさない。便 72 と main.rs で重なるので受付は便 72 の着地の後。

## 2. 範囲

- 入れる: --gate の口・設計文書の判定・印の判定・fixture 3 本・歯 7 本。
- 入れない: 印を書く側（便 72）・器への結線・席の手順（mk-run.sh）・面の名札。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| flag | 旗 | main.rs の --gate と --write-set |
| classify | 設計文書 | write-set の path を正本か否かに分ける |
| judge | 判定 | 印の 3 値と要約値から 0 / 1 / 2 |
| fixture | 凍結 | 印の写し 3 本（手書き） |
| teeth | 歯 | tests/gate.rs に gate_ の 7 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bv"
title = "命令 folio ceiling --gate --write-set <path>... が印 design-intent/preview/ceiling-stamp.yaml と便の書き換える file の一覧から 3 値（設計文書の正本を書き換えない便は通す・印が 4 観点合格で正本の要約値が同じなら通す・不合格が在れば止める・印が無い / 古い / まだ分からない が在れば まだ分からない）を返す（FR20 / AC18 の 6 場合）"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/main.rs", "+crates/folio/src/gate.rs", "+crates/folio/tests/gate.rs", "crates/folio/tests/stamp.rs", "crates/folio/tests/findings.rs", "crates/folio/tests/check.rs", "+tests/fixtures/ceiling/findings/stamp-pass.yaml", "+tests/fixtures/ceiling/findings/stamp-fail.yaml", "+tests/fixtures/ceiling/findings/stamp-unknown.yaml"]
verify = ["cargo nextest run -p folio --test gate gate_", "cargo nextest run -p folio --test stamp --test findings --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "gate_ の歯 7 本（6 場合 + preview / retired の除外）が緑、tests/stamp.rs・tests/findings.rs・tests/check.rs の既存の歯が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
