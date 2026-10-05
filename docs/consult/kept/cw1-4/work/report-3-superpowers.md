# superpowers（obra）の逆解析 — なぜ人気か・開発に効く部分・tsuzuri との差・tsuzuri の開発への活かし方

- 日: 2026-10-05（UTC）・窓 cw1・読んだ物: README・RELEASE-NOTES・skills 15 本の SKILL.md・hooks/hooks.json・hooks/session-start・docs/testing.md・docs/porting-to-a-new-harness.md（全部 raw.githubusercontent.com を WebFetch で読んだ・clone は殻の allowlist で不可）
- 著者: Jesse Vincent（Request Tracker の作者・Perl 5 の pumpking・Keyboardio 共同創業）。初版 2025-10（Claude Code の plugin の公開と同じ日）。GitHub の頁の字では 295.3k stars（2026-03 の外部記事では 93k）。MIT。

## 1. 仕組み（何がどう動くか）

1. **入口は settings hook 1 本**。`hooks/hooks.json` の SessionStart（matcher `startup|clear|compact`）が `hooks/session-start` を撃ち、`skills/using-superpowers/SKILL.md` の全文を `hookSpecificOutput.additionalContext` で注入する。harness ごとに出力の形だけ替える（Cursor・Copilot は top-level の `additional_context`）。文書の字どおり「The bootstrap is the entire integration. Without it, the skill files are inert」。
2. **以後は model が自分で skill を読む**。using-superpowers は「関連する skill を**どんな応答や行動の前にも**先に呼べ」「1% でも当たると思えば必ず使え」「process の skill（brainstorming・systematic-debugging）→ 実装の skill の順」「user の指示 > skill > 既定」を命じる。skill は folder 1 つ＝SKILL.md 1 枚で、必要な物だけ読む（progressive disclosure・context を汚さない）。
3. **流れは固定の 1 本**: brainstorming（Spike／Bounded／Architectural に分類・承認の門）→ using-git-worktrees（隔離）→ writing-plans（2〜5 分の task・決定の記録）→ subagent-driven-development か executing-plans（実装）→ test-driven-development（RED-GREEN-REFACTOR・test の前に書いた code は捨てる）→ requesting-code-review（spec 準拠 → 品質）→ finishing-a-development-branch（merge／PR／残す の選びは人）。
4. **skill の書き方自体が skill**（writing-skills）: description は「Use when …」の引き金だけ（workflow を書くと model が description だけ見て済ませる）、frequently-loaded は 200 語以下、Iron Law・rationalization table・red flags・flowchart で「飛ばす」失敗を構造で封じる、skill も TDD で書く（pressure scenario → RED → SKILL.md → GREEN・subagent probe で測る）。
5. **測る**: `tests/` は hook と server の統合試験、`evals/` は Quorum（本物の coding agent CLI を Gauntlet QA agent が駆動し、受入基準と決定的な post-check で採点）。公開 CI は静的な門だけ、live の sweep は夜間。
6. **版の歩み**（RELEASE-NOTES）: v2 skill を別 repo に → v3 公式 skills 体系に合流・命令形へ書き直し → v4 2 段 review・「executable specifications」（flowchart を正本） → v5 brainstorming の hard gate と必須 checklist（「model が brainstorming を飛ばす」対策）・browser の brainstorm companion → v6 controller は read-only の観察者・単一 reviewer が diff を file で読む・vendor-neutral な語（"dispatch a subagent"）・16 harness → v6.1〜6.2 bootstrap の圧縮（図を散文に・売り文句を削る・subagent probe で挙動を検証）→ v6.3 「儀式は task に比例」（spike/bounded/architectural）→ v6.4 inline 実行の復活・diagnosing-superpowers・「plan は決定の記録で code の写しではない」（token 1/3 で 9/9）。

## 2. なぜ人気か（仕組みから読める理由・推し量り）

- **導入の摩擦が 0 に近い**: `/plugin install superpowers@claude-plugins-official` 1 行。設定も project の file も要らない（SessionStart の注入だけ）。16 の harness で同じ skills が動くので乗り換えの壁が無い。
- **効き目が初回から見える**: 「書く前に問う」「test を先に」「終わったと言う前に証拠」は、既定の agent の一番目立つ失敗（飛び付き実装・虚偽の完了報告）に直撃する。使う側は自分の規律を書かずに済む。
- **文書の工学が本気**: 「model が飛ばす」問題を prose の説得でなく hard gate・Iron Law・rationalization table・flowchart で封じ、skill を eval で測って直す。版ごとに失敗の形（飛ばす・儀式が重い・token を食う）に対して構造の対策を打っている。
- **方法論としての一貫性**: TDD・root cause first・YAGNI・小さい task・spec 準拠 review・人の承認は入口と出口だけ、という開発者の常識を agent の手順に写した。人が納得できる物を agent にやらせる。
- **著者の信用と発信**: 長年の OSS 保守者。blog と Discord と release note で理由を語る。

## 3. 開発に効く部分（抜き出す価値の高い技法）

| 技法 | 中身 | 効く理由 |
|---|---|---|
| Iron Law | 「test を見て落ちるのを見なければ、それが正しい物を測っているか分からない」「root cause の前に fix なし」「証拠の前に主張なし」 | 1 文で止まる条件が決まる。例外を潰す文（"no exceptions: don't keep as reference"）を添える |
| rationalization table / red flags | 「too simple to test」「I'll test after」「emergency」… を列挙して全部却下 | model が自分に言い訳する語をあらかじめ塞ぐ。probe で集めた実測の語彙 |
| trigger-only description | description に手順を書かない。「Use when …」と症状だけ | 手順を書くと model が本文を読まずに description で済ませる（実測） |
| plan = 決定の記録 | task は「signature・test の assert・spec の値」だけ。「plan が code より長ければ code を書いてしまっている」。self-review で spec との比の検査 | 実装者に判断を残さず、token を 1/3 に |
| ledger（Ruling: decision — why — cost if wrong） | 逸脱は必ず記帳。黙った破棄の禁止 | 後から追える。tsuzuri の台帳と同じ思想 |
| controller は read-only | 判断は implementer（文脈を持つ）と人に返す。controller は fix しない（文脈を汚さない） | 席の context を守る。tsuzuri の席の文脈の分離（相談の窓・便）と同じ |
| review の 2 段→1 段・diff を file で読む | spec 準拠と品質。再 review は fix の diff だけ。5 round で打ち切り | 審査の形が決まっている |
| 止まる 4 つ | 不可逆・security・worktree の外の副作用・plan が壊れている時だけ止まる。「task の間で人に確認しない」 | 自律時間を伸ばす。tsuzuri の憲法 A-1（消す・外へ出す・使う は確認）と同じ線 |
| skill の TDD と eval | pressure scenario → 文を直す → subagent probe → eval harness | 文の効きを測る文化 |
| 圧縮の規律 | 図を散文に、売り文句を削る、200 語以下、per-harness の表を外す | context は有限。tsuzuri の条 P-13（注入は 8,000 byte 以下）と同じ問題 |

## 4. tsuzuri との差

| 観点 | superpowers | tsuzuri |
|---|---|---|
| 守りの置き場 | **model に読ませる文**（hard gate・flowchart）。機械の門は無い（finish の `discard` の typed 確認くらい） | **機械が落とす**（条 P-6 床と天井・P-7 fail-closed・P-14 編集時に止める・N-5 文章にしか無い規則は規則でない）。文は天井 |
| 判断の記録 | plan ごとの ledger file（workspace・git log で復元） | 台帳（bd）と裁定 id・判断の記録 ADR・憲法の改訂手続き |
| 役割 | controller／implementer／reviewer（subagent） | 席（orchestrator）／係（便の runner）／検証役（qv）・gate | 
| 縛り方 | brief と report の file・prompt の雛形 | 契約（write-set・verify・size）・歯・床・極性 |
| 人の位置 | brainstorming と finish で承認。途中は止まらない | 裁定だけ人（north_star）。決定の画面で承認 |
| 測り | skill を eval で測る（Quorum・probe） | 歯と床で機械検査。席への文の効きは grill（検証役）が手で測る |
| 配り | 16 harness・vendor-neutral な語 | Claude Code と器 scribe2 に固定（host 固有の分岐は N-7 で禁じるが vendor は 1 つ） |
| 大きさ | skills 15 本・数千語・plugin 1 つ | 憲法 39 条・SRS・ADR 50 本・Rust の器 3 つ（tsuzuri・scribe2・folio） |
| 対象 | 1 人の開発者の 1 session の品質 | 持ち主が裁定だけで設計から運用まで任せる仕組み全体 |

要するに superpowers は **天井（AI の側）の技法を極めた物**で、tsuzuri は **床（機械の側）を極めた物**。重なるのは「人は承認だけ・途中で止まらない・逸脱は記帳・context を汚さない」の思想。tsuzuri に薄いのは「席と係に読ませる文の工学と、その効きを測る仕組み」。superpowers に無いのは「破られたら機械が止める線」。

## 5. tsuzuri の開発への活かし方（具体の案・採否は席と持ち主）

1. **注入の形を借りる（最優先・memo t3-hub.74.59・条 P-13）**: tsuzuri の plugin の hooks.json に SessionStart（matcher `startup|clear|compact`）を足し、憲法の写し（前文・規範文・役割・8,000 byte 以下）を `additionalContext` で決定的に注入する。superpowers がやっている事そのもので、器の変更が要らない。Claude Mods が入れば `prompt.context` か `session.start` へ移す（report-4）。
2. **書法を借りる**: 席の手引き・係の brief・tsuzuri の plugin の skill を「executable specification」の形に書き直す。Iron Law 1 文・hard gate・rationalization table・red flags・trigger-only の description・200〜500 語。特に「席が hook の 1 行を飛ばす」「係が verify を飛ばす」「完了を言う前に証拠が無い」の 3 つに。
3. **測りを借りる**: 席と係への注入文を pressure scenario で測る「文の歯」を作る（subagent probe・no-guidance の対照・5 回以上）。grill の手の測りを機械の sweep に寄せる。superpowers-evals の Quorum の設計（Gauntlet QA agent＋決定的 post-check）が参考。
4. **係の作業規律を写す**: verification-before-completion（5 段の門）・systematic-debugging（4 段・3 回失敗で設計を疑う）・receiving-code-review（追従の語の禁止・検証してから直す）を係の手引きか skill に。
5. **契約の書き方に「plan は決定の記録」を入れる**: 契約表の行は signature・test の assert・spec の値を書き、実装の写しを書かない。self-review で spec との比を見る。
6. **採らない物**: 丸ごとの install（「skill を先に」の注入が憲法の注入と競合・`docs/superpowers/` を repo に書く）・using-git-worktrees と subagent-driven-development（便と worktree と二重）・brainstorm の browser companion（WebSocket server・telemetry）・dispatching-parallel-agents（席の便の並列と二重）。

## 6. 根拠の出所
- README・RELEASE-NOTES・skills/*/SKILL.md・hooks/hooks.json・hooks/session-start・docs/testing.md・docs/porting-to-a-new-harness.md（raw.githubusercontent.com・2026-10-05 読み）。
- 外部: horadecodar の紹介記事（著者の背景・2026-03 の 93k stars）。
- 注: 「なぜ人気か」の節は仕組みと版の歩みからの推し量り。stars の数は GitHub の頁の字をそのまま写した。
