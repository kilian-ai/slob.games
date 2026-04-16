-- storage-inspector.lua
-- Seeded VFS tool for inspecting localStorage usage.
--
-- Host contract:
--   input.quota_bytes       number   (estimated total quota)
--   input.used_bytes        number   (total bytes used by all localStorage keys)
--   input.pvfs_bytes        number   (bytes used by traits.pvfs key)
--   input.keys              array    [{key, size}]   all localStorage keys sorted by size desc
--   input.games             array    [{id, name, scope, size, hash, active}]   games in collection
--   input.sprites           array    [{path, size}]   sprite/resource files in VFS
--   input.other_vfs         array    [{path, size}]   non-game, non-sprite VFS entries
--   input.games_total       number   (total bytes of game content)
--   input.sprites_total     number   (total bytes of sprites/resources)
--   input.error             string?

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

local function pad_right(s, width)
  s = tostring(s or '')
  if #s >= width then return s end
  return s .. string.rep(' ', width - #s)
end

local function bar(used, total, width)
  if total <= 0 then return '' end
  local filled = math.floor((used / total) * width + 0.5)
  if filled > width then filled = width end
  local empty = width - filled
  return '[' .. string.rep('#', filled) .. string.rep('.', empty) .. ']'
end

local input = input or {}

if input.error then
  line('Error: ' .. tostring(input.error))
  return
end

local quota = tonumber(input.quota_bytes) or 0
local used = tonumber(input.used_bytes) or 0
local pvfs = tonumber(input.pvfs_bytes) or 0
local games_total = tonumber(input.games_total) or 0
local sprites_total = tonumber(input.sprites_total) or 0

line(repeat_char('=', 60))
line('  localStorage Inspector')
line(repeat_char('=', 60))

-- Quota overview
line('')
line('QUOTA')
local pct = 0
if quota > 0 then pct = math.floor(used / quota * 100 + 0.5) end
line('  Used:      ' .. fmt_size(used) .. ' / ' .. fmt_size(quota) .. '  (' .. pct .. '%)')
line('  Free:      ' .. fmt_size(math.max(0, quota - used)))
line('  ' .. bar(used, quota, 40))

-- Key breakdown
line('')
line('TOP KEYS BY SIZE')
local keys = type(input.keys) == 'table' and input.keys or {}
if #keys == 0 then
  line('  (no keys)')
else
  for i, k in ipairs(keys) do
    if i > 15 then
      line('  ... and ' .. (#keys - 15) .. ' more keys')
      break
    end
    local kpct = ''
    if used > 0 then
      kpct = string.format(' (%d%%)', math.floor((tonumber(k.size) or 0) / used * 100 + 0.5))
    end
    line('  ' .. pad_right(tostring(k.key), 35) .. fmt_size(k.size) .. kpct)
  end
end

-- pvfs breakdown
line('')
line('VFS BREAKDOWN  (traits.pvfs)')
line('  Total pvfs:    ' .. fmt_size(pvfs))
line('  Game content:  ' .. fmt_size(games_total))
line('  Sprites:       ' .. fmt_size(sprites_total))
local other_total = math.max(0, pvfs - games_total - sprites_total)
line('  Other VFS:     ' .. fmt_size(other_total))
if pvfs > 0 then
  line('')
  line('  ' .. bar(games_total, pvfs, 30) .. ' games')
  line('  ' .. bar(sprites_total, pvfs, 30) .. ' sprites')
  line('  ' .. bar(other_total, pvfs, 30) .. ' other')
end

-- Games detail
line('')
local games = type(input.games) == 'table' and input.games or {}
line('GAMES  (' .. #games .. ' total, ' .. fmt_size(games_total) .. ')')
if #games == 0 then
  line('  (none)')
else
  for _, g in ipairs(games) do
    local marker = g.active and ' *' or ''
    local scope = tostring(g.scope or 'internal')
    local tag = ''
    if scope == 'external' then tag = ' [ext]' end
    local hash = tostring(g.hash or ''):sub(1, 8)
    if #hash > 0 then hash = '  #' .. hash end
    line(string.format('  %s %s%s%s%s',
      fmt_size(g.size),
      pad_right(tostring(g.name or g.id or '?'), 28),
      tag, hash, marker))
  end
end

-- Sprites detail
line('')
local sprites = type(input.sprites) == 'table' and input.sprites or {}
line('SPRITES & RESOURCES  (' .. #sprites .. ' files, ' .. fmt_size(sprites_total) .. ')')
if #sprites == 0 then
  line('  (none)')
else
  for i, sp in ipairs(sprites) do
    if i > 20 then
      line('  ... and ' .. (#sprites - 20) .. ' more')
      break
    end
    line('  ' .. fmt_size(sp.size) .. '  ' .. tostring(sp.path))
  end
end

-- Other VFS files
local other_vfs = type(input.other_vfs) == 'table' and input.other_vfs or {}
if #other_vfs > 0 then
  line('')
  line('OTHER VFS FILES  (' .. #other_vfs .. ')')
  for i, f in ipairs(other_vfs) do
    if i > 15 then
      line('  ... and ' .. (#other_vfs - 15) .. ' more')
      break
    end
    line('  ' .. fmt_size(f.size) .. '  ' .. tostring(f.path))
  end
end

line('')
line(repeat_char('=', 60))
