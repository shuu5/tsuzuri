# superpowers の徹底解析 — tsuzuri の 3 つの痛み（pipeline の並列性・繰り返す失敗・設計のぶれ）に当てて

- 日: 2026-10-06（UTC）・窓 cw1。持ち主の求め: tsuzuri は design-intent と beads で設計図‐issue‐code のグラフを組み、それを LLM に渡して pipeline を並列化し、速く高い精度で巨大な project を組み上げたい。いま並列性が上がらず、何度も failed し、設計がぶれる。superpowers にそれに役立つ知見が在るはずだから徹底的に調べよ。
- 読んだ物（全部一次資料・raw.githubusercontent.com を WebFetch）: subagent-driven-development の SKILL.md 全文と implementer／task-reviewer／re-review の 3 雛形と scripts 3 本（task-brief・review-package・sdd-workspace）、executing-plans の SKILL.md と scripts 2 本（task-start・task-done）、writing-plans・brainstorming（＋spec-document-reviewer）・requesting-code-review（code-reviewer.md）・dispatching-parallel-agents・using-git-worktrees・test-driven-development（＋writing-good-tests.md）の各 SKILL.md、docs/superpowers/specs の 5 本（fix-loop-redesign・plan-scoped-workspace の eval 結果・strict-cost-sdd・task-scoped-review-dispatch・document-review-system）と plan 1 本、RELEASE-NOTES（v5.0.5〜v6.4.2）、docs/testing.md・porting-to-a-new-harness.md、外部の分析 2 本。
- tsuzuri の側の材料: fleet/events.jsonl（2026-09-27〜10-06・便 749 本・work/pipeline-stats.py）、台帳の写し（700 bead・work/ledger-stats.py）、scribe2 の pipeline.md・dispatcher.md・row-review.md・rules/manifest.toml、tsuzuri の ADR-51・ADR-59・ADR-63・設計ノート surface-wave29a。

## 0. 結論（先に）

1. **superpowers の仕組みは「plan 1 本を 1 つの controller が直列に実行し、task ごとに新しい実装役と審査役を起こす」形で、並列は原理的に薄い**（並列の skill は「独立な障害を同時に調べる」だけ）。tsuzuri が求める「グラフから便を並列に起こす」機能は superpowers には無い。**ただし、tsuzuri の並列が上がらない理由は上限（pipe.max_live=16）ではなく、行の供給と交わりと審査の重さにあり、そこに superpowers の知見（task の切り方・Interfaces・pre-flight の衝突表・同形の task の束ね・controller は手を動かさない）が直に効く**。
2. **繰り返す失敗**は、tsuzuri の実測で「契約審査の非 PASS 20%（149/746）・その内訳は全部契約表の行の自己矛盾の型（section-material-missing 59・literal-mismatch 27・vacuous-assert 20・goal-done-contradiction 18）・行の審査の分析で 76 件の 96% が設計の側」。superpowers はまさにこの「plan の文書の欠陥が実装の段で見つかる」問題を、plan の self-review の 5 項・task brief の正確な値・「diff から確かめられない」の第 3 の判定・fix の周の上限（3 回は同じ実装役を再開・4〜5 回は強い model で新しく・上限で controller が裁定）・fix の diff だけの再審査で潰してきた。**最大の費用の揺れは審査の周の数**（彼らの実測）で、tsuzuri の FAIL → 起草へ戻す 1 周 約 17 万 token（ADR-63）と同じ構図。
3. **設計のぶれ**は、tsuzuri の 10 日で 問い 142・判断の記録 178・設計ノート 93・design-intent の commit 100 本超／日の峰、起草係の予算の越え 87%（ADR-63）に出ている。superpowers の答えは「儀式は task に比例させる」（spike／bounded／architectural の 3 段で、bounded は chat の 2 文で済ます）、「spec が拘束し、plan はその論証、判断は ledger の ruling（決めた事・理由・外れた時の費え）」、「止まるのは 4 種だけ」、「plan は決定の記録で code の写しでない（比の検査）」。tsuzuri は全部を ADR＋ノート＋行で持ち、起草係が差の file（完成 code）まで書くので、重さが一様で持ち主への問いが多い。
4. **tsuzuri に無くて superpowers に在る物は 3 つ**: (a) plan の文書の自己検査の型（Global Constraints・Interfaces・Review Focus・5 項の self-review・比の規則）、(b) 失敗の周の構造（再開・上限・裁定・scoped な再審査・「2 度目の fix の波は無い」）、(c) **仕組みの変更を planted-defect の eval で測る文化**（Sonnet の controller は 5 回中 4 回欠陥を通した・haiku の審査役は 10 の欠陥を 0 件・文書の審査の subagent の周は品質に差なし → 削除）。tsuzuri は ADR-63 で日次の物差しを置いたが、審査の雛形を替える前に planted-defect で測る歯は無い。
5. **superpowers に無くて tsuzuri に在る物**: グラフ（要件‐ADR‐条‐行‐bead‐commit）、機械の床（受付・gate・write-set の排他・CAS の着地）、複数 plan の列と口座の群、台帳と裁定 id、差の file による 91% の 1 本目の着地。superpowers は 1 session・1 plan・1 人の開発者の道具で、評価も 5 反復の「煙の強さ」。**写すのは仕組みの型であって、道具そのものではない。**

## 1. tsuzuri の実測（2026-09-27〜10-06・fleet の記録・work/pipeline-stats-result.txt）

| 測り | 値 |
|---|---|
| 便 | 作った 749・終わった 554（着地 530・停止 24） |
| 日ごとの便 | 作った 13〜182・着地 9〜95（09-27 の 182 が峰・10-01〜03 は 38〜41） |
| 1 便の長さ | 中央 17.5 分・上位 1 割 50.8 分・最長 445 分 |
| 同時に走った便の最大／日 | 3〜11（上限 pipe.max_live=16 に届いた日は無い）。≥1 本が走っていた時間は 1 日 487〜1,252 分 |
| 契約審査（Reviewed） | PASS 597・FAIL 75（literal-mismatch 27・vacuous-assert 20・goal-done-contradiction 18・other 5・section-material-missing 3・teeth-outside-write-set 2）・INCONCLUSIVE 74（section-material-missing 59・unparsed 9・ほか 6）→ 非 PASS 20% |
| gate（Gated） | PASS 850・FAIL 45・INCONCLUSIVE 9 → 非 PASS 6%（追随の再 gate を含む 1,441 回） |
| bead ごとの便の数 | 1 本 409・2 本 81・3 本 24・4 本以上 20（534 bead のうち 23% が撃ち直し） |
| dispatch の hold | 115（理由の多くが「器を書く便は同時に 1 本」「器の入れ替え待ち」「行の審査 FAIL を見て直すまで」） |
| 台帳（10 日） | 便 441・問い 142（全部 closed）・memo 82。問いは 1 日 0〜41（09-29 に 41・10-04 に 30） |
| 設計の量 | ADR 178・設計ノート 93・design-intent の commit は 1 日 4〜111 |
| ADR-63 の数（席の調べ） | 便は差を当てるだけ（149/149）・1 本目の着地 91%・契約審査が便の新しい token の 62%・起草係の予算の越え 87%（中央 1.69 倍）・gate PASS 133 便のうち 53 便に質の指摘が捨てられている |
| row-review.md の数 | FAIL／INCONCLUSIVE 76 件の 96% は設計の側が根・runner が主因 2・審査役の誤り 0 |

読み: **着地は確かだが、並列は上限の 1/5〜2/3 で推移し、審査の非 PASS が 5 本に 1 本、問いが日に数十**。並列の上限は効いていない。効いているのは行の供給（起草係 1 体が差の file まで書く重さ・予算の越え）、行の交わり（write-set の排他・depends の数珠・器を書く便 1 本）、審査の重さ（1 便の token の 62%）、そして設計の側の欠陥で便が往復する事。

## 2. superpowers の仕組みの精密な記述（tsuzuri の語に写しながら）

### 2.1 artifact と役割
- **spec**（brainstorming の出口・docs/superpowers/specs/）: 拘束の正本。「The spec is the binding authority, the plan is its argument, and your judgment settles what neither answers.」
- **plan**（writing-plans・docs/superpowers/plans/）: header（Goal・Architecture・Tech Stack・Spec・**Global Constraints**〔全 task に効く規則と spec の正確な値〕・**Review Focus**〔spec が暗に求めるが task の test が撃たない入力の類と故障の形を 5 つまで・担当 task に歯を留める〕）＋ task（**Files**〔create／modify／test の正確な path〕・**Interfaces**〔前の task から消費する物と後の task が依る物・署名だけ〕・**Steps**〔1 歩 1 動作: test を書く → 落ちるのを見る → 実装 → 通る → commit・test 名と assert・署名と spec の値・検証の命令と期待の出力〕）。右の大きさ: 「A task is the smallest unit with its own test cycle worth a fresh reviewer's gate」「Split only where meaningful rejection of one task is possible while approving neighbors」。比の規則: 「A plan longer than the code it describes has written the code instead.」
- **task brief**（scripts/task-brief）: plan から task N の見出し以下を awk で切り出した file。実装役は brief と Interfaces と Global Constraints と controller の裁定だけを受け、**session の履歴は受けない**（「A dispatch prompt describes one task, not the session's history」）。
- **report**（task-N-report.md）: 実装役が書く。何を作った・test・**TDD の証拠（RED の落ちと GREEN の通り）**・触った file・self-review・懸念。状態は DONE／DONE_WITH_CONCERNS／NEEDS_CONTEXT／BLOCKED の 4 語。「It is always OK to stop and say 'this is too hard for me.'」
- **review package**（scripts/review-package）: `git log --oneline`＋`git diff --stat`＋`git diff -U10` を 1 file に（`review-<base7>..<head7>.diff`・切り詰め無し）。審査役は **diff だけ読む**（「do not Read a changed file separately unless a hunk is cut off」・名指せる 1 つの危険だけ 1 回調べる・test の再走は疑いを名指した時だけ・package 全体の suite は撃たない）。
- **ledger**（`.superpowers/sdd/<plan>/progress.md`）: plan ごとの作業場（plan-scoped・identity 行で持ち主を名指す）。行の形は 6 つ固定: `Task N: complete (commits a..b, …)`／`Task N: fix round R/5 (X addressed, Y open — …)`／`Ruling: <決めた事> — <理由> — <外れた時の費え>`／`Pre-flight: …`／`Final: fixed …`／`Final: minor (deferred): …`。「The ledger is your recovery map」（文脈の圧縮を越えて再開する）。
- **役割**: controller（plan を 1 回読み・brief を切り・起こし・審査を起こし・裁定を ledger に書く。**自分で直さない・審査役に「これは見逃せ」と言わない・重さを先に決めない**）、implementer（1 task・新しい文脈・subagent を起こさない）、task reviewer（spec 準拠と品質の 2 判定を 1 回の読みで・file:line を必ず添える・「a stated rationale never downgrades a finding's severity」・第 3 の判定 **⚠️ Cannot verify from diff** は controller が自分で確かめる）、final reviewer（最も強い model・branch 全体・1 回だけ）。

### 2.2 実行の流れ（subagent-driven-development）
1. **Setup**: 作業場を作る・ledger の identity 行・plan を 1 回読む・**pre-flight の衝突走査**（file か interface を共有する task の対を全部表にし、何を produce し何を consume するかを並べ、plan の字に対して全部 rule してから Task 1 を起こす。「Rule on everything you find before execution begins」）。
2. **Task の周**: BASE の sha を記す → brief を切る → 実装役を起こす → report の 4 語で分岐（BLOCKED は文脈不足／推論／範囲／plan の誤りのどれかを判じて手を替える。「Never ignore an escalation or force the same model to retry without changes」）→ review package → task reviewer（Global Constraints を逐語で渡す）→ **fix の周**: spec ❌か Critical／Important か ⚠️ の確定で入る。**round 1〜3 は同じ実装役を再開**（文脈を保つ）、**round 4〜5 は強い model で新しい実装役**（「A prior implementer attempted this task N times; you own it now」）。**毎 round の終わりに fix の diff だけの scoped な再審査**（各 finding を ADDRESSED／NOT ADDRESSED・新しい破壊だけ見る・外は Out-of-Scope）。**round 5 で残れば controller が裁定**（審査役が誤り → 理由付きで park／本物だが致命でない → 延期で park／本物で荷重 → 最小の fix を rule して先へ）。「Adjudicating earlier to end a loop is pre-judging with a different name.」
3. **止まる 4 種だけ**: 不可逆・security・worktree の外の副作用・plan が全ての道を推測にする程壊れている。それ以外は決めて ledger に書き進む。「Do not pause to check in with your human partner between tasks.」
4. **終わり**: branch 全体の最終審査 1 回 → findings があれば **fix の subagent 1 体に全部渡して 1 回**＋scoped 再審査 1 回（「There is no second fix wave」）→ Rulings を全部まとめて人に渡す → 作業場を消す → finishing-a-development-branch（merge／PR／残す は人が選ぶ）。
5. **束ね**（v6.3）: 「Small same-shape tasks batch into one dispatch」。束ねの審査は brief の全 file が diff に在る事を確かめる。
6. **inline の形**（executing-plans）: 同じ作業場と ledger と止まる規則で、1 つの文脈が全 task を実装し最終審査 1 回。費用は安く、新しい文脈の利点を捨てる。handoff で両方の費用を示して 1 つ推す。

### 2.3 彼らが測って分かった事（docs/superpowers/specs・RELEASE-NOTES）
| 実測 | 出所 |
|---|---|
| 「review-loop count が run ごとの費用の揺れの最大」。周の上限が無い形は「implement, review, fix, review, review, fix…」で収束しなかった | fix-loop-redesign-design（2026-07-15） |
| 新しい実装役で fix すると文脈を作り直し task の枠を失う → **再開**が勝る。上限 3＋2、上限で裁定 | 同上 |
| 1 plan 約 $13。controller（opus・150 turn）$6〜7、実装役 10〜13 体 $5〜6。審査の周 2〜4 回が揺れの主因で、多くは **plan の曖昧さの誤読** | strict-cost-sdd-design（2026-06-10） |
| **Sonnet の controller は「plan 準拠の弁護」に崩れ、植えた欠陥を 5 回中 4 回通した**。haiku の task reviewer は 10 の欠陥を正しい重さで 0 件。「cheap reviewers rationalize defects rather than reject them」 | 同上（L2・L3） |
| 手書きの散文の plan は、opus が書いた完成 code の plan より実行費用が約 2 倍 | 同上（L1） |
| 品質の審査役が 1 task に 50 命令超・200 秒。controller が審査役に「これは plan が禁じている」と嘘の前提を与えて欠陥を無視させた。diff を prompt に貼ったのは 22 回中 2 回 | task-scoped-review-dispatch-design（2026-06-09） |
| 審査役の範囲を diff に絞り、2 審査役を 1 つ（2 判定）に → 64.9 分／21.2M → 54 分／14〜16M／$13（−20〜25%）・植えた欠陥の捕捉は維持。「禁止は裏目に出る → 肯定の手順（名指せる危険を調べよ）」 | 同上 |
| spec と plan の **文書審査の subagent の周（25 分）は、inline の self-review checklist（30 秒）と品質が同じ** → 削除 | RELEASE-NOTES v5.0.6 |
| plan ごとの作業場（identity 行）は誤帰属を構造で消す。5 反復×2 場面＝25/25 だが「smoke-strength signal, not statistical」。道具の呼びは 9.0 → 9.6 で減らない | plan-scoped-workspace-eval-results（2026-07-06） |
| 「plan は決定の記録。code の写しではない」で token 1/3・defect probe 9/9 | RELEASE-NOTES v6.4.2 |
| controller が審査役に見逃しを指示して欠陥が出荷された → 「Suppressing findings and pre-rating severity are now banned outright」。審査役は read-only で rationale に懐疑的 | RELEASE-NOTES v6.0.0 |

## 3. tsuzuri の 3 つの痛みに当てる

### 3.1 pipeline の並列性が上がらない

**事実**: 上限 16 に対し峰 3〜11。hold 115 の多くが「器を書く便は同時に 1 本」「入れ替え待ち」。1 便 中央 17.5 分。契約審査が token の 62%。起草係が差の file を書き終えるまで行は列に入らない。

**superpowers から写せる物**
1. **Interfaces を行に持たせ、pre-flight の衝突表を機械で組む**。superpowers の plan は task ごとに「消費する署名／提供する署名」を書き、controller が Task 1 の前に「file か interface を共有する対」を全部表にして rule する。tsuzuri の行は write-set と depends しか持たず、交わりは write-set の file 単位でしか測れない。**行に `interfaces: {consumes: [...], produces: [...]}` を足し、tz derive がノートごとに衝突表（write-set の交差・interface の供給と消費の対）を導出して契約表の導出物に置く**。得: (a) 並列の 係 が隣の行の契約を知って起草できる（今は depends の鎖で直列化するか、着地を待つ）、(b) dispatcher が「file は交わらないが interface が合わない」組を起こさない、(c) 行の審査（row-review）が衝突表の未解決を 1 件ずつ名指せる。
2. **同形の小さい行を 1 便に束ねる**。superpowers v6.3: 小さい同形の task は 1 回の dispatch に束ね、審査は brief の全 file が diff に在る事を確かめる。tsuzuri は行 1 本 ＝ 便 1 本 ＝ 契約審査＋gate の全周。S の行が多い日（10-05 は 102 便）は審査の固定費が並列を食う。**同じノートの、depends の無い S の行 k 本を 1 便に束ねる口**（契約は k 行の和・verify は和・審査は行ごとの done を照らす）を dispatcher に。費用の 62% を占める審査が k 分の 1 になる。
3. **task の右の大きさ**: 「1 つの test の周を持ち、隣を通しながら 1 つだけ落とせる最小の単位」。tsuzuri の行は done の項目 8〜12・歯 5〜10 本・verify 5〜10 行が普通（surface-wave29a）。superpowers なら 2〜5 分の task 数本。**行を割る物差しは「隣を通して 1 行だけ落とせるか」**で、落とせないなら 1 行に束ね、落とせるなら割る。
4. **controller は手を動かさない**（「No controller fixes — controller context stays clean」）。tsuzuri の席は起草・審査の見・hold・問いを全部持つ。ADR-59／63 で係の型と門に落とし始めているので同じ向き。**席が差の file を直す・審査の判定を言い換える事を門で断る**（superpowers は「審査役に見逃しを指示する」を禁止にした）。
5. **並列の skill の教え**: 「fixing one might fix others」なら並列にしない。tsuzuri の同じ設計から出た兄弟の行が同じ理由で 4 秒差で落ちた（row-review.md §1）＝関連する失敗を並列に起こした例。row-review の「兄弟の待ち」はその答えで、superpowers の pre-flight の表と同じ思想。

**superpowers に無い物**: 複数 plan の列・口座の群・write-set の排他・CAS の着地・便の process の封じ込め。並列の本体は tsuzuri が持っている。足すのは「行の供給を速くする（起草の重さを下げる＝§3.3）」「交わりを interface で測る」「小さい行を束ねる」の 3 つ。

### 3.2 何度も failed する

**事実**: 契約審査の非 PASS 20%。型は全部「行の文書の自己矛盾」（節の材料が無い・字の不一致・空振りの assert・goal と done の矛盾）。bead の 23% が 2 便以上。1 周 約 17 万 token。gate の非 PASS 6%。row-review の分析: 96% が設計の側。ADR-63: 契約審査の落ち 10 件のうち 7 件は機械で見られる型。

**superpowers から写せる物**
1. **plan の self-review の 5 項を、行を出す前の係の床と手引きに**: (1) spec の全要件が task に写っているか（tsuzuri: 要件 req と done の対応）、(2) 何も決めない step・不要な本体を持つ step が無いか、(3) **署名が task 間で一致しているか**（literal-mismatch の根）、(4) Review Focus の各項に歯が在るか（vacuous-assert の根: 「every test names the break it catches」「mutation check: 1 つの変異に 1 つの歯が落ちる」「mirrored assertion は常に通る」）、(5) **比**: plan が spec の何倍かを数える（goal-done-contradiction・section-material-missing の根は節と行の字が離れる事）。ADR-63 の決定 (4)（出す前の床: preflight・tz check・derive --check・表の検査の rc の記録）は (1)(5) の機械の側。**(3)(4) は機械で全部は見られないので、係の手引き（agent-discipline）に superpowers の rationalization table の形で入れる**。
2. **writing-good-tests の規則を vacuous-assert の対策に**: 「The mock earns no assertions」「Derive expectations independently（期待値を被試験 code で作らない）」「Test behavior, not text（source を grep する歯は source が source である事しか示さない）」「Mutation check」。tsuzuri の gate の観点 teeth-nonvacuous の雛形と、係の手引きの字に。
3. **fix の周を便の中に作る**（最大の効き）。今は Reviewed FAIL → 行が列から外れ → 席／係が直す → 設計の PR → 新しい便＝全周。superpowers は同じ task の中で **実装役を再開**して finding を逐語で渡し（round 1〜3）、**fix の diff だけ**を再審査し、round 4〜5 で強い model に替え、上限で controller が裁定する。tsuzuri に写すなら: 契約審査 FAIL の型が「行の字の直しで済む物」（literal-mismatch・goal-done-contradiction）は、**同じ便の中で起草係を再開して行だけを直させ、再審査は差分の鍵だけ**（row-review の「撃ち直さない」の鍵と同じ）。gate FAIL は **runner を同じ worktree で再開**し finding を渡し、再 gate は fix の diff の観点だけ。回数は rules 行で上限（superpowers は 5・tsuzuri は regate が INCONCLUSIVE だけで FAIL は終端）。得は 1 周 17 万 → 数万。
4. **「diff から確かめられない」の第 3 の判定**を契約審査と gate に。審査役が触っていない code に在る要件を黙って PASS にせず、⚠️ で controller（席）に返す。tsuzuri の INCONCLUSIVE は「材料が足りない」で、「審査役の目の外」とは別。ADR-63 の決定 (6)（判定と数の食い違いを FAIL に読む）と並べる。
5. **report file と RED→GREEN の証拠**: 実装役は report に「落ちた test の出力 → 通った出力」を貼り、審査役は **読んで確かめ、再走しない**。tsuzuri の gate は verify を再実走する（確かだが重い）。runner の report に RED／GREEN の証拠を要求し、契約審査はそれを読む形にすれば、gate の再実走は着地の確かめに絞れる。
6. **実装役の「できない」の口**: NEEDS_CONTEXT／BLOCKED／DONE_WITH_CONCERNS。tsuzuri の runner は質問（Questioned・6 件）か Failed（6 件）しか無い。「同じ model に手を替えず再試行させない」を dispatcher の規則に（今は resume が同じ形で撃ち直す）。

**superpowers が教える「やらない方がよい事」**
- 文書審査の subagent の周を足さない（品質に差が無く 25 分）。tsuzuri の行の審査（pre-merge の lens）は彼らが捨てた形に近い。**行の審査は、機械の床（ADR-63 (4)(5)）＋係の inline の self-review を先に置き、lens は planted-defect で効きを測ってから残す**。
- 安い model に判断を渡さない（Sonnet controller 4/5 失敗・haiku 審査役 0/10）。tsuzuri の先撃ちの sonnet lens は「PASS 11 回のうち 2 回が本番で FAIL」（row-review.md）で同じ現象。判断の段は opus のまま、機械で決まる物だけ安くする（彼らの「mechanical, never judgmental」の線）。
- 審査役に「これは無視せよ」と言わない・重さを先に決めない。

### 3.3 設計がぶれる・設計の並列性が上がらない

**事実**: 問い 142／10 日（峰 41／日）、ADR 178、ノート 93、起草係の予算の越え 87%、席の記憶だけの決まり（ADR-63 (ク)）、差の file 196 本 6.2 MB。

**superpowers から写せる物**
1. **儀式を task に比例させる**（brainstorming の分類）: spike（答えが知りたい・code は捨てる）／bounded（repo に流れが在る変更・chat の 2 文の設計で実装へ）／architectural（新しい subsystem・spec と plan が要る）。「When in doubt between two paths, take the heavier one」。tsuzuri は全部が architectural の形（ADR＋ノート＋行＋差の file）。**行の重さを 3 段にし、bounded は ADR を起こさず行だけ（design の pointer は既存の節）、spike は相談の窓（この窓の用途）**。問いの数と ADR の数が下がる。
2. **spec が拘束・plan は論証・判断は ruling**。tsuzuri は要件書と憲法が spec、ノートの節と行が plan。ぶれの多くは「plan の段で spec を書き直す」事（ノートの版が 1 日に 27 回・surface-wave29a）。**行の起草で要件と条を変えない（変えるなら別の問い）**を手引きと床に。ruling の形 `決めた事 — 理由 — 外れた時の費え` を席と係の ledger（台帳の notes）の定型に。持ち主の常設の裁定「推奨で進める」（2026-09-28）は superpowers の「止まるのは 4 種だけ」と同じで、**4 種（不可逆・security・外への副作用・全ての道が推測）以外は席が rule して記帳し、問いにしない**を規則の行にすれば、問いの日 41 は減る。
3. **plan は決定の記録で code の写しでない**。tsuzuri の差の file は完成 code（ADR-63 (ア)）。得は 91% の着地。失う物は起草係の重さ（予算の越え 87%）と、行の供給が差の file の完成を待つ事＝並列の天井。superpowers の 2 つの実測（完成 code の plan は実行が 2 倍安い／決定だけの plan は token 1/3 で 9/9）は両立する: **「決定（署名・test の assert・spec の値・置き場）」を行に書き、完成 code は便の runner に書かせる形へ戻すと、起草は軽く並列になり、便は検証だけでなく実装を担う**。試すなら bounded の行から（差の file は任意に）。ADR-63 の retreat (7)（着地 1 本の token が 1.1 倍）と first_run_landed_3d（85%）で測れる。
4. **plan ごとの作業場と identity 行**: tsuzuri の起草の置き場（drafts/w*）は係ごと。ノートごとの衝突表と ledger を tz が置けば、係が隣の係の決めを読める（設計の並列）。
5. **文の工学**: 手引きは「Use when …」の引き金だけの description・200〜500 語・Iron Law・rationalization table（Excuse／Reality）。ADR-59 の agent-discipline（2,000 字以下）に同じ形を。superpowers の v6.1〜6.2 の「圧縮の運動」（売り文句を削る・図を散文に）は P-13 の 8,000 byte と同じ問題意識。

### 3.4 グラフから LLM に渡す物（持ち主の構想の芯）

superpowers の task brief は「task の字＋Interfaces＋Global Constraints＋controller の裁定」で、**session の履歴は渡さない**。tsuzuri の ADR-51 の近い仕様（要件・ADR・条・規則・受入の束・中央 4.6 KB）はその上位互換になれる。足す欄は 3 つ:

| 欄 | superpowers | tsuzuri の今 | 足し方 |
|---|---|---|---|
| Global Constraints | plan の頭に全 task に効く規則と spec の値 | 憲法の規範文の写し（ADR-51 (3)・約 15 KB）と規則の行 | ノートごとの「全行に効く値」（規則の行の id と値）を tz derive が 1 塊に |
| Interfaces | task ごとに consumes／produces の署名 | 無い（write-set と depends のみ） | 行に `interfaces` を足し、衝突表を導出 |
| Review Focus | spec が暗に求めるが歯の無い入力の類を 5 つまで・担当 task を留める | 無い（gate の観点は質の 5 つ・契約審査は適合の 3 つ） | 行かノートに `review-focus` を足し、契約審査の材料と done-teeth の照らしに |

さらに **report file**（runner の RED／GREEN の証拠）と **review package**（diff＋stat＋log の 1 file・審査役は diff だけ読む）を器の材料の形に。審査の token が 62% なら、「diff だけ読む・名指せる危険だけ調べる・suite は撃たない」の scope budget が最も効く節約で、彼らは −20〜25% を測った。

## 4. 採らない物・注意

- superpowers を丸ごと席や係に載せる（所見 cw1-4 と同じ理由。注入の競合・repo への書き物・worktree と便の二重）。
- 文書審査の subagent の周を増やす（差なしの実測）。
- 安い model に判断の段を渡す。
- superpowers の eval は 5 反復で「smoke-strength」。数は参考値で、tsuzuri の規模（10 日 749 便）では自分で測る方が確か。ADR-63 の物差しに **planted-defect の歯**（審査の雛形を替える前後で同じ欠陥を植えた便を撃つ）を足す。
- superpowers の controller は 1 人の開発者の session で、tsuzuri の席のような長命・多 plan・口座の群の運用は想定外。写すのは型。

## 5. 次の一手の案（効きと手の軽さの順・採否は席と持ち主）

1. **fix の周を便の中に**（契約審査 FAIL の字の直しは同じ便で係を再開・gate FAIL は runner を再開・fix の diff だけ再審査・上限は rules 行）— 失敗の往復 17 万／周を数万に。ADR の改め（pipeline の Failed の終端・FR14 resume）。
2. **行に interfaces と review-focus の欄**、tz derive でノートごとの衝突表 — 並列の係と dispatcher の判じを interface で。ADR-51 の context 欄の隣。
3. **係の手引きに plan の self-review 5 項と writing-good-tests の規則**（rationalization table の形）— vacuous-assert・literal-mismatch・goal-done-contradiction を出す前に。ADR-59 の agent-discipline の改め。
4. **行の重さの 3 段**（spike／bounded／architectural）と **ruling の定型と止まる 4 種** — 問いと ADR の数を下げる。憲法 A-1 と常設の裁定と整合。
5. **同形の S の行の束ね** — dispatcher の口 1 つ。審査の固定費を割る。
6. **planted-defect の歯** — 審査の雛形（lens.txt・lens-contract.txt）を替える前後で測る。ADR-63 (6)(7) の直前に。
7. **差の file を任意にする試し**（bounded の行から）— 起草の重さと並列の天井を測る。ADR-32 決定 (9) と ADR-63 retreat (7) の線で。

## 6. 出所
- superpowers: SKILL.md 群・雛形 3 本・scripts 5 本・docs/superpowers/specs の 5 本・RELEASE-NOTES（2026-10-06 読み）。
- tsuzuri: fleet/events.jsonl（work/pipeline-stats-result.txt）・台帳の写し（work/ledger-stats-result.txt）・scribe2 docs/design の pipeline.md・dispatcher.md・row-review.md・rules/manifest.toml・ADR-51・ADR-59・ADR-63・surface-wave29a。
