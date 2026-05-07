/**
 * traits.build relay — Cloudflare Worker + Durable Objects
 *
 * One RelaySession DO per pairing code. The DO holds all in-flight state
 * in memory, so long-poll coordination is instant and zero-latency.
 *
 * One GameRoomV3 DO (global) for automatic game sync between all clients.
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
 *   GET  /sync                → WebSocket upgrade → GameRoomV3 (automatic game sync)
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

// ── AES-256-GCM encryption for user secrets ──────────────────────────────────

async function deriveSecretKey(secret, username) {
  const key = await _getHmacKey(secret);
  const data = new TextEncoder().encode('user_secrets:' + username);
  const derived = await crypto.subtle.sign('HMAC', key, data);
  return crypto.subtle.importKey('raw', derived, { name: 'AES-GCM' }, false, ['encrypt', 'decrypt']);
}

async function encryptSecret(value, secret, username) {
  const key = await deriveSecretKey(secret, username);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, new TextEncoder().encode(value));
  const combined = new Uint8Array(iv.length + ct.byteLength);
  combined.set(iv);
  combined.set(new Uint8Array(ct), iv.length);
  return btoa(String.fromCharCode(...combined));
}

async function decryptSecret(encrypted, secret, username) {
  const key = await deriveSecretKey(secret, username);
  const raw = Uint8Array.from(atob(encrypted), c => c.charCodeAt(0));
  const iv = raw.slice(0, 12);
  const ct = raw.slice(12);
  const pt = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, ct);
  return new TextDecoder().decode(pt);
}

// Rate limiting constants for auth endpoints
const MAX_AUTH_ATTEMPTS = 5;
const AUTH_COOLDOWN_MS = 60_000; // 60 seconds

// ── CORS ─────────────────────────────────────────────────────────────────────

function cors() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET,POST,PUT,PATCH,DELETE,OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type,Authorization,X-Game-Owner",
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

function normalizeResourcesMap(input) {
  const src = (input && typeof input === 'object') ? input : {};
  const out = {};
  for (const key of Object.keys(src).sort()) {
    const path = String(key || '').trim();
    if (!path || path === 'canvas/app.html' || path === 'canvas/games.json') continue;
    if (path.startsWith('/') || path.includes('..')) continue;
    const val = src[key];
    if (typeof val !== 'string' || !val) continue;
    out[path] = val;
  }
  return out;
}

function parseResourcesField(raw) {
  try {
    if (!raw) return {};
    const parsed = (typeof raw === 'string') ? JSON.parse(raw) : raw;
    if (Array.isArray(parsed)) return {};
    if (typeof parsed === 'object') return normalizeResourcesMap(parsed);
    return {};
  } catch (_) {
    return {};
  }
}

function encodeResourcesField(resources) {
  // Store full normalized map so resources are durable and available
  // to late-joining clients even when no peer is online.
  return JSON.stringify(normalizeResourcesMap(resources));
}

function resourcePaths(resources) {
  // Accept either a full {path: dataUri} map or an already-stripped path list.
  if (Array.isArray(resources)) return resources.filter(p => typeof p === 'string' && p);
  const normalized = normalizeResourcesMap(resources);
  return Object.keys(normalized).sort();
}

function parseManifestField(raw) {
  // Read DB field — may be old-format {path:data} object or new-format ["path",...] array
  try {
    if (!raw) return [];
    const parsed = typeof raw === 'string' ? JSON.parse(raw) : raw;
    if (Array.isArray(parsed)) return parsed.filter(p => typeof p === 'string' && p);
    if (typeof parsed === 'object') return Object.keys(parsed).sort();
    return [];
  } catch (_) { return []; }
}

function resourceBytes(resources) {
  const normalized = normalizeResourcesMap(resources);
  let total = 0;
  for (const val of Object.values(normalized)) total += String(val || '').length;
  return total;
}

async function packageHash16(content, resources) {
  // Hash stays content-based for stable identity/versioning semantics.
  return sha256hex16(String(content || ''));
}

function makeReleaseVersion() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, '0');
  const y = String(d.getUTCFullYear()).slice(-2);
  const mo = p(d.getUTCMonth() + 1);
  const da = p(d.getUTCDate());
  const hh = p(d.getUTCHours());
  const mm = p(d.getUTCMinutes());
  const ss = p(d.getUTCSeconds());
  return `${y}${mo}${da}.${hh}${mm}${ss}`;
}

// ── Durable Object: GameRoomV3 ────────────────────────────────────────────────
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

const MAX_GAME_SIZE = 256 * 1024; // 256KB HTML content per game
const MAX_GAME_PACKAGE_SIZE = 8 * 1024 * 1024; // HTML + resources bundle cap (sprites included)
const MAX_TOTAL_GAMES = 500;
const DEFAULT_EXTERNAL_POOL_SIZE = 64;

// ── Top-level GitHub publisher (used by Worker-level /sync/internal/*/github-publish) ──
// Heuristic for paths that are sprite/audio assets and should be exploded
// into per-file GitHub blobs (so they can be browsed and managed individually).
function _isSpriteOrMediaPath(p) {
  const s = String(p || '').toLowerCase();
  if (!s) return false;
  if (s.startsWith('sprites/') || s.startsWith('canvas/sprites/')) return true;
  if (s.startsWith('assets/') || s.startsWith('images/') || s.startsWith('textures/') || s.startsWith('audio/')) return true;
  return /\.(png|jpe?g|gif|webp|svg|mp3|wav|ogg|mp4|webm|aac|flac|atlas)$/.test(s);
}

// Decode a value that might be a data URL, raw base64, or text into base64
// suitable for the GitHub Contents API.
function _toBase64ForGitHub(value) {
  const s = String(value || '');
  if (!s) return '';
  // data URL: strip prefix and return the base64 part as-is
  const m = s.match(/^data:[^;,]+;base64,(.*)$/i);
  if (m) return m[1];
  // raw base64 looking string (long, only base64 chars)
  if (s.length > 32 && /^[A-Za-z0-9+/=\s]+$/.test(s)) {
    // Likely already base64
    return s.replace(/\s+/g, '');
  }
  // Treat as text — utf-8 encode then base64
  try { return btoa(unescape(encodeURIComponent(s))); }
  catch (_) { return ''; }
}

async function _ghPutFile(BASE, headers, path, base64Content, message) {
  // Read existing sha first (so subsequent updates can overwrite)
  const ex = await fetch(`${BASE}/${path}`, { headers });
  const exData = ex.ok ? await ex.json().catch(() => ({})) : {};
  const sha = exData.sha;
  const put = await fetch(`${BASE}/${path}`, {
    method: 'PUT', headers,
    body: JSON.stringify({ message, content: base64Content, ...(sha ? { sha } : {}) }),
  });
  if (!put.ok) {
    const err = await put.json().catch(() => ({}));
    throw new Error(`GitHub PUT ${path} failed (${put.status}): ${err.message || ''}`);
  }
  return await put.json().catch(() => ({}));
}

async function _ghDeleteFile(BASE, headers, path, message) {
  const ex = await fetch(`${BASE}/${path}`, { headers });
  if (!ex.ok) return false; // already gone
  const exData = await ex.json().catch(() => ({}));
  const sha = exData.sha;
  if (!sha) return false;
  const del = await fetch(`${BASE}/${path}`, {
    method: 'DELETE', headers,
    body: JSON.stringify({ message, sha }),
  });
  return del.ok;
}

async function _ghListDir(BASE, headers, dirPath) {
  const r = await fetch(`${BASE}/${dirPath}`, { headers });
  if (!r.ok) return [];
  const d = await r.json().catch(() => null);
  return Array.isArray(d) ? d : [];
}

async function publishGameToGitHub(row, token, repo) {
  const owner = normalizeSlug(row.owner, 'public');
  const gameId = normalizeSlug(row.game_id, 'game');
  const BASE = `https://api.github.com/repos/${repo}/contents`;
  const headers = {
    Authorization: `token ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': 'slob-games-relay/1.0',
    Accept: 'application/vnd.github.v3+json',
  };

  const resources = parseResourcesField(row.resources);
  const gamePayload = {
    name: row.name,
    content: row.content,
    version: row.version || '',
    checksum: row.checksum || row.content_hash,
    content_hash: row.content_hash,
    owner, game_id: gameId,
    updated: row.updated, size: row.size,
    resources,
  };
  const gamePath = `games/${owner}/${gameId}.json`;
  const gameContent = btoa(unescape(encodeURIComponent(JSON.stringify(gamePayload, null, 2))));

  const existResp = await fetch(`${BASE}/${gamePath}`, { headers });
  const existData = existResp.ok ? await existResp.json().catch(() => ({})) : {};
  const gameSha = existData.sha;

  const gamePut = await fetch(`${BASE}/${gamePath}`, {
    method: 'PUT', headers,
    body: JSON.stringify({
      message: `publish: ${owner}/${gameId} v${row.version || 'latest'}`,
      content: gameContent,
      ...(gameSha ? { sha: gameSha } : {}),
    }),
  });
  if (!gamePut.ok) {
    const err = await gamePut.json().catch(() => ({}));
    throw new Error(`GitHub game write failed (${gamePut.status}): ${err.message || ''}`);
  }

  // Explode sprite/media resources to per-file blobs under games/{owner}/{gameId}/
  // for browseability and management. Errors here are non-fatal — the game JSON
  // itself still has resources inlined so playback works either way.
  const spriteResults = [];
  for (const p of Object.keys(resources)) {
    if (!_isSpriteOrMediaPath(p)) continue;
    const b64 = _toBase64ForGitHub(resources[p]);
    if (!b64) continue;
    const safe = String(p).replace(/^\/+/, '').replace(/\.\./g, '');
    if (!safe || safe.length > 240) continue;
    const filePath = `games/${owner}/${gameId}/${safe}`;
    try {
      await _ghPutFile(BASE, headers, filePath, b64, `sprite: ${owner}/${gameId} ${safe}`);
      spriteResults.push(safe);
    } catch (e) {
      console.warn('[publish-sprite-failed]', filePath, String(e?.message || e).slice(0, 200));
    }
  }

  for (let attempt = 0; attempt < 2; attempt++) {
    const idxResp = await fetch(`${BASE}/games/index.json`, { headers });
    const idxData = idxResp.ok ? await idxResp.json().catch(() => ({})) : {};
    const idxSha = idxData.sha;
    let index = { games: [] };
    if (idxData.content) {
      try { index = JSON.parse(decodeURIComponent(escape(atob(idxData.content.replace(/\n/g, ''))))); } catch (_) {}
    }
    const entry = {
      id: `${owner}/${gameId}`, owner, game_id: gameId, name: row.name,
      checksum: row.checksum || row.content_hash, content_hash: row.content_hash,
      size: row.size, updated: row.updated, version: row.version || '',
      published: true,
      sprite_count: spriteResults.length,
    };
    const idx = index.games.findIndex(g => g.id === entry.id);
    if (idx >= 0) {
      // Preserve published flag if it was explicitly set to false (disabled)
      const prev = index.games[idx];
      if (prev && prev.published === false) entry.published = false;
      index.games[idx] = entry;
    } else {
      index.games.push(entry);
    }
    index.games.sort((a, b) => a.id.localeCompare(b.id));
    const idxContent = btoa(unescape(encodeURIComponent(JSON.stringify(index, null, 2))));
    const idxPut = await fetch(`${BASE}/games/index.json`, {
      method: 'PUT', headers,
      body: JSON.stringify({
        message: `index: upsert ${owner}/${gameId}`,
        content: idxContent,
        ...(idxSha ? { sha: idxSha } : {}),
      }),
    });
    if (idxPut.ok) break;
    if (idxPut.status === 409 && attempt === 0) continue;
    const err = await idxPut.json().catch(() => ({}));
    throw new Error(`GitHub index write failed (${idxPut.status}): ${err.message || ''}`);
  }

  const rawBase = `https://raw.githubusercontent.com/${repo}/main`;
  return {
    ok: true,
    raw_url: `${rawBase}/${gamePath}`,
    index_url: `${rawBase}/games/index.json`,
    sprites: spriteResults,
    sprite_count: spriteResults.length,
  };
}

// ── Top-level GitHub manager helpers (delete/disable/rename) ──
function _b64Encode(str) {
  const bytes = new TextEncoder().encode(String(str || ''));
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
}
function _b64Decode(b64) {
  const bin = atob(String(b64 || '').replace(/\s+/g, ''));
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}

async function _ghLoadIndex(BASE, headers) {
  const r = await fetch(`${BASE}/games/index.json`, { headers });
  if (!r.ok) return { sha: null, index: { games: [] } };
  const d = await r.json().catch(() => ({}));
  let index = { games: [] };
  if (d && d.content) {
    try { index = JSON.parse(_b64Decode(d.content)); }
    catch (_) {}
  }
  return { sha: d?.sha || null, index };
}

async function _ghSaveIndex(BASE, headers, index, sha, message) {
  const body = JSON.stringify(index, null, 2);
  const content = _b64Encode(body);
  const put = await fetch(`${BASE}/games/index.json`, {
    method: 'PUT', headers,
    body: JSON.stringify({ message, content, ...(sha ? { sha } : {}) }),
  });
  if (!put.ok) {
    const err = await put.json().catch(() => ({}));
    throw new Error(`GitHub index write failed (${put.status}): ${err.message || ''}`);
  }
}

async function deleteGameFromGitHub(owner, gameId, token, repo) {
  const BASE = `https://api.github.com/repos/${repo}/contents`;
  const headers = {
    Authorization: `token ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': 'slob-games-relay/1.0',
    Accept: 'application/vnd.github.v3+json',
  };
  // Delete game JSON
  await _ghDeleteFile(BASE, headers, `games/${owner}/${gameId}.json`, `delete: ${owner}/${gameId}`);
  // Delete sprite folder recursively (one level only — flat sprite folder)
  let removedFiles = 0;
  async function rmDir(dir) {
    const items = await _ghListDir(BASE, headers, dir);
    for (const it of items) {
      if (it.type === 'file') {
        if (await _ghDeleteFile(BASE, headers, it.path, `delete: ${it.path}`)) removedFiles++;
      } else if (it.type === 'dir') {
        await rmDir(it.path);
      }
    }
  }
  await rmDir(`games/${owner}/${gameId}`);
  // Remove from index
  for (let attempt = 0; attempt < 3; attempt++) {
    const { sha, index } = await _ghLoadIndex(BASE, headers);
    const id = `${owner}/${gameId}`;
    const before = (index.games || []).length;
    index.games = (index.games || []).filter(g => g.id !== id);
    if (index.games.length === before) return { ok: true, removed_files: removedFiles, in_index: false };
    try {
      await _ghSaveIndex(BASE, headers, index, sha, `index: remove ${id}`);
      return { ok: true, removed_files: removedFiles, in_index: true };
    } catch (e) {
      if (attempt < 2) continue;
      throw e;
    }
  }
  return { ok: true, removed_files: removedFiles };
}

async function patchGameOnGitHub(owner, gameId, patch, token, repo) {
  const BASE = `https://api.github.com/repos/${repo}/contents`;
  const headers = {
    Authorization: `token ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': 'slob-games-relay/1.0',
    Accept: 'application/vnd.github.v3+json',
  };
  const id = `${owner}/${gameId}`;
  // Update index entry first
  let updatedIndex = false;
  for (let attempt = 0; attempt < 3; attempt++) {
    const { sha, index } = await _ghLoadIndex(BASE, headers);
    const idx = (index.games || []).findIndex(g => g.id === id);
    if (idx < 0) return { ok: false, error: 'not in index' };
    const entry = index.games[idx];
    if (typeof patch.published === 'boolean') entry.published = patch.published;
    if (typeof patch.name === 'string' && patch.name.trim()) entry.name = patch.name.trim().slice(0, 100);
    index.games[idx] = entry;
    try {
      await _ghSaveIndex(BASE, headers, index, sha, `index: patch ${id}`);
      updatedIndex = true;
      break;
    } catch (e) {
      if (attempt < 2) continue;
      throw e;
    }
  }
  // Also rename inside game.json so the activated game shows the new name
  if (typeof patch.name === 'string' && patch.name.trim()) {
    try {
      const gamePath = `games/${owner}/${gameId}.json`;
      const r = await fetch(`${BASE}/${gamePath}`, { headers });
      if (r.ok) {
        const d = await r.json().catch(() => ({}));
        if (d && d.content) {
          let game = {};
          try { game = JSON.parse(_b64Decode(d.content)); }
          catch (_) {}
          game.name = patch.name.trim().slice(0, 100);
          const content = _b64Encode(JSON.stringify(game, null, 2));
          await fetch(`${BASE}/${gamePath}`, {
            method: 'PUT', headers,
            body: JSON.stringify({ message: `rename: ${id}`, content, sha: d.sha }),
          });
        }
      }
    } catch (e) {
      console.warn('[rename-game-json]', String(e?.message || e).slice(0, 200));
    }
  }
  return { ok: true, updated_index: updatedIndex };
}

// ── Worker-level /sync/internal/* handler — replaces broken DO routes using AUTH_KV ──
// Key scheme:
//   mygame:{owner}:{game_id} → full record { content, name, version, resources, ... }
//   mygames:{owner} → array of summaries (no content)
async function handleInternalRoutes(url, request, env, ctx) {
  if (!env.AUTH_KV) return json({ error: 'storage not configured' }, 503);
  if (!env.RELAY_SECRET) return json({ error: 'RELAY_SECRET not configured' }, 503);

  // /sync/internal/... → /internal/...
  const path = url.pathname.startsWith('/sync/internal/')
    ? url.pathname.slice(5)
    : url.pathname;

  const authHeader = request.headers.get('Authorization') || '';
  const token = authHeader.startsWith('Bearer ') ? authHeader.slice(7).trim() : null;
  const tokenPayload = token ? await verifyUserToken(token, env.RELAY_SECRET) : null;
  const user = tokenPayload?.sub || null;
  if (!user) return json({ error: 'auth required' }, 401);

  // KV helpers for user games
  const getGame = async (owner, gameId) => {
    return (await env.AUTH_KV.get(`mygame:${owner}:${gameId}`, { type: 'json' })) || null;
  };
  const putGame = async (row) => {
    await env.AUTH_KV.put(`mygame:${row.owner}:${row.game_id}`, JSON.stringify(row));
    const summaries = (await env.AUTH_KV.get(`mygames:${row.owner}`, { type: 'json' })) || [];
    const summary = {
      content_hash: row.content_hash, checksum: row.checksum || row.content_hash,
      owner: row.owner, game_id: row.game_id, name: row.name, version: row.version || '',
      size: row.size, updated: row.updated, scope: row.scope || 'internal',
      published: row.published ? 1 : 0,
      forked_from_hash: row.forked_from_hash || null,
      resource_paths: row.resource_paths || [],
    };
    const idx = summaries.findIndex(g => g.game_id === row.game_id);
    if (idx >= 0) summaries[idx] = summary; else summaries.push(summary);
    await env.AUTH_KV.put(`mygames:${row.owner}`, JSON.stringify(summaries));
  };
  const deleteGame = async (owner, gameId) => {
    await env.AUTH_KV.delete(`mygame:${owner}:${gameId}`);
    const summaries = (await env.AUTH_KV.get(`mygames:${owner}`, { type: 'json' })) || [];
    await env.AUTH_KV.put(`mygames:${owner}`, JSON.stringify(summaries.filter(g => g.game_id !== gameId)));
  };
  const getUserRole = async (username) => {
    const u = await env.AUTH_KV.get(`user:${username}`, { type: 'json' });
    return u?.role || 'user';
  };

  // GET /internal/games — list user's games
  if (path === '/internal/games' && request.method === 'GET') {
    const summaries = (await env.AUTH_KV.get(`mygames:${user}`, { type: 'json' })) || [];
    summaries.sort((a, b) => String(b.updated || '').localeCompare(String(a.updated || '')));
    return json(summaries);
  }

  // POST /internal/score — forward to DO for source-of-truth score write + broadcast.
  // Worker has already verified the bearer token; DO will re-verify via authUser.
  if (path === '/internal/score' && request.method === 'POST') {
    try {
      const room = env.GAME_ROOM.get(env.GAME_ROOM.idFromName('global6'));
      const doUrl = new URL(request.url);
      doUrl.pathname = '/internal/score';
      // Re-create a Request preserving method, headers (incl. Authorization), and body
      return await room.fetch(new Request(doUrl.toString(), request));
    } catch (e) {
      console.error('[score-fwd]', String(e?.message || e).slice(0, 300));
      return json({ error: 'service temporarily unavailable' }, 503);
    }
  }

  // PATCH /internal/game/:gameId/publish
  const publishMatch = path.match(/^\/internal\/game\/([^/]+)\/publish$/);
  if (publishMatch && request.method === 'PATCH') {
    const gameId = normalizeSlug(publishMatch[1], '');
    if (!gameId) return json({ error: 'missing game id' }, 400);
    const body = await request.json().catch(() => ({}));
    const role = await getUserRole(user);
    const requestedOwner = normalizeSlug(
      request.headers.get('X-Game-Owner') || body.owner || user,
      user
    );
    const owner = requestedOwner || user;
    if (owner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);
    const row = await getGame(owner, gameId);
    if (!row) return json({ error: 'not found' }, 404);
    const explicit = (typeof body.published === 'boolean') ? (body.published ? 1 : 0) : null;
    const newVal = (explicit === null) ? (row.published ? 0 : 1) : explicit;
    await putGame({ ...row, published: newVal });
    return json({ ok: true, owner, game_id: gameId, published: !!newVal });
  }

  // PATCH /internal/game/:gameId/github-publish
  const githubPublishMatch = path.match(/^\/internal\/game\/([^/]+)\/github-publish$/);
  if (githubPublishMatch && request.method === 'PATCH') {
    const gameId = normalizeSlug(githubPublishMatch[1], '');
    if (!gameId) return json({ error: 'missing game id' }, 400);
    const githubToken = env.GITHUB_TOKEN;
    const githubRepo = env.GITHUB_REPO;
    if (!githubToken || !githubRepo) return json({ error: 'GitHub publishing not configured on this relay' }, 503);
    const body = await request.json().catch(() => ({}));
    const requestedOwner = normalizeSlug(body.owner || user, user);
    const role = await getUserRole(user);
    if (requestedOwner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);
    const row = await getGame(requestedOwner, gameId);
    if (!row) return json({ error: 'game not found' }, 404);
    try {
      const result = await publishGameToGitHub(row, githubToken, githubRepo);
      return json(result);
    } catch (err) {
      return json({ error: String(err?.message || err) }, 502);
    }
  }

  // PUT /internal/game/:gameId — save
  if (path.startsWith('/internal/game/') && request.method === 'PUT') {
    const gameId = normalizeSlug(path.slice('/internal/game/'.length), '');
    if (!gameId) return json({ error: 'missing game id' }, 400);
    const body = await request.json().catch(() => ({}));
    const content = String(body.content || '');
    if (!content) return json({ error: 'missing content' }, 400);
    if (content.length > MAX_GAME_SIZE) return json({ error: 'too large' }, 413);
    const name = String(body.name || gameId).slice(0, 100);
    const version = String(body.version || makeReleaseVersion());
    const updated = new Date().toISOString();
    const prev = await getGame(user, gameId);
    const prevResources = parseResourcesField(prev?.resources);
    const resourcesMap = (body.resources === undefined)
      ? prevResources
      : parseResourcesField(body.resources);
    const paths = resourcePaths(resourcesMap);
    const resourcePayloadBytes = resourceBytes(resourcesMap);
    const packageSize = content.length + resourcePayloadBytes;
    if (packageSize > MAX_GAME_PACKAGE_SIZE) return json({ error: 'package too large' }, 413);
    const prevPublished = prev ? (prev.published ?? 0) : 0;
    const size = content.length;
    const checksum = await packageHash16(content);
    const nextScope = String(body.scope || (prev && prev.scope) || 'internal').trim().toLowerCase() === 'external'
      ? 'external' : 'internal';
    await putGame({
      content_hash: checksum, name, content, updated, size,
      owner: user, game_id: gameId, scope: nextScope, version, checksum,
      resources: encodeResourcesField(resourcesMap),
      forked_from_hash: prev?.forked_from_hash || null,
      published: prevPublished,
      resource_paths: paths,
    });
    // If already published, auto-mirror the new content + sprites to GitHub so
    // updates (especially new/changed sprite files) flow through without the user
    // toggling publish off/on. Fire-and-forget; failures are logged.
    if (prevPublished && env.GITHUB_TOKEN && env.GITHUB_REPO) {
      const row = await getGame(user, gameId);
      if (row) {
        const mirrorTask = (async () => {
          try {
            const r = await publishGameToGitHub(row, env.GITHUB_TOKEN, env.GITHUB_REPO);
            console.log('[github-mirror-on-update]', user, gameId, 'sprites=', r.sprite_count);
          } catch (e) {
            console.warn('[github-mirror-on-update] failed', user, gameId, String(e?.message || e).slice(0, 200));
          }
        })();
        if (ctx && typeof ctx.waitUntil === 'function') ctx.waitUntil(mirrorTask);
      }
    }
    return json({ ok: true, owner: user, game_id: gameId, content_hash: checksum, checksum, version, published: !!prevPublished });
  }

  // GET /internal/game/:gameId — get full content
  if (path.startsWith('/internal/game/') && request.method === 'GET') {
    const gameId = normalizeSlug(path.slice('/internal/game/'.length), '');
    if (!gameId) return json({ error: 'missing game id' }, 400);
    const owner = url.searchParams.get('owner') || user;
    const row = await getGame(owner, gameId);
    if (!row) return json({ error: 'not found' }, 404);
    const resources = parseResourcesField(row.resources);
    return json({ ...row, resource_paths: Object.keys(resources).sort(), resources });
  }

  // DELETE /internal/game/:gameId — delete
  if (path.startsWith('/internal/game/') && request.method === 'DELETE') {
    const gameId = normalizeSlug(path.slice('/internal/game/'.length), '');
    if (!gameId) return json({ error: 'missing game id' }, 400);
    const owner = url.searchParams.get('owner') || user;
    const role = await getUserRole(user);
    if (owner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);
    const row = await getGame(owner, gameId);
    if (!row) return json({ error: 'not found' }, 404);
    await deleteGame(owner, gameId);
    return json({ ok: true, deleted: gameId });
  }

  // POST /internal/sprites/upload — upload one or many sprite files to the
  // shared `games/_shared/sprites/` folder on GitHub. Body shape:
  //   { files: { "sprites/foo.png": "<dataUrl|base64|text>", ... } }
  // The basename is what gets written to GitHub; folder prefixes are dropped
  // so all clients converge on a flat shared namespace.
  if (path === '/internal/sprites/upload' && request.method === 'POST') {
    if (!env.GITHUB_TOKEN || !env.GITHUB_REPO) return json({ error: 'GitHub not configured' }, 503);
    let body = {};
    try { body = await request.json(); } catch (_) { return json({ error: 'invalid JSON' }, 400); }
    const filesIn = (body && body.files && typeof body.files === 'object') ? body.files : {};
    const paths = Object.keys(filesIn);
    if (!paths.length) return json({ error: 'no files' }, 400);
    const BASE = `https://api.github.com/repos/${env.GITHUB_REPO}/contents`;
    const headers = {
      Authorization: `token ${env.GITHUB_TOKEN}`,
      'Content-Type': 'application/json',
      'User-Agent': 'slob-games-relay/1.0',
      Accept: 'application/vnd.github.v3+json',
    };
    const wrote = [];
    const skipped = [];
    const failed = [];
    for (const p of paths) {
      try {
        if (!_isSpriteOrMediaPath(p)) { skipped.push({ path: p, reason: 'not media' }); continue; }
        const b64 = _toBase64ForGitHub(filesIn[p]);
        if (!b64) { skipped.push({ path: p, reason: 'empty' }); continue; }
        const basename = String(p).split('/').pop().replace(/\.\./g, '').replace(/[^A-Za-z0-9._-]/g, '_');
        if (!basename) { skipped.push({ path: p, reason: 'bad basename' }); continue; }
        const filePath = `games/_shared/sprites/${basename}`;
        await _ghPutFile(BASE, headers, filePath, b64, `shared sprite: ${basename} (by ${user})`);
        wrote.push(basename);
      } catch (e) {
        failed.push({ path: p, error: String(e?.message || e).slice(0, 200) });
      }
    }
    // Update games/_shared/sprites/index.json with the current listing.
    try {
      const list = await _ghListDir(BASE, headers, 'games/_shared/sprites');
      const idx = { sprites: list.filter(it => it && it.type === 'file' && it.name !== 'index.json').map(it => ({
        name: it.name, size: it.size, sha: it.sha, download_url: it.download_url,
      })) };
      const idxContent = btoa(unescape(encodeURIComponent(JSON.stringify(idx, null, 2))));
      await _ghPutFile(BASE, headers, 'games/_shared/sprites/index.json', idxContent, `shared sprites: index (${idx.sprites.length})`);
    } catch (e) {
      console.warn('[shared-sprites:index] failed', String(e?.message || e).slice(0, 200));
    }
    return json({ ok: true, wrote, skipped, failed });
  }

  // POST /internal/games/upload-tmp — dump orphan VFS HTML "games" (those not
  // tracked by games.json) into the GitHub `tmp/` folder for later triage.
  // Body shape: { files: { "<source-path>": "<html string>", ... } }
  // Each file is written as `tmp/<sha8>.html` (content-addressed) so that
  // duplicate content from multiple clients naturally collapses to one blob.
  if (path === '/internal/games/upload-tmp' && request.method === 'POST') {
    if (!env.GITHUB_TOKEN || !env.GITHUB_REPO) return json({ error: 'GitHub not configured' }, 503);
    let body = {};
    try { body = await request.json(); } catch (_) { return json({ error: 'invalid JSON' }, 400); }
    const filesIn = (body && body.files && typeof body.files === 'object') ? body.files : {};
    const paths = Object.keys(filesIn);
    if (!paths.length) return json({ error: 'no files' }, 400);
    const BASE = `https://api.github.com/repos/${env.GITHUB_REPO}/contents`;
    const headers = {
      Authorization: `token ${env.GITHUB_TOKEN}`,
      'Content-Type': 'application/json',
      'User-Agent': 'slob-games-relay/1.0',
      Accept: 'application/vnd.github.v3+json',
    };
    async function _sha8(text) {
      const buf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(String(text || '')));
      const arr = new Uint8Array(buf);
      let hex = '';
      for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, '0');
      return hex.slice(0, 16);
    }
    const wrote = [];
    const skipped = [];
    const failed = [];
    const seen = new Set();
    for (const p of paths) {
      const content = String(filesIn[p] || '');
      if (!content || content.length < 32) { skipped.push({ path: p, reason: 'empty' }); continue; }
      try {
        const hash = await _sha8(content);
        if (seen.has(hash)) { skipped.push({ path: p, reason: 'dup-in-batch', hash }); continue; }
        seen.add(hash);
        const filePath = `tmp/${hash}.html`;
        // If file already exists, skip — this endpoint is idempotent and dedupes
        // across clients via content-addressing.
        const ex = await fetch(`${BASE}/${filePath}`, { headers });
        if (ex.ok) { skipped.push({ path: p, reason: 'exists', hash, target: filePath }); continue; }
        const b64 = _toBase64ForGitHub(content);
        if (!b64) { skipped.push({ path: p, reason: 'encode-failed', hash }); continue; }
        await _ghPutFile(BASE, headers, filePath, b64, `tmp game: ${hash} (from ${user}, src=${String(p).slice(0, 80)})`);
        wrote.push({ path: p, hash, target: filePath, size: content.length });
      } catch (e) {
        failed.push({ path: p, error: String(e?.message || e).slice(0, 200) });
      }
    }
    return json({ ok: true, wrote, skipped, failed });
  }

  return json({ error: 'not found' }, 404);
}


// ── Worker-level /sync/admin/* and /admin/* handler — replaces broken DO admin routes ──
async function handleAdminRoutes(url, request, env) {
  if (!env.AUTH_KV) return json({ error: 'storage not configured' }, 503);
  if (!env.RELAY_SECRET) return json({ error: 'RELAY_SECRET not configured' }, 503);

  // Normalize /sync/admin/... → /admin/...
  const path = url.pathname.startsWith('/sync/admin/')
    ? url.pathname.slice(5)
    : url.pathname;

  const authHeader = request.headers.get('Authorization') || '';
  const token = authHeader.startsWith('Bearer ') ? authHeader.slice(7).trim() : null;
  const tokenPayload = token ? await verifyUserToken(token, env.RELAY_SECRET) : null;
  const user = tokenPayload?.sub || null;
  if (!user) return json({ error: 'auth required' }, 401);

  const kvGetUser = async (username) => {
    return (await env.AUTH_KV.get(`user:${username}`, { type: 'json' })) || null;
  };
  const kvPutUser = async (row) => {
    await env.AUTH_KV.put(`user:${row.username}`, JSON.stringify(row));
    if (row.email) await env.AUTH_KV.put(`user_email:${row.email.toLowerCase()}`, row.username);
    const rawIndex = await env.AUTH_KV.get('users:index', { type: 'json' });
    const index = rawIndex || [];
    const summary = { username: row.username, email: row.email, role: row.role, created: row.created, last_login: row.last_login };
    const idx = index.findIndex(u => u.username === row.username);
    if (idx >= 0) index[idx] = summary; else index.push(summary);
    await env.AUTH_KV.put('users:index', JSON.stringify(index));
  };
  const kvDeleteUser = async (username) => {
    const u = await kvGetUser(username);
    if (u?.email) await env.AUTH_KV.delete(`user_email:${String(u.email).toLowerCase()}`);
    await env.AUTH_KV.delete(`user:${username}`);
    const rawIndex = await env.AUTH_KV.get('users:index', { type: 'json' });
    const index = rawIndex || [];
    await env.AUTH_KV.put('users:index', JSON.stringify(index.filter(u => u.username !== username)));
  };

  const role = (await kvGetUser(user))?.role || 'user';
  if (role !== 'admin') return json({ error: 'admin required' }, 403);

  // GET /admin/users
  if (path === '/admin/users' && request.method === 'GET') {
    const index = (await env.AUTH_KV.get('users:index', { type: 'json' })) || [];
    index.sort((a, b) => String(a.created || '').localeCompare(String(b.created || '')));
    return json(index);
  }

  // GET /admin/games — aggregate all users' games
  if (path === '/admin/games' && request.method === 'GET') {
    const index = (await env.AUTH_KV.get('users:index', { type: 'json' })) || [];
    const all = [];
    for (const u of index) {
      const games = (await env.AUTH_KV.get(`mygames:${u.username}`, { type: 'json' })) || [];
      for (const g of games) all.push(g);
    }
    all.sort((a, b) => `${a.owner}/${a.name}`.localeCompare(`${b.owner}/${b.name}`));
    return json({ external: all, internal: [] });
  }

  // GET /admin/stats
  if (path === '/admin/stats' && request.method === 'GET') {
    const index = (await env.AUTH_KV.get('users:index', { type: 'json' })) || [];
    let total = 0, published = 0, draft = 0, bytes = 0;
    for (const u of index) {
      const games = (await env.AUTH_KV.get(`mygames:${u.username}`, { type: 'json' })) || [];
      total += games.length;
      for (const g of games) {
        if (g.published) published++; else draft++;
        bytes += g.size || 0;
      }
    }
    return json({
      ok: true,
      generated_at: new Date().toISOString(),
      runtime: { active_websockets: 0, auth_rate_limit_entries: 0 },
      games: { total, internal: 0, external: total, published, draft, content_bytes_total: bytes },
      resources: { cached_rows: 0, cached_bytes_total: 0 },
      users: { total: index.length, highscore_rows: 0 },
      storage: { sqlite_page_count: 0, sqlite_page_size: 0, sqlite_db_bytes_approx: 0 },
    });
  }

  // PUT /admin/users/:username — edit role/email/password
  const userEdit = path.match(/^\/admin\/users\/([^/]+)$/);
  if (userEdit && request.method === 'PUT') {
    const target = decodeURIComponent(userEdit[1]);
    const exists = await kvGetUser(target);
    if (!exists) return json({ error: 'user not found' }, 404);
    const body = await request.json().catch(() => ({}));
    const updated = { ...exists };
    if (body.role && ['user', 'admin'].includes(body.role)) updated.role = body.role;
    if (body.email && typeof body.email === 'string' && body.email.includes('@')) updated.email = body.email;
    if (body.password && typeof body.password === 'string' && body.password.length >= 4) {
      const newSalt = generateSalt();
      const newHash = await pbkdf2Hash(body.password, newSalt);
      updated.password_hash = newHash;
      updated.salt = newSalt;
    }
    await kvPutUser(updated);
    return json({ ok: true, updated: target, role: updated.role });
  }

  // DELETE /admin/users/:username
  if (userEdit && request.method === 'DELETE') {
    const target = decodeURIComponent(userEdit[1]);
    if (target === user) return json({ error: 'cannot delete yourself' }, 400);
    if (!(await kvGetUser(target))) return json({ error: 'user not found' }, 404);
    await kvDeleteUser(target);
    return json({ ok: true, deleted: target });
  }

  // DELETE /admin/games/:hash — delete game by content hash (search across users)
  const adminGameDel = path.match(/^\/admin\/games\/([^/]+)$/);
  if (adminGameDel && request.method === 'DELETE') {
    const hash = decodeURIComponent(adminGameDel[1]);
    const index = (await env.AUTH_KV.get('users:index', { type: 'json' })) || [];
    for (const u of index) {
      const games = (await env.AUTH_KV.get(`mygames:${u.username}`, { type: 'json' })) || [];
      const hit = games.find(g => g.content_hash === hash || g.checksum === hash);
      if (hit) {
        await env.AUTH_KV.delete(`mygame:${u.username}:${hit.game_id}`);
        await env.AUTH_KV.put(`mygames:${u.username}`, JSON.stringify(games.filter(g => g.game_id !== hit.game_id)));
        return json({ ok: true, deleted: hash });
      }
    }
    return json({ error: 'game not found' }, 404);
  }

  // Stub admin secrets endpoints (not stored in AUTH_KV — return empty / not-supported)
  if (path.match(/^\/admin\/users\/[^/]+\/secrets$/) && request.method === 'GET') {
    return json([]);
  }
  if (path.match(/^\/admin\/users\/[^/]+\/secrets\/[^/]+$/)) {
    return json({ error: 'admin secrets storage not available on KV backend' }, 503);
  }

  return json({ error: 'not found' }, 404);
}


export class GameRoomV6 {
  constructor(state, env) {
    console.log('[GameRoomV6] constructor start');
    this.state = state;
    this.env = env;
    this.authAttempts = new Map();
  }

  // ── KV storage helpers ────────────────────────────────────────────────────

  async _getGame(hash) {
    return (await this.state.storage.get(`game:${hash}`)) || null;
  }

  async _getGamesIndex() {
    return (await this.state.storage.get('games:index')) || [];
  }

  async _getGameByOwnerGameId(owner, gameId) {
    const hash = await this.state.storage.get(`gameid:${owner}:${gameId}`);
    if (!hash) return null;
    return this._getGame(hash);
  }

  async _putGame(row) {
    await this.state.storage.put(`game:${row.content_hash}`, row);
    if (row.owner && row.game_id) {
      await this.state.storage.put(`gameid:${row.owner}:${row.game_id}`, row.content_hash);
    }
    const index = await this._getGamesIndex();
    const existing = index.findIndex(g => g.content_hash === row.content_hash);
    const prev = existing >= 0 ? index[existing] : {};
    const summary = {
      content_hash: row.content_hash, name: row.name, size: row.size,
      updated: row.updated, owner: row.owner, game_id: row.game_id,
      scope: row.scope, version: row.version, checksum: row.checksum,
      published: row.published, forked_from_hash: row.forked_from_hash || null,
      resources: row.resources,
      highscore: prev.highscore, highscore_player: prev.highscore_player,
    };
    if (existing >= 0) index[existing] = summary;
    else index.push(summary);
    await this.state.storage.put('games:index', index);
  }

  async _deleteGame(hash) {
    const row = await this._getGame(hash);
    if (!row) return;
    await this.state.storage.delete(`game:${hash}`);
    if (row.owner && row.game_id) {
      await this.state.storage.delete(`gameid:${row.owner}:${row.game_id}`);
    }
    const index = await this._getGamesIndex();
    await this.state.storage.put('games:index', index.filter(g => g.content_hash !== hash));
  }

  async _getUser(username) {
    return (await this.state.storage.get(`user:${username}`)) || null;
  }

  async _getUserByEmail(email) {
    const username = await this.state.storage.get(`user_email:${email.toLowerCase()}`);
    if (!username) return null;
    return this._getUser(username);
  }

  async _putUser(row) {
    await this.state.storage.put(`user:${row.username}`, row);
    if (row.email) await this.state.storage.put(`user_email:${row.email.toLowerCase()}`, row.username);
    const index = await this._getUsersIndex();
    const idx = index.findIndex(u => u.username === row.username);
    const summary = { username: row.username, email: row.email, role: row.role, created: row.created, last_login: row.last_login };
    if (idx >= 0) index[idx] = summary;
    else index.push(summary);
    await this.state.storage.put('users:index', index);
  }

  async _deleteUser(username) {
    const row = await this._getUser(username);
    if (!row) return;
    await this.state.storage.delete(`user:${username}`);
    if (row.email) await this.state.storage.delete(`user_email:${row.email.toLowerCase()}`);
    const index = await this._getUsersIndex();
    await this.state.storage.put('users:index', index.filter(u => u.username !== username));
  }

  async _getUsersIndex() {
    return (await this.state.storage.get('users:index')) || [];
  }

  async _getUserRole(username) {
    const u = await this._getUser(username);
    return u?.role || 'user';
  }

  async _getScore(gameHash) {
    return (await this.state.storage.get(`score:${gameHash}`)) || null;
  }

  async _putScore(gameHash, score, player) {
    const updated = new Date().toISOString();
    await this.state.storage.put(`score:${gameHash}`, { game_hash: gameHash, score, player, updated });
    const index = await this._getGamesIndex();
    const idx = index.findIndex(g => g.content_hash === gameHash);
    if (idx >= 0) {
      index[idx].highscore = score;
      index[idx].highscore_player = player;
      await this.state.storage.put('games:index', index);
    }
  }

  async _getAllScores() {
    const map = await this.state.storage.list({ prefix: 'score:' });
    return Array.from(map.values());
  }

  async _getResource(gameHash, path) {
    return (await this.state.storage.get(`res:${gameHash}:${encodeURIComponent(path)}`)) || null;
  }

  async _putResources(gameHash, resources) {
    const hash = String(gameHash || '').trim();
    if (!hash) return 0;
    const normalized = normalizeResourcesMap(resources);
    const entries = Object.entries(normalized);
    if (!entries.length) return 0;
    const batch = {};
    for (const [path, value] of entries) {
      batch[`res:${hash}:${encodeURIComponent(path)}`] = value;
    }
    const existing = (await this.state.storage.get(`res_paths:${hash}`)) || [];
    const pathSet = new Set([...existing, ...Object.keys(normalized)]);
    batch[`res_paths:${hash}`] = Array.from(pathSet);
    await this.state.storage.put(batch);
    return entries.length;
  }

  async _getSecretKeys(username) {
    return (await this.state.storage.get(`secret_keys:${username}`)) || [];
  }

  async _getSecret(username, key) {
    return (await this.state.storage.get(`secret:${username}:${encodeURIComponent(key)}`)) || null;
  }

  async _putSecret(username, key, encryptedValue, updated) {
    await this.state.storage.put(`secret:${username}:${encodeURIComponent(key)}`, { value: encryptedValue, updated });
    const keys = await this._getSecretKeys(username);
    if (!keys.includes(key)) {
      keys.push(key);
      await this.state.storage.put(`secret_keys:${username}`, keys);
    }
  }

  async _deleteSecret(username, key) {
    await this.state.storage.delete(`secret:${username}:${encodeURIComponent(key)}`);
    const keys = (await this._getSecretKeys(username)).filter(k => k !== key);
    await this.state.storage.put(`secret_keys:${username}`, keys);
  }

  _trackFailedAuth(username) {
    const attempt = this.authAttempts.get(username) || { count: 0, lastAttempt: 0 };
    if (Date.now() - attempt.lastAttempt >= AUTH_COOLDOWN_MS) attempt.count = 0;
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

  normalizeExternalGameRow(row, includeResources = false) {
    const owner = normalizeSlug(row?.owner || 'public', 'public');
    const gameId = this.deriveGameId(row?.name || row?.content_hash || 'untitled', row?.game_id || '');
    const { resources: _raw, ...rest } = (row || {});
    const resources = parseResourcesField(_raw);
    const mapKeys = Object.keys(resources);
    const paths = mapKeys.length ? mapKeys.sort() : parseManifestField(_raw);
    const out = {
      ...rest,
      owner,
      game_id: gameId,
      scope: row?.scope || 'external',
      version: row?.version || '',
      checksum: row?.checksum || row?.content_hash || '',
      resource_paths: paths,
    };
    if (includeResources) out.resources = resources;
    return out;
  }

  broadcast(message) {
    for (const sock of this.state.getWebSockets()) {
      try { sock.send(message); } catch (_) {}
    }
  }

  getExternalPoolLimit() {
    const raw = Number(this.env?.EXTERNAL_POOL_SIZE || DEFAULT_EXTERNAL_POOL_SIZE);
    if (!Number.isFinite(raw) || raw < 1) return DEFAULT_EXTERNAL_POOL_SIZE;
    return Math.floor(raw);
  }

  async trimExternalPool() {
    const keep = this.getExternalPoolLimit();
    const index = await this._getGamesIndex();
    const external = index.filter(g => g.scope === 'external');
    if (external.length <= keep) return 0;
    external.sort((a, b) => b.updated.localeCompare(a.updated)); // newest first
    const toDelete = external.slice(keep);
    for (const g of toDelete) await this._deleteGame(g.content_hash);
    return toDelete.length;
  }

  // ── GitHub publishing ─────────────────────────────────────────────────────
  async publishToGitHub(row, token, repo) {
    const owner = normalizeSlug(row.owner, 'public');
    const gameId = normalizeSlug(row.game_id, 'game');
    const BASE = `https://api.github.com/repos/${repo}/contents`;
    const headers = {
      Authorization: `token ${token}`,
      'Content-Type': 'application/json',
      'User-Agent': 'slob-games-relay/1.0',
      Accept: 'application/vnd.github.v3+json',
    };

    // 1. Write games/{owner}/{game_id}.json (full content)
    const resources = parseResourcesField(row.resources);
    const gamePayload = {
      name: row.name,
      content: row.content,
      version: row.version || '',
      checksum: row.checksum || row.content_hash,
      content_hash: row.content_hash,
      owner,
      game_id: gameId,
      updated: row.updated,
      size: row.size,
      resources,
    };
    const gamePath = `games/${owner}/${gameId}.json`;
    const gameContent = btoa(unescape(encodeURIComponent(JSON.stringify(gamePayload, null, 2))));

    const existResp = await fetch(`${BASE}/${gamePath}`, { headers });
    const existData = existResp.ok ? await existResp.json().catch(() => ({})) : {};
    const gameSha = existData.sha;

    const gamePut = await fetch(`${BASE}/${gamePath}`, {
      method: 'PUT',
      headers,
      body: JSON.stringify({
        message: `publish: ${owner}/${gameId} v${row.version || 'latest'}`,
        content: gameContent,
        ...(gameSha ? { sha: gameSha } : {}),
      }),
    });
    if (!gamePut.ok) {
      const err = await gamePut.json().catch(() => ({}));
      throw new Error(`GitHub game write failed (${gamePut.status}): ${err.message || ''}`);
    }

    // 2. Update games/index.json — retry once on 409 SHA conflict
    for (let attempt = 0; attempt < 2; attempt++) {
      const idxResp = await fetch(`${BASE}/games/index.json`, { headers });
      const idxData = idxResp.ok ? await idxResp.json().catch(() => ({})) : {};
      const idxSha = idxData.sha;
      let index = { games: [] };
      if (idxData.content) {
        try { index = JSON.parse(decodeURIComponent(escape(atob(idxData.content.replace(/\n/g, ''))))); } catch (_) {}
      }
      const entry = {
        id: `${owner}/${gameId}`,
        owner,
        game_id: gameId,
        name: row.name,
        checksum: row.checksum || row.content_hash,
        content_hash: row.content_hash,
        size: row.size,
        updated: row.updated,
        version: row.version || '',
      };
      const idx = index.games.findIndex(g => g.id === entry.id);
      if (idx >= 0) index.games[idx] = entry;
      else index.games.push(entry);
      index.games.sort((a, b) => a.id.localeCompare(b.id));

      const idxContent = btoa(unescape(encodeURIComponent(JSON.stringify(index, null, 2))));
      const idxPut = await fetch(`${BASE}/games/index.json`, {
        method: 'PUT',
        headers,
        body: JSON.stringify({
          message: `index: upsert ${owner}/${gameId}`,
          content: idxContent,
          ...(idxSha ? { sha: idxSha } : {}),
        }),
      });
      if (idxPut.ok) break;
      if (idxPut.status === 409 && attempt === 0) continue; // SHA conflict, retry with fresh SHA
      const err = await idxPut.json().catch(() => ({}));
      throw new Error(`GitHub index write failed (${idxPut.status}): ${err.message || ''}`);
    }

    const rawBase = `https://raw.githubusercontent.com/${repo}/main`;
    return {
      ok: true,
      raw_url: `${rawBase}/${gamePath}`,
      index_url: `${rawBase}/games/index.json`,
    };
  }

  async fetch(request) {
    try {
      return await this._fetch(request);
    } catch(e) {
      console.error('[DO fetch]', String(e && e.message), String(e && e.stack).slice(0,200));
      throw e;
    }
  }

  async _fetch(request) {
    const url = new URL(request.url);

    // ── REST API (non-WebSocket) ──
    if (request.headers.get("Upgrade") !== "websocket") {
      // /health responds immediately — no DB work, confirms DO is alive.
      if (url.pathname === '/health') return json({ ok: true });

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

        const existUser = await this._getUser(username);
        const existEmail = await this._getUserByEmail(email);
        if (existUser || existEmail) return json({ error: "username or email already exists" }, 409);

        const salt = generateSalt();
        const hashed = await pbkdf2Hash(password, salt);
        const created = new Date().toISOString();
        await this._putUser({ username, email, password_hash: hashed, salt, role: 'user', created, last_login: '' });
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

        const row = await this._getUser(username);
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
            await this._putUser({ ...row, password_hash: newHash, salt: newSalt });
          }
        }

        if (!valid) {
          this._trackFailedAuth(username);
          return json({ error: "invalid credentials" }, 401);
        }

        // Success — clear rate limit counter and update last_login
        this.authAttempts.delete(username);
        await this._putUser({ ...row, last_login: new Date().toISOString() });
        const token = await signUserToken(username, new URL(request.url).origin, this.env.RELAY_SECRET);
        return json({ ok: true, username, token, role: row.role || 'user' });
      }

      // GET /auth/me — get current user info (including role)
      if (url.pathname === "/auth/me" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const row = await this._getUser(user);
        if (!row) return json({ error: "user not found" }, 404);
        const { password_hash, salt, ...safe } = row;
        return json({ ok: true, ...safe });
      }

      // POST /auth/refresh — issue a fresh token (extends session)
      if (url.pathname === "/auth/refresh" && request.method === "POST") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const token = await signUserToken(user, new URL(request.url).origin, this.env.RELAY_SECRET);
        return json({ ok: true, token, username: user });
      }

      // GET /auth/secrets — get all secrets for authenticated user (decrypted)
      if (url.pathname === "/auth/secrets" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (!this.env.RELAY_SECRET) return json({ error: "encryption not configured" }, 503);
        const keys = await this._getSecretKeys(user);
        const secrets = [];
        for (const key of keys) {
          const s = await this._getSecret(user, key);
          if (!s) continue;
          try {
            const val = await decryptSecret(s.value, this.env.RELAY_SECRET, user);
            secrets.push({ key, value: val, updated: s.updated });
          } catch (_) {
            secrets.push({ key, value: null, updated: s.updated, error: 'decrypt failed' });
          }
        }
        return json(secrets);
      }

      // PUT /auth/secrets/:key — store a secret (encrypted)
      const authSecretPut = url.pathname.match(/^\/auth\/secrets\/([^/]+)$/);
      if (authSecretPut && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (!this.env.RELAY_SECRET) return json({ error: "encryption not configured" }, 503);
        const key = decodeURIComponent(authSecretPut[1]);
        if (!key || key.length > 100) return json({ error: "invalid key" }, 400);
        const body = await request.json().catch(() => ({}));
        const value = String(body.value || '');
        if (!value) return json({ error: "value required" }, 400);
        const encrypted = await encryptSecret(value, this.env.RELAY_SECRET, user);
        await this._putSecret(user, key, encrypted, new Date().toISOString());
        return json({ ok: true, key });
      }

      // DELETE /auth/secrets/:key — delete a secret
      const authSecretDel = url.pathname.match(/^\/auth\/secrets\/([^/]+)$/);
      if (authSecretDel && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const key = decodeURIComponent(authSecretDel[1]);
        await this._deleteSecret(user, key);
        return json({ ok: true, deleted: key });
      }

      // ── Admin endpoints (require admin role) ──

      // GET /admin/users — list all registered users
      if (url.pathname === "/admin/users" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const index = await this._getUsersIndex();
        index.sort((a, b) => (a.created || '').localeCompare(b.created || ''));
        return json(index);
      }

      // GET /admin/games — list all games (external + internal) with owner info + high scores
      if (url.pathname === "/admin/games" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const index = await this._getGamesIndex();
        const external = index.sort((a, b) => `${a.owner}/${a.name}`.localeCompare(`${b.owner}/${b.name}`));
        return json({ external, internal: [] });
      }

      // GET /admin/stats — relay diagnostics (admin only)
      if (url.pathname === "/admin/stats" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);

        const index = await this._getGamesIndex();
        const externalGames = index.filter(g => g.scope === 'external').length;
        const internalGames = index.filter(g => g.scope === 'internal').length;
        const publishedGames = index.filter(g => g.published).length;
        const draftGames = index.filter(g => !g.published).length;
        const totalGameContentBytes = index.reduce((s, g) => s + (g.size || 0), 0);
        const usersIndex = await this._getUsersIndex();
        const scores = await this._getAllScores();
        const externalPoolLimit = this.getExternalPoolLimit();

        return json({
          ok: true,
          generated_at: new Date().toISOString(),
          runtime: {
            active_websockets: this.state.getWebSockets().length,
            auth_rate_limit_entries: this.authAttempts.size,
          },
          games: {
            total: index.length,
            internal: internalGames,
            external: externalGames,
            published: publishedGames,
            draft: draftGames,
            external_pool_limit: externalPoolLimit,
            external_over_limit: Math.max(0, externalGames - externalPoolLimit),
            content_bytes_total: totalGameContentBytes,
          },
          resources: { cached_rows: 0, cached_bytes_total: 0 },
          users: { total: usersIndex.length, highscore_rows: scores.length },
          storage: {
            sqlite_page_count: 0,
            sqlite_page_size: 0,
            sqlite_db_bytes_approx: 0,
          },
        });
      }

      // DELETE /admin/users/:username — delete a user (admin only, cannot delete self)
      const adminUserDelete = url.pathname.match(/^\/admin\/users\/([^/]+)$/);
      if (adminUserDelete && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserDelete[1]);
        if (target === user) return json({ error: "cannot delete yourself" }, 400);
        if (!(await this._getUser(target))) return json({ error: "user not found" }, 404);
        await this._deleteUser(target);
        return json({ ok: true, deleted: target });
      }

      // PUT /admin/users/:username — edit user role/email (admin only)
      const adminUserEdit = url.pathname.match(/^\/admin\/users\/([^/]+)$/);
      if (adminUserEdit && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const role = await this._getUserRole(user);
        if (role !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserEdit[1]);
        const body = await request.json().catch(() => ({}));
        const exists = await this._getUser(target);
        if (!exists) return json({ error: "user not found" }, 404);
        const updated = { ...exists };
        if (body.role && ['user', 'admin'].includes(body.role)) updated.role = body.role;
        if (body.email && typeof body.email === 'string' && body.email.includes('@')) updated.email = body.email;
        if (body.password && typeof body.password === 'string' && body.password.length >= 4) {
          const newSalt = crypto.randomUUID();
          const newHash = await pbkdf2Hash(body.password, newSalt);
          updated.password_hash = newHash;
          updated.salt = newSalt;
        }
        await this._putUser(updated);
        return json({ ok: true, updated: target });
      }

      // DELETE /admin/games/:hash — delete a game (external or internal by content_hash)
      const adminGameDelete = url.pathname.match(/^\/admin\/games\/([^/]+)$/);
      if (adminGameDelete && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const hash = decodeURIComponent(adminGameDelete[1]);
        if (!(await this._getGame(hash))) return json({ error: "game not found" }, 404);
        await this._deleteGame(hash);
        this.broadcast(JSON.stringify({ type: 'game-deleted', content_hash: hash }));
        return json({ ok: true, deleted: hash });
      }

      // PUT /admin/games/:hash/assign — change game owner (admin only)
      const adminGameAssign = url.pathname.match(/^\/admin\/games\/([^/]+)\/assign$/);
      if (adminGameAssign && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const hash = decodeURIComponent(adminGameAssign[1]);
        const body = await request.json().catch(() => ({}));
        const newOwner = (body.owner || '').trim();
        if (!newOwner) return json({ error: "owner required" }, 400);
        const game = await this._getGame(hash);
        if (!game) return json({ error: "game not found" }, 404);
        await this._putGame({ ...game, owner: newOwner });
        return json({ ok: true, hash, owner: newOwner });
      }

      // GET /admin/users/:username/secrets — list user's secret keys (admin only, no values)
      const adminUserSecrets = url.pathname.match(/^\/admin\/users\/([^/]+)\/secrets$/);
      if (adminUserSecrets && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserSecrets[1]);
        const keys = await this._getSecretKeys(target);
        const rows = [];
        for (const key of keys) {
          const s = await this._getSecret(target, key);
          if (s) rows.push({ key, updated: s.updated });
        }
        return json(rows);
      }

      // PUT /admin/users/:username/secrets/:key — set a secret for any user (admin only)
      const adminUserSecretPut = url.pathname.match(/^\/admin\/users\/([^/]+)\/secrets\/([^/]+)$/);
      if (adminUserSecretPut && request.method === "PUT") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        if (!this.env.RELAY_SECRET) return json({ error: "encryption not configured" }, 503);
        const target = decodeURIComponent(adminUserSecretPut[1]);
        const key = decodeURIComponent(adminUserSecretPut[2]);
        if (!key || key.length > 100) return json({ error: "invalid key" }, 400);
        const body = await request.json().catch(() => ({}));
        const value = String(body.value || '');
        if (!value) return json({ error: "value required" }, 400);
        const encrypted = await encryptSecret(value, this.env.RELAY_SECRET, target);
        await this._putSecret(target, key, encrypted, new Date().toISOString());
        return json({ ok: true, username: target, key });
      }

      // DELETE /admin/users/:username/secrets/:key — delete a user's secret (admin only)
      const adminUserSecretDel = url.pathname.match(/^\/admin\/users\/([^/]+)\/secrets\/([^/]+)$/);
      if (adminUserSecretDel && request.method === "DELETE") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        if (await this._getUserRole(user) !== 'admin') return json({ error: "admin required" }, 403);
        const target = decodeURIComponent(adminUserSecretDel[1]);
        const key = decodeURIComponent(adminUserSecretDel[2]);
        await this._deleteSecret(target, key);
        return json({ ok: true, username: target, deleted: key });
      }

      // GET /config/github-catalog — returns GitHub raw index URL if GITHUB_REPO is set
      if (url.pathname === '/config/github-catalog' && request.method === 'GET') {
        const repo = this.env.GITHUB_REPO || '';
        const catalogUrl = repo
          ? `https://raw.githubusercontent.com/${repo}/main/games/index.json`
          : null;
        return json({ url: catalogUrl, repo: repo || null });
      }

      // GET /games — list all published games (with high score)
      if (url.pathname === "/games" && request.method === "GET") {
        const index = await this._getGamesIndex();
        const rows = index.filter(g => g.published).map(r => this.normalizeExternalGameRow(r));
        rows.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
        return json(rows);
      }

      // GET /games.toml — export published game manifests as TOML
      if (url.pathname === '/games.toml' && request.method === 'GET') {
        const index = await this._getGamesIndex();
        const rows = index.filter(g => g.published).map(r => this.normalizeExternalGameRow(r));
        rows.sort((a, b) => `${a.owner}/${a.game_id}`.localeCompare(`${b.owner}/${b.game_id}`));
        const out = rows.map((g) => [
          '[[game]]',
          `id = "${g.owner}/${g.game_id}"`,
          `name = "${String(g.name || '').replace(/"/g, '\\"')}"`,
          `owner = "${g.owner}"`,
          `game_id = "${g.game_id}"`,
          `version = "${g.version || ''}"`,
          `checksum = "${g.checksum || g.content_hash}"`,
          `content_hash = "${g.content_hash}"`,
          `published = true`,
          `size = ${Number(g.size || 0)}`,
          `updated = "${g.updated}"`,
        ].join('\n')).join('\n\n');
        return new Response(out, { status: 200, headers: { 'Content-Type': 'text/plain; charset=utf-8', ...cors() } });
      }

      // GET /internal/games — list authenticated user's games (with high score)
      if (url.pathname === "/internal/games" && request.method === "GET") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const index = await this._getGamesIndex();
        const rows = index.filter(g => g.owner === user);
        rows.sort((a, b) => b.updated.localeCompare(a.updated));
        return json(rows);
      }

      // POST /internal/score — submit a high score (auth required, server forces player=user)
      // Body: { game_hash, score }
      if (url.pathname === "/internal/score" && request.method === "POST") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const body = await request.json().catch(() => ({}));
        const gameHash = String(body.game_hash || "").trim().toLowerCase();
        const incoming = Math.floor(Number(body.score));
        if (!/^[0-9a-f]{8,128}$/.test(gameHash)) return json({ error: "invalid game_hash" }, 400);
        if (!Number.isFinite(incoming) || incoming < 0 || incoming > 1e9) {
          return json({ error: "invalid score" }, 400);
        }
        // Rate limit: 1 submission / 2s per user+game
        const rlKey = `rl:score:${user}:${gameHash}`;
        const last = (await this.state.storage.get(rlKey)) || 0;
        const now = Date.now();
        if (now - last < 2000) return json({ error: "rate limited" }, 429);
        await this.state.storage.put(rlKey, now);
        // Only write if higher than existing
        const existing = await this._getScore(gameHash);
        if (existing && incoming <= existing.score) {
          return json({ ok: true, accepted: false, current: existing });
        }
        await this._putScore(gameHash, incoming, user);
        const msg = JSON.stringify({ type: "score-update", game_hash: gameHash, score: incoming, player: user });
        for (const sock of this.state.getWebSockets()) {
          try { sock.send(msg); } catch (_) {}
        }
        return json({ ok: true, accepted: true, score: incoming, player: user });
      }

      // PATCH /internal/game/:gameId/publish — toggle published flag
      const publishMatch = url.pathname.match(/^\/internal\/game\/([^/]+)\/publish$/);
      if (publishMatch && request.method === 'PATCH') {
        const user = await this.authUser(request);
        if (!user) return json({ error: 'auth required' }, 401);
        const gameId = normalizeSlug(publishMatch[1], '');
        if (!gameId) return json({ error: 'missing game id' }, 400);
        const body = await request.json().catch(() => ({}));

        const role = await this._getUserRole(user);
        const requestedOwner = normalizeSlug(
          request.headers.get('X-Game-Owner') || body.owner || user,
          user
        );
        const owner = requestedOwner || user;
        if (owner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);

        const row = await this._getGameByOwnerGameId(owner, gameId);
        if (!row) return json({ error: 'not found' }, 404);
        const explicit = (typeof body.published === 'boolean') ? (body.published ? 1 : 0) : null;
        const newVal = (explicit === null) ? (row.published ? 0 : 1) : explicit;
        await this._putGame({ ...row, published: newVal });
        if (newVal === 0) {
          // Notify clients to remove unpublished game from their catalog
          this.broadcast(JSON.stringify({ type: 'game-deleted', content_hash: row.content_hash }));
        } else {
          // Re-broadcast the game to all clients
          const full = await this._getGame(row.content_hash);
          if (full) {
            const norm = this.normalizeExternalGameRow(full, true);
            this.broadcast(JSON.stringify({ type: 'sync', games: [norm] }));
          }
        }
        return json({ ok: true, owner, game_id: gameId, published: !!newVal });
      }

      // PATCH /internal/game/:gameId/github-publish — mirror game to GitHub
      const githubPublishMatch = url.pathname.match(/^\/internal\/game\/([^/]+)\/github-publish$/);
      if (githubPublishMatch && request.method === 'PATCH') {
        const user = await this.authUser(request);
        if (!user) return json({ error: 'auth required' }, 401);
        const gameId = normalizeSlug(githubPublishMatch[1], '');
        if (!gameId) return json({ error: 'missing game id' }, 400);

        const githubToken = this.env.GITHUB_TOKEN;
        const githubRepo = this.env.GITHUB_REPO;
        if (!githubToken || !githubRepo) return json({ error: 'GitHub publishing not configured on this relay' }, 503);

        const body = await request.json().catch(() => ({}));
        const requestedOwner = normalizeSlug(body.owner || user, user);
        const role = await this._getUserRole(user);
        if (requestedOwner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);

        const row = await this._getGameByOwnerGameId(requestedOwner, gameId);
        if (!row) return json({ error: 'game not found' }, 404);

        try {
          const result = await this.publishToGitHub(row, githubToken, githubRepo);
          return json(result);
        } catch (err) {
          return json({ error: String(err?.message || err) }, 502);
        }
      }

      // POST /internal/fork — fork a game into authenticated user's collection
      if (url.pathname === "/internal/fork" && request.method === "POST") {
        const user = await this.authUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const body = await request.json().catch(() => ({}));
        const sourceHash = String(body.source_hash || '').trim();
        if (!sourceHash) return json({ error: "source_hash required" }, 400);

        const src = await this._getGame(sourceHash);
        if (!src) return json({ error: "source game not found" }, 404);

        const gameId = this.deriveGameId(src.name, body.game_id);
        const version = String(body.version || makeReleaseVersion());
        const srcResources = parseResourcesField(src.resources);
        const checksum = await packageHash16(src.content);
        const updated = new Date().toISOString();
        const size = src.content.length;

        const prevFork = await this._getGameByOwnerGameId(user, gameId);
        if (prevFork) await this._deleteGame(prevFork.content_hash);
        await this._putGame({
          content_hash: checksum, name: String(body.name || src.name).slice(0, 100), content: src.content,
          updated, size, owner: user, game_id: gameId, scope: 'internal', version,
          checksum, resources: encodeResourcesField(srcResources), forked_from_hash: sourceHash, published: 0,
        });
        await this._putResources(checksum, srcResources);
        return json({ ok: true, owner: user, game_id: gameId, forked_from_hash: sourceHash, checksum, version });
      }

      // GET /game/:hash — full HTML content of one game
      if (url.pathname.startsWith("/game/") && request.method === "GET") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const row = await this._getGame(hash);
        if (!row) return json({ error: "not found" }, 404);
        return json(this.normalizeExternalGameRow(row, true));
      }

      // PUT /game/:hash — update game content, broadcast to all connected clients
      if (url.pathname.startsWith("/game/") && request.method === "PUT") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        const existing = await this._getGame(hash);
        if (!existing) return json({ error: "not found" }, 404);

        const body = await request.json();
        const content = body.content;
        if (!content || typeof content !== 'string') return json({ error: "missing content" }, 400);
        if (content.length > MAX_GAME_SIZE) return json({ error: "too large" }, 413);

        const existingResources = parseResourcesField(existing[0].resources);
        const incomingResources = (body.resources === undefined)
          ? existingResources
          : parseResourcesField(body.resources);
        const paths = resourcePaths(incomingResources);
        const resourcePayloadBytes = resourceBytes(incomingResources);
        const packageSize = content.length + resourcePayloadBytes;
        if (packageSize > MAX_GAME_PACKAGE_SIZE) {
          return json({ error: "package too large" }, 413);
        }
        const size = content.length;

        // Compute new hash (content only, resources are P2P metadata)
        const newHash = await packageHash16(content);
        const name = body.name || existing[0].name;
        const owner = normalizeSlug(body.owner || 'public', 'public');
        const gameId = this.deriveGameId(name, body.game_id);
        const version = String(body.version || makeReleaseVersion());
        const updated = new Date().toISOString();

        // Delete old row and any duplicates by name/owner+gameId in external scope
        await this._deleteGame(hash);
        const curIndex = await this._getGamesIndex();
        for (const g of curIndex.filter(g => g.name === name.slice(0, 100) && g.scope === 'external' && g.content_hash !== newHash)) {
          await this._deleteGame(g.content_hash);
        }
        for (const g of (await this._getGamesIndex()).filter(g => g.owner === owner && g.game_id === gameId && g.scope === 'external' && g.content_hash !== newHash)) {
          await this._deleteGame(g.content_hash);
        }
        await this._putGame({
          content_hash: newHash, name: name.slice(0, 100), content, updated, size,
          owner, game_id: gameId, scope: 'external', version, checksum: newHash,
          resources: encodeResourcesField(incomingResources), published: 1,
        });
        await this._putResources(newHash, incomingResources);
        await this.trimExternalPool();

        // Broadcast updated game to all connected WebSocket clients
        const msg = JSON.stringify({
          type: 'sync',
          games: [{
            content_hash: newHash, checksum: newHash, owner, game_id: gameId,
            scope: 'external', version, name: name.slice(0, 100), content,
            resources: incomingResources, resource_paths: paths, updated,
          }]
        });
        for (const sock of this.state.getWebSockets()) {
          try { sock.send(msg); } catch (_) {}
        }

        return json({ ok: true, old_hash: hash, content_hash: newHash, name, size });
      }

      // GET /scores — all high scores
      if (url.pathname === "/scores" && request.method === "GET") {
        return json(await this._getAllScores());
      }

      // DELETE /game/:hash — remove a game
      if (url.pathname.startsWith("/game/") && request.method === "DELETE") {
        const hash = url.pathname.slice(6);
        if (!hash) return json({ error: "missing hash" }, 400);
        if (!(await this._getGame(hash))) return json({ error: "not found" }, 404);
        await this._deleteGame(hash);
        return json({ ok: true, deleted: hash });
      }

      // PUT /internal/game/:gameId — save authenticated user's game
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
        const version = String(body.version || makeReleaseVersion());
        const updated = new Date().toISOString();

        const prev = await this._getGameByOwnerGameId(user, gameId);
        const prevResources = parseResourcesField(prev?.resources);
        const resourcesMap = (body.resources === undefined)
          ? prevResources
          : parseResourcesField(body.resources);
        const paths = resourcePaths(resourcesMap);
        const resourcePayloadBytes = resourceBytes(resourcesMap);
        const packageSize = content.length + resourcePayloadBytes;
        if (packageSize > MAX_GAME_PACKAGE_SIZE) return json({ error: 'package too large' }, 413);
        const prevPublished = prev ? (prev.published ?? 0) : 0;
        const size = content.length;
        const checksum = await packageHash16(content);
        const nextPublished = prevPublished;
        const nextScope = String(body.scope || (prev && prev.scope) || 'internal').trim().toLowerCase() === 'external' ? 'external' : 'internal';

        if (prev) await this._deleteGame(prev.content_hash);
        await this._putGame({
          content_hash: checksum, name, content, updated, size, owner: user, game_id: gameId,
          scope: nextScope, version, checksum,
          resources: encodeResourcesField(resourcesMap),
          forked_from_hash: (prev?.forked_from_hash) || null,
          published: nextPublished,
        });
        await this._putResources(checksum, resourcesMap);
        await this.trimExternalPool();

        // Publish state is authoritative on the private row itself.
        // Content edits do not implicitly change published/draft.
        if (nextPublished) {
          const msg = JSON.stringify({
            type: 'sync',
            games: [{
              content_hash: checksum,
              checksum,
              owner: user,
              game_id: gameId,
              scope: nextScope,
              version,
              name,
              content,
              resources: resourcesMap,
              resource_paths: paths,
              updated
            }]
          });
          for (const sock of this.state.getWebSockets()) {
            try { sock.send(msg); } catch (_) {}
          }
        }

        return json({ ok: true, owner: user, game_id: gameId, content_hash: checksum, checksum, version, published: !!nextPublished });
      }

      // GET /internal/game/:gameId — get full content of authenticated user's game
      if (url.pathname.startsWith('/internal/game/') && request.method === 'GET') {
        const user = await this.authUser(request);
        if (!user) return json({ error: 'auth required' }, 401);
        const gameId = normalizeSlug(url.pathname.slice('/internal/game/'.length), '');
        if (!gameId) return json({ error: 'missing game id' }, 400);
        const owner = url.searchParams.get('owner') || user;
        const row = await this._getGameByOwnerGameId(owner, gameId);
        if (!row) return json({ error: 'not found' }, 404);
        const resources = parseResourcesField(row.resources);
        return json({ ...row, resource_paths: Object.keys(resources).sort(), resources });
      }

      // DELETE /internal/game/:gameId — delete authenticated user's game
      if (url.pathname.startsWith('/internal/game/') && request.method === 'DELETE') {
        const user = await this.authUser(request);
        if (!user) return json({ error: 'auth required' }, 401);
        const gameId = normalizeSlug(url.pathname.slice('/internal/game/'.length), '');
        if (!gameId) return json({ error: 'missing game id' }, 400);
        const owner = url.searchParams.get('owner') || user;
        const role = await this._getUserRole(user);
        if (owner !== user && role !== 'admin') return json({ error: 'forbidden' }, 403);
        const row = await this._getGameByOwnerGameId(owner, gameId);
        if (!row) return json({ error: 'not found' }, 404);
        await this._deleteGame(row.content_hash);
        return json({ ok: true, deleted: gameId });
      }

      return json({ error: "WebSocket upgrade required or use REST endpoints" }, 426);
    }

    // ── WebSocket upgrade ──
    const pair = new WebSocketPair();
    this.state.acceptWebSocket(pair[1]);
    const index = await this._getGamesIndex();
    const hashes = index.filter(g => g.published).map(g => g.content_hash);
    pair[1].send(JSON.stringify({ type: 'catalog', hashes }));

    // Send all high scores to the new client
    const scoreRows = await this._getAllScores();
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
        const wanted = data.hashes.slice(0, 50);
        const gameRows = [];
        for (const h of wanted) {
          const row = await this._getGame(h);
          if (row && row.published) gameRows.push(this.normalizeExternalGameRow(row, true));
        }
        if (gameRows.length > 0) ws.send(JSON.stringify({ type: 'games', games: gameRows }));
        break;
      }

      case 'push': {
        // Anonymous push disabled — all games managed via auth'd REST API
        ws.send(JSON.stringify({ type: 'ack', added: 0 }));
        break;
      }

      case 'score': {
        // Client reports a high score: { game_hash, score, player? }
        if (!data.game_hash || typeof data.game_hash !== 'string') return;
        const incoming = Math.floor(Number(data.score));
        if (!Number.isFinite(incoming) || incoming < 0) return;
        const player = (typeof data.player === 'string' ? data.player : '').slice(0, 50);

        // Only store if it's higher than existing (or same score adding player info)
        const existing = await this._getScore(data.game_hash);
        if (existing) {
          const dominated = incoming < existing.score ||
            (incoming === existing.score && (!player || existing.player));
          if (dominated) {
            ws.send(JSON.stringify({ type: 'score-update', game_hash: data.game_hash, score: existing.score, player: existing.player || '' }));
            return;
          }
        }

        await this._putScore(data.game_hash, incoming, player);

        // Broadcast new high score to ALL clients (including sender)
        const msg = JSON.stringify({ type: 'score-update', game_hash: data.game_hash, score: incoming, player });
        for (const sock of this.state.getWebSockets()) {
          try { sock.send(msg); } catch (_) {}
        }
        break;
      }

      case 'need-resources': {
        // Resource request — serve from durable cache first, then forward misses to peers.
        const nonce = data.nonce || '';
        const items = Array.isArray(data.items) ? data.items.slice(0, 64) : [];
        const legacyPaths = Array.isArray(data.paths) ? data.paths.slice(0, 50) : [];
        const hits = [];
        const misses = [];

        if (items.length > 0) {
          for (const item of items) {
            const hash = String(item && item.game_hash || '').trim();
            const path = String(item && item.path || '').trim();
            if (!hash || !path) continue;
            const value = await this._getResource(hash, path);
            if (value) {
              hits.push({ game_hash: hash, path, value });
            } else {
              misses.push({ game_hash: hash, path });
            }
          }
        }

        if (hits.length > 0) {
          ws.send(JSON.stringify({ type: 'have-resources', nonce, items: hits }));
        }

        if (misses.length === 0 && legacyPaths.length === 0) return;

        const fwd = JSON.stringify({
          type: 'need-resources',
          nonce,
          items: misses,
          paths: legacyPaths,
        });
        for (const sock of this.state.getWebSockets()) {
          if (sock !== ws) try { sock.send(fwd); } catch (_) {}
        }
        break;
      }

      case 'have-resources': {
        // P2P resource response — persist to durable cache and forward to others.
        if (!data.nonce) return;
        const items = Array.isArray(data.items) ? data.items : [];
        if (items.length > 0) {
          const grouped = {};
          for (const item of items) {
            const hash = String(item && item.game_hash || '').trim();
            const path = String(item && item.path || '').trim();
            const value = String(item && item.value || '');
            if (!hash || !path || !value) continue;
            if (!grouped[hash]) grouped[hash] = {};
            grouped[hash][path] = value;
          }
          for (const hash of Object.keys(grouped)) {
            await this._putResources(hash, grouped[hash]);
          }
        }
        // Limit forwarded payload to ~900KB to stay under WS frame limits
        const raw = JSON.stringify(data);
        if (raw.length > 900_000) return;
        for (const sock of this.state.getWebSockets()) {
          if (sock !== ws) try { sock.send(raw); } catch (_) {}
        }
        break;
      }

      case 'need-game': {
        // P2P game request — check DB first, then forward to peers
        const hash = String(data.hash || '').trim();
        const nonce = String(data.nonce || '');
        if (!hash || !nonce) return;
        const gameRow = await this._getGame(hash);
        if (gameRow && gameRow.published && gameRow.content) {
          const norm = this.normalizeExternalGameRow(gameRow, true);
          ws.send(JSON.stringify({ type: 'have-game', nonce, hash, name: norm.name || '', content: norm.content || '' }));
        } else {
          // Forward to all other clients
          const fwd = JSON.stringify({ type: 'need-game', hash, nonce });
          for (const sock of this.state.getWebSockets()) {
            if (sock !== ws) try { sock.send(fwd); } catch (_) {}
          }
        }
        break;
      }

      case 'have-game': {
        // P2P game response from a peer — forward to all other clients
        const nonce = String(data.nonce || '');
        if (!nonce) return;
        const raw2 = JSON.stringify(data);
        if (raw2.length > 900_000) return;
        for (const sock of this.state.getWebSockets()) {
          if (sock !== ws) try { sock.send(raw2); } catch (_) {}
        }
        break;
      }
    }
  }

  webSocketClose(ws, code, reason) {}
  webSocketError(ws, error) {}
}

// Keep legacy class exports so existing DO dependencies can still load.
export class GameRoomV5 extends GameRoomV6 {}
export class GameRoomV4 extends GameRoomV6 {}
export class GameRoomV2 extends GameRoomV6 {}
export class GameRoom extends GameRoomV6 {}

// ── Main Worker ───────────────────────────────────────────────────────────────

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    // CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: cors() });
    }

    if (url.pathname === "/health") {
      return new Response("ok", { headers: cors() });
    }

    // Auth routes — handled directly in Worker using AUTH_KV (bypasses broken DO)
    // Also handles /sync/auth/* (frontend uses these paths)
    if (url.pathname.startsWith('/auth/') || url.pathname.startsWith('/sync/auth/') || url.pathname === '/config/github-catalog') {
      if (!env.AUTH_KV) return json({ error: "auth storage not configured" }, 503);
      if (!env.RELAY_SECRET) return json({ error: "RELAY_SECRET not configured" }, 503);

      // Normalize /sync/auth/* → /auth/* for routing below
      const authPath = url.pathname.startsWith('/sync/auth/')
        ? url.pathname.slice(5)  // strip '/sync'
        : url.pathname;

      // KV helpers scoped to AUTH_KV
      const kvGetUser = async (username) => {
        return (await env.AUTH_KV.get(`user:${username}`, { type: 'json' })) || null;
      };
      const kvGetUserByEmail = async (email) => {
        const username = await env.AUTH_KV.get(`user_email:${email.toLowerCase()}`);
        if (!username) return null;
        return kvGetUser(username);
      };
      const kvPutUser = async (row) => {
        await env.AUTH_KV.put(`user:${row.username}`, JSON.stringify(row));
        if (row.email) await env.AUTH_KV.put(`user_email:${row.email.toLowerCase()}`, row.username);
        const rawIndex = await env.AUTH_KV.get('users:index', { type: 'json' });
        const index = rawIndex || [];
        const summary = { username: row.username, email: row.email, role: row.role, created: row.created, last_login: row.last_login };
        const idx = index.findIndex(u => u.username === row.username);
        if (idx >= 0) index[idx] = summary; else index.push(summary);
        await env.AUTH_KV.put('users:index', JSON.stringify(index));
      };
      const kvAuthUser = async (req) => {
        const authHeader = req.headers.get('Authorization') || '';
        const token = authHeader.startsWith('Bearer ') ? authHeader.slice(7).trim() : null;
        if (!token) return null;
        const payload = await verifyUserToken(token, env.RELAY_SECRET);
        return payload?.sub || null;
      };

      // POST /auth/register
      if (authPath === '/auth/register' && request.method === 'POST') {
        const body = await request.json().catch(() => ({}));
        const username = normalizeSlug(body.username || '', '');
        const email = String(body.email || '').trim().toLowerCase();
        const password = String(body.password || '');
        if (!username || username.length < 3) return json({ error: "username must be at least 3 chars" }, 400);
        if (!/^\S+@\S+\.\S+$/.test(email)) return json({ error: "invalid email" }, 400);
        if (password.length < 6) return json({ error: "password must be at least 6 chars" }, 400);
        const existUser = await kvGetUser(username);
        const existEmail = await kvGetUserByEmail(email);
        if (existUser || existEmail) return json({ error: "username or email already exists" }, 409);
        const salt = generateSalt();
        const hashed = await pbkdf2Hash(password, salt);
        const created = new Date().toISOString();
        await kvPutUser({ username, email, password_hash: hashed, salt, role: 'user', created, last_login: '' });
        const token = await signUserToken(username, url.origin, env.RELAY_SECRET);
        return json({ ok: true, username, token, role: 'user' });
      }

      // POST /auth/login
      if (authPath === '/auth/login' && request.method === 'POST') {
        const body = await request.json().catch(() => ({}));
        const username = normalizeSlug(body.username || '', '');
        const password = String(body.password || '');
        if (!username || !password) return json({ error: "username and password required" }, 400);
        const row = await kvGetUser(username);
        if (!row) return json({ error: "invalid credentials" }, 401);
        let valid = false;
        if (row.salt) {
          valid = (await pbkdf2Hash(password, row.salt)) === row.password_hash;
        } else {
          const hashed = await legacyPasswordHash(username, password, env.RELAY_SECRET);
          valid = (hashed === row.password_hash);
          if (valid) {
            const newSalt = generateSalt();
            const newHash = await pbkdf2Hash(password, newSalt);
            await kvPutUser({ ...row, password_hash: newHash, salt: newSalt });
          }
        }
        if (!valid) return json({ error: "invalid credentials" }, 401);
        await kvPutUser({ ...row, last_login: new Date().toISOString() });
        const token = await signUserToken(username, url.origin, env.RELAY_SECRET);
        return json({ ok: true, username, token, role: row.role || 'user' });
      }

      // GET /auth/me
      if (authPath === '/auth/me' && request.method === 'GET') {
        const user = await kvAuthUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const row = await kvGetUser(user);
        if (!row) return json({ error: "user not found" }, 404);
        const { password_hash, salt, ...safe } = row;
        return json({ ok: true, ...safe });
      }

      // POST /auth/refresh
      if (authPath === '/auth/refresh' && request.method === 'POST') {
        const user = await kvAuthUser(request);
        if (!user) return json({ error: "auth required" }, 401);
        const token = await signUserToken(user, url.origin, env.RELAY_SECRET);
        return json({ ok: true, token, username: user });
      }

      // GET /config/github-catalog
      if (authPath === '/config/github-catalog' && request.method === 'GET') {
        const repo = env.GITHUB_REPO || '';
        const catalogUrl = repo ? `https://raw.githubusercontent.com/${repo}/main/games/index.json` : null;
        return json({ url: catalogUrl, repo: repo || null });
      }

      return json({ error: "not found" }, 404);
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

    // /sync routes → global GameRoomV3
    if (url.pathname === "/sync" || url.pathname.startsWith("/sync/")) {
      // Handle /sync/health directly in Worker — DO may be unavailable
      if (url.pathname === "/sync/health") {
        return json({ ok: true });
      }

      // Handle /sync/games with GitHub catalog fallback when DO is unavailable
      if (url.pathname === "/sync/games" && request.method === "GET") {
        if (env.GITHUB_REPO) {
          try {
            const catUrl = `https://raw.githubusercontent.com/${env.GITHUB_REPO}/main/games/index.json`;
            const catRes = await fetch(catUrl, { headers: { 'User-Agent': 'slob-games-relay/1.0' } });
            if (catRes.ok) {
              const cat = await catRes.json();
              const games = (cat.games || []).map(g => ({
                owner: g.owner || 'public',
                game_id: g.game_id || '',
                name: g.name || '',
                checksum: g.checksum || g.content_hash || '',
                content_hash: g.content_hash || g.checksum || '',
                size: g.size || 0,
                updated: g.updated || '',
                version: g.version || '',
                scope: 'external',
                published: 1,
                resource_paths: [],
              }));
              return json(games);
            }
          } catch (_) {}
        }
        return json([]);
      }

      // Worker-level /sync/game/:hash → fetch single game JSON from GitHub by content_hash
      const syncGameMatch = url.pathname.match(/^\/sync\/game\/([^/]+)$/);      if (syncGameMatch && request.method === 'GET') {
        const hash = decodeURIComponent(syncGameMatch[1]).trim().toLowerCase();
        if (!hash) return json({ error: 'missing hash' }, 400);
        if (!env.GITHUB_REPO) return json({ error: 'catalog not configured' }, 503);
        try {
          const catUrl = `https://raw.githubusercontent.com/${env.GITHUB_REPO}/main/games/index.json`;
          const catRes = await fetch(catUrl, { headers: { 'User-Agent': 'slob-games-relay/1.0' } });
          if (!catRes.ok) return json({ error: 'catalog fetch failed' }, 502);
          const cat = await catRes.json();
          const entry = (cat.games || []).find(g =>
            String(g.content_hash || '').toLowerCase() === hash ||
            String(g.checksum || '').toLowerCase() === hash
          );
          if (!entry || !entry.owner || !entry.game_id) return json({ error: 'not found' }, 404);
          const fileUrl = `https://raw.githubusercontent.com/${env.GITHUB_REPO}/main/games/${entry.owner}/${entry.game_id}.json`;
          const fileRes = await fetch(fileUrl, { headers: { 'User-Agent': 'slob-games-relay/1.0' } });
          if (!fileRes.ok) return json({ error: 'game file fetch failed' }, 502);
          const game = await fileRes.json();
          // Normalize shape expected by frontend
          return json({
            content_hash: entry.content_hash || entry.checksum || hash,
            checksum: entry.checksum || entry.content_hash || hash,
            owner: entry.owner,
            game_id: entry.game_id,
            name: game.name || entry.name || '',
            content: game.content || '',
            version: game.version || entry.version || '',
            size: game.size || entry.size || 0,
            updated: game.updated || entry.updated || '',
            scope: 'external',
            published: 1,
            resources: game.resources || {},
            resource_paths: game.resources ? Object.keys(game.resources).sort() : [],
          });
        } catch (e) {
          return json({ error: String(e?.message || e) }, 502);
        }
      }

      // Forward /sync/internal/* to Worker-level handler (uses AUTH_KV instead of broken DO)
      if (url.pathname.startsWith('/sync/internal/')) {
        return handleInternalRoutes(url, request, env, ctx);
      }

      // ── /sync/github/* — public read endpoints for the dashboard ──
      // GET /sync/github/games → enriched index.json (includes published flag, sprite_count)
      if (url.pathname === '/sync/github/games' && request.method === 'GET') {
        if (!env.GITHUB_REPO) return json({ error: 'catalog not configured' }, 503);
        try {
          // Use the Contents API (not raw.githubusercontent.com) so we don't
          // hit aggressive CDN caching after deletes/edits.
          const apiUrl = `https://api.github.com/repos/${env.GITHUB_REPO}/contents/games/index.json`;
          const apiHeaders = {
            'User-Agent': 'slob-games-relay/1.0',
            Accept: 'application/vnd.github.v3+json',
            'Cache-Control': 'no-cache',
            ...(env.GITHUB_TOKEN ? { Authorization: `token ${env.GITHUB_TOKEN}` } : {}),
          };
          const r = await fetch(apiUrl, { headers: apiHeaders, cf: { cacheTtl: 0 } });
          if (!r.ok) return json({ games: [] });
          const d = await r.json().catch(() => ({}));
          let cat = { games: [] };
          if (d && d.content) {
            try { cat = JSON.parse(_b64Decode(d.content)); } catch (_) {}
          }
          return new Response(JSON.stringify(cat), {
            status: 200,
            headers: { 'Content-Type': 'application/json', 'Cache-Control': 'no-store', ...cors() },
          });
        } catch (e) {
          return json({ error: String(e?.message || e) }, 502);
        }
      }

      // GET /sync/github/sprites → list shared sprite files in games/_shared/sprites
      if (url.pathname === '/sync/github/sprites' && request.method === 'GET') {
        if (!env.GITHUB_REPO) return json({ error: 'catalog not configured' }, 503);
        try {
          const apiUrl = `https://api.github.com/repos/${env.GITHUB_REPO}/contents/games/_shared/sprites`;
          const apiHeaders = {
            'User-Agent': 'slob-games-relay/1.0',
            Accept: 'application/vnd.github.v3+json',
            ...(env.GITHUB_TOKEN ? { Authorization: `token ${env.GITHUB_TOKEN}` } : {}),
          };
          const r = await fetch(apiUrl, { headers: apiHeaders });
          if (!r.ok) return json({ files: [] });
          const items = await r.json().catch(() => []);
          const files = (Array.isArray(items) ? items : [])
            .filter(it => it && it.type === 'file' && it.name !== 'index.json')
            .map(it => ({
              name: it.name,
              path: it.path,
              size: it.size,
              sha: it.sha,
              download_url: it.download_url,
            }));
          return json({ files });
        } catch (e) {
          return json({ error: String(e?.message || e) }, 502);
        }
      }

      // GET /sync/github/games/:owner/:gameId/sprites → list sprite files for a game
      const ghSpritesMatch = url.pathname.match(/^\/sync\/github\/games\/([^/]+)\/([^/]+)\/sprites$/);
      if (ghSpritesMatch && request.method === 'GET') {
        if (!env.GITHUB_REPO) return json({ error: 'catalog not configured' }, 503);
        const owner = normalizeSlug(ghSpritesMatch[1], '');
        const gameId = normalizeSlug(ghSpritesMatch[2], '');
        if (!owner || !gameId) return json({ error: 'bad path' }, 400);
        const apiUrl = `https://api.github.com/repos/${env.GITHUB_REPO}/contents/games/${owner}/${gameId}`;
        try {
          const apiHeaders = {
            'User-Agent': 'slob-games-relay/1.0',
            Accept: 'application/vnd.github.v3+json',
            ...(env.GITHUB_TOKEN ? { Authorization: `token ${env.GITHUB_TOKEN}` } : {}),
          };
          const r = await fetch(apiUrl, { headers: apiHeaders });
          if (!r.ok) return json({ files: [] });
          const items = await r.json().catch(() => []);
          const files = [];
          async function walk(arr, prefix) {
            for (const it of (Array.isArray(arr) ? arr : [])) {
              if (it.type === 'file') {
                const p = (prefix ? (prefix + '/') : '') + it.name;
                files.push({
                  path: p,
                  size: it.size,
                  download_url: it.download_url,
                  sha: it.sha,
                });
              } else if (it.type === 'dir') {
                const sub = await fetch(`${apiUrl}/${it.name}`, { headers: apiHeaders });
                if (sub.ok) {
                  const subItems = await sub.json().catch(() => []);
                  await walk(subItems, (prefix ? (prefix + '/') : '') + it.name);
                }
              }
            }
          }
          await walk(items, '');
          return json({ owner, game_id: gameId, files });
        } catch (e) {
          return json({ error: String(e?.message || e) }, 502);
        }
      }

      // ── /sync/github-mgr/* — destructive endpoints (require auth token) ──
      const ghMgrMatch = url.pathname.match(/^\/sync\/github-mgr\/games\/([^/]+)\/([^/]+)$/);
      if (ghMgrMatch && (request.method === 'DELETE' || request.method === 'PATCH')) {
        if (!env.RELAY_SECRET) return json({ error: 'RELAY_SECRET not configured' }, 503);
        const authHeader = request.headers.get('Authorization') || '';
        const token = authHeader.startsWith('Bearer ') ? authHeader.slice(7).trim() : null;
        const tokenPayload = token ? await verifyUserToken(token, env.RELAY_SECRET) : null;
        if (!tokenPayload?.sub) return json({ error: 'auth required' }, 401);
        const ghToken = env.GITHUB_TOKEN;
        const ghRepo = env.GITHUB_REPO;
        if (!ghToken || !ghRepo) return json({ error: 'GitHub not configured' }, 503);
        const owner = normalizeSlug(ghMgrMatch[1], '');
        const gameId = normalizeSlug(ghMgrMatch[2], '');
        if (!owner || !gameId) return json({ error: 'bad path' }, 400);
        try {
          if (request.method === 'DELETE') {
            const result = await deleteGameFromGitHub(owner, gameId, ghToken, ghRepo);
            return json({ ok: true, ...result });
          } else {
            const body = await request.json().catch(() => ({}));
            const patch = {};
            if (typeof body.published === 'boolean') patch.published = body.published;
            if (typeof body.name === 'string' && body.name.trim()) patch.name = body.name.trim();
            const result = await patchGameOnGitHub(owner, gameId, patch, ghToken, ghRepo);
            return json(result);
          }
        } catch (e) {
          console.error('[github-mgr]', request.method, owner, gameId, String(e?.message || e).slice(0, 400));
          return json({ ok: false, error: String(e?.message || e).slice(0, 400) }, 502);
        }
      }

      // Forward /sync/admin/* to Worker-level handler
      if (url.pathname.startsWith('/sync/admin/')) {
        return handleAdminRoutes(url, request, env);
      }

      const room = env.GAME_ROOM.get(env.GAME_ROOM.idFromName("global6"));

      // WebSocket upgrade: /sync
      if (url.pathname === "/sync") {
        if (request.headers.get("Upgrade") !== "websocket") {
          return json({ error: "WebSocket upgrade required" }, 426);
        }
        try {
          return await room.fetch(request);
        } catch (e) {
          console.error('[sync-ws-error]', String(e?.message || e).slice(0, 300));
          return new Response('Service unavailable', { status: 503 });
        }
      }

      // REST: /sync/games → DO /games
      // REST: /sync/game/:hash → DO /game/:hash
      // REST: /sync/scores → DO /scores
      const doPath = url.pathname.slice(5); // strip '/sync'
      const doUrl = new URL(request.url);
      doUrl.pathname = doPath;
      try {
        const doRes = await room.fetch(new Request(doUrl.toString(), request));
        // Ensure CORS headers are present on all DO responses
        if (!doRes.headers.get('Access-Control-Allow-Origin')) {
          const patched = new Response(doRes.body, {
            status: doRes.status,
            statusText: doRes.statusText,
            headers: { ...Object.fromEntries(doRes.headers.entries()), ...cors() },
          });
          return patched;
        }
        return doRes;
      } catch (e) {
        console.error('[sync-do-error]', String(e?.message || e).slice(0, 300));
        return json({ error: "service temporarily unavailable" }, 503);
      }
    }

    return json({ error: "not found" }, 404);
  },
};
