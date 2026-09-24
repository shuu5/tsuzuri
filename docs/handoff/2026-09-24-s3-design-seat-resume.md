# s3:design 席の作業記憶（起こし直しの前・2026-09-24T13:1xZ）

次の席が最初に読む file。読む順: この file → docs/handoff/2026-09-24-v3-kickoff.md → docs/grill/2026-09-24-D-tech-stack.md。

## 1. いまどこか
- 論点 D（技術スタック）の 1 問を持ち主に提示済み・**答え待ち**。問いと推奨は docs/grill/2026-09-24-D-tech-stack.md §5（推奨 = (a) Rust の同期 server + build 無しの HTML/JS・撤退条件つき）。裁定はまだ無い＝ADR・vocabulary・srs には何も書いていない。
- 討論は「器の穴」の直しを待って止めていた。持ち主の指示: 討論の前に scribe2 側を直させる。

## 2. 器の穴（持ち主の裁定・2026-09-24T13:1xZ・逐語は scribe2 の台帳へ）
- 要点: 新しい置き場の立ち上げは `scribe2 init` 1 発で終わる簡便さが要る。今日は手作業 7 手（state dir・口座の symlink・host.toml の写し・起動行・git 設定・tmux・seat launch の長い引数）で、vessel init と .vessel.toml が抜けた。seat launch の引数の長さも同じ穴。
- 追加の裁定（13:2xZ・逐語は scribe2 の台帳へ）: 人間向けの説明が 0 で初見の人には知りようがない／人が打つ command は最小限にし、それで器が十全に動くようにする。s3 の読み = 穴は 3 つで 1 組（1 発でない・説明 0・command が多い）。目標は人が打つのは init と doctor の 2 つを上限に、残りは器が自分で行う。
- 報告先: scribe2 席（session 名 scribe2-2a）へ Claude Code の session 間メッセージで 3 通送付済み（欠落の実測 → 訂正: 器の穴として直す便に起票せよ → 説明 0・command 最小限の裁定）。scribe2 側が台帳に起票し ADR を書く。
- 起票済み（2026-09-24・scribe2 席の報告）: scribe2 の台帳 **s2-07l.609**（逐語と UTC の分つき・P1・open）。設計 doc と ADR は scribe2 側で書く。`bd --readonly show s2-07l.609` で読める。
- scribe3 側の直し（scribe2 席が実施・13:03Z）: `.vessel`（untracked・name=scribe2 version=2）と `.vessel.toml`（commit aeb81b1・allowed-commands=["git"]・common-verify=["git diff --quiet"]・requirements=design-intent/srs.yaml）。`scribe2 vessel show .` が stateDir を返すことを実測。
- v3 への持ち越し: 論点 A で「器の口として init を 1 本持つ」を要件候補に（grill 記録 §7）。

## 3. 起こし直した席が最初に確かめること
1. SessionStart の出力に `[scribe2/SessionStart]` の名乗りと orchestrator の指示文（決定はしごの行）が入っているか。無ければ state dir の inject.jsonl に session-start-header が無いことを実測して scribe2-2a へ 1 行返す。
2. `git status`（.vessel は untracked のままでよい・folio2 と同じ）。`folio check --dir design-intent` は違反 0・まだ分からない 2 が正常。
3. 持ち主の答えが来ていなければ、論点 D の 1 問（grill 記録 §5）を再掲して待つ。答えが来ていれば ADR-2（技術スタック）を design-intent/adr/ に起こし、語を vocabulary.yaml へ、要件を srs.yaml へ書く。

## 4. 決めごと（この席で確かめた運用）
- grill は 1 論点 1 問・推奨 1 つ・散文で問う（AskUserQuestion は使わない・grill-me skill）。
- 討論の記録は docs/grill/ の markdown、正本は design-intent/ の YAML。逐語は tracked に写さない（台帳へ）。
- この repo は remote 無し・master へ直接 commit（前席と同じ運用）。
