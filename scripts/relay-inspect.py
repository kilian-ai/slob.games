#!/usr/bin/env python3
"""
relay-inspect.py — show everything stored in the slob.games relay

Usage:
  python3 scripts/relay-inspect.py
  python3 scripts/relay-inspect.py --token <jwt>   # show your own drafts too
"""

import sys
import json
import ssl
import urllib.request
from datetime import datetime

RELAY = "https://relay.slob.games"
UA    = {"User-Agent": "Mozilla/5.0"}
CTX   = ssl.create_default_context()

def fetch(path, token=None):
    headers = dict(UA)
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(f"{RELAY}{path}", headers=headers)
    try:
        with urllib.request.urlopen(req, context=CTX, timeout=10) as r:
            return json.loads(r.read()), r.status
    except urllib.error.HTTPError as e:
        return json.loads(e.read() or b"{}"), e.code
    except Exception as e:
        return {"error": str(e)}, 0

def fmt_size(n):
    n = int(n or 0)
    if n >= 1024*1024: return f"{n/1024/1024:.1f} MB"
    if n >= 1024:      return f"{n/1024:.1f} KB"
    return f"{n} B"

def fmt_date(s):
    try:
        dt = datetime.fromisoformat(str(s).replace("Z", "+00:00"))
        return dt.strftime("%Y-%m-%d %H:%M")
    except Exception:
        return str(s or "")

token = None
if "--token" in sys.argv:
    idx = sys.argv.index("--token")
    if idx + 1 < len(sys.argv):
        token = sys.argv[idx + 1]

print("=" * 60)
print(f"  slob.games relay: {RELAY}")
print("=" * 60)

# ── Published games ──────────────────────────────────────────
print("\n📦 PUBLISHED GAMES  (scope=external, published=true)")
games, status = fetch("/sync/games")
if isinstance(games, list):
    if not games:
        print("  (none)")
    for g in games:
        hs = f"  hs={g.get('highscore', '—')}" if g.get("highscore") else ""
        print(f"  [{g.get('content_hash','?')[:8]}] {g.get('name','?')!r:<30} "
              f"{fmt_size(g.get('size',0)):>8}  {fmt_date(g.get('updated',''))}{hs}")
else:
    print(f"  Error ({status}): {games}")

# ── High scores ──────────────────────────────────────────────
print("\n🏆 HIGH SCORES")
scores, status = fetch("/sync/scores")
if isinstance(scores, list):
    if not scores:
        print("  (none)")
    for s in scores:
        print(f"  game={s.get('game_hash','?')[:8]}  score={s.get('score','?'):<8} "
              f"player={s.get('player','—')!r:<16} {fmt_date(s.get('updated',''))}")
else:
    print(f"  Error ({status}): {scores}")

# ── My games (if token provided) ─────────────────────────────
if token:
    print("\n👤 MY GAMES  (/internal/games)")
    my, status = fetch("/internal/games", token=token)
    if isinstance(my, list):
        if not my:
            print("  (none)")
        for g in my:
            pub = "pub" if g.get("published") else "draft"
            print(f"  [{g.get('content_hash','?')[:8]}] [{pub}] {g.get('name','?')!r:<28} "
                  f"{fmt_size(g.get('size',0)):>8}  {fmt_date(g.get('updated',''))}")
    else:
        print(f"  Error ({status}): {my}")

# ── TOML manifest ────────────────────────────────────────────
print("\n📋 GAMES MANIFEST  (/sync/games.toml)")
toml_raw = None
req = urllib.request.Request(f"{RELAY}/sync/games.toml", headers=UA)
try:
    with urllib.request.urlopen(req, context=CTX, timeout=10) as r:
        toml_raw = r.read().decode()
except Exception as e:
    toml_raw = f"Error: {e}"
if toml_raw and toml_raw.strip():
    print(toml_raw[:1200])
    if len(toml_raw) > 1200:
        print("  [truncated]")
else:
    print("  (empty)")

print("\n" + "=" * 60)
print(f"  published={len(games) if isinstance(games,list) else '?'}  "
      f"scores={len(scores) if isinstance(scores,list) else '?'}")
print("=" * 60)
