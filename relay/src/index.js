/**
 * traits.build relay — Cloudflare Worker + Durable Objects
 *
 * One RelaySession DO per pairing code. The DO holds all in-flight state
 * in memory, so long-poll coordination is instant and zero-latency.
 *
 * One GameRoom DO (global) for automatic game sync between all clients.
 * Games are stored in SQLite and synced via WebSocket.
 *
 * Routes:
 *   GET  /health
 *   POST /relay/register      → { code }
 *   POST /relay/connect       { code } → { token, code }   (HMAC-signed token)
 *   GET  /relay/poll?code=    → {id, path, args} when a call arrives, 204 on timeout
 *   POST /relay/call          { code|token, path, args } → { result, error }
 *   POST /relay/respond       { code, id, result }
 *   GET  /relay/status?code=  → { active, age_seconds, code }
 *   GET  /relay/status?token= → same, validated from signed token
 *   GET  /sync                → WebSocket upgrade → GameRoom (automatic game sync)
 *
 * Signed tokens (requires RELAY_SECRET worker secret):
 *   After a client enters the 4-char pairing code, call /relay/connect to get a
 *   HMAC-SHA256 signed token { code, relay, iat, exp }. The token is stateless —
 *   the relay verifies its signature without any persistent store. Clients save the
 *   token in localStorage and use it for all future status checks and calls without
 *   re-entering the pairing code.
 *
 *   Setup:  npx wrangler secret put RELAY_SECRET
 *           (generate with: openssl rand -base64 32)
 */

// ── HMAC-SHA256 token signing (Web Crypto) ────────────────────────────────────

async function _getHmacKey(secret) {
  return crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify'],
  );
}

const TOKEN_TTL_SECS = 86400 * 30; // 30 days

async function signToken(code, relayOrigin, secret) {
  const payload = {
    code,
    relay: relayOrigin,
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + TOKEN_TTL_SECS,
  };
  const payloadBytes = new TextEncoder().encode(JSON.stringify(payload));
  const key = await _getHmacKey(secret);
  const sig = await crypto.subtle.sign('HMAC', key, payloadBytes);
  const payloadB64 = btoa(JSON.stringify(payload));
  const sigB64 = btoa(String.fromCharCode(...new Uint8Array(sig)));
  return `${payloadB64}.${sigB64}`;
}

async function verifyToken(token, secret) {
  try {
    const dot = token.lastIndexOf('.');
    if (dot === -1) return null;
    const payloadB64 = token.slice(0, dot);
    const sigB64    = token.slice(dot + 1);
    const payload   = JSON.parse(atob(payloadB64));
    // Check expiry client-side before hitting crypto
    if (!payload.exp || Date.now() / 1000 > payload.exp) return null;
    const key       = await _getHmacKey(secret);
    const sigBytes  = Uint8Array.from(atob(sigB64), c => c.charCodeAt(0));
    const dataBytes = new TextEncoder().encode(JSON.stringify(payload));
    const valid     = await crypto.subtle.verify('HMAC', key, sigBytes, dataBytes);
    return valid ? payload : null;
  } catch(_) { return null; }
}

// ── CORS ─────────────────────────────────────────────────────────────────────

function cors() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type,Authorization",
  };
}

function json(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...cors() },
  });
}

// ── Pairing code generation ───────────────────────────────────────────────────

const CODE_CHARS = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // unambiguous chars

function generateCode() {
  const buf = new Uint8Array(4);
  crypto.getRandomValues(buf);
  return Array.from(buf, (v) => CODE_CHARS[v % CODE_CHARS.length]).join("");
}

function normalizeCode(code) {
  if (!code) return null;
  const normalized = String(code).trim().toUpperCase();
  return /^[A-Z0-9]{4}$/.test(normalized) ? normalized : null;
}

// ── Durable Object: RelaySession ──────────────────────────────────────────────
//
// One instance per pairing code (created via idFromName(code)).
// All relay coordination happens in-memory — no KV writes needed.
//
// In-memory state:
//   pendingRequest  — a request the Mac hasn't picked up yet (phone arrived first)
//   pollResolve     — the Mac's waiting resolve() (poller arrived first)
//   resultResolvers — Map<id, resolve> for open phone /relay/call Promises

export class RelaySession {
  constructor(state, env) {
    this.created = Date.now();
    this.lastPollAt = null;     // timestamp of last /poll from Mac
    this.pendingRequest = null; // { id, path, args }
    this.pollResolve = null;    // fn(request) — Mac's waiting resolver
    this.resultResolvers = new Map(); // id → fn(result)
  }

  async fetch(request) {
    const url = new URL(request.url);

    switch (url.pathname) {
      case "/register": return this._register();
      case "/poll":    return this._poll();
      case "/call":    return this._call(request);
      case "/respond": return this._respond(request);
      case "/status":  return this._status();
      default:         return new Response("not found", { status: 404 });
    }
  }

  _register() {
    this.created = Date.now();
    this.lastPollAt = null;
    this.pendingRequest = null;
    this.pollResolve = null;
    this.resultResolvers.clear();
    return json({ ok: true });
  }

  // Mac long-polls here. Resolves immediately if a request is already waiting,
  // otherwise suspends for up to 29s then returns 204 (Mac should re-poll).
  _poll() {
    this.lastPollAt = Date.now(); // track liveness for _status()
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        this.pollResolve = null;
        resolve(new Response(null, { status: 204, headers: cors() }));
      }, 29_000);

      const deliver = (req) => {
        clearTimeout(timer);
        this.pollResolve = null;
        resolve(json(req));
      };

      if (this.pendingRequest) {
        // A call was already queued before Mac reconnected — deliver immediately.
        const req = this.pendingRequest;
        this.pendingRequest = null;
        deliver(req);
      } else {
        this.pollResolve = deliver;
      }
    });
  }

  // Phone calls a trait via relay. Suspends until Mac responds or 60s timeout.
  async _call(request) {
    const body = await request.json();
    const id = crypto.randomUUID();
    const req = { id, path: body.path, args: body.args ?? [] };

    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        this.resultResolvers.delete(id);
        resolve(json({ error: "Relay timeout (60s)", result: null }, 504));
      }, 60_000);

      this.resultResolvers.set(id, (result) => {
        clearTimeout(timer);
        resolve(json(result));
      });

      // Wake the Mac if it's polling, otherwise queue the request.
      if (this.pollResolve) {
        this.pollResolve(req);
      } else {
        this.pendingRequest = req;
      }
    });
  }

  // Mac sends back the result for a previous request.
  async _respond(request) {
    const body = await request.json();
    const resolve = this.resultResolvers.get(body.id);
    if (!resolve) {
      return json({ error: "No pending request with that id" }, 404);
    }
    this.resultResolvers.delete(body.id);
    resolve(body); // body contains { id, result, error? }
    return json({ ok: true });
  }

  _status() {
    // Mac is considered connected if it's currently in a poll OR polled within the
    // last 35s (29s poll timeout + 6s grace for reconnect).
    const macConnected =
      this.pollResolve !== null ||
      (this.lastPollAt !== null && Date.now() - this.lastPollAt < 35_000);
    return json({
      active: macConnected,
      age_seconds: Math.floor((Date.now() - this.created) / 1000),
    });
  }
}

// ── SHA-256 hash helper (first 16 hex chars) ──────────────────────────────────

async function sha256hex16(str) {
  const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(str || ''));
  return [...new Uint8Array(buf)].map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 16);
}

// ── Durable Object: GameRoom ──────────────────────────────────────────────────
//
// Single global instance for automatic game sync across all slob.games clients.
// Games stored in SQLite, synced via WebSocket with hibernation.
//
// Protocol:
//   connect     → server sends { type:"catalog", hashes:["abc...","def...",...] }
//   client→srv  { type:"need", hashes:[...] }       → server sends { type:"games", games:[...] }
//   client→srv  { type:"push", games:[{name,content,content_hash},...] }
//                   → server stores, broadcasts { type:"sync", games:[...] } to others
//                   → server sends { type:"ack", added:N } to sender

const MAX_GAME_SIZE = 256 * 1024; // 256KB per game
const MAX_TOTAL_GAMES = 500;

export class GameRoom {
  constructor(state, env) {
    this.state = state;
    this.sql = state.storage.sql;
    this.sql.exec(`CREATE TABLE IF NOT EXISTS games (
      content_hash TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      content TEXT NOT NULL,
      updated TEXT NOT NULL,
      size INTEGER NOT NULL
    )`);
    this.sql.exec(`CREATE TABLE IF NOT EXISTS scores (
      game_hash TEXT PRIMARY KEY,
      score INTEGER NOT NULL,
      player TEXT NOT NULL DEFAULT '',
      updated TEXT NOT NULL
    )`);
  }

  async fetch(request) {
    const url = new URL(request.url);

    // ── REST API (non-WebSocket) ──
    if (request.headers.get("Upgrade") !== "websocket") {
      // GET /games — list all games (name, hash, size, updated)
      if (url.pathname === "/games" && request.method === "GET") {
        const rows = this.sql.exec(
          "SELECT content_hash, name, size, updated FROM games ORDER BY updated DESC"
        ).toArray();
        return json(rows);
      }

      // GET /game/:hash — full HTML content of one game
      if (url.pathname.startsWith("/game/") && request.method === "GET") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const rows = this.sql.exec(
          "SELECT content_hash, name, content, updated FROM games WHERE content_hash = ?", hash
        ).toArray();
        if (rows.length === 0) return json({ error: "not found" }, 404);
        return json(rows[0]);
      }

      // PUT /game/:hash — update game content, broadcast to all connected clients
      if (url.pathname.startsWith("/game/") && request.method === "PUT") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const existing = this.sql.exec(
          "SELECT content_hash, name FROM games WHERE content_hash = ?", hash
        ).toArray();
        if (existing.length === 0) return json({ error: "not found" }, 404);

        const body = await request.json();
        const content = body.content;
        if (!content || typeof content !== 'string') return json({ error: "missing content" }, 400);
        if (content.length > 256 * 1024) return json({ error: "too large" }, 413);

        // Compute new hash
        const newHash = await sha256hex16(content);
        const name = body.name || existing[0].name;
        const updated = new Date().toISOString();

        // Delete old row, insert with new hash
        this.sql.exec("DELETE FROM games WHERE content_hash = ?", hash);
        this.sql.exec(
          "INSERT INTO games (content_hash, name, content, updated, size) VALUES (?, ?, ?, ?, ?)",
          newHash, name.slice(0, 100), content, updated, content.length
        );

        // Broadcast updated game to all connected WebSocket clients
        const msg = JSON.stringify({
          type: 'sync',
          games: [{ content_hash: newHash, name: name.slice(0, 100), content, updated }]
        });
        for (const sock of this.state.getWebSockets()) {
          try { sock.send(msg); } catch (_) {}
        }

        return json({ ok: true, old_hash: hash, content_hash: newHash, name, size: content.length });
      }

      // GET /scores — all high scores
      if (url.pathname === "/scores" && request.method === "GET") {
        const rows = this.sql.exec("SELECT game_hash, score, player, updated FROM scores").toArray();
        return json(rows);
      }

      return json({ error: "WebSocket upgrade required or use REST endpoints" }, 426);
    }

    // ── WebSocket upgrade ──
    const pair = new WebSocketPair();
    this.state.acceptWebSocket(pair[1]);

    // Send catalog (hashes only) to the new client
    const rows = this.sql.exec("SELECT content_hash FROM games").toArray();
    const hashes = rows.map(r => r.content_hash);
    pair[1].send(JSON.stringify({ type: 'catalog', hashes }));

    // Send all high scores to the new client
    const scoreRows = this.sql.exec("SELECT game_hash, score, player FROM scores").toArray();
    if (scoreRows.length > 0) {
      pair[1].send(JSON.stringify({ type: 'scores', scores: scoreRows }));
    }

    return new Response(null, { status: 101, webSocket: pair[0] });
  }

  async webSocketMessage(ws, message) {
    let data;
    try { data = JSON.parse(message); } catch (_) { return; }

    switch (data.type) {
      case 'need': {
        // Client wants full content for specific hashes
        if (!Array.isArray(data.hashes) || data.hashes.length === 0) return;
        // Limit to 50 at a time
        const wanted = data.hashes.slice(0, 50);
        const ph = wanted.map(() => '?').join(',');
        const rows = this.sql.exec(
          `SELECT content_hash, name, content, updated FROM games WHERE content_hash IN (${ph})`,
          ...wanted
        ).toArray();
        if (rows.length > 0) {
          ws.send(JSON.stringify({ type: 'games', games: rows }));
        }
        break;
      }

      case 'push': {
        // Client pushes new games
        if (!Array.isArray(data.games) || data.games.length === 0) return;
        const countRow = this.sql.exec("SELECT COUNT(*) as c FROM games").toArray();
        let count = countRow[0]?.c || 0;
        const added = [];

        for (const g of data.games.slice(0, 20)) { // max 20 per push
          if (!g.content || typeof g.content !== 'string') continue;
          if (!g.name || typeof g.name !== 'string') continue;
          if (!g.content_hash || typeof g.content_hash !== 'string') continue;
          if (g.content.length > MAX_GAME_SIZE) continue;
          if (g.content.length === 0) continue;
          if (count >= MAX_TOTAL_GAMES) break;

          // Verify content hash matches (don't trust client blindly)
          const verified = await sha256hex16(g.content);
          if (verified !== g.content_hash) continue;

          // Check if already stored
          const exists = this.sql.exec(
            "SELECT 1 FROM games WHERE content_hash = ?", g.content_hash
          ).toArray();
          if (exists.length > 0) continue;

          const updated = new Date().toISOString();
          this.sql.exec(
            "INSERT INTO games (content_hash, name, content, updated, size) VALUES (?, ?, ?, ?, ?)",
            g.content_hash, g.name.slice(0, 100), g.content, updated, g.content.length
          );
          added.push({ content_hash: g.content_hash, name: g.name.slice(0, 100), content: g.content, updated });
          count++;
        }

        // Broadcast new games to all OTHER connected clients
        if (added.length > 0) {
          const msg = JSON.stringify({ type: 'sync', games: added });
          for (const sock of this.state.getWebSockets()) {
            if (sock !== ws) {
              try { sock.send(msg); } catch (_) {}
            }
          }
        }
        ws.send(JSON.stringify({ type: 'ack', added: added.length }));
        break;
      }

      case 'score': {
        // Client reports a high score: { game_hash, score, player? }
        if (!data.game_hash || typeof data.game_hash !== 'string') return;
        const incoming = Math.floor(Number(data.score));
        if (!Number.isFinite(incoming) || incoming < 0) return;
        const player = (typeof data.player === 'string' ? data.player : '').slice(0, 50);

        // Only store if it's higher than existing
        const existing = this.sql.exec(
          "SELECT score FROM scores WHERE game_hash = ?", data.game_hash
        ).toArray();

        if (existing.length > 0 && existing[0].score >= incoming) {
          // Not a new high score — just send back the current best
          ws.send(JSON.stringify({
            type: 'score-update',
            game_hash: data.game_hash,
            score: existing[0].score
          }));
          return;
        }

        const updated = new Date().toISOString();
        this.sql.exec(
          `INSERT INTO scores (game_hash, score, player, updated) VALUES (?, ?, ?, ?)
           ON CONFLICT(game_hash) DO UPDATE SET score=excluded.score, player=excluded.player, updated=excluded.updated`,
          data.game_hash, incoming, player, updated
        );

        // Broadcast new high score to ALL clients (including sender)
        const msg = JSON.stringify({
          type: 'score-update',
          game_hash: data.game_hash,
          score: incoming,
          player
        });
        for (const sock of this.state.getWebSockets()) {
          try { sock.send(msg); } catch (_) {}
        }
        break;
      }
    }
  }

  webSocketClose(ws, code, reason) {}
  webSocketError(ws, error) {}
}

// ── Main Worker ───────────────────────────────────────────────────────────────

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: cors() });
    }

    if (url.pathname === "/health") {
      return new Response("ok", { headers: cors() });
    }

    // POST /relay/register
    if (url.pathname === "/relay/register" && request.method === "POST") {
      let preferred = null;
      try {
        const text = await request.text();
        if (text) {
          const body = JSON.parse(text);
          preferred = normalizeCode(body.code);
        }
      } catch (_) {
      }
      const code = preferred || generateCode();
      const stub = env.RELAY.get(env.RELAY.idFromName(code));
      await stub.fetch(new Request("http://do/register", { method: "POST" }));
      return json({ code });
    }

    // GET /relay/poll?code=XXXX
    if (url.pathname === "/relay/poll" && request.method === "GET") {
      const code = url.searchParams.get("code");
      if (!code) return json({ error: "missing code" }, 400);
      return env.RELAY.get(env.RELAY.idFromName(code)).fetch(
        new Request("http://do/poll")
      );
    }

    // POST /relay/connect  { code } → { token, code }  (issues signed token)
    if (url.pathname === "/relay/connect" && request.method === "POST") {
      if (!env.RELAY_SECRET) return json({ error: "Token signing not configured on relay" }, 503);
      const body = await request.json().catch(() => ({}));
      const code = normalizeCode(body.code);
      if (!code) return json({ error: "invalid code" }, 400);
      // Verify Mac is actually polling before issuing a token
      const stub = env.RELAY.get(env.RELAY.idFromName(code));
      const statusData = await stub.fetch(new Request("http://do/status")).then(r => r.json());
      if (!statusData.active) return json({ error: "No helper connected with this code" }, 404);
      const token = await signToken(code, new URL(request.url).origin, env.RELAY_SECRET);
      return json({ ok: true, token, code });
    }

    // POST /relay/call  { code|token, path, args }
    if (url.pathname === "/relay/call" && request.method === "POST") {
      const body = await request.json();
      let code = normalizeCode(body.code);
      // Accept signed token in place of code
      if (!code && body.token && env.RELAY_SECRET) {
        const payload = await verifyToken(body.token, env.RELAY_SECRET);
        if (!payload) return json({ error: "Invalid or expired relay token" }, 401);
        code = payload.code;
      }
      if (!code) return json({ error: "missing code or token" }, 400);
      return env.RELAY.get(env.RELAY.idFromName(code)).fetch(
        new Request("http://do/call", {
          method: "POST",
          body: JSON.stringify({ ...body, code }),
          headers: { "Content-Type": "application/json" },
        })
      );
    }

    // POST /relay/respond  { code, id, result }
    if (url.pathname === "/relay/respond" && request.method === "POST") {
      const body = await request.json();
      if (!body.code) return json({ error: "missing code" }, 400);
      return env.RELAY.get(env.RELAY.idFromName(body.code)).fetch(
        new Request("http://do/respond", {
          method: "POST",
          body: JSON.stringify(body),
          headers: { "Content-Type": "application/json" },
        })
      );
    }

    // GET /relay/status?code=XXXX  or  ?token=XXX
    if (url.pathname === "/relay/status" && request.method === "GET") {
      let code = url.searchParams.get("code");
      // Accept signed token in place of code
      const token = url.searchParams.get("token");
      if (!code && token && env.RELAY_SECRET) {
        const payload = await verifyToken(token, env.RELAY_SECRET);
        if (!payload) return json({ error: "Invalid or expired relay token" }, 401);
        code = payload.code;
      }
      if (!code) return json({ error: "missing code or token" }, 400);
      const stub = env.RELAY.get(env.RELAY.idFromName(code));
      const res  = await stub.fetch(new Request("http://do/status"));
      const data = await res.json();
      return json({ ...data, code }); // always include resolved code in response
    }

    // /sync routes → global GameRoom
    if (url.pathname === "/sync" || url.pathname.startsWith("/sync/")) {
      const room = env.GAME_ROOM.get(env.GAME_ROOM.idFromName("global"));

      // WebSocket upgrade: /sync
      if (url.pathname === "/sync") {
        if (request.headers.get("Upgrade") !== "websocket") {
          return json({ error: "WebSocket upgrade required" }, 426);
        }
        return room.fetch(request);
      }

      // REST: /sync/games → DO /games
      // REST: /sync/game/:hash → DO /game/:hash
      // REST: /sync/scores → DO /scores
      const doPath = url.pathname.slice(5); // strip '/sync'
      const doUrl = new URL(request.url);
      doUrl.pathname = doPath;
      return room.fetch(new Request(doUrl.toString(), request));
    }

    return json({ error: "not found" }, 404);
  },
};
