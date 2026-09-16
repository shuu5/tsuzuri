# 図の凍結 anchor（FR15 / AC12・判断の記録 ADR-4 の帰結・P-10.1）

- `anchor/spec.json` = 図の型付き記述 1 本（構成図・仕上がりの段 showcase・日本語の名札）／ `anchor/body.svg` = それから期待される図の本体（SVG）の写し。道具の版 = rules 行 R-15（2.16.0）。版上げで body.svg が変われば anchor が落ち「まだ分からない」に落とす（P-10.3）。体裁: 行内の様式 0・色の直書き 0・意味 class は `design-intent/preview/parts.json` の figure_body_classes に全て含まれる（実測 2026-09-16）。sha256(body.svg) = fcab70afb67eeb45c1b4159affd2dc68a29375ad7a6b38a04f22a51063968577
- `fails-showcase.json` = 仕上がりの段の検査を通らない記述（AC12 の落ちる側）。格子の行列だけ持ち、間隔（gapX / gapY / cellW / cellH）と線の名札のずらし（labelDy）を持たない＝調査 2026-09-15 の往復 2 の形（診断: 線の名札 6 件が隣の箱に重なる）。**本便では道具を実行していない**（A-3.1・依存を足す便で初めて実行）ので、落ちることの再確認はその便まで「まだ分からない」。
- 出所 = 調査報告 2026-09-15-archify/samples/ja-round4.{json,svg}（版管理の外の写し）。床の script はこの fixture を読まない（床への取り込みは M1 の folio）。
