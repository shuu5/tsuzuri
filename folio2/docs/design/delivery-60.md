# 設計: 便 60 — 図の道具の凍結 anchor を実行時に照合し、落ちたら判定を「まだ分からない」に落とす（FR15 の未実装の節・天井の 10 周目の実態 F-3 の是正）

- 要件: FR15（図の道具の境界）/ FR5（3 値の判定）
- 条: P-10.1（検査は独立した凍結 anchor を持つ）/ P-10.3（anchor が維持できなくなった検査の結果は「まだ分からない」に落とす）/ P-4.1
- 出所: 天井の 10 周目（2026-09-21・main 39b6590）の実態 F-3（止める・反証 支持）= 要件書 FR15 の規範文「凍結 anchor（型付き記述 1 本と図の本体の写し）が落ちたら判定を「まだ分からない」に落とす」と判断の記録 ADR-4 決定 (6)・規則の表 R-15 の注に対し、実装（crates/folio/src/figure.rs）は凍結 anchor を一切読まず、anchor を読むのは歯（crates/folio/tests/figure.rs）だけ。道具の版が変わって図の本体が変われば folio figure --write と folio build --write は新しい図を書いて 0 を返す。
- 根拠の判断: ADR-4 決定 (6)（発効済み）の実装であり新しい判断は無い。凍結 anchor の置き場（tests/fixtures/figure/anchor/ の spec.json と body.svg）は変えない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bi が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 凍結 anchor を実装に埋める。crates/folio/src/figure.rs（正規化 470 行・余地 1030）に、凍結 anchor の 2 file を compile 時に取り込む定数を 2 つ置く: 型付き記述 = include_str!(../../../tests/fixtures/figure/anchor/spec.json)・図の本体の写し = include_str!(../../../tests/fixtures/figure/anchor/body.svg)（path は figure.rs からの相対・fixture の file は 1 byte も変えない）。実行時に file を探さないので、配信先や写しの置き場に anchor を要求しない。

(b) 照合の関数を 1 つ足す（anchor_holds の名・引数は正本の置き場 dir）。型付き記述の定数を既存の読み手（yaml-rust2 は JSON も読める）で Value に読み、既存の derive と同じ経路（to_json で 1 行に正規化 → tool_path(dir) の道具へ deliver・型は architecture）で図の本体を導出し、図の本体の写しの定数と byte で比べる。同じなら Ok(())。違えば Err で、文言は「凍結 anchor が落ちた（図の道具の出力が固定の写しと違う・道具の版か写しが変わった。P-10.3）」。道具が起動できない・検査を通らないは既存の Err がそのまま上がる。結果は process の中で 1 回だけ計算して以後は再利用する（std::sync::OnceLock・道具の呼び出しを図ごとに増やさない。nextest は歯ごとに process を分けるので歯どうしは混ざらない）。

(c) 照合を掛ける場所は 2 つ。① 命令 folio figure（run）: --write でも --check でも、図を導出する前に anchor_holds を呼び、Err なら Outcome::unknown（終了 2・stderr は既存の接頭辞「folio figure: まだ分からない: 」+ 文言）で終え、出力 file は書かず前の生成物も上書きしない。② 面の経路（render・設計ノートの面 face_note と入口 build が図ごとに呼ぶ）: 導出の前に anchor_holds を呼び、Err ならそのまま Err を返す（面の生成器と build は既存の規則で「まだ分からない」に落ちる）。anchor が合うときの出力は 1 byte も変えない（既存の凍結の歯 figure_write_matches_the_frozen_anchor が緑のまま）。

(d) 歯（crates/folio/tests/figure.rs・正規化 約 300 行・関数名は anchor_ で始める・今この語で始まる歯は無い）。写しの作り方は既存の fixture_copy と同じ（vendor/archify/ を一時 dir の親へ写す）。改変 = 写した vendor/archify/renderers/shared/utils.mjs の中の、図の本体に出る class の字 m-default を m-defaultx に書き換える（写しだけ・repo の vendor は触らない。この字は凍結 anchor の body.svg に実在し、道具の出力が変わる最小の改変）。
1. anchor_drift_makes_figure_write_unknown: 改変した写しで folio figure --write（fig-anchor）が終了 2・stderr に「凍結 anchor が落ちた」を含み、出力 file が無い。
2. anchor_drift_keeps_the_previous_output: 出力先に前の生成物（任意の 1 行）を置いてから改変した写しで --write → 終了 2・前の生成物の byte が不変。
3. anchor_drift_makes_figure_check_unknown: 改変した写しで --check → 終了 2（不合格 1 ではない）・stderr に「凍結 anchor が落ちた」。
4. anchor_drift_makes_the_note_face_unknown: 改変した写しで folio face --face note --id full --write（tests/face_note.rs と同じ呼び方・図の行を持つ写し）→ 終了 2・stderr に「凍結 anchor が落ちた」・面の file を書かない。
5. anchor_holds_on_the_untouched_copy: 改変しない写しで --write（fig-anchor）→ 終了 0・出力が body.svg と byte 一致（既存の凍結の歯と同じ判定を anchor_ の名で持つ = 照合が正の側でも回っている）。
6. 回帰（期待不変・verify の 2 行目）: 既存の figure_ の歯すべて（凍結 anchor との一致・決定的な再実行・--check の 3 値・通らない図は書かない）。

(e) 大きさと接続。新規 file は無い。figure.rs（定数 2 + 関数 1 + 呼び出し 2・約 +45 行）・tests/figure.rs（約 +90 行）。size S。face.rs・site.rs・main.rs・fixture・design-intent は触らない（FR15 の注の更新は席の一括で行う）。外部 crate は増やさない。

## 2. 範囲

- 入れる: 凍結 anchor の compile 時の取り込み・実行時の照合・命令と面の経路の 2 か所への接続・歯 5 本。
- 入れない: anchor の更新の手順（道具の版上げの便で扱う・A-3.1）・往復の数え（R-7・folio の外 = ADR-4 決定 (7)）・要件書 FR15 の注（席の一括）・道具の要約値（R-15）の実行時の照合（別の便で問う）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| const | 定数 | figure.rs に anchor の 2 file を include_str! で持つ |
| holds | 照合 | anchor_holds: 導出と写しの byte 比較・OnceLock で 1 回 |
| hook | 接続 | run と render の導出の前に照合 |
| teeth | 歯 | tests/figure.rs に anchor_ の 5 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない（OnceLock は標準 library）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bi"
title = "図の道具の凍結 anchor（tests/fixtures/figure/anchor/ の型付き記述と図の本体の写し）を figure.rs に compile 時に取り込み、folio figure（--write / --check）と面の経路（render）の導出の前に照合して、落ちたら判定を まだ分からない（終了 2）に落とし出力も前の生成物も書かない（FR15 の未実装の節・ADR-4 決定 (6)・P-10.3・天井の 10 周目の実態 F-3）"
req = ["FR15", "FR5"]
section = "1"
write-set = ["crates/folio/src/figure.rs", "crates/folio/tests/figure.rs"]
verify = ["cargo nextest run -p folio --test figure anchor_", "cargo nextest run -p folio --test figure figure_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "anchor_ の歯 5 本（改変した写しで figure --write が終了 2 で出力を書かない・前の生成物が不変・--check が終了 2・設計ノートの面が終了 2・改変しない写しで --write が終了 0 で body.svg と一致）が緑、既存の figure_ の歯が全部緑（出力は 1 byte も変わらない）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
