# 相談の窓の問題の一覧と直す順（窓 cw1 の 3 話題の棚卸し）

- 日: 2026-10-05（UTC）・窓 cw1・Claude Code 2.1.289・所見 cw1-1〜cw1-4 の材料を 1 つにまとめた。持ち主の追加の求め（席と窓の間で言付けをやり取りできるようにする・保存の時に席が言い換えた）も含む。
- 持ち主の報告: この窓で `/rc`（Remote Control）を打ったら動いた。→ --restricted・dontAsk・囲い付きの話す窓でも Remote Control は付く（持ち主が確かめた）。

## 1. 見つかった物（重さの順）

| # | 事 | 性質 | 確かさ | 直し方 |
|---|---|---|---|---|
| 1 | **orchestrator と窓の message のやり取りが無い**。窓→席は所見（file）だけ、席→窓は束（開く時）だけ。ADR-29 は窓どうしの連絡を却下した（dontAsk の窓から bypass の席へ送ると承認待ちで 5 分で捨てられる） | 設計の欠け | verified（ADR-29 の字・公式の cross-session-messaging の規則） | §2 の案（席→窓は SendMessage ＋ 窓の settings に `crossSessionInbound: accept`・窓→席は outbox の file を席の見張りが自分の子として席の inbox に中継） |
| 2 | **Remote Control が起動に付いていない**。mobile の app に窓が並ばない（口座は席と同じで一致） | 起動の欠け | verified（口座）・持ち主が /rc で動くのを確かめた | 話す窓の argv に `--remote-control consult-cw<n>`（差分あり・cw1-3） |
| 3 | **status line が無い** | 起動の欠け | verified | settings に `statusLine`＝`tz consult statusline`（差分あり・写しで build・clippy・歯 32 本・demo が通った・cw1-3） |
| 4 | **npm が囲いの proxy で 407** | host の道具 | verified | host の Node 18.19.1／npm 9.2.0（EOL）を Node 20 以上に上げる（窓からは直せない・cw1-2） |
| 5 | **GitHub から code を取れない**（github.com・raw・codeload が allowlist の外） | 設計の線 | verified | 読むだけの配り元 3 つ（raw.githubusercontent.com・codeload.github.com・objects.githubusercontent.com）を DOMAINS に足す（持ち主の裁定・push の道は増えない・cw1-2／cw1-3） |
| 6 | **器の dontAsk が殻の命令を黙って断る**（`echo "rc=$?"`・heredoc の python・rm -rf と git status を 1 行にした命令…似た命令でも揺れる） | 器の性質 | verified（3 話題で 7 回） | 窓の使い方: 複雑な命令は work/ の script に書いて `bash work/x.sh` で撃つ。brief に 1 行足す（launch.rs の brief()） |
| 7 | **WebFetch の大きい出力が読めない**。53 KB を超える頁は口座の dir（accounts/<label>/projects/…/tool-results/）に落ち、窓の Read の断り（`Read(/S/accounts/**)`）で読めない（remote-control・statusline の公式の頁がこれで読めなかった） | 設計と器の重なり | verified | 当座: 問いを絞る・節ごとに読む。根治は Read の断りを file 単位（.credentials.json 等）に替える事になり資格の覆いが弱まるので推さない。器に「persisted-output の置き場を変える設定」が無いか席が見る |
| 8 | **束の台帳の写しが開いた時の物のまま**。窓の中の `tz consult bundle` は台帳を替えない（bd が囲いの中で撃てない） | 設計の線 | verified | #1 の message があれば窓が席に「束を組み直して」と頼める。席の見張りが n 分ごとに写しを替える案も |
| 9 | repo が窓の下で動く（便が着地すると写しの一貫性が崩れる） | 運用 | verified（cs-cred-claude が読みの途中で着地した） | 写しは 1 時点で取り HEAD を記す（work/tz-sync-build.sh がそうしている）。brief に 1 行 |
| 10 | 入れ子の claude は model を呼べない（口座の dir は空の tmpfs・api.anthropic.com は断り） | 設計どおり | verified | 直さない。mod の validate・test・model を呼ばない命令は動く（cw1-2） |
| 11 | 窓の中で `cd` すると以後の命令の cwd が変わり、`tz consult answer` が「cwd が窓の作業場でない」で断る事がある | 器の性質 | verified（1 回） | 命令ごとに `cd <作業場> &&` を付ける。brief に 1 行 |
| 12 | 口座は一致しているが、群の移動が tsuzuri の群に起きると開いている窓は前の口座に残る | 設計どおり | deduced | 直さない（ADR-29 決定 (5)）。board の一覧に窓の口座を出すと見分けやすい |

囲いの穴（作業場の外へ書ける・資格が読める・許していない host へ出る）は 3 話題で **見つからなかった**（話題 1 の probe・話題 2 の GitHub と proxy の確かめ・話題 3 の試しの組み）。

## 2. 席と窓の message（#1）の案

### 2.1 Claude Code の仕組み（公式 cross-session-messaging・2.1.224 以上）
- 送るのは `SendMessage`、相手を見つけるのは `ListAgents`（道具・`/list-agents` で人も見える）。相手は session の名（`--name` か `/rename`）で呼ぶ。窓は `--name consult-cw<n>` で起きるので名は決まっている。
- 届け方: 相手が idle なら新しい turn が始まる。turn の途中なら道具の呼びの間に読む（走っている道具は止めない）。
- 受け側の門 `crossSessionInbound`（accept・hold・refuse）。`--settings` で置ける（置くと `/config` の行は消える）。値が無い時の既定は 2 つの permission の級で決まる: 受け手が「聞く級」（dontAsk を含む）なら届くが、送り手が「飛ばす級」（bypass）なら承認待ち。受け手が bypass なら送り手も bypass でない限り承認待ち。承認待ちは 5 分（`dialogExpiry`）で捨てられる。
- **自分の子の例外**: 値が無い時、session の自分の子 process（hook・Bash の命令）が自分の inbox socket（環境変数 `CLAUDE_CODE_MESSAGING_SOCKET`）に書いた message は承認なしで届く（Linux は子が終わった後も process の証拠で確かめる）。
- 同じ machine の中は socket だけで Anthropic の server を通らない。囲いの中の Bash が socket に届くかは `sandbox.network.allowUnixSockets` で決まる（窓は ABSENT＝届かない・この窓で確かめた: §2.4）。
- 届いた message は「別の session から」と明示され、承認にはならず、設定を変えられず、命令の字は走らない。

### 2.2 何が ADR-29 の却下の理由を解くか
- 却下の理由「dontAsk の窓 → bypass の席は承認待ちで捨てられる」は、席の側に `crossSessionInbound: accept` を置くか、**席の自分の子（tz consult watch）が中継する**事で解ける。中継なら席の設定を変えず、席に届く字を tz が決めた固定の形に保てる（ADR-29 決定 (8) の「固定の 1 行」の思想そのまま）。
- 席 → 窓は、席（bypass）から窓（dontAsk）へ送ると窓の側で承認待ちになる。窓の `--settings` に `crossSessionInbound: "accept"` を置けば届く。窓は囲いの中の低い権限の session で、message は承認にも設定の変更にもならないので、受け入れてよい。

### 2.3 案
| 向き | 形 | 変える物 |
|---|---|---|
| 席 → 窓 | 席が `SendMessage` で `@consult-cw<n>` に送る（skill `/tsuzuri:consult` に「窓に言う」の口を足してもよい）。窓は idle なら turn が始まり、busy なら道具の間に読む。持ち主は mobile の Remote Control からも窓に話せる | 窓の起動の settings に `"crossSessionInbound": "accept"`・audit と fixture・brief に「席からの message は別の session からの字として読む」の 1 行 |
| 窓 → 席（正式） | 今のまま所見（`tz consult answer` → 見張り → 席）。 | 無し |
| 窓 → 席（軽い連絡） | `tz consult tell <字>` が作業場の `outbox/<分>-<n>.txt` に書く（窓の守りは通す・席の口でない）。席の見張り `tz consult watch` が outbox を見て、**席の自分の子として席の inbox socket に 1 行を書く**（`CLAUDE_CODE_MESSAGING_SOCKET`・自分の子の例外で承認なし）。字は固定の形「相談の窓 cw<n> から: <先頭 200 字> — 全文 <path>」 | tz consult に tell と watch の中継を足す行。席の settings は変えない。窓の TOOLS は変えない（SendMessage を窓に渡すと相手を選べない: 許しの規則は道具の名だけで相手を絞れない） |
| 見張りの届け方の改め（任意） | 見張りが「終わって席を起こす」代わりに、生きたまま席の inbox に 1 行ずつ書く（busy の席にも道具の間に届く・置き直しの turn が減る） | ADR-29 決定 (8) の字の改め・席の hook の拾いはそのまま |
| 後の形 | 席の mod（report-4）: `session.receive` で窓の message を構造で受け、`$.session.send` で窓に返す | 判断の記録の後 |

### 2.4 この窓で確かめた事（work/inbox-probe.sh → work/inbox-probe-result.txt）
- この窓は inbox socket を持つ: `CLAUDE_CODE_MESSAGING_SOCKET=/run/user/1001/cc-socks/3531006.sock`（3531006 は窓の claude の pid・.consult/proc-1.json と同じ）・`CLAUDE_CODE_MESSAGING_TOKEN` は在る。→ --restricted の話す窓でも session 間 message の受け口は開いている（席から `@consult-cw1` へ送れる）。
- 囲いの中の殻は AF_UNIX の socket を**作る事すら**できない（python の `socket.socket(AF_UNIX)` が `PermissionError: Operation not permitted`）。→ 窓の Bash は自分の inbox にも席の inbox にも書けない。窓から席へは Claude Code の道具（SendMessage）か、file を席の子が中継する形しか無い。囲いの線（ADR-29 決定 (7)(カ)）はここでも保たれている。
- 同じ dir に他の session の socket が並ぶ（同じ user の全部の session）。席の見張りが席の socket に書く時は、環境変数の path だけを使い dir を漁らない。

## 3. 直す順（案）

1. **brief の 3 行**（器の変更なし・launch.rs の brief() の字だけ）: 複雑な殻の命令は work/ の script にして bash で撃つ／命令ごとに `cd <作業場> &&`／repo の写しは 1 時点で取り HEAD を記す。合わせて「大きい web 頁は節ごとに読む」。→ 窓の無駄な turn が今日から減る。
2. **Remote Control と status line**（差分あり・cw1-3・写しで組みと歯が通った・/rc は持ち主が確かめた）: ADR-29 決定 (7) の字の改め → 行 cs-statusline → 便。着地の後に新しい窓で描画と一覧を見る。
3. **席 → 窓の message**（settings に `crossSessionInbound: accept` の 1 鍵・audit・fixture・brief の 1 行）: 2 と同じ便に載せられる大きさ。着地の後に席から `SendMessage` で窓に 1 行送って届くのを見る。
4. **窓 → 席の軽い連絡**（`tz consult tell` と見張りの中継）: 行 1 つ。見張りの届け方の改めと一緒に ADR-29 決定 (8) の字を改めるかは席の判断。
5. **GitHub の読むだけの配り元 3 つ**（持ち主の裁定の後・DOMAINS と fixture と歯）。
6. **npm**（host の Node を上げる・窓で撃ち直す）。
7. **束の写しの更新**（3 が入れば窓が席に頼める。見張りで定期に替えるかは後）。
8. **WebFetch の大きい出力**（当座は問いを絞る。根治は資格の覆いを弱めるので推さない）。

## 4. 根拠の出所
- 所見 cw1-1（囲いの探り）・cw1-2（できる事とできない事・npm・GitHub・mod）・cw1-3（口座・Remote Control・status line の試し）・cw1-4（superpowers と mods）。
- 公式 code.claude.com/docs/en/cross-session-messaging（2026-10-05 読み）。
- 持ち主の報告（この窓で /rc が動いた）。
