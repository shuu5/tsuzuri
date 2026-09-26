# grill 論点 B — 裁定面（裁定を取る対話面）の最小の形

- 出自: 持ち主の要望（v3 kickoff）「GUI を追加し口座と pipeline の管理だけでなく project ごとの台帳の提示とすり合わせを可能にする」。scribe2 席の叩き台 = 「本命は台帳・契約・裁定を持ち主に見せて合意を取る面。裁定は GUI が受け取り、UTC の分つきの id を機械が発行して台帳へ直接記帳する」。
- 状態: 討論中（1 問目を提示）

## 1. 決まっていること（verified）
- 面の技術は Rust の wasm（Leptos）・server は同期の Rust（ADR-3）。表示面は端末の Chrome を app mode で席が起動して操作する（ADR-5）= 裁定面もその窓に出せる。
- 憲法 P-12.1: 持ち主の承認は R-8 が指す **1 つの対話面**を通ったものだけを受け取る。R-8 の今の値 = 「持ち主と orchestrator 席の対話面（記帳先 = 台帳の notes の逐語 + 文書の承認欄 + 器の承認の event）」。D-3: 問いは対話面の散文で 1 論点 1 問・選択式の窓は使わない。
- 比較試作（ADR-2・design-note bakeoff-surface §6）で裁定面の輪（台帳の一覧 → 詳細 → 逐語の入力 → `POST /api/rulings` → 機械が `<item>@<UTC の分>#<連番>` の id を発行 → notes に追記 → 一覧を読み直す）は 3 つの client で動いた。契約 v0.2（§7）の決めも在る。
- 今日の事故の型（scribe2 席の実測）: 裁定 id の分の写し忘れ・逐語の tracked への混入・口座の自動移動の停止 = 対話面が無く、席が chat から手で写しているため。

## 2. 残る決め（deduced）
1. **裁定面 v1 の範囲**（この 1 問）。
2. 裁定 id の形（機械発行 `<item>@<UTC の分>#n` に統一するか・今の「user <UTC の分>Z」を併記するか）。
3. R-8 の値の改訂（対話面 = GUI を指す・規則の行の値の変更 = 裁定 id が要る・条文は凍結済みで触らない）。
4. 席への通知（裁定が入ったら席が知る形 = 器の event → SessionStart / UserPromptSubmit の brief に載る、か tick）。
5. 口座・便の管理の面は v1 の後。

## 3. 事実（verified・2026-09-25）
- scribe2 の docs/design/dialogue-surface.md（次の 1 手が先頭・事実の 4 区分）が既に在り、裁定面はその carrier になる（kickoff §2）。
- scribe2 の器の承認の event（憲法 C7・A1）は「逐語つきの event」を要求する。裁定面が発行する id と逐語を同じ event に書けば、台帳の notes・文書の承認欄・event の 3 か所が 1 つの出所から出る。
- 台帳（beads）は host の bd で読み書きできる（`bd --readonly show` / `bd update --append-notes`）。裁定面の server は席の代わりに bd を撃つ = 起票の門（PreToolUse）の外側なので、形の判定は server 自身が持つ必要がある（notes-replace を撃たない・親の無い create を撃たない）。

## 4. 候補（v1 の範囲）
| 候補 | 範囲 | 評価 |
|---|---|---|
| (a) **裁定の輪だけ** | 台帳の bead の一覧と詳細（open の epic・便・memo）・席が置いた「問い」（1 論点 1 問・推奨 1 つ・散文）の表示・逐語の入力 → 機械が裁定 id を発行し、台帳の notes と器の event に記帳し、席へ通知。文書の承認欄は席が id を写す（v1 では機械が書かない） | 今日の事故の型を全部塞ぐ。試作で輪は実証済み。口座・便の管理は入れない |
| (b) (a) + 文書の承認欄の自動記入 | 裁定面が design-intent の ADR の approval 欄も書く | 出所が 1 つになるが、面が tracked の file を書く = commit の主体と権能（edit-design-intent は席の権能）の設計が要る。v1.1 へ |
| (c) (a) + 口座と便の管理 | 口座の一覧・移動・便の起動と停止 | kickoff の要望に在るが「付随」。裁定の輪が動いてから |

## 5. s3 席の推奨
**(a) 裁定の輪だけ**を v1 とする。「問い」は席が台帳に置く（memo の子の bead か、bead の欄）ので、grill の 1 問は chat でなく裁定面に出る = R-8 の値を「裁定面」へ改める裁定を同じ ADR で取る。文書の承認欄は v1 では席が写す（写し忘れは床の check が数える = 既に在る門）。

## 6. 問い（1 問）
裁定面 v1 の範囲を **(a) 裁定の輪だけ**（台帳の一覧と詳細・席の問いの表示・逐語の入力 → 機械が裁定 id を発行して台帳と器の event に記帳・席へ通知。口座と便の管理と承認欄の自動記入は後）にしてよいか。あわせて、kickoff の要望で「伝えきれていない」と言っていた裁定面まわりの要望があれば、ここで足してほしい。

## 8. 持ち主の要件を受けた練り直し（2026-09-25T01:50Z・検討 agent 3 つの出力を席が統合・出力の原本は scratchpad の design-b-ux / -ledger-view / -mechanism.md）
持ち主の要件（user 2026-09-25T01:43Z・逐語は t3-hub.1）: まとめて承認・全体の方針の自由記述・席がリアルタイムに面を更新して面越しの対話・台帳と epic の全体像が直感的に分かる高機能な表示面。

### 8.1 全体の形（席の統合案）
- **正本は台帳 1 つ**。問い・裁定・方針・席の受けは全部 beads の bead と notes に残り、server は「即時の知らせ」と「記帳の代行」だけを持つ揮発の写し（機構の (γ)）。server が落ちても問いは台帳に残り、起動時に拾う。
- **対話の単位を台帳で表す（新しい型は足さない）**:
  | 単位 | 置き場 | 形 |
  |---|---|---|
  | 問い | 論点の bead の**子の bead**（1 論点 1 問・label で「問い」と印・束は label）| description = 平易な説明・推奨 1 つ・理由・前提にした方針の id。open = 未回答 |
  | 裁定 | 問いの bead の notes に追記し、server が閉じる | `裁定 <問い id>@<UTC の分>#n（束 <束 id>）: <逐語>` |
  | 方針 | 根の直下の memo「方針」の notes に追記（範囲の札つき・裁定の控えとは別）| `方針 <範囲>@<UTC の分>#n: <逐語>`。問いを閉じない・承認として数えない |
  | 席の受け | 問いの bead の notes に追記 | `受け <席>@<UTC の分>: <1〜2 行・信頼度の語つき>` |
  却下: 問いを notes の定型行に書く（自由文を判定に使う・「答えた」の印が付かない）／description に置く（上書きで履歴が消える）／方針を問いと同じ一覧に混ぜる。
- **まとめて承認**: 1 問 1 id + 束の id（`batch@<分>#n`）。束の各行に「含める」と個別の欄、個別の文が無い行は束の逐語。**見た版への裁定**（面が表示した問いの版を送り、食い違えば 409 で 1 byte も書かない）。全件を先に検めてから順に bdw で書き、途中で落ちたら巻き戻さず「束 <id> 中断: k/N」を控えに残して赤字で出す。冪等は「notes に同じ id が在れば飛ばす」。**取り消しは可逆**（消さず、新しい裁定が前の id を名指して問いを open に戻す・承認欄の検査は取り消されていない最新の id だけ有効）。
- **方針の自由記述**: 面の下端に常時の入力欄（範囲: この論点 / 全体）。席は次の問いを置く前に範囲内の方針を読む（問いの要求が premises に方針の id を持たないと server が 409 = 歯にできる）。方針に「全部推奨で」と書かれても席は問いを閉じず、束の承認として押してもらう（線引き）。
- **面越しの対話の体験**（縦の並び）: (1) 次の 1 手 1 行 → (2) 未回答の束（論点ごとに 1 枚・A-1 の印〔消す / 出す / 使う〕は束に入れず単独）→ (3) 時系列（問い・裁定・方針・受けを 1 本・発言者の印）→ (4) 方針の欄。上端に席の状態「待ち / 考え中（問い X）/ 応答なし」。裁定から 2 分で受けが無ければ「席に届いていない」を赤で出す（黙って待たせない）。答えた後は「待ち（赤）→ 答え済・席が未読（灰）→ 記帳済で消える」の 3 段。
- **chat との分担**: R-8 を面へ改めた後、chat は承認を運ばない（作業の指示・「詳細版で」・緊急の停止・面が落ちたときの連絡だけ）。chat で承認らしい文が来たら席は記帳せず面の問いを指し返す。
- **今日の事故の型を塞ぐ**: id は server だけが発行（席は打たない）／server の書き口は bdw と器の event だけで repo の file を書く経路を持たない／A-1 の 3 クラスは問いに印を付け、束に入れない・対象と先と鍵を面に明示・個別の逐語を必須。

### 8.2 台帳と epic の全体像（表示面）
- 持ち主が知りたい 9 問: Q1 私の答え待ちは何か・Q2 何が動いて何が止まっているか（理由）・Q3 今日何が着地したか・Q4 この束はどこまで来たか・Q5 次に何が動けるか・Q6 これは何のためでどこから来たか・Q7 どれが難航しているか・Q8 未整理の memo・Q9 器は黙って止まっていないか（.618 の実例: 口座待ちで 2 時間、どの面も名指さなかった）。
- 現物の形（scribe2 の台帳 668 本・epic 1 本に子 618）: 親子の木は平たく全体像の束にならない → **中間の束は設計 doc（契約表の pointer）で作る**（台帳を 1 byte も変えない）。status の実際は open / closed の 2 値 → **動いている / 止まっているの正本は器の event**（C15: 器が台帳に書くのは close だけ）。
- 眺めの評価: **今の板**（待ちの帯・状態の 5 列・今日の着地）と **束**（epic → 設計 doc → 行 → memo・進み棒）が Q1〜Q9 の 8 問に届き、CSS grid と生の SVG だけで 1280 幅と 390 幅に成立。親子の木は作らない（平たい）。依存の図は 1 bead の周り 2 段だけ（後の版・今は文字の鎖で足りる）。時間軸（14 日の着地・便の回数）は v2。
- **直感的の物差し（受入条件）**: (1) 開いた最初の画面で scroll せず Q1〜Q3 の数が 3 秒で読める（1280・390 の両方）(2) どの open の bead にも home から 2 click 以内・一番古い問いには 1 click (3) 390 幅で横 scroll 0（席が CDP で実測し report に残す）(4) 器の event から面まで 5 秒以内・「最終更新」を表示・読めないときは 0 件でなく「測れていない」(5) 見出しは「待ち・動いている・止まっている・着地」の 4 語・題は 40 字・止まっている便には理由の 1 行を必ず。
- 問いと裁定待ちの見え方: 上部の帯「あなた待ち N」（全眺めに固定・押すと古い順）・bead の「問」の印に待ちの長さと「止めている便 n 本」・論点ごとの束に「3 問」。data の源は構造化した記録（問いの bead・rulings の記録・器の event）だけ。notes の自由文を正規表現で拾わない。
- ADR-5 との結び: 眺め・絞り込み・bead・問いの全部に URL を持たせ、席は「ここを見て」と URL を開くだけで持ち主の目を連れて行く。

### 8.3 機構（Rust の同期 server + Leptos + beads + 器の hook）
- **push は SSE**（裁定面は低頻度の文字・EventSource の再接続と Last-Event-ID の再送が標準）。WebSocket は表示面の画面配信の用。server は 1 process・1 port・口の前置きで分け（/api/ledger・/api/rulings・/api/surface/*・/api/stage/*）、裁定面の thread は Chrome の死に巻き込まれない。
- **台帳の監視は std の poll**（thread 1 本・500 ms ごとに .beads/issues.jsonl の mtime と長さ・器の seat state と event log も同じ間隔）。席が問いを置く = **bdw で書く + server へ合図の POST の 2 手**（合図が落ちても poll が拾う）。CDP で裁定面の中身を変える形は採らない（正本を経ない）。
- **server の台帳の書きは門の外なので、4 形を型で表せなくする**: 閉じた enum LedgerWrite（AppendNotes・CloseItem・CreateChild〔parent 必須〕）から argv を組み常に bdw を経る。歯で全 variant を起票の門の判定の純関数に掛けて deny 0。
- **持ち主の裁定が席に届く形**（決定はしご: 要る → 在る = dispatcher の deliver_within〔send-keys に入力欄の門・settle・Enter の再送〕→ 標準 = Stop hook の block → 足すのは 2 つ）: 席が**待ち**なら server が器の口（新設・中身は deliver_within 1 回）で 1 行の指し示し「裁定 <id> が届いた」を送り、UserPromptSubmit が event log から逐語を注入（日本語と長文を打鍵に通さない）。席が**考え中**なら送らず、Stop hook が未読を見つけて block と逐語で turn を続けさせる。60 s の tick は予備。席の自前の poll は採らない（token を焼く）。衝突: FR44「差し込みは 3 つだけ」の 4 つ目になる（裁定が要る）。
- 面に出す最小の event 3 つ: SeatState（器の打刻から・席の手 0）・QuestionPosted（合図の POST 1 本）・RulingRead（器の注入の記録から・席の手 0）。
- 契約 v0.5 の草案: GET /api/ledger と /api/ledger/<id>・GET /api/surface/events（SSE）・GET /api/surface/state・POST /api/surface/questions（合図）・POST /api/rulings（+ in_reply_to）・POST /api/rulings/batch・POST /api/policy。contract-rs に足す型は 9 つ（SurfaceEvent・SurfaceState・QuestionNudge・RulingRequest の拡張・BatchRulingRequest・BatchRulingResponse・ItemOutcome・PolicyRequest・LedgerWrite）。
- 未確認 2 点（歯で測る）: bd の auto-export が issues.jsonl に出る遅れ／Stop hook の block で turn を続けさせる形の器での扱い。

### 8.4 v1 の範囲（練り直し）と受入条件
| v1 | 後の版 |
|---|---|
| 問い / 裁定 / 方針 / 受けの台帳の形・束の記帳（1 問 1 id + 束 id・一部別答え・見た版の 409）・取り消し（追記と reopen）・方針の欄と premises の 409・SSE の即時表示・席の状態と「届いていない」の表示・席を起こす口・A-1 の印の束除外・眺め = 今の板 + 束（設計 doc 単位）・全部に URL | v1.1: 承認欄の自動記入・「決まったこと」板。v2: 時間軸（14 日・便の回数）・口座と便の管理。v3: 依存の図（周り 2 段） |

受入条件（実行できる歯・server の cargo test を一時の beads と偽 bdw・偽 tmux で回す）: (1) 3 問の束 → 裁定 id 3 つ別々・同じ束 id・3 つの notes の末尾がその行・closed (2) A-1 の印を含む束は 400 で台帳の file は byte 単位で不変 (3) 見た版が古い要求は 409 で何も書かない・取り消しは前の id を名指して open に戻す (4) 範囲内の新しい方針を premises に挙げない問いは 409・承認欄の検査は方針の id を拒む (5) 問いを bdw で置くと合図ありで 200 ms・なしで 1.5 s 以内に SSE へ届く・裁定の POST 1 本で notes 1 行と event 1 行が同じ id と逐語・待ちの席は送達 1 回・考え中の席は送達 0 回で Stop hook が block (6) 束で 2 件目が失敗しても応答が item ごとの成否を持ち、再起動後に失敗分だけ撃ち直して重複 0 (7) LedgerWrite の全 variant が起票の門で deny 0 (8) 直感的の物差し 5 本（8.2）。

### 8.5 決めの列（1 論点 1 問で順に問う・推奨つき）
1. **v1 の範囲を 8.4 の形にするか**（推奨: する。§6 の (a) に束の承認・方針の欄・即時の往復・席を起こす口・今の板と束の眺めを足し、口座の管理と承認欄の自動記入は後）。
2. **承認の event を誰が書くか**（推奨: 裁定面の server が逐語つきの event を直に書く = R-8 と scribe2 由来の「承認の event は orchestrator 席からだけ」の値を「裁定面の server」を含む形へ改める。席に写させると写し忘れの型が残る）。
3. **席を起こす 4 つ目の差し込み（FR44）を開けるか**（推奨: 開ける。1 行の指し示しだけ・逐語は hook が注入。開けないと待ちの席は 60 s の tick まで知らず体験が崩れる）。
4. **束の「推奨で」の逐語**（推奨: 持ち主が束の欄に打った文を逐語とし、空なら押せない。button の文字を逐語にすると P-12 の「言葉どおり」が薄れる）。
5. **中間の束は設計 doc で作る／動いている・止まっているの正本は器の event／home は待ちの帯 + 今の板**（推奨: いずれもそのとおり）。
6. 文書の承認欄の逐語（P-12.2 は承認欄に逐語を求める・凍結済みの条文）: 推奨は今のまま（承認欄の逐語は許された 1 か所・混入の事故は他の tracked への写しの話）。改めるなら憲法の改訂 = 別の裁定。

## 9. 問い（1 問・2 問目以降は 8.5 の順）
裁定面 v1 の範囲を **§8.4 の形**にしてよいか。提示面（HTML）は tailnet の site に置いた（URL は chat で）。

## 11. 広げ直し — beads と design-intent を 1 つのグラフにして裁定面の土台にする（2026-09-25T02:17Z・検討 agent 3 つ〔graph-model・graph-survey・graph-view〕の出力を席が統合・原本は scratchpad の design-g-*.md）
持ち主の裁定（user 2026-09-25T02:09Z・逐語は t3-hub.1）: 裁定の材料である beads と design-intent はそれ自体が高度に連携し、両方ともグラフ化でき、その統合グラフを表示面に出し、裁定と考察の材料に使い、持ち主に伝わりやすく AI にも見落としなく精査できる仕組みとして全体像を把握したうえで仕上げる。これが v3 を新しい repo に切った理由。

### 11.1 前提の実測（verified）
- folio2: FR14「機械が読む id の索引」（節点 11 種・辺 17 型・毎回組み直す導出物・文書は節点にしない = ADR-14）と FR21「全体像を短い出力」= `folio graph --print / --digest`。folio2 自身は節点 224・辺 1116。
- scribe: beads の issue グラフが前提の土台（scribe v1「issue グラフ無しに scribe は成立しない」）。scribe2 は契約表 → `bd create --graph` の plan JSON（ledger-form §5・§9）。台帳 669 本・parent-child 651・blocks 266・relates-to 182・discovered-from 43・次数の中央値 2・最大 619（根の epic）。graphify（code のグラフ）は scribe v1 の token 節約の候補で scribe2 では範囲外。
- 両者の結びは 5 本（契約 bead の pointer 行 `design = <doc>#<row>`・ADR の approval.ruling の台帳 id・memo の出所節・ADR の produced・xtask の契約表 → 台帳）だが、多くが散文か片側だけで機械が辺として辿れない。
- 不具合と直し（済・aa3964a）: tsuzuri の rules.yaml が引用符つきの key と id で書かれ、folio の 2 つの読み手（YAML の側と行の逐語を byte で切る側）が食い違って `folio graph --print` が組めなかった。値を 1 字も変えずに裸の形へ直し、凍結の照合は合格のまま、節点 161・辺 254 で組める。床がこの食い違いを捕まえない穴は folio2 席へ便の候補として送った。

### 11.2 統合グラフの模型
- **節点 18 種**（§14 で走行を足して 19 種・mock の宣言と同じ。以前「17 種」と書いたのは数え違い）= folio の 11（条・規範文・規則行・目的・要件・非機能要件・受入基準・制約・登場人物・出力・判断の記録）+ **設計ノートの行**（id `<note>#<row>`・ADR-14 の「id を持つ行」の範囲）+ bead 系 4 種（epic・契約・memo・問い = 4 象限の純関数で分ける）+ 裁定 + 方針。文書は節点にしない（ADR-14 を保つ。「設計 doc 単位の束」は行の file 属性で束ねる眺め）。
- **辺** = folio の 17 型 + beads の 4 型 + **結びの 6 型**:
  | 結びの辺 | 向き | 張る者 | 正本 |
  |---|---|---|---|
  | design | 契約 → 設計ノートの行 | xtask の plan と席 | bead の acceptance の pointer 行 |
  | ruled_by | ADR・規則行・発効文書 → 裁定 | 席が発効時に書く | 承認欄の型付きの 2 欄（台帳 id と裁定 id） |
  | answers | 裁定 → 問い | 裁定面の server | notes の定型行 |
  | touches | 問い → 任意の節点 | 席が問いを置くとき | bead の metadata の閉じた key |
  | premises | 問い → 方針 | 席 | metadata（§8.1 の 409 と同じ正本） |
  | source | memo → 裁定・run | 器が memo の plan に写す | metadata |
  「裁定 → 変えた節点」は書かず ruled_by の逆向きとして導く（同じことを 2 か所に書かない・P-6.4）。散文の走査は辺を作らず、「文で指したなら欄にもある」の検査にだけ使う。
- **id の名前空間**: 内部の key は `<repo>:<id>`。形は 4 族で互いに素（folio の id は大文字で始まる・bead は小文字 prefix と `-`・行は `#` を含む・裁定と方針は語で始まる）。repo をまたぐと R-1 が衝突するので修飾は必須。
- **見落としを無くす不変条件 10 本**（3 値・読めなければ「まだ分からない」）: G1 辺の両端が実在／G2 open の契約は実在する設計の行をちょうど 1 つ指す／G3 発効した ADR・規則行は ruled_by を持ち、先の bead の notes にその裁定の行が在る／G4 問いは touches を 1 つ以上持ち、見た版から要約値が変われば古い印／G5 孤児 0（全 bead が根から parent-child で届く・未着地の契約行はどれかの bead に指される）／G6 blocks に循環無し・parent-child は木／G7 宙に浮いた裁定 0／G8 4 象限の違反 0／G9 本文で名指した id は欄か辺にもある（detect → 2 周の実測の後に deny）／G10 id の 4 族が互いに素で repo 内で一意。
- **計算の置き場**: 毎回組み直す導出物（repo に書かない・folio の流儀）。入力は design-intent の YAML・design-note・issues.jsonl・宣言した外部の台帳（scribe2 は読むだけ）・器の event log。実測 folio graph 0.08 s・issues.jsonl 7.4 MB の読み 0.04 s → 統合で 0.5 s 未満（inferred）・server の 500 ms の poll ごとに組み直せる。器の event（動いている・止まっている・着地）は節点の属性で辺にしない（正本は event・C15）。**ADR-4 への収め方**: folio の crate は設計の索引だけ（台帳を読まない）→ `--json`（節点 5 欄・辺 3 欄）の型を面の契約の型の crate に置いて境界にし、結合と G1〜G10 は scribe の core が持つ。
- **口**: `tz graph --print / --digest / --around <id> --hops 2 / --check / --json / --dot`（graphviz は依存に足さない・SVG は面が JSON から描く）。

### 11.3 AI が見落としなく精査する仕組み
- **骨は folio2 の天井の教訓**: 束を切らない。近傍の一覧は「ここだけ読め」でなく「ここから見ろ」。関係の有無を機械が決めない（P-19.2 道具の判断代行の禁止）→ **近傍の全節点に席が処分を宣言する形**（touches か not-relevant〔理由 1 句〕）で、振っていない節点が 1 つでも残れば断る。判断は席に残り、黙って落とすことだけを塞ぐ。
- **精査の問いの型（閉じた一覧）**: around（近傍 2 段）・impact（裁定の touches から依存の逆向きへ 2 段）・trace（bead → pointer → 行 → basis → 条）・cites（条を根拠にする ADR と便）・unruled（発効 ADR − 台帳で解ける ADR）・orphan / dangling・changed（要約値の差 + 2 段）。出力は全型同じ形（`id / 種類 / 題 36 字 / 辺の型 / 段 / 経由 id`・末尾に `# shown=n total=m`・切った周は `# cut reason=hub|cap`・読めない周は `# unmeasured`）。**hub の扱い**: 親子は子 → 親の 1 向きだけ辿り兄弟へ広げない・次数が閾値（初期値 30・fixture で決める）を超える節点は畳んで総数を出す。
- **材料の束（bundle）**: 組む時点は 3 つ（問いを置く前・裁定を受けた後・ADR を accepted にする前）。中身は起点 → 常に入れる集合（範囲内の方針・同じ論点の未答の問い・起点が挙げる条）→ 近傍 2 段 → 参考（散文の言及・v1.1）→ 切った跡と測れなかった跡。上限は節ごと 60 行・全体 200 行（folio2 の実測: 2 段の近傍は中央値 49・最大 166）。天井から流用 = 束の要約値・記録の read 欄・3 値。流用しない = 観点ごとの束（N-5.1）・面の HTML の写し・反証の段（v2）。
- **門で止める**: 問いの起票（PreToolUse の台帳の guard）= touches 無し・束の要約値が無いか古い・id が解けない・1 段の近傍に処分の無い節点 → deny（欠けた id と次の 1 手を名指す）。床（tz check）= accepted の ADR の basis が 1 段の条を処分していない・unruled → fail。文書を変えない裁定は「まだ分からない」を brief に出す（fail にしない）。**偽陽性の切り方**: 門は 1 段だけ・種類は条・規則行・発効 ADR・要件の 4 つ・hub は除く・処分が 30 を超える起点は「まだ分からない」で起点を細かくさせる。撤退条件（P-4）: 20 問で not-relevant が 9 割超／天井の「止める」が処分済みの近傍の外に置かれた。
- **席への注入**（実測: SessionStart 合計 3,783 byte・R-1 の上限 8,000）: グラフの節は **1,000 byte 以下**。SessionStart に `[GRAPH-UNREAD]`（受けの無い裁定・上位 3）・`[GRAPH-OPENQ]`・`[GRAPH-GAP] unruled/dangling/orphan → tz graph gaps`・`[GRAPH-CUT] / [GRAPH-UNMEASURED]`。UserPromptSubmit は裁定が届いた turn だけ `[GRAPH-IMPACT] <裁定 id> touches=<8 つまで> total=n → tz graph impact`（400 byte 以下）。読みに失敗しても注入全体は黙らせず、その種類だけ UNMEASURED（fail-open・scribe2 の recent と同じ極性）。
- **席と持ち主は同じ列を見る**: 同じ問いの関数から 1 つの結果の列 → 席は TSV / JSON・面は同じ JSON から近傍図。描き方の側で絞りも並べ替えもしない（id の集合と順が一致・歯で固定）。id は生のまま短く（`P-12.2`・`FR5`・`<doc>#<id>`・`t3-hub.1`）・面では札と題と種類の 1 語・札ごとに URL `/g/<id>`。辺の名は席には型の名、持ち主には平易な語（写像は 1 つの表）。**題は 36 字に揃える**（folio の索引の定数・§8.2 の 40 字を改める）。

### 11.4 表示面での見せ方
- **図は新しい頁にせず、問いの card と束の中に畳む**。形は 4 つだけ: 近傍図（1 節点の周り 2 段・層の帯 × 距離の列・上り = 根拠側を左・下りを右・最長路で層分け・力学配置は使わない・表示 40 節点まで・1 列 8・超えた分は「他 n 件」）・影響の差分（字下げの木・3 段で切る・段ごとの件数）・文字の鎖（`<ul>` だけ・390 幅の既定・1 節点 8 本まで）・見落としの一覧（孤児・切れた辺・根拠の無い便を表で）。2 部の対応図（設計の行 ⇄ 便）と俯瞰（種類 × 種類の数の行列・絵にしない）は v1.1。
- **見た目の固定**: 節点の形と色は **層の 4 語**（決まり = 条・規範文・規則行 = 四角／設計 = 目的〜出力 = 角丸／判断 = ADR・問い・裁定・方針 = 菱形／作業 = epic・便・memo = 円）。辺は **族の 4 語**（根拠 = basis・relations.*・結び = 細い実線／止める = blocks = 赤い太線／含む = in-article・parent-child = 点線／変える = amends・裁定 → 変えた節点 = 橙の矢印）。型の名は hover と詳細でだけ。未回答の問いは赤い縁取り・閉じた便は灰。凡例は 4 語 × 2。種類 → 層と型 → 族の写像が全部を覆うことを単体の歯で固定（漏れた型は build が落ちる）。
- **裁定の場面との結び**: 問いの card に「材料 ▸ 触れる 7・止めている便 2・切った 0」を 1 行 → 1 click で近傍図が card の中に開く → 同じ card の button で 2 click 目に裁定。束の承認 = 全問が触れる節点の和集合 +「重なり 2」（衝突の兆し）。裁定の直後 = 時系列に「変わった 4」→ 押すと影響の差分（源は裁定 id を持つ台帳と索引の記録だけ）。方針の欄 = 範囲がこの論点なら効く節点を文字の鎖で 1 行。URL は `/g/<id>?view=near|impact|pair&k=2` の 1 族だけ（増殖の防止）。席は CDP でこの URL を開いて持ち主の目を連れて行く。
- **伝わりやすさの物差し（受入条件 5 本）**: (1) 問いの card から触れる節点の一覧まで 1 click・材料を開いてから押すまで 2 click 以内（席が CDP で数える）(2) 凡例は層 4 語と族 4 語・写像の網羅を歯で (3) 題は 36 字・id は副 (4) 390 幅では図が文字の鎖に落ち横 scroll 0（CDP で実測）(5) 表示は 40 節点まで・「切った n」と索引の時刻を必ず出す・読めないときは「測れていない」。
- 計算は wasm の中で 1,000 節点・2,000 辺の幅優先を 1 回（配置は表示する 40 まで・inferred・実測はまだ）。

### 11.5 v1 の範囲（§8.4 を改める）と受入条件
| v1（§8.4 に足す） | 後の版 |
|---|---|
| 統合グラフの導出（節点 17 種・辺 27 型・repo で修飾した key）・`tz graph` の 5 型（around・impact・trace・gaps・bundle）と `--check`（G1〜G10）・材料の束の file・問いの起票の門（処分の宣言・1 段・4 種）・床の unruled・SessionStart の 3 行と裁定の turn の IMPACT 1 行・面 = 近傍図（card に畳む・2 段・40）+ 文字の鎖 + 影響の差分 + 見落としの一覧 + `/g/<id>`・設計ノートの行を節点に・承認欄の型付き 2 欄・題 36 字 | v1.1: changed・散文の参考の列・2 部の対応図・俯瞰の行列・ADR の accepted の床の処分。v2: 撤退条件の結線・天井の整合の観点が束を読む・「仮に変えたら」・時間軸。v3: ADR の反証の束・段数を持ち主が変える |

受入条件（歯・一時の beads と toy の design-intent で回す）: (1) 子 619 の hub を持つ fixture で around 2 段 → hub が展開されず `# cut reason=hub total=619`・snapshot 一致 (2) accepted の ADR の裁定が台帳に無ければ `tz check` が id と次の 1 手を名指して落ち、台帳が読めなければ「まだ分からない」で合格にしない (3) 問いの起票 3 通り: 処分の無い id が 1 つでも在れば deny してその id を列挙・全部処分すれば allow・束の要約値が古ければ deny (4) 未読の裁定 50 件で SessionStart のグラフの節が 1,000 byte 以下・CUT が総数を持つ・台帳が読めない周は UNMEASURED で他の行は出る (5) 同じ問いで席の TSV と面の JSON の id の列が順も含めて一致・題は同じ関数で 36 字 (6) G1〜G10 の fixture（各 1 本の違反）で名指しが出る (7) 近傍図: 種類 → 層・型 → 族の写像が全部を覆う・390 幅で横 scroll 0・「切った n」が常に出る。

### 11.6 決めの列（1 論点 1 問で順に問う・推奨つき）
1. **全体の形**: 統合グラフを「毎回組み直す導出物」として 1 つ持ち、裁定面・席の精査・床の検査の 3 つがそれを共有する形にするか（推奨: する。folio の crate は索引だけ・結合と検査は scribe の core・境界は JSON の型）。
2. **設計ノートの行を索引の節点に足すか**（節点の閉じた一覧が変わる = 判断の記録）（推奨: 足す。契約 bead の pointer の先が節点になり結びが辺として閉じる）。
3. **承認欄と規則行の裁定を「台帳 id」と「裁定 id」の型付き 2 欄に分けるか**（推奨: 分ける。今の括弧書きは機械が辺にできず G3 を測れない。ADR と規則の schema を改める）。
4. **裁定の記帳先を tsuzuri の台帳に一本化するか**（推奨: 以後は t3・過去の s2 は外部の台帳として読むだけ）。
5. **見落としの門を処分の宣言で fail-closed にするか**（推奨: する。問いの起票と ADR の accepted の 2 か所・1 段・4 種・hub は除く）。
6. **近傍の既定**（推奨: 束 2 段・門 1 段・親子は子 → 親だけ・次数 30 で畳む・fixture で決める）。
7. **近傍図を v1 に入れるか**（推奨: 入れる。card に畳む 2 段・40 まで）／辺の型を表示で 4 族に畳むか（推奨: 畳む）／俯瞰は数の行列か（推奨: 行列）。
8. **題の切り幅を 36 字に揃えるか**（推奨: 揃える）。
9. §8.5 の 2〜6（承認の event を server が書く・席を起こす差し込み・束の逐語・束は設計 doc・承認欄の逐語）は据え置き。

## 12. 問い（1 問・2 問目以降は §11.6 の順）
統合グラフを「毎回組み直す導出物」として 1 つ持ち、裁定面・席の精査・床の検査の 3 つがそれを共有する形（§11.2〜11.5）を v1 の土台にしてよいか。提示面は tailnet の site の同じ頁の §7 に足した。

## 14. 追加の要件 — pipeline の表示を beads と design-intent に連動させる（2026-09-25T04:15Z）
持ち主の裁定（user 2026-09-25T04:14Z・逐語は t3-hub.1）: 「beads design-intentの表示に加えてそれと連動する形でpipelineの表示もできる必要がある」。§8.5-6（承認欄の逐語は今のまま）は「推奨で良い」で仮に積んだ = 決めの列は全部積んだ。

### 14.1 前提の実測（verified・scribe2 の器）
- 便の段（pipeline.md §4）: Queued → Blocked（承認待ち）→ Spawned（runner が worktree で書く）→ Questioned（runner の質問）→ Implemented → Gated（verdict PASS / FAIL / INCONCLUSIVE）→ Landed｜Stopped｜Failed。段の正本は器の event log（`RunStage stage=… detail=…`）。
- event の種類（fleet/events.jsonl と pipe/<run>/events.jsonl・scribe2 の実数）: AllowanceMeasured 19,590・RunStage 2,285・RunDone 742・AllowanceUnmeasured 660・RunCreated 639・DispatchMark 537・SeatStopped 408・SeatSpawned 395・RunCost 323・RunStopped 91・QuestionRaised 56・SeatRegistered 34・GroupMovePending 27・QuestionAnswered 8・GroupPressureNotified 8。
- 走行（run）の id は `<bead id>-<UTC の秒>`（例 s2-07l.132-20260920T063806Z）。走行ごとの置き場 `<state dir>/pipe/<run>/` に contract.toml（契約の写し）・vessel.toml・lens.toml・verdict.json（gate）・review.json（審査）・repo（worktree）。1 つの契約 bead に走行が複数付く（やり直し・.531 は 5 回）。
- 便と設計の結び: 契約 bead の acceptance の pointer 行 `design = <doc>#<row>`（契約表の行）。走行 → 契約 bead は run id の前半。走行 → 口座は SeatSpawned / RunCost の account。走行 → 問い（runner の質問）は QuestionRaised（about・逐語）。

### 14.2 模型への足し方（§11.2 の「器の event は節点の属性で辺にしない」を改める）
- **走行（run）を節点の種類に足す**（節点 19 種目）。理由: 持ち主が見たい「この便は何回目の走行で、どこで止まったか」「この走行はどの設計の行と条に根拠づくか」「この質問はどの走行から出たか」は、走行が節点でないと辿れない（属性では 1 便 1 値しか持てず、やり直しの履歴が消える）。
- **辺（結びの 3 型を足す・計 30 型）**: `run_of`（走行 → 契約 bead・正本 = run id の前半 = 器の event RunCreated）・`raised`（走行 → 問い・正本 = QuestionRaised と問いの bead の metadata `source`）・`ran_by`（走行 → 口座・正本 = SeatSpawned の account。口座は節点にせず走行の属性の札で足りる = 却下案「口座を節点に」は増殖）。
- **走行の属性**（辺にしない）: 段（最新の RunStage）・段の理由（detail）・gate の verdict・審査の結果・費用（RunCost）・開始と終了の時刻（RunCreated / RunDone / RunStopped）・worktree の有無（retired）。正本は器の event log（読むだけ・台帳へ写さない = C15）。
- **入力の追加**: `<state dir>/fleet/events.jsonl` と `pipe/<run>/events.jsonl`・verdict.json・review.json。統合グラフの導出は state dir を宣言（host の面）から解く。読めなければ走行の種類だけ「まだ分からない」（他の種類は組む・fail-open の極性は §11.3 の注入と同じ）。
- **不変条件を 2 本足す（G11・G12）**: G11 走行は run_of をちょうど 1 つ持ち、先の bead が在る（宙に浮いた走行 0）／G12 Questioned の走行は raised の先の問いが在り、QuestionAnswered が在れば問いは closed（問いの状態と走行の段が食い違わない）。

### 14.3 眺め（§8.2 の「今の板」を pipeline の板に育てる）
- **pipeline の板**（home の「今の板」= 段ごとの列: 待ち〔Queued / Blocked〕・動いている〔Spawned / Implemented / Gated〕・止まっている〔Questioned / Failed / Stopped〕・着地〔Landed・今日〕）。各札 = 契約 bead の題 36 字 + 走行の回数 + 段の理由の 1 行（止まっているは必須）+ 口座の札 + 経過時間。Q2「なぜ止まっているか」と Q9「器は黙っていないか」（最後の event の時刻）に直接効く。
- **走行の時間軸**（契約 bead の頁）: 走行ごとに 1 行、段の遷移を横に並べる（Queued → Spawned → Gated(FAIL) → Failed / 2 回目 → Landed）。verdict と review の要点を札で。Q7「難航」に効く。
- **連動（同じ id で 3 つの面が光る）**: 板の札を押す → その契約 bead の近傍図（設計の行・条・問い）が card に開く → 設計の行を押すと同じ行を指す他の便が光る → 問いを押すと裁定面の card へ。逆に裁定面の問いの card の「止めている便 2」は板の札を指す。全部 `/g/<id>` の URL（走行は `/g/<run id>`）。
- **層の割り当て**: 走行は「作業」の層（円・小さめ）。辺の族: run_of と ran_by は「含む」（点線）・raised は「根拠」。段の色は板の 4 語（待ち・動いている・止まっている・着地）と同じ 4 色で、図でも同じ。
- **スマホ幅**: 板は列を縦に積む（空の列は畳む）・時間軸は段の語の鎖に落とす。横 scroll 0。

### 14.4 受入条件（足す 3 本）
(1) 5 回やり直した契約 bead の fixture で、走行の節点が 5 つ・run_of が 5 本・時間軸に 5 行・板には最新の段だけが 1 札で出る (2) Questioned の走行の fixture で、問いの bead が無ければ G12 が名指し、在れば板の札から問いの card へ 1 click で着く (3) state dir が読めない fixture で、走行の種類だけ「まだ分からない」と出て、設計と台帳の節点は組める。

### 14.5 問い（1 問）
pipeline の表示を「走行を節点の種類に足し（辺 3 型・不変条件 2 本）、home の今の板を段ごとの pipeline の板に育て、契約 bead の頁に走行の時間軸を置き、板・近傍図・裁定面が同じ id で連動する」形（§14.2〜14.4）にしてよいか。§11.2 の「器の event は節点の属性で辺にしない」は「走行だけ節点・他の event は走行の属性」に改める。推奨: この形。答えの後に mock（pipeline の板を含む）へ。

## 16. mock v1 への持ち主の評価と、v2 の設計指針（2026-09-25T07:58Z・逐語は t3-hub.1）
持ち主の本裁定は「このままでは不可」。評価は (0) 日本語入力が不可能（bug）(1) 文字が多すぎて処理しきれない = 形と構造で直感的に分からせ、説明は hover / tooltip に・初心者と経験者の mode 切替 (2) この project でしか通じない語が多い（板・逐語・束）= 一般の語に言い換え、専門語は丁寧に説明 (3) 構造化が甘い（「問 3」の下に「問」・番号の始まりが 3）(4) 節点は click しないと分からない = hover で最低限の題を (5) 節点の頁には短い概要が要り、folio と同じくエンジニア向けと非エンジニア向けを並べる (6) 単一 project の表示か（scribe2 の混ぜ方は中途半端）・project ごとに orchestrator が GUI を持つか、口座の群で複数 project を 1 面に出すか (7) 席の状態が「考え中」のままだったが実際は usage limit で長く止まっていた = pipeline と席の稼働状況はリアルタイムに更新し、文字でなく記号と UI で表す。

### 16.1 bug の原因と直し（verified・2026-09-25T07:57Z）
- 席が ssh で起こした Chrome の環境に IME の変数が 0 本（GTK_IM_MODULE / QT_IM_MODULE / XMODIFIERS）。ThinkPad の sway の session は fcitx5 が動き `QT_IM_MODULE=fcitx XMODIFIERS=@im=fcitx LANG=ja_JP.UTF-8` を持つが、ADR-5 の実証の起動行は display の 4 変数しか渡していなかった。
- 直し: 起動行に `LANG GTK_IM_MODULE QT_IM_MODULE XMODIFIERS` を足し、Chrome に `--enable-wayland-ime --wayland-text-input-version=3` を付ける。**もう 1 つの原因（verified）**: 同じ profile の Chrome が既に動いていると、新しい `google-chrome --app=…` は URL を既存の process に渡して終わり、窓は古い環境の process が開く（env と flag が効かない）。→ 席が起こす Chrome は **専用の profile（`--user-data-dir=~/.local/state/tsuzuri/chrome-app`）** で起こす（持ち主の普段の Chrome と混ざらない・CDP の port も別 = 9224）。mock の窓はこの形で起こし直した（Chrome の子 process の環境に IME の変数 3 本が載っていることを実測・親 process の /proc の environ は Chrome が書き換えるので見えない）・持ち主の打鍵で確認待ち。**ADR-5 の端末の一覧（器の宣言の [[device]] 行）に「画面の環境」だけでなく「入力の環境（IME の変数と flag）」と「専用の profile の置き場」を持たせる**（ADR-7 で ADR-5 に枝を足す）。

### 16.2 v2 の設計指針（本裁定の前に mock v2 を作る）
- **形で語る**: 説明の文は既定で出さない。使い方と専門語は hover / tooltip / 「?」の印で出す。初心者 mode（tooltip と手引きが出る）と経験者 mode（出ない）を切り替える（切替は URL と localStorage・既定は初心者）。文字の量の物差し: home の最初の画面の本文は 300 字以下（数字と題を除く）。
- **語の言い換え（この面の語彙表を 1 つ持ち、面の全部の見出しがそれを引く）**: 板 → 「pipeline ダッシュボード」／逐語 → 「あなたの言葉（原文のまま）」／束 → 「まとめて承認」／問い → 「質問」／裁定 → 「あなたの決定」／方針 → 「全体への指示」／席 → 「AI（orchestrator）」／便 → 「作業（run）」／走行 → 「実行」／近傍図 → 「つながり」／見落とし → 「抜けの検査」。内部の id と型の名は経験者 mode の tooltip でだけ出す。語彙表は design-intent/vocabulary.yaml の「面の語」の群として正本に置く（散文にしない）。
- **構造の型を固定**: 一覧の項目は「番号 + 題」か「印 + 題」のどちらか 1 種に統一・番号は 1 から・入れ子は 2 段まで・同じ種類の物は同じ形。
- **hover の情報**: 全部の節点と札に hover card（題 36 字・種類・状態・1 行の要約・id は小さく）。図の節点は hover で題を出し、click で頁へ。
- **節点の頁の概要**: 上に「非エンジニア向けの 1〜2 文」と「エンジニア向けの 1〜2 文」を並べる（folio の plain と同じ形）。出所: design-intent の節点は plain 欄と本文の先頭・bead は title と description の先頭（逐語は含めない）・走行は段と理由。要約は機械が切り出し、無ければ「要約なし」と出す（黙らない）。
- **単一 project が既定**: GUI は project（器の anchor）ごとに 1 つ、orchestrator の席が管理する（推奨）。理由: 台帳・design-intent・state dir が project 単位で、権能と口座も anchor 単位。**群の面**（口座の群 Tier1 の anchor 全部の「待ち・動いている・止まっている」の数だけを 1 面に並べ、押すと各 project の面へ）は後の版で足す = 複数 project を 1 面に混ぜない。scribe2 の data は mock の都合で混ぜただけで、v2 では「外部の台帳」として bead の頁の参照にだけ出す。
- **稼働状況はリアルタイム・記号で**: 席の状態は器の seat の state（busy / idle）だけでなく、**口座の残量の event（AllowanceMeasured・GroupMovePending）と tick の生存**を合わせて 5 値にする: 動いている（緑の点・脈）／待っている（灰の点）／限度で止まっている（橙の砂時計 + 再開の見込み時刻）／応答なし（赤の点・最後の event からの経過）／測れていない（点線の丸）。pipeline の作業も同じ 5 値の印。更新は SSE で 5 秒以内・「最終更新」の代わりに印の脈で鮮度を示し、止まったら灰に落ちる。usage limit で止まった事実は「AI（orchestrator）: 限度で停止・02:5x から・再開 15:59Z 見込み」の 1 行で出す。
- **受入条件（v2 に足す）**: (1) home の最初の画面の本文 300 字以下 (2) 全節点に hover card（欠け 0） (3) 面の見出しの語は語彙表の語だけ（歯で照合） (4) 一覧の項目の形が 1 種・番号は 1 から (5) 席の状態が usage limit の event を反映する fixture（AllowanceMeasured used_pct=100 → 限度で止まっている）(6) IME の env と flag を持つ起動行で日本語が入る（持ち主の実機で確認・記録に残す）。

### 16.3 問い
持ち主の評価は全部取り込む（裁定は要らない）。次の 1 問は mock v2 を見てから。

## 18. mock v2 への持ち主の評価と、mock v3 の指針（2026-09-25T14:29Z・逐語は t3-hub.1）
- **日本語入力は直った（verified・持ち主の実機）**: 専用 profile + IME の env + `--enable-wayland-ime` の起動行で入る。ADR-5 の端末の行に「入力の環境」と「専用 profile」を足す（ADR-7 で）。
- **概要はデータとして書く（今後の目標）**: mock の要約が薄いのは mock だから。本番は節点ごとの概要（非エンジニア向け / エンジニア向け）を design-intent（plain 欄）と台帳（bead の description の先頭の要約の欄）に**データとして**持ち、面は写すだけ。要約の無い節点は床が数える（「要約なし」を 0 にするのが目標）。
- **語は「英語を使うべき所は英語」**: pipeline の段（Queued / Running / Blocked / Landed / Failed …）・issue の状態（open / closed / in_progress）・run / ADR / epic などの技術語は英語のまま出し、「?」で注釈。言い換えるのは「逐語」のような一般に使わない日本語だけ（逐語 → あなたの言葉（原文のまま）・裁定 → あなたの決定・束 → まとめて承認・板 → dashboard）。語彙表を「英語のまま + 注釈」「言い換え」の 2 欄に直す。
- **地図は番号と複数の圧縮面**: 条・決まりの文・規則・ADR は id（番号）を必ず出す（順が分かる）。全体を見通す圧縮面は 1 つでは情報を落とすだけなので、**切り替えられる面を複数**持つ: (a) 圧縮（今の 4 帯・id と題）(b) 一覧（id + 題 + 概要 1 行・種類で絞る・並べ替え）(c) グラフ（節点と辺の図・層で配置・hub は畳む・hover で題）(d) 表（種類 × 種類の数の行列）。切替は 1 つの tab 列・URL に残す。
- **口座の面（account board）は器の側の別の面**: 口座の残量（各 window の used_pct・resets_at）と口座ごとの usage の負荷（どの project のどの席と作業が今どの口座を使っているか）を可視化する面は、複数 project × 複数口座の管理画面なので **project の dashboard とは別の面**として器（scribe の core）が管理する。関係: account board（上）→ 各 project の dashboard（下・詳細）。project の dashboard は各 project の orchestrator がデータのやり取りと browser の制御を行い、account board は器が管理する。**mock で示すこと**: 口座 × window の残量の表（used_pct の帯・reset までの時間）・口座の群（Tier1 / Tier2）と anchor（project）の対応・いま各口座を使っている席と作業（project 名・段・経過）・口座の移動の履歴（GroupMovePending）・限度で止まっている席の印・**project の dashboard への飛び方**（同じ窓で開くか、新しい窓〔端末の Chrome の app mode の別窓〕で開くか、の 2 案を mock で並べて示す。席の推奨: account board は器の窓 1 つ・project の dashboard は project ごとの窓 = ADR-5 の「席が窓を起こす」と一致・窓の一覧は account board が持つ）。
- **mock v3 の範囲**: (1) project dashboard（mock2）の直し = 語彙の 2 欄化・地図の番号と 4 面の切替・概要の欄の見本を数本だけ実文で（憲法の条の plain 欄・ADR の plain 欄は実データ）(2) account board（新・mock3/account/）= 実データ（scribe2 の state dir の host.toml の口座と群と anchor・fleet の AllowanceMeasured / AllowanceUnmeasured / GroupMovePending / SeatRegistered・各 project の seat の state）(3) 2 つの面の関係の図と、飛び方の 2 案。
- 受入条件（足す）: 地図の 4 面それぞれで id が全節点に出る・切替が URL に残る・account board の残量は event の実データから組み、読めない口座は「測れていない」・account board から各 project の dashboard へ 1 click・限度で止まった席が account board と project dashboard の両方で同じ記号。
- **層の抽象名は捨て、帯は出所の正本の名にする（持ち主 user 2026-09-25T14:55Z・逐語は t3-hub.1）**: 「決まり / 設計 / 判断 / 作業」の 4 層は、持ち主には「決まり = constitution の条項か・条と何が違うか・判断 = ADR か・設計 = SRS か」と疑問が尽きず、何と何がつながっているかを逆に分かりにくくした。§11.4 の「層で畳む」（§11.6-7 の 4 族と同じ発想）は**却下**し、帯の名を実際の正本の名にする。帯 = 出所の file（7 つ・順は憲法の順位と同じ）: **constitution**（design-intent/constitution.yaml・条 A-n と条の文 A-n.m）→ **rules**（design-intent/rules.yaml・rule D-n / R-n）→ **ADR**（design-intent/adr/ADR-n.yaml）→ **SRS**（design-intent/srs.yaml・requirement FR-n・tsuzuri は今 0 本）→ **design docs**（docs/design/*.md の id 付きの行）→ **beads**（.beads の台帳・epic / task / memo / question〔label=問い〕と、その notes に書かれた あなたの決定〔裁定〕・方針）→ **pipeline**（器の event の run）。節点の種類の面での名も抽象名を捨てる: 判断の記録 → ADR・規則行 → rule・規範文 → 条の文・契約 → task・走行 → run・設計ノートの行 → design doc の行・問い → question。形は帯ごとに固定しなくてよい（帯の名が横に常に見えるので形で覚えさせない）。色は帯ごとに 1 色。**辺の凡例も 4 族の抽象名（根拠 / 止める / 含む / 変える）を捨て、その眺めに実際に出ている辺の型の名（basis・blocks・parent-child・in-article・amends・ruled_by・design・answers・touches・run_of …）を線の見本つきで並べる**（族は線の style を決めるだけで名は出さない）。hover の card には「出所の file と行」を必ず出す。この直しは mock v3 の範囲に入れ、§11.6-7 の仮の裁定（4 族に畳む）は本裁定で改める（ADR-7 に書く）。
- **形と囲みの規則（持ち主の問い 2026-09-25T15:1xZ 台・逐語は t3-hub.1 に無し = 問いのみ）**: mock v2 の頭の記号は「層ごとの形」（四角 / 角丸 / 菱形 / 丸）で、question が bead なのに ADR と同じ菱形になっていた。つながり図の囲みの角の丸さ（rx 16 / 8 / 4）は頭の記号をなぞっただけの二重表現で意味は無かった（verified・ui.js）。v3 では **頭の記号は 2 種**（四角 = file に書かれた行 = constitution / rules / ADR / SRS / design docs・丸 = 台帳と器にある動くもの = beads / pipeline の run）・**囲みは 1 種**（rx 4）・意味を持つのは「記号の形・色 = 帯・縁 = 状態（open の question は赤・起点は太い・closed は灰の文字）」の 3 つだけで、凡例の先頭にこの 3 行を置く。
- **mock v3 完成（2026-09-25T15:35Z・受入 7 本合格・14 画面 × 2 幅 × 2 mode = 56 回・page error 0・横 scroll 0・逐語の混入 0・report = scratchpad/mock3-report.md）**: URL = http://100.127.217.108:8101/mock3/index.html（地図は map.html?view=compact|list|graph|table）・account board = mock3/account/index.html（`?at=<ISO>` で時点・「限度の時刻へ」）・関係 = mock3/account/relation.html。agent の判断（持ち主の確認待ち）: (1) 応答なしは「busy のまま 15 分途絶」に絞った（器の tick が疎で idle の席が全部赤になったため・器が定期の tick を記す規則があれば元に戻せる = 器の要件候補）(2) seven_day_model の限度は model が合う席だけ止め、run は model が event に無いので判定しない（= 器の要件候補: run の model を event に）(3) 12 h 以上動きの無い席は表から外し数だけ (4) design docs の帯は docs/design/ が無いので「測れていない」(5) グラフの面は hub と次数の大きい順に 40 を残す (6) v2 から持ち越しの CSS の bug（.st-run::after の脈の輪が文字の札にも当たり横 scroll が周期的に出る）を直した (7) 飛び方は案 2（新しい窓・名前つき target）を primary に。
- **持ち主の評価 1（2026-09-26 朝・逐語は t3-hub.1・23:48Z の note）**: 見た目はかなり改善。直し = (a) pipeline dashboard の Questioned / Failed / Stopped の列などで文字がはみ出す (b) account board の「口座の群」は古い仕組み。正しくは **一つの口座を共有する project の群**（群が占有する口座はその時点で 1 つ・群同士で重ならない・群の今の口座では新規の pipeline を起こせない）なので「口座の数」の表示は誤り。今占有している口座 1 つを出す (c) 「誰が」の擬人化は禁止。使い手は orchestrator と pipeline の 2 つなので **session** と呼ぶ (d) session の表の既定 = project で group 化 → role（orchestrator 上・pipeline 下）→ session 名 → 口座 → 段 → 経過 → 稼働の記録。稼働の記録は色と線の意味を「?」に注釈。sort profile を切替可能に。次に project board の評価が来る。
- **器の群の仕様（scribe2 の account-lifecycle.md §17 / §19 / §20 / §23 / §27 / §28 を読んだ・verified 2026-09-26）**: 宣言は host.toml の `[[account-group]]`（name / anchors / accounts の候補順）。今の口座の解決は 1 関数 `current_of` = 記録 > 種。記録は `~/.local/state/scribe2-host/groups/<群>.account`（account / ts / reason / previous・履歴は history/）。今は Tier1 = black1（17:07Z）・Tier2 = black4（02:19Z）。種 = 宣言順で前の群の種でない最初の候補（§28）。群の置き場の席は必ず群の今の口座で起きる（§20 形 3）。新規の便は各群の今の口座だけを候補から外す（§23）。稼働中の便は移動を妨げない（§27）。閾値は rules 行 3 本（fleet.group_pressure_5h/7d/model_pct）。event は GroupPressureNotified / GroupMovePending / GroupMoved / GroupMoveRefused。scribe2 の席（scribe2-d8）へ確認を送付済み（返事待ち）。直しの agent mock-v3-fix1（opus）を 2026-09-26T00:0xZ に起動（report = scratchpad/mock3-fix1-report.md）。
- **scribe2 の席の確認（scribe2-d8・main 3cbe744・2026-09-26T00:1xZ）**: 6 点とも席の読みどおり。補正 = (1) 種は Tier1 = black6・Tier2 = black1（宣言順で前の群の種を避ける `seed_groups`）で、Tier1 = black1・Tier2 = black4 は記録の値。(2) GroupMovePending = 退避が settle の窓の内に shell へ戻らなかった席（account = 移り先）。groups/ の <群>.request / .judged / lock は file で event ではない。(3) §27（live 便は移動を妨げない）は設計は着地・code は未着地（契約 s2-07l.640 走行中）→ mock は着地後の姿で作り、今日の binary の挙動を注記。(4) 群の門は群の anchor の席の全部に効き、tick が登録 row ≠ 記録の席を移動の周と判じる。閾値 = 5h 85 / 7d 95 / model 95。event は 4 種のみ。→ agent mock-v3-fix1 へ転送済み。
- **直し 1 完了（agent mock-v3-fix1・2026-09-26T00:10Z・report = scratchpad/mock3-fix1-report.md）**: はみ出し 926 件 → 0（17 画面 × 2 幅 × 2 mode = 68 回・列の見出しと札を折り返し・地図の表は 390 幅で 1 列の一覧・脈の輪を box-shadow に）。account board = project の群の枠（今の口座 1 つ・いつから・前の口座・逼迫の閾値に対する 3 窓・群の project の一致の印・候補の順）・口座の表に「占有」の列（Tier1 / Tier2 / pipeline が使える）・session の表（project → role → session → 口座 → 段 → 経過 → 稼働の記録・sort 4 つ・`?sort=`）・稼働の記録の「?」に 11 行の注釈・擬人化の語 0（grep）・移動の履歴は記録 1 件 = 1 行で GroupMoved / GroupMovePending と突き合わせ（Tier1 の 00:39Z は改名の記録で GroupMoved なし）。agent の判断: pipeline の行は未終了の run のうち worker が起きているか 60 分以内のもの・応答なしは busy 15 分 / worker 無しは 30 分。scribe2 の補正 4 点（種・GroupMovePending の意味・§27 未着地の注記・移動待ちの印）は agent へ送付済みで反映待ち。
- **持ち主の評価 2（2026-09-26 朝・project board・逐語は t3-hub.1 の 00:12Z の note）**: (a) はみ出しと重なりを直す (b) home に「この project が今使っている口座」の枠（usage limit・負荷・器が移動を指示した状態・口座の移動の履歴。account board は複数 project × 複数口座を並列に・project board は特定 project の口座を掘る）(c) つながりの辺が重なる → hover した節点を中心に、つながる辺と節点を両方向（根拠の側とその先・影響の側とその先）に強調、全部は光らせない算法 (d) 質問の card は 題 → 概要（非エンジニア / エンジニア）→ **理由（常に表示）** → 推奨 → つながり（畳んでよい）(e) 問い: question に答えると rules / ADR / SRS / task の bead が生成されるのか・答える行為のグラフの中の意味 (f) 決定済みの question を押しても頁に飛ばないのは mock ゆえか (g) 近傍図は根拠の側だけでなく末端の側（語の提案が要る）も 2 段・畳む / 開く / 段数の調整・epic を押したら子が出る (h) 地図のグラフは wheel で拡大・drag で移動・帯の名は左端に固定。→ agent mock-v3-fix1 へ直し 2 として送付（report = scratchpad/mock3-fix2-report.md）。
- **席の答え（e・f・g の語）**: (e) 答える = 裁定の節点（notes の定型行・`answers` 辺で問いへ）を作る行為。文書や task は自動生成しない。代わりに **処分の宣言**（§11.3・§11.6-5）で席が裁定の処分を宣言する: 改める ADR / rules 行 / SRS 行 / design doc の行（席が design-intent を編集し承認欄の型付き 2 欄で `ruled_by` → 裁定）・起こす task（epic の下の契約 bead・`design` 辺で design doc の行へ・pipeline の run になる）・変えない（理由 1 句）。裁定に処分が無ければ G7（宙に浮いた裁定）で床が落とし、発効した ADR / rules 行に `ruled_by` が無ければ G3 で落ちる。問いで止まっていた run（`blocks`・Questioned）は裁定で解けて Spawned に戻る（§14）。= 答えは「根拠の節点」であり、その後の文書と task はすべてその裁定を根拠に指す。(f) 本番は全節点が `/g/<id>` の頁を持つ（§11.4）。mock は頁を 3 つしか作っていなかった → 直し 2 で全節点を bead.html が id で描く。(g) 語の提案: **根拠（この節点が拠る先）← 節点 → 影響（この節点に拠るもの）**。影響 = parent-child の子・design で指される task・blocks で止まる run・ruled_by と answers の逆向き。
- **持ち主の問題点 3（2026-09-26T00:20Z・逐語は t3-hub.1）**: 候補の順は要るのか。要望は「Fable の枠と 7d の残量と reset までの時間を考慮して群ごとに最適な口座を動的に選ぶ」だった → そうなっていなければ scribe2 に報告して直させ、合意した形で mock を作る。加えて session の稼働の記録に 24h / 6h / 3h の切替 button。
- **席の実測（scribe2 main 3cbe744・verified）**: 群の移り先 `target_of`（hook/group.rs）は宣言順に走査して門（今の口座 / 他群の今の口座 / 退役 / live 便 / 逼迫）を通る**最初の**候補を返す = 残量と reset で順位を付けていない。器には動的な選定が既に在る（fleet/select.rs `pick`: session 用 = 逼迫度が最小・便用 = 7 日窓の reset が早い順・ADR-0042）ので、群の移り先だけが宣言順。→ scribe2（scribe2-d8）へ持ち主の逐語と席の提案を送付: accounts は候補の集合（順は tie-break）・鍵 = 7d とモデル別 7d の残量の小さい方が大きい順 → 7d の reset が早い順 → 宣言順・5h は門だけ・doctor に `next=`。合意の返事待ち。mock の「候補の順」の直しは保留（agent へ伝達済み・24h / 6h / 3h の切替は直し 2 に追加）。
- **scribe2 の合意（scribe2-d8・memo s2-07l.645・2026-09-26T00:4xZ）**: 群の移り先を残量の鍵で選ぶ形に合意。修正 3 点 = (A) 鍵 (a) のモデル別 7 日窓は最新の実測が持つ model の行の全部（Fable を名指さない）(B) 鍵は用途別の関数に閉じる（fleet/select.rs の pick の隣に群用の鍵・target_of と doctor の next= が同じ 1 本）(C) 5 時間窓は門だけ・鮮度の外の候補は 1 回測ってから並べ、測れない口座は候補から落とす。順 = 持ち主の裁定 → SRS FR38 / AC41 を「鍵で並べた先頭」に改める（/folio-architect は持ち主の手）+ ADR-0069（ADR-0049 の移り先を部分 supersede）→ account-lifecycle.md の § と行 → 契約 bead（id が出たら送られる）。**裁定は scribe2 の席で取る（tsuzuri は同じ文を見せるだけ・二重に取らない）**。mock は next= と同じ鍵で「次の移り先」を出す前提で作ってよい。
- **直し 2 完了（agent mock-v3-fix1・2026-09-26T00:31Z・report = scratchpad/mock3-fix2-report.md・受入 8 本合格・20 画面 × 2 幅 × 2 mode = 80 回・はみ出し 0・重なり 0〔文字の矩形 22,045〕・page error 0）**: home に「この project の口座」の枠（登録の口座・群と今の口座の一致 / 移動待ち・3 窓の meter に閾値の線・sparkline・負荷 = この project の orchestrator と他 project の session・移動の指示の 3 段〔承認 → 退避 / exit 待ち → 起こし直し〕・口座の履歴）。判定の関数 30 本を acct.js の `TZ.AcctModel` に集め account board と home が共有。hover 強調（両方向 各 2 段・hub は広げない・20 超で段 1・click で固定・Esc 解除・光る節点は最大 20）を近傍図と地図 (c) で同じ関数。質問の card = 題 → 概要（非エンジニア / エンジニア）→ 理由（常時）→ 推奨 → 答え → つながり（畳）。近傍図は根拠 ← 節点 → 影響の両側・段数 1 / 2 / 3・畳み（`?k=`・`?fold=`）・epic の頁で子 10 + 決定 10。地図 (c) は wheel / drag / pinch / 元に戻す・帯の名は左端に固定。全 199 節点が bead.html?id= で描ける。agent の判断: 図の節点の click は「固定」に替え頁へは 2 回押す / Enter / 一覧の link（1 click で頁へ移すなら固定を Shift+click に）・home の口座の枠は最初の画面の 300 字を保つため左列の下・退避の合図は event に無いので GroupMovePending と登録 row の変化から 3 段を導く。残り = 直し 3（稼働の記録の 24h / 6h / 3h・候補と次の移り先の鍵）。
- **持ち主の問題点 4（2026-09-26T00:34Z・逐語は t3-hub.1）= home の block の整理**: run の数の block と あなたの決定待ちの block は不要（数は tab の badge と次の一手で分かる・run と質問の数は account board の各 project の札へ）。次の一手は質問以外も来るようにしてもっと便利に。「この project の口座」と「orchestrator」の重複を整理。「移動の指示」は履歴でなく今の状態を compact に（平時は出さない・逼迫 / 移動中のときだけ）。口座の履歴は畳む。「この口座の負荷」は不要（群の orchestrator しか居ないのは自明）→ 群名 Tier1 を出し hover で所属 project。
- **席の答え（次の一手に来るもの）**: 閉じた一覧 7 種を優先順に = (a) 限度 / 移動（移り先なし → 口座を足すか待つ・席が戻らない → 窓を見る）(b) 応答なし（窓を見る / 起こし直す）(c) 止まっている run（Failed / FAIL / Stopped → 再走か止める）(d) 束の承認（2 問以上）(e) 質問（最古の open）(f) 発効待ち（裁定に処分の宣言が無い = G7・見るだけ）(g) なし。1 つを大きく + 残りを小さく列挙。→ agent へ直し 4 として送付（report = scratchpad/mock3-fix4-report.md）。
- **持ち主の裁定（scribe2 の席で・2026-09-26T00:43Z・逐語は scribe2 の memo s2-07l.645 の notes）= 群の移り先の形**: 門は不変（今の口座でない ∧ 他群の今の口座でない ∧ 退役でない ∧ 鮮度の内側で 3 窓とも閾値未満）。鍵 = (a) 7 日窓と「群の席の役割の model」（seat.model.orchestrator = fable）の 7 日窓の残量の小さい方が大きい順（役割の model の行が無い口座は最後）→ (b) 7 日窓の reset が早い順 → (c) 宣言順。**予約は記録せず周ごとに導出**: lock の内側で候補を鍵で並べ、群を宣言順に見て「どの群の今の口座でもなく先の群の予約でもない最上位」をその群の next に（群同士の移動と予約は排他・便の起動は排他しない）。doctor に next=<label|none>・board は同じ導出の値。n 群でも同じ。開いている 1 点 = 群の名を Tier<数字> に固定するか（推奨 = 固定・宣言順が昇順でなければ host.toml の読みで断る）。これから SRS FR38 / AC41 → ADR-0069 → account-lifecycle.md → 契約 bead。→ agent へ差分 3 点（役割の model・予約・表示）を送付。

## 19. 経緯
- 2026-09-25T01:31Z: 論点 B を開いた（1 問目 = v1 の範囲・推奨 (a) 裁定の輪だけ）。台帳の根の epic は scribe2 席の (a) の合図待ち
- 2026-09-25T01:4xZ: 根の epic は持ち主が素の terminal で置いた（t3-hub）。子の memo「裁定の控え」は席が canonical の bdw 経由で置いた（t3-hub.1・以後の裁定の逐語はここの notes）。席の対象は t3:orchestrator に確定。.vessel.toml に path の種別の宣言を足した（2481b39）。1 問目は答え待ち
- 2026-09-25T01:50Z: 検討 agent 3 つの出力を §8 に統合し、決めの列（§8.5）と 1 問（§9）を提示。HTML の提示面を tsuzuri-site/ruling-surface.html に置いた
- 2026-09-25T02:17Z: 持ち主の広げ直し（統合グラフ）を受け、検討 agent 3 つの出力を §11 に統合。rules.yaml の引用符の不具合を直した（aa3964a）。§12 の 1 問を提示
- 2026-09-25T02:28Z: 持ち主の裁定「それでよい。先に進んで。」= §12 の形（統合グラフを導出物として 1 つ持ち 3 つが共有）を v1 の土台に。あわせて「グラフの全体像を出すのはどうなった？」→ 次の 1 問 = 全体像の眺め（§11.4 では俯瞰を数の行列で v1.1 に置いていた・畳んだ地図を v1 に繰り上げる案を提示）。器が口座を移動中（黒 6 → 黒 1）のため作業記憶を残して待つ
- 2026-09-25T02:38Z: 持ち主の裁定「とりあえず推奨で進めて」= 層で畳んだ地図を v1 に繰り上げ（仮）。**最終の判断は他の論点も決めた後に mock を作り、それを見て行う** = 論点 B の決めの列（§11.6 の 2〜8・§8.5 の 2〜6）は「仮の裁定」として進め、mock（裁定面 + 統合グラフの地図・tsuzuri の実データで静的 HTML）で持ち主が見てから本裁定 → ADR-7
- 2026-09-25T02:41Z: 仮の裁定「推奨で良い」= §11.6-2 設計ノートの行を索引の節点に足す。次 = §11.6-3（承認欄と規則行の裁定を型付き 2 欄に）
- 2026-09-25T02:41Z: 仮の裁定「推奨で」= §11.6-3 承認欄と規則行の裁定を型付き 2 欄に。次 = §11.6-4（裁定の記帳先を tsuzuri の台帳に一本化）
- 2026-09-25T03:14Z: 仮の裁定「推奨で」= §11.6-4 記帳先を t3 に一本化（過去の s2 は外部の台帳として読むだけ）。席は口座 black1 で再開。次 = §11.6-5（見落としの門を処分の宣言で fail-closed に）
- 2026-09-25T03:19Z: 仮の裁定「推奨で良い」= §11.6-5 処分の宣言の門（問いの起票と ADR の accepted・1 段・4 種・hub 除く）。あわせて持ち主の裁定: bd init が作る余計な file（CLAUDE.md / AGENTS.md）は scribe2 に報告・project の CLAUDE.md は今は不要（pointer も不要・規則は global の CLAUDE.md と orchestrator の役割の指示文・project 固有の知識だけが project CLAUDE.md）・憲法は orchestrator の役割の指示文の時点で機械的に注入（散文の規律にしない = 規律はデータで）。2 file を git rm（可逆・履歴に残る）。次 = §11.6-6（近傍の既定）
- 2026-09-25T03:46Z: 仮の裁定「それでよい」= §11.6-6 近傍の既定。次 = §11.6-7（近傍図を v1 に・辺は 4 族・俯瞰は行列）
- 2026-09-25T03:49Z: 仮の裁定「推奨で進めて」= §11.6-7 近傍図 v1・4 族・俯瞰は行列。次 = §11.6-8（題 36 字）
- 2026-09-25T03:56Z: 仮の裁定「推奨で良い」= §11.6-8 題 36 字。統合グラフの決めの列は仮で全部積んだ。次 = §8.5-2（承認の event を裁定面の server が書く）
- 2026-09-25T03:57Z: 仮の裁定「推奨で良い」= §8.5-2 承認の event は裁定面の server が書く。次 = §8.5-3（席を起こす 4 つ目の差し込み）
- 2026-09-25T04:04Z: 仮の裁定「推奨で良い」= §8.5-3 席を起こす差し込みを開ける（1 行の指し示し）。次 = §8.5-4（束の逐語）
- 2026-09-25T04:09Z: 仮の裁定「それでよい」= §8.5-4 束の逐語。次 = §8.5-5（束は設計 doc・状態の正本は器の event・home は待ちの帯 + 今の板）
- 2026-09-25T04:12Z: 仮の裁定「それで良い」= §8.5-5 束は設計 doc・正本は event・home。次 = §8.5-6（承認欄の逐語は今のまま）
- 2026-09-25T04:15Z: 仮の裁定「推奨で良い」= §8.5-6 承認欄の逐語は今のまま（決めの列は全部積んだ）。追加の要件 = pipeline の表示の連動 → §14 を書き 1 問を提示
- 2026-09-25T04:19Z: 仮の裁定「よい」= §14 pipeline の連動。**決めの列は全部積んだ → mock の制作へ**（置き場 tsuzuri-site/mock/・実データ = tsuzuri の索引と台帳 + scribe2 の台帳と走行を外部の例として）
- 2026-09-25T04:41Z: mock 完成（tsuzuri-site/mock/・5 頁・graph.json 節点 1,548・辺 2,198・受入条件 7 本合格・report は scratchpad/mock-report.md）。逐語を含む元 data は配信の外へ移した。本裁定の前に見てほしい仮定: 板の 4 列への段の写像（Reviewed の扱い）・方針を近傍の辿りに含めるか（切った 12〜20 の原因）・節点の種類の数（宣言 19 vs §11.2 の 17）・ADR の発効が索引に無い（status の列）
- 2026-09-25T07:58Z: 持ち主の評価（mock v1 は不可・bug と 7 点）→ §16。IME の原因を実測して窓を起こし直した。器が口座を移動中（black1 → black6）のため作業記憶を残して待つ。次 = mock v2
- 2026-09-25T08:24Z: **mock v2 完成**（tsuzuri-site/mock2/・受入 8 本合格・1280 / 390・初心者 / 経験者・home の本文 219 字 / 204 字・語彙表に無い見出し 0・hover card の欠け 0・横 scroll 0・verbatim / notes の混入 0・report は scratchpad/mock2-report.md）。agent の発見: **器の record に「利用枠の限度で断られた」event が無い**（fleet の log に used_pct=100 は無く、state.jsonl は 03:14〜04:20Z に busy / idle の組が 12 回）→ 本番で「限度で止まっている」を正しく出すには器が限度の event を記録する必要（v3 の要件候補・scribe2 へ送付）。mock の 02:52Z used_pct=100 と 07:50Z GroupMovePending は見本の event と注記。ThinkPad は ssh が届かず（timeout）窓は開けていない → URL で見てもらう
- 2026-09-25T14:29Z: 持ち主の評価（mock v2）→ §18。日本語入力は直った（verified）。次 = mock v3（project dashboard の直し + account board の新設 + 2 面の関係）。器が口座を移動中（black1 → black5）
