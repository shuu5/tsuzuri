# 相談の窓 consult — 当座の運び方（器の対応を待たない形・判断の記録 ADR-10 決定 (9)）

本式（器が窓を起こし、所見の口が欄を検め、器の口が 1 行を送る）ができるまでの手順。型と置き場は本式と同じで、渡す所だけ人の手。

## 手順
1. **相談を開く**（orchestrator の窓で）: 持ち主が「相談を開いて: <題>」と言う。orchestrator が台帳に相談の bead を置き、材料の束を組む。
   - 置き場: `~/.local/state/scribe2-v2-state-tsuzuri/consult/<相談 id>/`
   - 束: `bundle/question.md`（問い・前提・望む形）と `bundle/reads.md`（読む file の一覧と理由）
2. **窓を開く**（持ち主）: tmux の新しい窓で repo の根に入り、model を選んで Claude Code を起こす。既定は Fable。
3. **最初の一言**（持ち主 → 相談の窓）: 「docs/consult/brief.md を読んで、相談 <相談 id> を始めて」。
4. **相談する**。相談の窓は要点を `notes.md` に書き足す。
5. **所見を出す**（相談の窓）: `finding-<n>.yaml` を置き場に書く。型は docs/consult/finding-template.yaml。
6. **知らせる**（持ち主 → orchestrator の窓）: 「相談 <相談 id> の所見 <n> を読んで」。
7. **処分**（orchestrator）: 所見を読み、採る・一部採る・採らない を理由つきで台帳に記録する。承認が要る決定は問いにして持ち主に出す。
8. **model を替える**: 窓を閉じ、2 と 3 をやり直す。新しい窓は束と控えと前の所見から続ける。

## 当座の穴（本式で塞ぐ）
- 手で開いた窓には器の guard が掛からない。相談の窓が repo を書かないことは brief の作法だけが守る。
- 所見の欄は機械が検めない。orchestrator が読むときに欠けを確かめる。
- 知らせの 1 行は持ち主が打つ。
