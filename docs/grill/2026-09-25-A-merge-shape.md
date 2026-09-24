# grill 論点 A — 合流の形と順序（討論の記録・正本は design-intent/ の YAML）

- 出自: docs/handoff/2026-09-24-v3-kickoff.md §2 論点 A・docs/handoff/2026-09-24-v2-leftovers-for-v3.md §2 の 2・3・8
- 状態: 裁定あり（2026-09-24T22:21Z「よい」・ADR-4 発効・記帳先 s2-07l.214）
- 逐語は台帳へ（この file には写さない）

## 1. 確認した事実（verified・2026-09-25）

| 事実 | 出所 |
|---|---|
| v3 の合流先 = 新しい repo（scribe3）。folio2 の持ち主が 2026-09-24 22:38 JST に承認（ADR-21 の問 2） | folio2/design-intent/adr/ADR-21.yaml |
| folio2 ADR-21: M3 の出口の利用者 = v3 の設計の置き場。出口の判定点 ① 置き場で床が合格（違反 0・まだ分からない 0 = 凍結 anchor が要る）② 契約表の導出物が v3 の repo に着地し差分 0 ③ v3 の便を運ぶ器がその導出物を読んで通す | 同 decision (1) |
| ADR-21 の撤退条件 (1): 判定点 ①〜③ の記帳より前に folio の実装が v3 の repo へ移る取り込みが着地したら、持ち主へ問い直す。(2): v3 が「器は契約を導出物の file でなく型で受ける」と決めたら判定点 ②③ を問い直す。(3): 発効から 30 日（2026-10-24）までに判定点 ① の記帳が無ければ問い直す | 同 retreat |
| scribe2 の crate: scribe2（core・約 65k 行）・scribe2-boundary（約 0.5k）・xtask（約 20k）。folio2 の crate: folio（約 28k・依存 clap + yaml-rust2） | 各 Cargo.toml・wc |
| scribe2 の init 1 発の設計 = docs/design/host-init.md + ADR-0063（main 564b4cd・2026-09-24 23:16 JST）。人が打つのは host init（host に 1 回）・init（repo に 1 回）・doctor の 3 つ。行 a〜d を便に起票済み（.612〜.615）。trust の書き手は再裁定待ち | scribe2 台帳 s2-07l.609 notes・host-init.md |
| 今の器（scribe2）は導出物の契約 file（toml）を読み goal 欄も受ける（s2-07l.586 着地） | folio2 ADR-21 context (3) |
| scribe3 の design-intent: 憲法は folio init の雛形（floor-declaration・条 1 本・draft・binding false）。ADR-1〜3・design-note 1 本 | folio check・repo |

## 2. 論点の中身（deduced）

論点 A は 4 つの決めに分かれる。
1. **repo と workspace**: scribe3 に 1 つの cargo workspace。crate は folio / scribe（core）/ scribe-boundary / surface-contract（面の契約の型・serde だけ）/ surface（Leptos の wasm）/ xtask。表示 server は boundary に置く（network と process を持つ側・core は async を持たない C13.3 と整合）。
2. **folio と scribe の境界**: 「設計ノートの契約表 → 導出物の file（toml）→ 器が読む」の既に在る形を採る（決定はしごの「既に在るか」で止まる）。同じ workspace で型を直に渡す形は、folio2 ADR-21 の撤退条件 (2) に当たるので、合流の後の別の判断に回す。
3. **code の持ち込み方**: scribe2 と folio2 の履歴を保って持ち込む（git subtree add か read-tree・元の repo は書き換えない・可逆）。書き直しはしない（裁定 0.1）。
4. **順序**: folio2 ADR-21 の撤退条件 (1) が「folio の実装が移る前に判定点 ①〜③」を求める。判定点 ① には凍結 anchor が要り、凍結には v3 の憲法 v1.0 の承認が要る（論点 E）。よって順序は **E（憲法の最小版）→ 面の実装を scribe3 の最初の便として今の器（scribe2）で運ぶ（契約表 → 導出物 → 器が読む = 判定点 ②③）→ code の持ち込み** になる。

## 3. 候補と評価

| 案 | 中身 | 利点 | 代償 |
|---|---|---|---|
| (a) 憲法 → 面の便 → code の持ち込み | 上の 4 の順 | folio2 の M3 の出口を素直に満たす・scribe2 の init 1 発と器の受付を scribe3 で実地に使う・設計と code が最初から揃う | v3 の repo に code が入るのが後になる（面の実装そのものは scribe2 の器で先に進むので体感の遅れは小さい） |
| (b) code を先に持ち込む | 今すぐ subtree で 2 repo を入れ、設計は追って書く | v3 が「動くもの」として早く見える | folio2 ADR-21 の撤退条件 (1) を踏み、持ち主に問い直しが立つ・設計 doc と code の乖離（scribe2 で起きた形）を最初から抱える |
| (c) 面を別 repo で先に作る | surface だけ別 repo で進め、後で合流 | 憲法を待たない | repo が 1 つ増え、合流が 2 回になる |

## 4. s3 席の推奨

**(a)**。順序は E → 面の便 → code の持ち込み。論点の順は D → **E → A の残り** → C → B → F に組み替える（E は「最小の憲法 v1.0」= scribe2 の C1〜C17 と folio2 の P / N を 1 本にする草案を s3 席が書き、持ち主は条ごとでなく 1 回で承認できる形にする）。

init の口: scribe2 の host-init.md の形（host init / init / doctor）をそのまま v3 の要件に採る。人が打つのは 3 つで、裁定の「init と doctor の 2 つ」より 1 つ多いが、host init は host に 1 回だけで repo ごとには打たない。

## 5. 持ち主への問い（1 問）

合流の順序を「憲法の最小版 → 面の実装を scribe3 の最初の便として今の器で運ぶ → scribe2 と folio2 の code を履歴つきで持ち込む」にしてよいか。前提 = v3 の repo に code が入るのは面の便の後になる（面の実装は待たない）。この前提を受け入れないなら推奨は (b) に変わり、folio2 側へ問い直しを送る。

合流について他に要望（伝えきれていない点）があれば、この答えに添えてほしい。

## 6. 経緯
- 2026-09-25: s3 席が事実を確認し §2〜§5 を持ち主へ提示（答え待ち）
- 2026-09-24T22:22Z: 持ち主「よい」→ ADR-4 を起こす（proposed）。次は論点 E（憲法の最小版 v1.0 の草案）
- 2026-09-25: scribe2 席の助言 = init は宣言が在れば skip なので .vessel.toml は scribe3 側で直接 cargo の形へ・trunk は器の上限（cargo / git / bats）の外なので xtask の subcommand から起こす（上限は動かさない）。ADR-4 の note に写した
