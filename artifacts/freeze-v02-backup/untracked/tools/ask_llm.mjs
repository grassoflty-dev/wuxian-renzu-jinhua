import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
const key = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
const q = process.argv[2];
const body = { model: 'deepseek-v4-flash-0731', messages: [{ role: 'user', content: q }], max_tokens: 2000 };
const res = await fetch('https://tokenrhythm.studio/v1/chat/completions', { method: 'POST', headers: { 'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
const t = await res.text();
if (!res.ok) { console.log('HTTP', res.status, t.slice(0,300)); process.exit(1); }
const d = JSON.parse(t);
console.log((d.choices?.[0]?.message?.content) || (d.choices?.[0]?.message?.reasoning_content) || '(empty)');