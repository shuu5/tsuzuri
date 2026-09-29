# vendor — folio が build 時に使う外部の道具の写し

- `archify/` = 図の道具 archify 2.16.0（本家 tt-a1i/archify・tag v2.16.0 = commit c826e6c3a7abad19c0f3cd1ca57207d54b1ad8de・許諾 MIT・`archify/LICENSE`）のうち、`deliver`（検査 + 描画 + 仕上がりの検査）に要る file だけの写し（43 file・`assets/template.html`・`bin/`・`renderers/`・`schemas/`・`scripts/check-render-output.mjs`・`package.json`・`LICENSE`）。実行時の依存は実行環境 Node.js（18 以上）だけで、npm の取り込みは行わない。
- 出所と版の固定は判断の記録 ADR-4 決定 (5) と rules 行 R-15（版の番号・commit・要約値）。要約値（sha256）の規則 = `vendor/archify/` の全 file を path の byte 順（`LC_ALL=C sort`）に並べ、その中身を区切りなしに連結した byte 列の sha256。歯（`crates/folio/tests/figure.rs`）が R-15 の値と突き合わせる。
- 写しを更新する（版上げ）ときは、凍結 anchor（`tests/fixtures/figure/anchor/`）が落ちる＝「まだ分からない」に落ちる（P-10.3）。版上げは A-3.1（依存の増減）の確認と R-15 の改訂（裁定 id・時刻）を要する。
- 道具の通信する命令（`brands capture`）と閲覧の仕掛け（`preview` / `--open`）は folio から呼ばない（ADR-4 決定 (2)(3)(5)）。
- 持ち主の承認（A-3.1）: 2026-09-19「承認する」（f2-648 notes・対話面 R-8）。
