-- relay-monitor.lua
-- Seeded VFS tool for inspecting the slob.games relay.
--
-- Host contract:
--   input.relay           string
--   input.games           array
--   input.scores          array
--   input.my_games        array|nil
--   input.manifest        string
--   input.token_provided  boolean
--   input.games_error     string|nil
--   input.scores_error    string|nil
--   input.my_games_error  string|nil
--   input.manifest_error  string|nil

local function line(s)
  print(s or '')
end

local function repeat_char(ch, n)
  local out = {}
  for i = 1, (n or 0) do out[i] = ch end
  return table.concat(out)
end

local function fmt_size(n)
  n = tonumber(n or 0) or 0
  if n >= 1024 * 1024 then return string.format('%.1f MB', n / 1024 / 1024) end
  if n >= 1024 then return string.format('%.1f KB', n / 1024) end
  return string.format('%d B', n)
end

local function fmt_date(s)
  s = tostring(s or '')
  if #s == 0 then return '' end
  s = s:gsub('T', ' '):gsub('Z', '')
  if #s >= 16 then return s:sub(1, 16) end
  return s
end

local function short_hash(s)
  s = tostring(s or '?')
  return s:sub(1, 8)
end

local function pad_right(s, width)
  s = tostring(s or '')
  if #s >= width then return s end
  return s .. string.rep(' ', width - #s)
end

local input = input or {}
local relay = tostring(input.relay or 'https://relay.slob.games')
local games = type(input.games) == 'table' and input.games or {}
local scores = type(input.scores) == 'table' and input.scores or {}
local my_games = type(input.my_games) == 'table' and input.my_games or nil
local manifest = tostring(input.manifest or '')
local token_provided = input.token_provided and true or false

line(repeat_char('=', 60))
line('  slob.games relay: ' .. relay)
line(repeat_char('=', 60))

line('')
line('PUBLISHED GAMES  (scope=external, published=true)')
if input.games_error then
  line('  Error: ' .. tostring(input.games_error))
elseif #games == 0 then
  line('  (none)')
else
  for _, g in ipairs(games) do
    local hs = ''
    if g.highscore ~= nil and tostring(g.highscore) ~= '' then
      hs = '  hs=' .. tostring(g.highscore)
    end
    local name = string.format('%q', tostring(g.name or '?'))
    line(string.format('  [%s] %s %8s  %s%s',
      short_hash(g.content_hash),
      pad_right(name, 30),
      fmt_size(g.size),
      fmt_date(g.updated),
      hs
    ))
  end
end

line('')
line('HIGH SCORES')
if input.scores_error then
  line('  Error: ' .. tostring(input.scores_error))
elseif #scores == 0 then
  line('  (none)')
else
  for _, s in ipairs(scores) do
    local player = string.format('%q', tostring(s.player or '—'))
    line(string.format('  game=%-8s  score=%-8s player=%-16s %s',
      short_hash(s.game_hash),
      tostring(s.score or '?'),
      player,
      fmt_date(s.updated)
    ))
  end
end

if token_provided then
  line('')
  line('MY GAMES  (/internal/games)')
  if input.my_games_error then
    line('  Error: ' .. tostring(input.my_games_error))
  elseif not my_games or #my_games == 0 then
    line('  (none)')
  else
    for _, g in ipairs(my_games) do
      local pub = g.published and 'pub' or 'draft'
      local name = string.format('%q', tostring(g.name or '?'))
      line(string.format('  [%s] [%s] %s %8s  %s',
        short_hash(g.content_hash),
        pub,
        pad_right(name, 28),
        fmt_size(g.size),
        fmt_date(g.updated)
      ))
    end
  end
end

line('')
line('GAMES MANIFEST  (/sync/games.toml)')
if input.manifest_error then
  line('  Error: ' .. tostring(input.manifest_error))
elseif #manifest == 0 then
  line('  (empty)')
else
  local preview = manifest
  if #preview > 1200 then
    preview = preview:sub(1, 1200)
  end
  line(preview)
  if #manifest > 1200 then
    line('  [truncated]')
  end
end

line('')
line(repeat_char('=', 60))
line(string.format('  published=%s  scores=%s', tostring(#games), tostring(#scores)))
line(repeat_char('=', 60))

__result = string.format('published=%s scores=%s', tostring(#games), tostring(#scores))
