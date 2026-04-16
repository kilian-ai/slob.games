(function () {
    const DEFAULT_MODEL = 'gpt-4o-mini';
    const DEFAULT_SYSTEM = 'You are a concise, tool-using terminal assistant.';
    const DEFAULT_TOOLS = ['sys.echo', 'sys.list', 'sys.search', 'sys.info'];

    function sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    function getSessions() {
        if (typeof window === 'undefined') return new Map();
        if (!(window.__traitsAgentSessions instanceof Map)) {
            window.__traitsAgentSessions = new Map();
        }
        return window.__traitsAgentSessions;
    }

    function printableToolName(tc) {
        return tc.trait || tc.name || 'tool';
    }

    async function streamText(text, write) {
        const full = String(text || '');
        if (!full) return;
        const chunks = full.split(/(\s+)/).filter(Boolean);
        for (const chunk of chunks) {
            write(chunk);
            await sleep(12);
        }
    }

    async function runToolBindingLoop(toolCalls, sdk, writeln) {
        if (!Array.isArray(toolCalls) || !toolCalls.length) return toolCalls || [];

        const updated = [];
        for (const tc of toolCalls) {
            const next = Object.assign({}, tc || {});
            const alreadyHasResult = Object.prototype.hasOwnProperty.call(next, 'result');
            if (!alreadyHasResult) {
                const traitPath = next.trait || (next.name ? String(next.name).replace(/_/g, '.') : '');
                if (traitPath) {
                    const argObj = (next.args && typeof next.args === 'object') ? next.args : {};
                    const args = Object.keys(argObj).sort().map(k => argObj[k]);
                    try {
                        const r = await sdk.call(traitPath, args);
                        next.result = r.ok ? r.result : { error: r.error || 'tool call failed' };
                    } catch (e) {
                        next.result = { error: e && e.message ? e.message : String(e) };
                    }
                }
            }
            writeln(`\x1b[90m⚡ ${printableToolName(next)}\x1b[0m`);
            updated.push(next);
        }
        return updated;
    }

    function parseAgentCommand(line) {
        const trimmed = String(line || '').trim();
        if (!trimmed) return { cmd: 'help' };

        const tokens = trimmed.split(/\s+/);
        if (tokens[0] === '@agent') tokens.shift();
        if (tokens[0] === 'agent') tokens.shift();

        if (!tokens.length) return { cmd: 'help' };

        const cmd = tokens[0].toLowerCase();
        const rest = tokens.slice(1);

        if (cmd === 'help' || cmd === 'list' || cmd === 'create' || cmd === 'query' || cmd === 'stop' || cmd === 'lua') {
            return { cmd, rest };
        }

        return { cmd: 'query', rest: tokens };
    }

    function usage(writeln) {
        writeln('\x1b[36mAgent Commands\x1b[0m');
        writeln('  @agent help');
        writeln('  @agent list');
        writeln('  @agent create [model]');
        writeln('  @agent query <text>');
        writeln('  @agent stop [sessionId]');
        writeln('  @agent lua <text>    (prints a Lua runner payload example)');
    }

    async function createSession(sdk, sessions, model) {
        const cfg = {
            action: 'create',
            model: model || DEFAULT_MODEL,
            system_prompt: DEFAULT_SYSTEM,
            tools: DEFAULT_TOOLS,
        };
        const res = await sdk.call('sys.rig_agent', [JSON.stringify(cfg)]);
        if (!res.ok || !res.result || !res.result.ok) {
            return { ok: false, error: res.error || (res.result && res.result.error) || 'create failed' };
        }
        const sid = res.result.sessionId;
        sessions.set('current', sid);
        return { ok: true, sid, result: res.result };
    }

    async function querySession(sdk, sid, message, write, writeln) {
        const payload = {
            action: 'query',
            sessionId: sid,
            message,
            model: DEFAULT_MODEL,
            system_prompt: DEFAULT_SYSTEM,
            tools: DEFAULT_TOOLS,
        };

        const res = await sdk.call('sys.rig_agent', [JSON.stringify(payload)], { stream: true });

        if (res && res.ok && res.stream) {
            let streamed = false;
            let combined = '';
            for await (const chunk of res.stream) {
                const text = typeof chunk === 'string' ? chunk : (chunk && chunk.result ? String(chunk.result) : '');
                if (!text) continue;
                streamed = true;
                combined += text;
                write(text);
            }
            if (streamed) writeln('');
            return { ok: true, response: combined, tool_calls: [] };
        }

        if (!res.ok || !res.result) {
            return { ok: false, error: res.error || 'query failed' };
        }

        if (res.result.ok === false) {
            return { ok: false, error: res.result.error || 'query failed' };
        }

        const response = typeof res.result.response === 'string'
            ? res.result.response
            : JSON.stringify(res.result.response || res.result, null, 2);

        await streamText(response, write);
        writeln('');

        const toolCalls = Array.isArray(res.result.tool_calls) ? res.result.tool_calls : [];
        const rebound = await runToolBindingLoop(toolCalls, sdk, writeln);
        return { ok: true, response, tool_calls: rebound };
    }

    function buildLuaRunnerExample(prompt) {
        return {
            script_path: 'traits/www/sdk/agent-runner.lua',
            payload: {
                prompt: prompt || 'summarize the current workspace',
                model: DEFAULT_MODEL,
                system_prompt: DEFAULT_SYSTEM,
                tools_csv: DEFAULT_TOOLS.join(','),
                max_steps: 8,
            },
            note: 'Load this payload into your Lua host and call sys.rig_agent in a turn loop.',
        };
    }

    async function handleAgentCommand(ctx) {
        const line = String((ctx && ctx.line) || '').trim();
        const sdk = ctx && ctx.sdk;
        const write = (ctx && ctx.write) || function () {};
        const writeln = (ctx && ctx.writeln) || function () {};

        if (!line || (!line.startsWith('@agent') && !line.startsWith('agent '))) {
            return false;
        }

        if (!sdk || typeof sdk.call !== 'function') {
            writeln('\x1b[31mAgent command requires SDK runtime\x1b[0m');
            return true;
        }

        const sessions = getSessions();
        const parsed = parseAgentCommand(line);

        if (parsed.cmd === 'help') {
            usage(writeln);
            return true;
        }

        if (parsed.cmd === 'list') {
            const current = sessions.get('current') || '';
            if (!current) {
                writeln('\x1b[90mNo active agent session\x1b[0m');
                return true;
            }
            writeln(`\x1b[36mCurrent session:\x1b[0m ${current}`);
            return true;
        }

        if (parsed.cmd === 'create') {
            const model = parsed.rest && parsed.rest[0] ? parsed.rest[0] : DEFAULT_MODEL;
            const created = await createSession(sdk, sessions, model);
            if (!created.ok) {
                writeln(`\x1b[31m${created.error}\x1b[0m`);
                return true;
            }
            writeln(`\x1b[32mAgent ready\x1b[0m ${created.sid} (${model})`);
            return true;
        }

        if (parsed.cmd === 'stop') {
            const sid = (parsed.rest && parsed.rest[0]) || sessions.get('current') || '';
            if (!sid) {
                writeln('\x1b[31mNo session to stop\x1b[0m');
                return true;
            }
            const payload = { action: 'stop', sessionId: sid };
            const res = await sdk.call('sys.rig_agent', [JSON.stringify(payload)]);
            if (!res.ok || !res.result || res.result.ok === false) {
                writeln(`\x1b[31m${res.error || (res.result && res.result.error) || 'stop failed'}\x1b[0m`);
                return true;
            }
            if (sessions.get('current') === sid) sessions.delete('current');
            writeln(`\x1b[32mStopped\x1b[0m ${sid}`);
            return true;
        }

        if (parsed.cmd === 'lua') {
            const prompt = parsed.rest.join(' ').trim();
            const example = buildLuaRunnerExample(prompt);
            writeln(JSON.stringify(example, null, 2));
            return true;
        }

        if (parsed.cmd === 'query') {
            const message = parsed.rest.join(' ').trim();
            if (!message) {
                writeln('\x1b[31mUsage: @agent query <text>\x1b[0m');
                return true;
            }

            let sid = sessions.get('current') || '';
            if (!sid) {
                const created = await createSession(sdk, sessions, DEFAULT_MODEL);
                if (!created.ok) {
                    writeln(`\x1b[31m${created.error}\x1b[0m`);
                    return true;
                }
                sid = created.sid;
                writeln(`\x1b[90mCreated session ${sid}\x1b[0m`);
            }

            writeln('\x1b[90mThinking...\x1b[0m');
            const result = await querySession(sdk, sid, message, write, writeln);
            if (!result.ok) {
                writeln(`\x1b[31m${result.error}\x1b[0m`);
            }
            return true;
        }

        usage(writeln);
        return true;
    }

    if (typeof window !== 'undefined') {
        window.handleAgentCommand = handleAgentCommand;
    }
})();
