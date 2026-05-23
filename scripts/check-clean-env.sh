#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Forge env precheck. Run before `pnpm tauri dev` to confirm the host is in
# a state where Forge MVP can take over ports 80/443/5353 and write its own
# /etc/resolver/test. Read-only — no sudo, no destructive actions.
#
# Usage: scripts/check-clean-env.sh

set -u

OK=0
WARN=0
ERR=0

ok()   { echo "  [✓] $1"; OK=$((OK+1)); }
warn() { echo "  [!] $1"; WARN=$((WARN+1)); }
err()  { echo "  [x] $1"; ERR=$((ERR+1)); }

section() { echo; echo "▶ $1"; }

section "Ports"
for port in 80 443 5353; do
    if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
        proc=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR==2 {print $1, "(pid "$2")"}')
        warn "Port $port is in use by $proc"
    else
        ok "Port $port is free"
    fi
done

section "DNS resolver"
RESOLVER="/etc/resolver/test"
if [ -f "$RESOLVER" ]; then
    expected=$'nameserver 127.0.0.1\nport 5353'
    actual=$(tr -d '\r' < "$RESOLVER")
    if [ "$actual" = "$expected" ] || [ "$actual" = "${expected}"$'\n' ]; then
        ok "$RESOLVER present and correct"
    else
        warn "$RESOLVER present but content differs (Forge will rewrite it)"
        echo "      actual content:"
        sed 's/^/        /' "$RESOLVER"
    fi
else
    ok "$RESOLVER absent — Forge wizard will create it"
fi

section "Competing app processes"
patterns=(
    "Herd.app/Contents"
    "de.beyondco.herd"
    "Local by Flywheel"
    "MAMP.app/Contents"
    "Valet"
)
hits=0
for pat in "${patterns[@]}"; do
    if pgrep -lf "$pat" >/dev/null 2>&1; then
        names=$(pgrep -lf "$pat" | head -3 | sed 's/^/        /')
        warn "Found process matching '$pat':"
        echo "$names"
        hits=$((hits+1))
    fi
done
if [ "$hits" -eq 0 ]; then
    ok "No competing dev environment app detected"
fi

section "Forge bundle directory"
ENGINES_DIR="$HOME/Library/Application Support/Forge/engines"
if [ -d "$ENGINES_DIR" ]; then
    bundles=$(find "$ENGINES_DIR" -mindepth 2 -maxdepth 2 -type d 2>/dev/null | sed "s|$ENGINES_DIR/|        |")
    if [ -n "$bundles" ]; then
        ok "Engines installed:"
        echo "$bundles"
    else
        ok "Engines dir exists but is empty"
    fi
else
    ok "No engines installed yet ($ENGINES_DIR absent)"
fi

echo
echo "Summary: $OK ok · $WARN warning · $ERR error"

if [ "$ERR" -gt 0 ]; then
    exit 1
elif [ "$WARN" -gt 0 ]; then
    exit 2
fi
exit 0
