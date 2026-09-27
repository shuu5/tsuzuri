# 画面の便を並べて走らせる段取り（2026-09-27・設計席の作業記憶）

持ち主の指摘（2026-09-27・逐語は t3-hub.1）= 最初の画面は見本（mock v3）に機能も見た目も遠い・速さを上げたい。作る順の計画（docs/handoff/2026-09-27-t3-build-plan.md・承認済み）の順は変えず、便を並べて走らせる。

## 1. 並べ方（write-set が重ならない便は同時に走る）
| 波 | 便 | 触る所 | 状態（10:00Z） |
|---|---|---|---|
| 0 | g-frame（枠と見た目）・c（導出グラフ） | 面・中核 | 着地 2a638ad・58a3ea6 |
| 1 | g-parts（block の読みの口と hover の card）∥ d（指標・未反映・次の一手・pipeline の板） | 面 ∥ 契約の型と中核 | 実装中 |
| 2 | e-read（読む側の口 4 つと board-changed）∥ g-pipe ∥ g-next ∥ g-ledger | 境界 ∥ 面の module 1 つずつ | 行は起草済み（stash と席の作業場の wave-t1.patch）・波 1 の着地の後に commit して起票 |
| 3 | g-map（地図の頁の殻と圧縮と一覧）∥ 席と口座の読み（中核の字の読みと server の口）∥ e（裁定の受付） | 面 ∥ 中核と境界 | 未起草 |
| 4 | 地図のグラフ・表・近傍 ∥ block「orchestrator と口座」∥ 問いの card の答えの欄 ∥ f（hook） | 面 ∥ 境界 | 未起草 |

block の便は自分の module の file と自分の歯の file と fixture だけを書く（便 g-parts が関数の外形と口の path を先に決める）。枠の歯 tests/frame.rs は block の便の write-set に入れない。

## 2. 口の path（閉じた 6 本）
/api/ledger（着地済み）・/api/next・/api/pipeline・/api/seat・/api/metrics・/api/graph。変化の知らせは /api/surface/events の ledger-changed と board-changed。

## 3. 器の読み口（scribe2 の回答 2026-09-27T09:5xZ・state dir に書かないことは scribe2 が実測）
- 跨版を約束している面は 2 つ: event log `<state dir>/fleet/events.jsonl`（schema 1）と `scribe2 fleet export --state-dir S`（JSON・1 行目が header・以降 run と seat の行・過去の run も全部入る）。
- run の stage は閉じた 11 値: Intake・Reviewed・Blocked・Spawned・Questioned・RateLimited・Implemented・Gated・Landed・Stopped・Failed。Gated の判定は export に無く event log の RunStage の detail に在る。
- 口座の残量: event log の AllowanceMeasured / AllowanceUnmeasured。text の口は `scribe2 fleet usage --show --state-dir S`（`--show` を付けずに撃たない = 測り直して event を書く）。
- 席の健康: `scribe2 seat tick status --state-dir S`（target・last・age・healthy・heartbeat）。群と口座: `scribe2 doctor --state-dir S`（seat の行と group の行・current と next）。
- file を直に読む場合（版の約束は jsonl だけ）: `<state dir>/seat/<session>_<window>/state.jsonl`（busy / idle）・tick-last・account・move-signal。移動の履歴は `<state dir の親>/scribe2-host/groups/`。
- 4 状態は面の側で合成する: 動いている / 待ち = state.jsonl・応答なし = busy のまま age が seat.tick_stale_s を越えた・限度 = 残量 100% 以上か tick-last の reason=account-pressed。上限で終わった turn は busy のまま残る。

## 4. 審査の diff の上限 150000 byte
除外の宣言も値を変える口も無い（scribe2 の memo s2-07l.691・今日 2 回）。大きい fixture と写しの asset は、実装役が書いた file を設計席が先に main へ置き（tests の下か面の asset）、`scribe2 pipe follow --run <id>` で run を載せ直す。契約の行を書くときは、大きい file を作らせない（fixture は小さく・件数の多い入力は歯の中で組む）。

## 5. 着地の後の手（remote の宣言が無い間）
RunDone（Landed）を event log で確かめてから、`git status` → `cargo run -q -p xtask -- check` → push → `bdw close` → `bdw dolt push` → `bash ~/.cache/tsuzuri-admin/dispatch.sh`。RunDone の前に main で cargo を回すと器の着地の検証と lock を取り合う。着地の間は作業の木を汚さない（起草中の行は stash か席の作業場へ）。

## 6. folio2 の天井の 1 周の所見（2026-09-27・写しでの検証）
支持の 止める 6 件は tsuzuri の文書の直し（srs AC14・srs FR18・surface-board §8・憲法 P-13 の機構・P-26.2・surface §17 の問いの見分け）。画面の波の後にまとめて起こす。憲法の字に触る 2 件は持ち主に問う。門は folio2 の便 174 の着地まで撃たない。報告は ~/.local/share/folio2/handoff-2026-09-27/t3v-report.md。
