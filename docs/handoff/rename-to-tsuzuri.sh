#!/usr/bin/env bash
# scribe3 → tsuzuri の改名（ADR-6）。既定は dry-run。RUN=1 で実行。OLD / NEW で向きを変えられる（戻し）。
set -u
OLD=${OLD:-scribe3}; NEW=${NEW:-tsuzuri}
P=$HOME/projects/local-projects; ST=$HOME/.local/state; ACC=$HOME/.claude-accounts
TS=$(date -u +%Y%m%dT%H%M%SZ)
run() { if [ "${RUN:-0}" = 1 ]; then "$@"; else echo "  dry: $*"; fi; }
step() { printf 'rename: %s %s\n' "$1" "$2"; }

# 0) 前提: 旧 dir で動く claude の席が無い・git が clean
if pgrep -f "claude.*" -a 2>/dev/null | grep -q "cwd=$P/$OLD" ; then :; fi
if [ -d "$P/$OLD" ] && ls -l /proc/[0-9]*/cwd 2>/dev/null | grep -q " $P/$OLD\$"; then
  step precheck "failed:席か process が $P/$OLD を cwd にしている（閉じてから）"; exit 1
fi
if [ -d "$P/$OLD" ] && ! git -C "$P/$OLD" diff --quiet; then step precheck "failed:$P/$OLD が dirty"; exit 1; fi
step precheck ok

# 1) repo と作業場と site の dir
for d in "" -spikes -site; do
  if [ -d "$P/$OLD$d" ] && [ ! -e "$P/$NEW$d" ]; then run mv "$P/$OLD$d" "$P/$NEW$d"; step "dir$d" ok
  elif [ -e "$P/$NEW$d" ]; then step "dir$d" skip; else step "dir$d" "skip:無し"; fi
done

# 2) state dir と git の設定
if [ -d "$ST/scribe2-v2-state-$OLD" ] && [ ! -e "$ST/scribe2-v2-state-$NEW" ]; then
  run mv "$ST/scribe2-v2-state-$OLD" "$ST/scribe2-v2-state-$NEW"; step state-dir ok
else step state-dir skip; fi
R=$P/$NEW; [ -d "$R" ] || R=$P/$OLD
if [ "$(git -C "$R" config --local scribe2.statedir 2>/dev/null)" != "$ST/scribe2-v2-state-$NEW" ]; then
  run git -C "$R" config --local scribe2.statedir "$ST/scribe2-v2-state-$NEW"; step git-config ok
else step git-config skip; fi

# 3) host の面の anchors（全 state dir の host.toml・.bak を残す）
for f in "$ST"/scribe2-v2-state*/host.toml; do
  if grep -q "$P/$OLD\"" "$f"; then
    run cp "$f" "$f.bak-rename-$TS"; run sed -i "s#$P/$OLD\"#$P/$NEW\"#g" "$f"; step "anchors:$(basename "$(dirname "$f")")" ok
  else step "anchors:$(basename "$(dirname "$f")")" skip; fi
done

# 4) 口座ごとの projects の dir（会話記録と memory）
for a in "$ACC"/*/projects; do
  o="$a/-home-shuu5-projects-local-projects-$OLD"; n="$a/-home-shuu5-projects-local-projects-$NEW"
  if [ -d "$o" ] && [ ! -e "$n" ]; then run mv "$o" "$n"; step "projects:$(basename "$(dirname "$a")")" ok
  else step "projects:$(basename "$(dirname "$a")")" skip; fi
done

# 5) 作業場の文中の旧 path
if [ -d "$P/$NEW-spikes" ]; then
  for f in README.md hub/index.html server/report.md; do
    ff="$P/$NEW-spikes/$f"; [ -f "$ff" ] || continue
    if grep -q "local-projects/$OLD" "$ff"; then run sed -i "s#local-projects/$OLD#local-projects/$NEW#g" "$ff"; step "spikes-text:$f" ok; else step "spikes-text:$f" skip; fi
  done
fi

echo "next= scribe2 席: seat tick uninstall（旧 unit）→ seat register/tick install/launch（新 state dir・target s3:design）。新 s3 席: 配信と試作 server の起こし直し・bd init"
