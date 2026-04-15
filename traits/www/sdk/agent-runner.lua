-- agent-runner.lua
-- Rig-style Lua agent runner for slob.games
--
-- Host contract expected by this script:
--   call_trait(path, args_table) -> table
--   stream_emit(text)            -> nil (optional)
--   input globals:
--     prompt, model, system_prompt, tools_csv, max_steps
--
-- In this project, the Rust side entrypoint is sys.rig_agent.

local function split_csv(s)
  local out = {}
  if type(s) ~= 'string' then return out end
  for part in string.gmatch(s, '([^,]+)') do
    part = string.gsub(part, '^%s+', '')
    part = string.gsub(part, '%s+$', '')
    if #part > 0 then table.insert(out, part) end
  end
  return out
end

local function emit(text)
  if type(stream_emit) == 'function' then
    stream_emit(tostring(text or ''))
  end
end

local function safe_call(path, args)
  if type(call_trait) ~= 'function' then
    return { ok = false, error = 'call_trait(path,args) bridge is not installed by host' }
  end
  local ok, res = pcall(call_trait, path, args)
  if not ok then
    return { ok = false, error = tostring(res) }
  end
  return res or { ok = false, error = 'empty trait response' }
end

local function run_tool_binding_loop(tool_calls)
  local out = {}
  if type(tool_calls) ~= 'table' then return out end

  for i, tc in ipairs(tool_calls) do
    local entry = tc
    if type(entry) == 'table' and entry.result == nil then
      local trait = entry.trait
      if trait == nil and type(entry.name) == 'string' then
        trait = string.gsub(entry.name, '_', '.')
      end

      if type(trait) == 'string' and #trait > 0 then
        local ordered_args = {}
        if type(entry.args) == 'table' then
          local keys = {}
          for k, _ in pairs(entry.args) do table.insert(keys, k) end
          table.sort(keys)
          for _, k in ipairs(keys) do table.insert(ordered_args, entry.args[k]) end
        end

        emit(string.format('[tool] %s', trait))
        local tres = safe_call(trait, ordered_args)
        entry.result = tres
      end
    end
    out[i] = entry
  end

  return out
end

local function run_agent()
  local p = prompt or 'hello'
  local m = model or 'gpt-4o-mini'
  local sys = system_prompt or 'You are a concise, tool-using assistant.'
  local tools = split_csv(tools_csv or 'sys.echo,sys.list,sys.search')
  local steps = tonumber(max_steps) or 8

  local create_cfg = {
    action = 'create',
    model = m,
    system_prompt = sys,
    tools = tools,
  }

  local created = safe_call('sys.rig_agent', { create_cfg })
  if not (type(created) == 'table' and created.ok and created.sessionId) then
    return {
      ok = false,
      error = (created and created.error) or 'failed to create rig session',
      create = created,
    }
  end

  local sid = created.sessionId
  local turns = {}
  local final_text = ''

  for i = 1, steps do
    local query_cfg = {
      action = 'query',
      sessionId = sid,
      message = (i == 1) and p or ('continue: ' .. (final_text or '')),
      model = m,
      system_prompt = sys,
      tools = tools,
    }

    local resp = safe_call('sys.rig_agent', { query_cfg })
    if not (type(resp) == 'table' and resp.ok) then
      table.insert(turns, { step = i, ok = false, error = resp and resp.error or 'query failed' })
      break
    end

    local text = resp.response or ''
    final_text = tostring(text)
    emit(final_text)

    local rebound = run_tool_binding_loop(resp.tool_calls)
    table.insert(turns, {
      step = i,
      ok = true,
      response = final_text,
      tool_calls = rebound,
    })

    if final_text ~= '' then
      break
    end
  end

  safe_call('sys.rig_agent', { { action = 'stop', sessionId = sid } })

  return {
    ok = true,
    sessionId = sid,
    response = final_text,
    turns = turns,
  }
end

__result = run_agent()
