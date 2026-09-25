# scribe3 → tsuzuri の改名の手順（ADR-6・2026-09-25）

前提: 持ち主の裁定で器の名は tsuzuri。改名は **この repo の席（scribe3-13）を閉じてから** 1 度で行う。script は `docs/handoff/rename-to-tsuzuri.sh`（既定は dry-run・`RUN=1` で実行・段ごとに ok / skip を出す・2 度目は全段 skip）。

## 名の写し（実測・2026-09-25T00:39Z（名）/ 00:42Z（prefix））
| 場所 | 今 | 後 | 誰が |
|---|---|---|---|
| repo の dir | ~/projects/local-projects/scribe3 | …/tsuzuri | script |
| 作業場・site の dir | scribe3-spikes・scribe3-site | tsuzuri-spikes・tsuzuri-site | script |
| state dir | ~/.local/state/scribe2-v2-state-scribe3 | …-tsuzuri | script |
| git の設定 scribe2.statedir | 旧 state dir | 新 state dir | script |
| host の面の anchors（Tier1）| 7 つの state dir の host.toml に旧 path | 新 path（.bak を残す）| script（scribe2 席が `scribe2 vessel show` で検査）|
| 口座ごとの projects の dir（7 口座）| ~/.claude-accounts/*/projects/-home-shuu5-projects-local-projects-scribe3 | …-tsuzuri（会話記録と memory が付いてくる）| script |
| tick の unit | scribe2-seat-tick-s3_design.{service,timer}（旧 state dir を焼いている）| 席を登録し直してから `seat tick install` | scribe2 席 |
| 席の登録 | seat/s3_design（target s3:design）| 新 state dir の下で `seat register` → `seat launch`（target は s3:design のまま可）| scribe2 席 |
| tmux | session s3 / window design | そのまま（tmux の名は端末の慣習・器の名ではない）| — |
| 配信 | tailnet-serve の docroot が旧 path（hub :8100・site :8101）| 止めて新 path で起こし直す | s3 席（新） |
| 生きた試作 server | 8791〜8793・8803（旧 path の dir を配る）| 止めて起こし直す（pid は tsuzuri-spikes/live/*/pid）| s3 席（新） |
| 憲法の id | scribe3-constitution | tsuzuri-constitution（済・凍結前）| s3 席（済）|
| 台帳 | 無し | `bd init --prefix t3` を新 dir で（裁定済み）| s3 席（新） |
| 作業場の文中の旧 path | README・hub/index.html・server/report.md | sed で新 path | script |

## 順序
1. 持ち主が scribe3-13 の席を閉じる（tmux の window は残してよいが、window の shell は `cd ~` で旧 dir から出る。script の precheck は旧 dir を cwd に持つ process が 1 つでもあれば断る）。ThinkPad への ssh tunnel（scratchpad の tunnel.pid）は閉じる前に止めた。ThinkPad の Chrome の窓は残っていてよい。
2. scribe2 席（か持ち主）が `RUN=1 bash ~/projects/local-projects/scribe3/docs/handoff/rename-to-tsuzuri.sh` を 1 度実行。
3. scribe2 席が tick の unit を外し（`seat tick uninstall`）、新 state dir で `seat register` → `seat tick install` → `seat launch --role orchestrator --target s3:design`。
4. 新しい s3 席が docs/handoff/2026-09-24-s3-design-seat-resume.md を読み、配信と試作 server を起こし直し、CLI と prefix の裁定を受けて `bd init`。

## 戻し
script は mv と sed だけ（.bak を残す）。戻すには同じ script を OLD と NEW を入れ替えて実行する（`OLD=tsuzuri NEW=scribe3`）。
