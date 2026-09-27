# grill 論点 H — 指し示し（持ち主が画面の一部や画像を文脈として席に渡す）（討論の記録・正本は design-intent/ の YAML）

- 出自: 持ち主の要望 user 2026-09-27T05:37Z（逐語は t3-hub.1）。「用意された button や入力欄では言えない指摘」= アプリやエラーの screenshot と、アプリの特定の部分を囲って文脈として渡す機能。remote の端末から席の host へ届くこと。
- 状態: **裁定あり（持ち主 user 2026-09-27T05:44Z・逐語は t3-hub.1）→ 判断の記録 ADR-9 発効**。問いは §7。
- 関係する決定: ADR-5（表示面 = 端末の Chrome を ssh と CDP で操作）・ADR-7（裁定面・承認は面だけ・文の予算）・器の seat deliver（scribe2 main bed08a9）。

## 1. 要件の読み（deduced）
| id | 要件 | 読み |
|---|---|---|
| H-R1 | screenshot を席に渡す | アプリの画面・エラーの画面・端末の上の何でも。画像だけでも成り立つ |
| H-R2 | 特定の部分を囲って渡す | 囲んだ範囲の画像と、その範囲に在る要素（DOM）の素性。フロントエンドの直しに必須 |
| H-R3 | remote から届く | 持ち主は tailnet 越しの端末に居る。席は host に居る。端末 → host の経路が要る |
| H-R4 | 一言を添える | 指し示しだけでは意図が決まらない。逐語の一言を添える（空は可・画像だけの指摘も受ける） |

今の穴（verified）: 席は ssh と tmux の中の Claude Code で、remote の端末から画像を貼れない。持ち主は画面の場所を言葉で説明するしかない。

## 2. 事実（verified・2026-09-27・host の Chrome 148.0.7778.96 の `/json/protocol`）
- CDP の Overlay（experimental の印つき）: `Overlay.setInspectMode` の mode は `searchForNode`・`searchForUAShadowDOM`・`captureAreaScreenshot`・`none`。event は `inspectNodeRequested {backendNodeId}`・`screenshotRequested {viewport}`・`inspectModeCanceled`。= DevTools の「要素を選ぶ」と「範囲を撮る」と同じ口。
- 要素の素性: `DOM.describeNode`・`DOM.getOuterHTML`・`DOM.getBoxModel`・`DOM.getNodeForLocation {x, y}`・`CSS.getComputedStyleForNode`・`Accessibility.getPartialAXTree`。
- 画面と DOM を同じ瞬間に写す: `Page.captureScreenshot {clip}` と `DOMSnapshot.captureSnapshot`（全節点の layout の箱を持つ）。
- 席は ADR-5 の形で端末の Chrome に CDP の tunnel を持つ（双方向）。event は同じ tunnel で戻る。
- 器の `scribe2 seat deliver --ruling <id>` は固定の 1 行だけを待ちの席へ送る。id は ASCII の英数字と . - _ : の 64 byte 以下。中身は記帳から読む。
- Claude Code は画像の file を path で読める（Read）。

不確か（uncertain・spike で確かめる）:
- Overlay の inspect mode が app mode の窓（menu 無し）と ssh 越しの CDP で、DevTools を開かずに動くか。
- browser の `getDisplayMedia`（端末の他の窓を取り込む口）は secure context が要る。board は tailnet の http なので、席が窓を起こすときの flag（専用の profile・ADR-7 決定 (14)）か tailnet の https で満たせるか。
- `paste` の event で貼った画像を http の origin で受けられるか（inferred: 受けられる。`navigator.clipboard.read` と違い利用者の操作の event）。

## 3. 模型（席の提案）
**指摘** = 持ち主が面で作る 1 件の記録。承認ではない（P-12）。問いを閉じない。

| 欄 | 中身 |
|---|---|
| id | server が発行。`point:<YYYYMMDDTHHMMZ>-<n>`（seat deliver の id-shape を通る） |
| 一言 | 持ち主の逐語（空でも可） |
| 出所 | stage（端末の Chrome）・paste（貼った画像）・capture（端末の画面の取り込み）・stream（配信の窓） |
| 画像 | 全体 1 枚と、囲みごとの切り抜き |
| 囲み | 画像の座標の矩形の列（1 件に複数可）。種類は 囲み・要素 |
| 頁の文脈 | URL・題・viewport・DPR・scroll・時刻（出所が stage か stream のときだけ） |
| 要素 | 囲みごとに節点の列（上限つき）。selector の path・tag・id・class・役割と名前（a11y）・字・箱・主な computed style・outerHTML の抜粋・source の手がかり（data 属性） |
| 直前の異常 | console の error と失敗した request（直前の数十秒・上限つき） |
| 宛先 | 今の論点の bead（席が状態に出している論点）。無ければ根 |

**置き場**
- 画像と DOM の写しは `<state_dir>/points/<id>/`（machine-local・repo に入れない）。画面には患者や個人の情報が写りうる。版管理と台帳の remote へ出さない（N-6・D-4）。
- 台帳には bead 1 本（一言・置き場の path・要約値）。指摘は「反映されていない要望」として未反映の一覧に載る（ADR-7 決定 (8)）。
- 消すのは A-1（消す）の問い。自動では消さない（N-1）。面は置き場の大きさを出す。

**導出グラフ**
- 節点の種類に「指摘」を足す（21 種目）。席が処分を宣言するとき `touches`（指摘 → 変えた節点）を張る。処分の無い指摘は不変条件で数える（候補 G-13 = 宙に浮いた指摘 0）。

**席に届く形**
1. server が bead と file を書く。
2. 席が待ちなら `scribe2 seat deliver --ruling point:…` の 1 行。考え中なら停止の hook が未読を拾う（裁定と同じ 2 経路・ADR-7 決定 (4)）。
3. hook が注入するのは 一言・切り抜きの path・要素の要約（上限つき）・「承認ではない」の 1 行。画像そのものは席が path で読む。
4. 席は指摘ごとに処分を宣言する（改める・起こす・変えない）。

器の固定の 1 行は「裁定 <id> が届いた」で、指摘でも字は「裁定」になる。機械の上は通る（id の接頭辞で hook が見分ける）。字を一般にするかは器の側の memo の候補（急がない）。

## 4. 取り込みの経路（候補と評価）
| 案 | 持ち主の手 | 仕組み | DOM | 届く範囲 | 評価 |
|---|---|---|---|---|---|
| (a) stage を撮る | board の button 1 つ → board の上で囲む | server が CDP で screenshot と DOMSnapshot を同時に取る。囲みから要素への対応は写しの上の純関数 | 有 | 端末の Chrome に出ている頁 | **v1**。頁が後で変わっても写しで解ける。関数は host の cargo test に乗る |
| (b) stage の上で指す | board の button → アプリの窓で要素を click か範囲を drag | Overlay.setInspectMode。1 回で mode を戻す。Esc で取り消し | 有 | 同上 | **v1**（spike の後）。いちばん自然。アプリの DOM を汚さない |
| (c) 画像を貼る・落とす | OS の screenshot を board に貼る → 囲む | browser の paste と drop。upload | 無 | 端末に見える物すべて（native・エラー・他の窓） | **v1**。remote から画像を渡す最短の道 |
| (d) 端末の画面を取り込む | board の button → 窓を選ぶ → 囲む | getDisplayMedia で 1 frame | 無 | 同上 | v1.1（secure context の spike の後）。OS の道具が要らなくなる |
| (e) 配信の窓で囲む | スマホの窓で drag | 自前の client が座標を送る。server が headless Chrome で解く | 有 | ssh の届かない端末 | v1.1（配信の窓の便と一緒） |
| 却下: アプリの頁に道具の帯を注入する | 頁の中の button | CDP で script を注入 | 有 | — | 却下。開発中のアプリの DOM と layout を汚し、歯と干渉する |
| 却下: 端末側に常駐の道具 | — | agent を入れる | — | — | 却下。ADR-5 決定 (4)（端末側の常駐の agent は作らない） |

決定はしごの読み: 要る → 既に在る（CDP の口と browser の paste）→ 足すのは board の「指し示しの欄」と囲みから要素への純関数と置き場だけ。

## 5. 囲みから要素への対応（純関数・core）
- 入力: DOMSnapshot の layout の箱の列と、囲みの矩形。出力: 節点の列（上限 12・初期値）。
- 選び方: 箱が囲みに 50% 以上入る節点のうち、いちばん外の 1 つ（囲み全体を言う要素）と、字か操作を持つ葉の節点。順は paint の順。
- 出さないもの: 入力欄の値（password・個人の情報）は伏せる。outerHTML の抜粋は上限つき。
- 歯: 固定の snapshot の fixture と矩形で、節点の id の列が snapshot と一致。上限を超える囲みは切って総数を出す（黙って落とさない）。

## 6. 対象の範囲との関係
指し示しの (c)(d) は画像だけで成り立つので、native のアプリやスマホのアプリにも届く。要素の素性まで渡せるのは DOM を持つ対象（web と WebView 系）だけ。対象の範囲の決めは論点 C §12。

## 7. 問い（1 問）
指し示しの v1 を「board の指し示しの欄で、(a) stage を撮って囲む・(b) stage の上で要素を指すか範囲を囲む・(c) 画像を貼って囲む」の 3 つにし、(d) 端末の画面の取り込みと (e) 配信の窓は次の版にしてよいか。指摘は承認として数えず、画像は state dir に置いて版管理へ出さない。推奨: この形。答えの後に spike（Overlay の inspect mode を app mode の窓で）→ ADR → 要件と便の行。

## 8. 経緯
- 2026-09-27T05:37Z: 持ち主の要望。席が CDP の定義を host の Chrome で確かめ、案を起草。
- 2026-09-27T05:44Z: 持ち主の裁定「よい」→ ADR-9（accepted）・要件 FR18・受入 AC15。次 = spike（要素を選ぶ口を app mode の窓と ssh 越しの CDP で）→ 設計ノートと便の行（便 a の着地の後）。
