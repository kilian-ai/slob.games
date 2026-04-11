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

const USER_TOKEN_TTL_SECS = 86400 * 30; // 30 days

async function signUserToken(username, relayOrigin, secret) {
  const payload = {
    sub: username,
    relay: relayOrigin,
    typ: 'user',
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + USER_TOKEN_TTL_SECS,
  };
  const payloadBytes = new TextEncoder().encode(JSON.stringify(payload));
  const key = await _getHmacKey(secret);
  const sig = await crypto.subtle.sign('HMAC', key, payloadBytes);
  const payloadB64 = btoa(JSON.stringify(payload));
  const sigB64 = btoa(String.fromCharCode(...new Uint8Array(sig)));
  return `${payloadB64}.${sigB64}`;
}

async function verifyUserToken(token, secret) {
  try {
    const dot = token.lastIndexOf('.');
    if (dot === -1) return null;
    const payloadB64 = token.slice(0, dot);
    const sigB64 = token.slice(dot + 1);
    const payload = JSON.parse(atob(payloadB64));
    if (payload.typ !== 'user' || !payload.sub) return null;
    if (!payload.exp || Date.now() / 1000 > payload.exp) return null;
    const key = await _getHmacKey(secret);
    const sigBytes = Uint8Array.from(atob(sigB64), c => c.charCodeAt(0));
    const dataBytes = new TextEncoder().encode(JSON.stringify(payload));
    const valid = await crypto.subtle.verify('HMAC', key, sigBytes, dataBytes);
    return valid ? payload : null;
  } catch (_) {
    return null;
  }
}

function normalizeSlug(input, fallback = 'game') {
  const s = String(input || '')
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return s || fallback;
}

async function sha256hex(str) {
  const data = new TextEncoder().encode(str);
  const hash = await crypto.subtle.digest("SHA-256", data);
  const bytes = new Uint8Array(hash);
  return Array.from(bytes).map(b => b.toString(16).padStart(2, "0")).join("");
}

// Legacy password hash (SHA-256) — used only for migration of pre-PBKDF2 accounts
async function legacyPasswordHash(username, password, secret) {
  return sha256hex(`${username}:${password}:${secret || ''}`);
}

// PBKDF2 password hashing — 100k iterations, SHA-256, 256-bit output
async function pbkdf2Hash(password, salt) {
  const enc = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    'raw', enc.encode(password), 'PBKDF2', false, ['deriveBits']
  );
  const bits = await crypto.subtle.deriveBits(
    { name: 'PBKDF2', salt: enc.encode(salt), iterations: 100000, hash: 'SHA-256' },
    keyMaterial, 256
  );
  return Array.from(new Uint8Array(bits)).map(b => b.toString(16).padStart(2, '0')).join('');
}

function generateSalt() {
  const buf = new Uint8Array(16);
  crypto.getRandomValues(buf);
  return Array.from(buf).map(b => b.toString(16).padStart(2, '0')).join('');
}

// Rate limiting constants for auth endpoints
const MAX_AUTH_ATTEMPTS = 5;
const AUTH_COOLDOWN_MS = 60_000; // 60 seconds

// ── CORS ─────────────────────────────────────────────────────────────────────

function cors() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET,POST,PUT,DELETE,OPTIONS",
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
const DEFAULT_EXTERNAL_POOL_SIZE = 64;

export class GameRoom {
  constructor(state, env) {
    this.state = state;
    this.env = env;
    this.sql = state.storage.sql;
    this.sql.exec(`CREATE TABLE IF NOT EXISTS games (
      content_hash TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      content TEXT NOT NULL,
      updated TEXT NOT NULL,
      size INTEGER NOT NULL
    )`);
    const gameCols = this.sql.exec("PRAGMA table_info(games)").toArray().map(r => r.name);
    if (!gameCols.includes('owner')) this.sql.exec("ALTER TABLE games ADD COLUMN owner TEXT NOT NULL DEFAULT 'public'");
    if (!gameCols.includes('game_id')) this.sql.exec("ALTER TABLE games ADD COLUMN game_id TEXT NOT NULL DEFAULT ''");
    if (!gameCols.includes('scope')) this.sql.exec("ALTER TABLE games ADD COLUMN scope TEXT NOT NULL DEFAULT 'external'");
    if (!gameCols.includes('version')) this.sql.exec("ALTER TABLE games ADD COLUMN version TEXT NOT NULL DEFAULT ''");
    if (!gameCols.includes('checksum')) this.sql.exec("ALTER TABLE games ADD COLUMN checksum TEXT NOT NULL DEFAULT ''");

    this.sql.exec(`CREATE TABLE IF NOT EXISTS internal_games (
      owner TEXT NOT NULL,
      game_id TEXT NOT NULL,
      name TEXT NOT NULL,
      content TEXT NOT NULL,
      content_hash TEXT NOT NULL,
      checksum TEXT NOT NULL,
      version TEXT NOT NULL,
      forked_from_hash TEXT,
      updated TEXT NOT NULL,
      size INTEGER NOT NULL,
      PRIMARY KEY (owner, game_id)
    )`);
    this.sql.exec("CREATE INDEX IF NOT EXISTS idx_internal_games_owner ON internal_games(owner)");

    this.sql.exec(`CREATE TABLE IF NOT EXISTS users (
      username TEXT PRIMARY KEY,
      email TEXT UNIQUE NOT NULL,
      password_hash TEXT NOT NULL,
      created TEXT NOT NULL
    )`);
    const userCols = this.sql.exec("PRAGMA table_info(users)").toArray().map(r => r.name);
    if (!userCols.includes('salt')) this.sql.exec("ALTER TABLE users ADD COLUMN salt TEXT NOT NULL DEFAULT ''");
    if (!userCols.includes('role')) this.sql.exec("ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user'");
    if (!userCols.includes('last_login')) this.sql.exec("ALTER TABLE users ADD COLUMN last_login TEXT NOT NULL DEFAULT ''");

    // Seed kilian-ai as admin if exists
    this.sql.exec("UPDATE users SET role = 'admin' WHERE username = 'kilian-ai' AND role != 'admin'");

    // In-memory rate limiting for auth endpoints
    this.authAttempts = new Map(); // username → { count, lastAttempt }
    this.sql.exec(`CREATE TABLE IF NOT EXISTS scores (
      game_hash TEXT PRIMARY KEY,
      score INTEGER NOT NULL,
      player TEXT NOT NULL DEFAULT '',
      updated TEXT NOT NULL
    )`);
  }

  _trackFailedAuth(username) {
    const attempt = this.authAttempts.get(username) || { count: 0, lastAttempt: 0 };
    // Reset counter if cooldown has expired
    if (Date.now() - attempt.lastAttempt >= AUTH_COOLDOWN_MS) {
      attempt.count = 0;
    }
    attempt.count++;
    attempt.lastAttempt = Date.now();
    this.authAttempts.set(username, attempt);
  }

  async authUser(request) {
    const auth = request.headers.get('Authorization') || '';
    const bearer = auth.toLowerCase().startsWith('bearer ') ? auth.slice(7).trim() : '';
    const headerToken = request.headers.get('X-Slob-Token') || '';
    const url = new URL(request.url);
    const token = bearer || headerToken || url.searchParams.get('token') || '';
    if (!token || !this.env.RELAY_SECRET) return null;
    const payload = await verifyUserToken(token, this.env.RELAY_SECRET);
    return payload?.sub || null;
  }

  deriveGameId(name, explicit) {
    return normalizeSlug(explicit || name, 'untitled');
  }

  normalizeExternalGameRow(row) {
    const owner = normalizeSlug(row?.owner || 'public', 'public');
    const gameId = this.deriveGameId(row?.name || row?.content_hash || 'untitled', row?.game_id || '');
    return {
      ...row,
      owner,
      game_id: gameId,
      scope: row?.scope || 'external',
      version: row?.version || '',
      checksum: row?.checksum || row?.content_hash || '',
    };
  }

  getExternalPoolLimit() {
    const raw = Number(this.env?.EXTERNAL_POOL_SIZE || DEFAULT_EXTERNAL_POOL_SIZE);
    if (!Number.isFinite(raw) || raw < 1) return DEFAULT_EXTERNAL_POOL_SIZE;
    return Math.floor(raw);
  }

  trimExternalPool() {
    const keep = this.getExternalPoolLimit();
    const rows = this.sql.exec(
      "SELECT content_hash FROM games WHERE scope = 'external' ORDER BY updated DESC, rowid DESC"
    ).toArray();
    if (rows.length <= keep) return 0;
    const toDelete = rows.slice(keep);
    for (const row of toDelete) {
      this.sql.exec("DELETE FROM games WHERE content_hash = ?", row.content_hash);
    }
    return toDelete.length;
  }

  async fetch(request) {
    const url = new URL(request.url);

    // ── REST API (non-WebSocket) ──
    if (request.headers.get("Upgrade") !== "websocket") {
      // POST /auth/register — create user + issue token
      if (url.pathname === "/auth/register" && request.method === "POST") {
        if (!this.env.RELAY_SECRET) return json({ error: "RELAY_SECRET not configured" }, 503);
        const body = await request.json().catch(() => ({}));
        const username = normalizeSlug(body.username || '', '');
        const email = String(body.email || '').trim().toLowerCase();
        const password = String(body.password || '');
        if (!username || username.length < 3) return json({ error: "username must be at least 3 chars" }, 400);
        if (!/^\S+@\S+\.\S+$/.test(email)) return json({ error: "invalid email" }, 400);
        if (password.length < 6) return json({ error: "password must be at least 6 chars" }, 400);

        const exists = this.sql.exec("SELECT 1 FROM users WHERE username = ? OR email = ?", username, email).toArray();
        if (exists.length > 0) return json({ error: "username or email already exists" }, 409);

        const salt = generateSalt();
        const hashed = await pbkdf2Hash(password, salt);
        const created = new Date().toISOString();
        this.sql.exec(
          "INSERT INTO users (username, email, password_hash, salt, created) VALUES (?, ?, ?, ?, ?)",
          username, email, hashed, salt, created
        );
        const token = await signUserToken(username, new URL(request.url).origin, this.env.RELAY_SECRET);
        return json({ ok: true, username, token, role: 'user' });
      }
      // ── verify creds + issue token (with rate limiting) ──
      if (url.pathname === "/auth/login" && request.method === "POST") {
        if (!this.env.RELAY_SECRET) return json({ error: "RELAY_SECRET not configured" }, 503);
        const body = await request.json().catch(() => ({}));
        const username = normalizeSlug(body.username || '', '');
        const password = String(body.password || '');
        if (!username || !password) return json({ error: "username and password required" }, 400);

        // Rate limiting: block after MAX_AUTH_ATTEMPTS failures within cooldown window
        const attempt = this.authAttempts.get(username);
        if (attempt && attempt.count >= MAX_AUTH_ATTEMPTS && Date.now() - attempt.lastAttempt < AUTH_COOLDOWN_MS) {
          return json({ error: "too many attempts, try again later" }, 429);
        }

        const row = this.sql.exec(
          "SELECT username, password_hash, salt FROM users WHERE username = ?", username
        ).toArray()[0];
        if (!row) {
          this._trackFailedAuth(username);
          return json({ error: "invalid credentials" }, 401);
        }

        let valid = false;
        if (row.salt) {
          // PBKDF2 path
          const hashed = await pbkdf2Hash(password, row.salt);
          valid = (hashed === row.password_hash);
        } else {
          // Legacy SHA-256 path — migrate on success
          const hashed = await legacyPasswordHash(username, password, this.env.RELAY_SECRET);
          valid = (hashed === row.password_hash);
          if (valid) {
            // Migrate to PBKDF2
            const newSalt = generateSalt();
            const newHash = await pbkdf2Hash(password, newSalt);
            this.sql.exec(
              "UPDATE users SET password_hash = ?, salt = ? WHERE username = ?",
              newHash, newSalt, username
            );
          }
        }

        if (!valid) {
          this._trackFailedAuth(username);
          return json({ error: "invalid credentials" }, 401);
        }

        // Success — clear rate limit counter and update last_login
        this.authAttempts.delete(username);
        this.sql.exec("UPDATE users SET last_login = ? WHERE username = ?", new Date().toISOString(), username);
        const userRole = this.sql.exec("SELECT role FROM users WHERE username = ?", username).toArray()[0]?.role || 'user';
        const token = await signUserToken(username, new URL(request.url).origin, this.env.RELAY_SECRET);
        return json({ ok: true, username, token, role: userRole });
      }

      // GET /auth/me — get current user info (including role)
      if (url.pathname === "/auth/me" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const row = this.sql.exec(
          "SELECT username, email, role, created, last_login FROM users WHERE username = ?", user
        ).toArray()[0];
        if (!row) return json({ error: "user not found" }, 404);
        return json({ ok: true, ...row });
      }

      // ── Admin endpoints (require admin role) ──

      // GET /admin/users — list all registered users
      if (url.pathname === "/admin/users" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const rows = this.sql.exec(
          "SELECT username, email, role, created, last_login FROM users ORDER BY created ASC"
        ).toArray();
        return json(rows);
      }

      // GET /admin/games — list all games (external + internal) with owner info
      if (url.pathname === "/admin/games" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const external = this.sql.exec(
          "SELECT content_hash, name, size, updated, owner, game_id, scope FROM games ORDER BY owner ASC, name ASC"
        ).toArray();
        const internal = this.sql.exec(
          "SELECT owner, game_id, name, content_hash, size, updated, forked_from_hash FROM internal_games ORDER BY owner ASC, game_id ASC"
        ).toArray();
        return json({ external, internal });
      }

      // DELETE /admin/users/:username — delete a user (admin only, cannot delete self)
      const adminUserDelete = url.pathname.match(/^\/admin\/users\/([^/]+)$/);
      if (adminUserDelete && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserDelete[1]);
        if (target === user) return json({ error: "cannot delete yourself" }, 400);
        const exists = this.sql.exec("SELECT username FROM users WHERE username = ?", target).toArray()[0];
        if (!exists) return json({ error: "user not found" }, 404);
        this.sql.exec("DELETE FROM users WHERE username = ?", target);
        return json({ ok: true, deleted: target });
      }

      // PUT /admin/users/:username — edit user role/email (admin only)
      const adminUserEdit = url.pathname.match(/^\/admin\/users\/([^/]+)$/);
      if (adminUserEdit && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserEdit[1]);
        const body = await request.json().catch(() => ({}));
        const exists = this.sql.exec("SELECT username FROM users WHERE username = ?", target).toArray()[0];
        if (!exists) return json({ error: "user not found" }, 404);
        if (body.role && ['user', 'admin'].includes(body.role)) {
          this.sql.exec("UPDATE users SET role = ? WHERE username = ?", body.role, target);
        }
        if (body.email && typeof body.email === 'string' && body.email.includes('@')) {
          this.sql.exec("UPDATE users SET email = ? WHERE username = ?", body.email, target);
        }
        if (body.password && typeof body.password === 'string' && body.password.length >= 4) {
          const newSalt = crypto.randomUUID();
          const newHash = await pbkdf2Hash(body.password, newSalt);
          this.sql.exec("UPDATE users SET password_hash = ?, salt = ? WHERE username = ?", newHash, newSalt, target);
        }
        return json({ ok: true, updated: target });
      }

      // DELETE /admin/games/:hash — delete a game (external or internal by content_hash)
      const adminGameDelete = url.pathname.match(/^\/admin\/games\/([^/]+)$/);
      if (adminGameDelete && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const hash = decodeURIComponent(adminGameDelete[1]);
        const ext = this.sql.exec("SELECT content_hash FROM games WHERE content_hash = ?", hash).toArray()[0];
        const intl = this.sql.exec("SELECT content_hash FROM internal_games WHERE content_hash = ?", hash).toArray()[0];
        if (!ext && !intl) return json({ error: "game not found" }, 404);
        if (ext) this.sql.exec("DELETE FROM games WHERE content_hash = ?", hash);
        if (intl) this.sql.exec("DELETE FROM internal_games WHERE content_hash = ?", hash);
        this.broadcast(JSON.stringify({ type: 'game-deleted', content_hash: hash }));
        return json({ ok: true, deleted: hash });
      }

      // PUT /admin/games/:hash/assign — change game owner (admin only)
      const adminGameAssign = url.pathname.match(/^\/admin\/games\/([^/]+)\/assign$/);
      if (adminGameAssign && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = this.sql.exec("SELECT role FROM users WHERE username = ?", user).toArray()[0]?.role;
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const hash = decodeURIComponent(adminGameAssign[1]);
        const body = await request.json().catch(() => ({}));
        const newOwner = (body.owner || '').trim();
        if (!newOwner) return json({ error: "owner required" }, 400);
        const ext = this.sql.exec("SELECT content_hash FROM games WHERE content_hash = ?", hash).toArray()[0];
        const intl = this.sql.exec("SELECT owner, game_id FROM internal_games WHERE content_hash = ?", hash).toArray()[0];
        if (!ext && !intl) return json({ error: "game not found" }, 404);
        if (ext) this.sql.exec("UPDATE games SET owner = ? WHERE content_hash = ?", newOwner, hash);
        if (intl) this.sql.exec("UPDATE internal_games SET owner = ? WHERE owner = ? AND game_id = ?", newOwner, intl.owner, intl.game_id);
        return json({ ok: true, hash, owner: newOwner });
      }

      // GET /games — list all external games
      if (url.pathname === "/games" && request.method === "GET") {
        const rows = this.sql.exec(
          "SELECT content_hash, name, size, updated, owner, game_id, scope, version, checksum FROM games WHERE scope = 'external' ORDER BY name ASC"
        ).toArray().map((r) => this.normalizeExternalGameRow(r));
        return json(rows);
      }

      // GET /games.toml — export external game manifests as TOML
      if (url.pathname === '/games.toml' && request.method === 'GET') {
        const rows = this.sql.exec(
          "SELECT content_hash, name, size, updated, owner, game_id, version, checksum FROM games WHERE scope = 'external' ORDER BY owner ASC, game_id ASC"
        ).toArray().map((r) => this.normalizeExternalGameRow(r));
        const out = rows.map((g) => {
          return [
            '[[game]]',
            `id = "${g.owner}/${g.game_id}"`,
            `name = "${String(g.name || '').replace(/"/g, '\\"')}"`,
            `owner = "${g.owner}"`,
            `game_id = "${g.game_id}"`,
            `version = "${g.version || ''}"`,
            `checksum = "${g.checksum || g.content_hash}"`,
            `content_hash = "${g.content_hash}"`,
            `size = ${Number(g.size || 0)}`,
            `updated = "${g.updated}"`,
          ].join('\n');
        }).join('\n\n');
        return new Response(out, {
          status: 200,
          headers: { 'Content-Type': 'text/plain; charset=utf-8', ...cors() }
        });
      }

      // GET /internal/games — list authenticated user's internal games
      if (url.pathname === "/internal/games" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const rows = this.sql.exec(
          "SELECT owner, game_id, name, content_hash, checksum, version, size, updated, forked_from_hash FROM internal_games WHERE owner = ? ORDER BY updated DESC",
          user
        ).toArray();
        return json(rows);
      }

      // POST /internal/fork — fork external game into authenticated user's internal room
      if (url.pathname === "/internal/fork" && request.method === "POST") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const body = await request.json().catch(() => ({}));
        const sourceHash = String(body.source_hash || '').trim();
        if (!sourceHash) return json({ error: "source_hash required" }, 400);

        const src = this.sql.exec(
          "SELECT content_hash, name, content FROM games WHERE content_hash = ?", sourceHash
        ).toArray()[0];
        if (!src) return json({ error: "source game not found" }, 404);

        const gameId = this.deriveGameId(src.name, body.game_id);
        const version = String(body.version || 'v1');
        const checksum = await sha256hex16(src.content);
        const updated = new Date().toISOString();

        this.sql.exec(
          `INSERT OR REPLACE INTO internal_games
           (owner, game_id, name, content, content_hash, checksum, version, forked_from_hash, updated, size)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
          user, gameId, String(body.name || src.name).slice(0, 100), src.content,
          sourceHash, checksum, version, sourceHash, updated, src.content.length
        );
        return json({ ok: true, owner: user, game_id: gameId, forked_from_hash: sourceHash, checksum, version });
      }

      // GET /game/:hash — full HTML content of one game
      if (url.pathname.startsWith("/game/") && request.method === "GET") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const rows = this.sql.exec(
          "SELECT content_hash, name, content, updated, owner, game_id, scope, version, checksum FROM games WHERE content_hash = ?", hash
        ).toArray().map((r) => this.normalizeExternalGameRow(r));
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
        const owner = normalizeSlug(body.owner || 'public', 'public');
        const gameId = this.deriveGameId(name, body.game_id);
        const version = String(body.version || '');
        const updated = new Date().toISOString();

        this.sql.exec("DELETE FROM games WHERE name = ? AND scope = 'external'", name.slice(0, 100));
        this.sql.exec("DELETE FROM games WHERE owner = ? AND game_id = ? AND scope = 'external'", owner, gameId);

        // Delete old row, insert with new hash
        this.sql.exec("DELETE FROM games WHERE content_hash = ?", hash);
        this.sql.exec(
          `INSERT INTO games
           (content_hash, name, content, updated, size, owner, game_id, scope, version, checksum)
           VALUES (?, ?, ?, ?, ?, ?, ?, 'external', ?, ?)` ,
          newHash, name.slice(0, 100), content, updated, content.length,
          owner, gameId, version, newHash
        );
        this.trimExternalPool();

        // Broadcast updated game to all connected WebSocket clients
        const msg = JSON.stringify({
          type: 'sync',
          games: [{
            content_hash: newHash,
            checksum: newHash,
            owner,
            game_id: gameId,
            scope: 'external',
            version,
            name: name.slice(0, 100),
            content,
            updated
          }]
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

      // DELETE /game/:hash — remove a game
      if (url.pathname.startsWith("/game/") && request.method === "DELETE") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const exists = this.sql.exec("SELECT 1 FROM games WHERE content_hash = ?", hash).toArray();
        if (exists.length === 0) return json({ error: "not found" }, 404);
        this.sql.exec("DELETE FROM games WHERE content_hash = ?", hash);
        return json({ ok: true, deleted: hash });
      }

      // PUT /internal/game/:gameId — update authenticated user's internal game
      if (url.pathname.startsWith('/internal/game/') && request.method === 'PUT') {
        const user = await this.authUser(request);
        if (!user) return json({ error: 'auth required' }, 401);
        const gameId = normalizeSlug(url.pathname.slice('/internal/game/'.length), '');
        if (!gameId) return json({ error: 'missing game id' }, 400);
        const body = await request.json().catch(() => ({}));
        const content = String(body.content || '');
        if (!content) return json({ error: 'missing content' }, 400);
        if (content.length > MAX_GAME_SIZE) return json({ error: 'too large' }, 413);

        const name = String(body.name || gameId).slice(0, 100);
        const checksum = await sha256hex16(content);
        const version = String(body.version || 'v1');
        const updated = new Date().toISOString();

        this.sql.exec(
          `INSERT OR REPLACE INTO internal_games
           (owner, game_id, name, content, content_hash, checksum, version, forked_from_hash, updated, size)
           VALUES (?, ?, ?, ?, ?, ?, ?,
             COALESCE((SELECT forked_from_hash FROM internal_games WHERE owner = ? AND game_id = ?), NULL),
             ?, ?)`,
          user, gameId, name, content, checksum, checksum, version, user, gameId, updated, content.length
        );
        return json({ ok: true, owner: user, game_id: gameId, content_hash: checksum, checksum, version });
      }

      return json({ error: "WebSocket upgrade required or use REST endpoints" }, 426);
    }

    // ── WebSocket upgrade ──
    const pair = new WebSocketPair();
    this.state.acceptWebSocket(pair[1]);

    // Send catalog (hashes only) to the new client
    const rows = this.sql.exec("SELECT content_hash FROM games WHERE scope = 'external'").toArray();
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
          `SELECT content_hash, name, content, updated, owner, game_id, scope, version, checksum FROM games WHERE content_hash IN (${ph})`,
          ...wanted
        ).toArray().map((r) => this.normalizeExternalGameRow(r));
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
          if (count >= MAX_TOTAL_GAMES) {
            // Hard safety cap: evict the oldest external row to make room.
            const oldest = this.sql.exec(
              "SELECT content_hash FROM games WHERE scope = 'external' ORDER BY updated ASC, rowid ASC LIMIT 1"
            ).toArray()[0];
            if (!oldest) break;
            this.sql.exec("DELETE FROM games WHERE content_hash = ?", oldest.content_hash);
            count = Math.max(0, count - 1);
          }

          // Verify content hash matches (don't trust client blindly)
          const verified = await sha256hex16(g.content);
          if (verified !== g.content_hash) continue;

          // Check if already stored
          const exists = this.sql.exec(
            "SELECT 1 FROM games WHERE content_hash = ?", g.content_hash
          ).toArray();
          if (exists.length > 0) continue;

          const owner = normalizeSlug(g.owner || 'public', 'public');
          const gameId = this.deriveGameId(g.name, g.game_id);
          const version = String(g.version || '');

          this.sql.exec("DELETE FROM games WHERE name = ? AND scope = 'external'", g.name.slice(0, 100));
          // Keep only one current version per identity (<owner>/<game_id>) in external pool.
          const sameIdentity = this.sql.exec(
            "SELECT content_hash FROM games WHERE owner = ? AND game_id = ? AND scope = 'external'",
            owner, gameId
          ).toArray();
          for (const row of sameIdentity) {
            this.sql.exec("DELETE FROM games WHERE content_hash = ?", row.content_hash);
            count = Math.max(0, count - 1);
          }

          const updated = new Date().toISOString();
          this.sql.exec(
            `INSERT INTO games
             (content_hash, name, content, updated, size, owner, game_id, scope, version, checksum)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'external', ?, ?)`,
            g.content_hash, g.name.slice(0, 100), g.content, updated, g.content.length,
            owner, gameId, version, verified
          );
          const trimmed = this.trimExternalPool();
          if (trimmed > 0) {
            count = Math.max(0, count - trimmed);
          }
          added.push({
            content_hash: g.content_hash,
            checksum: verified,
            owner,
            game_id: gameId,
            scope: 'external',
            version,
            name: g.name.slice(0, 100),
            content: g.content,
            updated
          });
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

        // Only store if it's higher than existing (or same score adding player info)
        const existing = this.sql.exec(
          "SELECT score, player FROM scores WHERE game_hash = ?", data.game_hash
        ).toArray();

        if (existing.length > 0) {
          const dominated = incoming < existing[0].score ||
            (incoming === existing[0].score && (!player || existing[0].player));
          if (dominated) {
            ws.send(JSON.stringify({
              type: 'score-update',
              game_hash: data.game_hash,
              score: existing[0].score,
              player: existing[0].player || ''
            }));
            return;
          }
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
