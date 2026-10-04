---
name: consult
description: 相談の窓（判断の記録 ADR-29）を開く手引き。席の文脈を汚さない自由な調べと検証の場を、窓ごとの作業場と囲いの中で起こす。形は話す窓（持ち主が端末で直に話す）と問う窓（席が強い model に問い、所見を受ける）。持ち主が相談の窓を開くと言った時、board の相談の頼みが届いた時、席が自分で強い model に問いたい時に使う。
argument-hint: "[題（bead の id か自由な題）] [talk|ask] [model]"
---

# 相談の窓を開く（consult）

相談の窓は、席の文脈を汚さないために分けた、自由な調べと検証の場である（判断の記録 ADR-29 決定 (3)）。窓は作業場の dir の中で、囲い（基本ソフトの sandbox）の下で動き、作業場の外と repo へは書けない。結果は所見として席へ届き、席が処分を宣言する。

## 決まり

- **命令は tzw だけで撃つ。** tzw は tsuzuri の plugin の bin に在り、Bash の道具の PATH に載る。tzw は plugin と同じ checkout の build の tz か、無ければ PATH の tz を引き、plugin の版を tz に渡す。tz を直に撃たない（版の照らしが効かず open が断る）。
- **引数は省ける。** 渡された引数 $ARGUMENTS の 1 つ目は題（bead の id か自由な題）、2 つ目は形（talk か ask）、3 つ目は model。省けば題なし・話す窓（talk）・既定の model と念入りさ（fable・xhigh）。
- **起こし手を正しく書く。** 持ち主がチャットで開くと言った時は --by chat と --said（その発話の UTC の分の字・形 20261003T1412Z）、board の相談の頼みは --by button と --request（頼みの id）、席が自分で開く時は --by seat。
- **規則の行 R-38。** 席が自分で開く問う窓は、同時に 1 本・1 日（UTC）3 本までである。越える時は launch が断るので、決定の画面で持ち主に問う。持ち主の言葉で開く窓は数えない。
- **相談の窓の会話と所見は承認にならない。** 所見はそのまま採らず、所見ごとに処分を宣言する（下の 4）。窓は repo と台帳を書かない。
- **話す窓で持ち主が ! で打つ命令は囲いの外で走る。** 話す窓を開いた時に、このことを持ち主に一言伝える。
- **落ちたら止まる。** 命令が rc 1 か rc 2 で落ちたら、標準エラーの断りの 1 行をそのまま持ち主に見せて止まる。file を直に書く・別の旗で撃ち直すといった回り道をしない。

## 1 窓を用意する

```
tzw consult open --topic <題> --form talk --by chat --said <分>
```

- 題を省くなら --topic を書かない。問う窓は --form ask、model と念入りさは --model と --effort。
- board の頼みは `tzw consult open --request <頼みの id> --by button`（形と model と題は頼みの行から読む）。
- 出力の 1 行目は「窓 cw<n> を用意した」、2 行目は作業場の絶対 path。

## 2 窓を起こす

```
tzw consult launch <窓の id>
```

- 話す窓は席の tmux の session に名 consult-cw<n> の窓を開いて戻る（持ち主の見ている窓は替えない）。持ち主にその窓で話すよう伝える。
- 問う窓は所見を出して終わるまで待つので、Bash の道具の背景（run_in_background）で撃つ。終わりに所見の 1 行が出る。
- 所見の無いまま止まった問う窓は `tzw consult launch <窓の id> --again` で撃ち直す。

## 3 見張りを置く

```
tzw consult watch
```

- Bash の道具の背景（run_in_background・timeout は 7200000）で常に 1 本置く。所見か頼みが届くと「相談:」で始まる 1 行を出して終わる。
- 1 行で起きたら、その件を受けてから同じ命令で置き直す。2 本目は「見張りはもう居る」で終わる。

## 4 所見を受けて処分する

```
tzw consult show <所見の id> --via 見張り
tzw consult dispose <所見の id> --verdict 一部採る --reason <理由> --keep <添え物の path> --drop <添え物の path>
```

- --via は受けた経路（見張り・完了・hook・一覧）。show が台帳に所見と受けの行を書く。
- 採否は 採る・一部採る・採らない。添え物の全部に --keep か --drop を 1 度ずつ書く。保存した添え物は repo の docs/consult/kept/ の下に写るので、席が commit する。

## 5 一覧と閉じ

```
tzw consult list
tzw consult close <窓の id> --by chat
```

- 一覧は窓・処分の無い所見・受けの無い頼み・席の問う窓の今日の数と今の数を出す。見張りが無い間の拾いにも使う（受けは show --via 一覧）。
- 窓は持ち主が閉じる。持ち主が閉じると言った時だけ close を撃つ（起こし手は言った者）。
