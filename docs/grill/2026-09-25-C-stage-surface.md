# grill 論点 C — 表示面（session が制御する browser の面）の残り（討論の記録・正本は design-intent/ の YAML）

- 出自: docs/handoff/2026-09-24-v3-kickoff.md §1（持ち主の逐語 3 点目）・§2 論点 C・docs/handoff/2026-09-24-v2-leftovers-for-v3.md §2 の 1・比較試作の結果（design-note/bakeoff-surface.yaml §6〜§7）
- 状態: 討論中。裁定 1（2026-09-24T22:50Z・逐語は台帳へ）= click と入力も最初から用意する（対話的な表示面）。持ち主の指摘 = menu の無い独立した簡素な browser を組み込み remote でも local でも使えること（試作は Firefox で開いた）。§8 で答える
- 逐語は台帳へ

## 1. 決まっていること（verified）
- 表示面の形 = 1 つの表示 server + 契約 1 本 + menu の無い browser の面。席は命令（URL・viewport・reload）を送り、面は iframe を描いて受領を返す。remote / local の差は URL だけ。比較試作で 3 つの client が動作を実証（ADR-2・ADR-3）。
- 契約 v0.2 の決め（設計ノート bakeoff-surface §7・席の決定）: 状態は最後に適用できた値・server が値を検める・復元にも受領・受領は client id つき・起動時刻を持つ。
- 面の技術 = Rust の wasm（Leptos）・server は Rust の同期 server（ADR-3・ADR-4）。
- 憲法 P-20.3: 器は席の状態を描画からでなく構造から読む。N-5.2: app ごとの表示の仕組みを増設しない。N-6: 見せる先は端末内と tailnet の中だけ。

## 2. 残る決め（deduced）
1. **席の目（headless の client）の持ち方**。持ち主の browser は「見る」ための client。席が自分の app を確かめるには screenshot と DOM を読む目が要る。
2. **スマホの模擬の段**。iframe では DPR と UA を模擬できない（試作で実測・状態表示に文字で出しただけ）。本物の模擬は headless の browser 側で行うのが自然。2 段目（本物の emulator の画面配信）は要るときに足す（kickoff §2 のまま）。
3. **器が席の状態を読む口**。Claude Code の hook（SessionStart・UserPromptSubmit・PreToolUse・PostToolUse・Notification・Stop・SubagentStop・SessionEnd）と状態 file が構造の口（inferred・公式 doc の hook の一覧）。表示 server はこの event を受けて席の状態（待ち・実行中・止まった）を面に出せる。dialog の literal を足す形は持ち込まない（P-20.3）。

## 3. 事実（verified・2026-09-25）
| 事実 | 出所 |
|---|---|
| host に Google Chrome 148 が在る。Playwright の chromium（1140 / 1223）と headless shell も ~/.cache/ms-playwright に在る | which・ls |
| headless の Chrome の CLI 1 発で、試作の表示面（port 8791）を 390×844・DPR 3・iPhone の UA で撮ると 1170×2532 の PNG（約 184 KB）が出る。--dump-dom で DOM も取れる（約 3.3 KB） | 実測（scratchpad/eye.png） |
| 比較試作では 3 つの agent が 1 つの Playwright MCP を共有し、tab の取り違えと screenshot の置き場の問題が出た。folio2 の作法 D-6 は「並列の agent に MCP を共有させない」 | 各 report.md・folio2 rules D-6 |
| scribe2 の core は子 process を撃たず境界 crate が Invocation で撃つ（ADR-0062）。core は async を持たない（v3 憲法 P-26.3） | scribe2 host-init.md §2 |

## 4. 候補と評価（席の目）
| 案 | 中身 | 利点 | 代償 |
|---|---|---|---|
| (a) 表示 server が headless の Chrome を CLI で撃つ | 契約に「撮る」「DOM を読む」の口を足し、server が host の Chrome を子 process として 1 発ずつ撃つ（viewport・DPR・UA は Chrome の引数で本物の模擬） | 依存 0（host の Chrome）・core に async を持ち込まない・席ごとに MCP を持たずに済む・スマホの模擬 1 段目がここで本物になる | 対話（click・入力・待ち）はできない。要るときは CDP（Chrome DevTools Protocol）の client を足す判断が要る（A-3） |
| (b) 席が自分の Playwright MCP を使う | 今の形。席ごとに browser を持ち、表示 server は関与しない | 対話ができる・今日動く | 席の目が器の外に散る・並列の agent が MCP を共有して衝突（実測）・スマホの模擬が席ごとの設定になる・app ごとの仕組みに近づく（N-5.2 の趣旨に反する） |
| (c) 表示 server が CDP で 1 つの browser を常駐で持つ | server が Chrome を remote-debugging で起こし、websocket で命令 | 対話も screenshot も 1 か所 | CDP の client（依存 1 本か自前の websocket）と常駐 process の管理（P-23 の予算・P-11 の停止検知）が最初から要る |

## 5. s3 席の推奨
**(a)** を採り、(c) は撤退先として置く。契約 v0.2 に「撮る（screenshot・引数 = viewport・DPR・UA・待ち時間）」「DOM を読む」の 2 口を足し、応答は file の path でなく byte を返す（席は file を介さず読める）。席の状態の口は hook の event を表示 server が受ける形で、面は「席は何をしているか」を 1 行で出す（論点 B の裁定面と同じ状態表示）。2 段目の emulator は要るときに ADR で足す。

## 6. 問い（1 問）
席の目は「表示 server が host の headless の Chrome を 1 発ずつ撃つ（依存 0・対話なし・スマホの模擬は本物）」でよいか。前提 = 席の目に要るのは screenshot と DOM の静止画で、click や入力の対話は当面要らない。対話が最初から要るなら推奨は (c) に変わり、CDP の client の依存を A-3 で問う。

表示面について伝えきれていない要望（emulator・複数の app の同時表示・録画など）があれば、この答えに添えてほしい。

## 7. 経緯
- 2026-09-25: s3 席が headless の Chrome の実測を行い §3〜§6 を持ち主へ提示（答え待ち）
- 2026-09-24T22:50Z: 持ち主の答え = 対話（click・入力）は最初から要る。指摘 = 独立した簡素な browser（menu 無し）を remote / local の両方で。→ §4 の (a) は落ち、(c) を土台に §8 の形を提案

## 8. 対話を最初から持つ形（2026-09-24T22:50Z・s3 席の提案）

### 8.1 見落としの認め
§4〜§6 は「席の目」だけを問い、持ち主の窓（menu の無い独立した簡素な browser）を browser の tab に任せていた。これは持ち主の逐語（kickoff §1「ユーザー操作用のメニューなどを排除した session 側がコントロールする表示面」）の読み落とし。改める。

### 8.2 形（推奨）
**描くのは 1 つ。見るのは 2 つ。**
- **描く**: 表示 server が Chrome を 1 つ常駐で持ち、CDP（Chrome DevTools Protocol・websocket）で制御する。navigate・viewport（本物の DPR と UA の模擬）・reload・click・入力・待ち・screenshot・DOM の読みは全部この 1 つに対して行う。
- **席の目**: 同じ CDP に screenshot / DOM / console を求める（client 2 = headless の目）。
- **持ち主の窓**: 表示 server がその Chrome の画面を配信（CDP の screencast・JPEG の連続）し、持ち主の窓は画面を映して click と key を server へ返すだけの簡素な面。窓の中身は canvas 1 枚と状態 1 行で menu は無い。持ち主の操作も席の操作も同じ Chrome に届くので、「席が今何をしているか」を持ち主がそのまま見られる。
- **窓の殻（remote / local で同じ page）**: local = 器が Chrome の app mode（--app=URL・tab も menu も無い窓）で開く。remote（別の PC・スマホ）= 同じ page を PWA として home 画面 / desktop に入れると menu の無い独立窓になる（tailnet の HTTPS は tailscale serve が持つ・この host で既に /dash に使用中）。Tauri の殻は「後から任意」のまま。
- **裁定面**: 同じ窓の中の tab（対話面）。これは普通の page（Leptos）。
- **なぜ iframe を捨てるか**: 開発中の app は別 origin（別 port）なので、iframe の中へ click や入力を送ることも DOM を読むことも browser が禁じる。試作の表示面が「見るだけ」で済んだのはこのため。対話を最初から持つなら描画を server 側の Chrome に寄せるしかない。
- **スマホの模擬**: 1 段目 = CDP の device emulation（DPR・UA・touch）で本物に近い。2 段目 = 本物の emulator の画面配信は同じ「画面を配信し入力を返す」形に載るので、要るときに足しても面の作りは変わらない。

### 8.3 依存（A-3 の材料・verified）
- CDP の client = 同期の websocket 1 本。候補 tungstenite 0.30（default-features 無し）= 解決 17 crate（自前を含む）。async の実行系は要らず、core でなく境界（server）の crate に置く（憲法 P-26.3）。自前で websocket を書く案（依存 0・約 200〜300 行）は決定はしごの「1 行で書けるか」を越えるので退ける。
- Chrome は host の物（Google Chrome 148 が在る・Playwright の chromium も在る）。器が Invocation で撃つ（scribe2 ADR-0062 の形）。
- 画面配信は SSE（JPEG を base64）で始めて依存を足さず、帯域が足りなければ websocket へ（同じ tungstenite で足りる）。

### 8.4 代償と撤退
- 画面は画像なので持ち主の窓では文字の選択や拡大の鮮明さが落ちる（裁定面は普通の page なので影響なし）。
- 常駐 Chrome の停止検知と再起動（P-11）と予算（P-23）を server が持つ。
- 撤退（measure）: tailnet 越しの配信で 1 秒あたりの frame が閾値（rules 行で凍結・初期値は面の便で測る）を割るなら、持ち主の窓を「iframe で直に描く（見るだけ）」へ切り替える口を残す（契約の navigate / viewport は両方に効く）。

### 8.5 問い（1 問）
表示面を「server が持つ 1 つの Chrome を CDP で動かし、持ち主の窓はその画面を映して click と key を返すだけの簡素な面（local は app mode の窓・remote は PWA の独立窓）」の形にしてよいか。前提 = 依存を 1 本（同期の websocket）足す（A-3）。この前提を受け入れないなら、自前の websocket（約 300 行）で同じ形にする。
- 2026-09-24T22:57Z: 持ち主の裁定 = 「Rust・Chrome・入力や click の反映まで含めて試作して検証してから決める」。→ 作業場に契約 v0.3（contract-stage.md）と検証用 target-app を置き、agent 2 つ（stage-server = Rust + CDP + 画面配信 + 入力の中継・client-window = Leptos の窓 + PWA + app mode）を opus・予算 700k / 600k で並列に起動（2026-09-24T22:57Z）。結果は §9 に追記し、本決定の問いを出し直す

## 9. 対話的な表示面の試作の結果（2026-09-24T23:57Z・agent 2 つの report から転記・作業場 stage-server/ と client-window/）

**動いた（verified）**。Rust の同期 server が host の Chrome を起こし、CDP を websocket 1 本で握り、命令 8 種（navigate・viewport・reload・click・type・key・scroll・wait）・screenshot（PNG の byte）・DOM・console・画面配信（JPEG の SSE）・窓からの入力の中継（mouse・wheel・key・text）が全部動いた。歯は unit 10 本と Chrome を実際に起こす統合 2 本（検証用 app と、比較試作の裁定面へ navigate して行を click し逐語を打って記帳し jsonl が 1 行増える実 app の周）で、5 回連続で緑。持ち主の窓（Leptos）は frame を描き、click（スマホ viewport の縮小表示でも座標が合う）・文字入力（IME の確定を含む）・Backspace・wheel・2 秒の再接続・PWA の service worker が動いた。

| 物差し | stage-server | 窓 |
|---|---|---|
| click → 次の frame（localhost） | 中央値 35 ms（命令）/ 24 ms（窓からの入力） | — |
| frame | 60 fps・1 frame 9〜13 KB（JPEG q60） | 窓側で 22〜33 fps |
| 依存 | 23 crate（tungstenite の必須依存で +12）・cargo deny 全部 ok | 171 crate（Leptos と同じ） |
| build clean / 差分 | 2.6 s / 0.28 s | 24 s / 3 s |
| 手書き | 新規 1,513 + 写し 435 + 歯 612 + 型 165 | 614 |
| 型の住処 | 共有 crate（contract-rs/src/stage.rs） | 共有 crate・手写し 0 |
| --no-sandbox | 不要（adequately sandboxed を確認） | — |

**動かなかった・欠けている（verified）**。(1) headless の Chrome の画面配信は DPR を無視し、390×844 dpr 3 でも 390×844 の JPEG が届く（screenshot は 1170×2532 で正しい）＝窓はスマホの画面を CSS px の解像度でしか見られない。(2) websocket の握手の照合（Sec-WebSocket-Accept）を省いた（相手は自分が起こした localhost の Chrome）。(3) I/O thread が 2 ms ごとに送信の列を見に戻る（待機中 CPU 約 0.5%）。(4) Chrome が死んだら server の再起動が要る（自動再起動なし・黙らず 500 を返す）。(5) app mode の窓（local の殻）は host に画面が無く未実行。(6) 本物の IME と soft keyboard は未試験。(7) tailnet 越しの遅延と fps は未測定（localhost のみ）→ hub の生きた窓で持ち主が体感できる。

**読み（deduced）**。形は成立した。async の実行系は要らず、依存は websocket 1 本（+12 crate）で済む。DPR の件は「headed の Chrome（Xvfb か本物の画面）で配信する」「配信は CSS px のまま・鮮明さが要るときは screenshot を併用」「CDP の Emulation で width と height を dpr 倍にして dpr 1 で描く（layout が変わるので不可）」の 3 択で、推奨は 2 つ目（窓は見て操作する面・精査は screenshot）。

## 10. 持ち主の実測と直し（2026-09-25T00:16Z）
- 持ち主の裁定（逐語は台帳へ）: tailnet 越し（Firefox・RTT 30〜50 ms の経路〔tailscale ping で実測: Windows の desktop 32 ms・ThinkPad 49 ms〕）で、動きがもっさり・scroll が遅い・page2 への遷移が非常に遅い・文字入力の表示が遅い・日本語入力の確定で二重表示。**解決は必須**。
- 原因（code を読んで特定・deduced）: (1) 入力が 1 event ごとに新しい TCP 接続の POST で順送り（server は全応答 Connection: close）→ 1 event ≈ 2 RTT。(2) 画面配信に流量制御が無く 60 fps を SSE へ押し込む → TCP の buffer に溜まり古い画面を見る。(3) base64 の JPEG。(4) 日本語入力は input と compositionend の両方で送っており、Firefox は compositionend の後に isComposing=false の input が来るので二重。
- 直し（契約 v0.4・作業場 contract-stage-v04.md）: 窓と server を WebSocket 1 本に（下りは binary の JPEG・上りは入力と受領）・未受領 1 枚の流量制御と最新優先・RTT に応じた quality の自動調整・mousemove と wheel の rAF ごとの間引き・IME は Chrome と Firefox の両順序を純関数の歯で通し本物の Firefox でも確認・状態の行に RTT を表示。RTT 40 ms を模した中継で直す前後を測る。
- agent 2 つ（server・窓・opus）を 2026-09-25T00:16Z に起動。

