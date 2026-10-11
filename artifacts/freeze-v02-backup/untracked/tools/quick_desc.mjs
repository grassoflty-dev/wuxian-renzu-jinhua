// quick_desc.mjs — quick description with reasoning_content support
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
const key = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'),'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
const fp = process.argv[2];
const prompt = process.argv[3] || '简短描述这张图';
const dataUrl = { type: 'image_url', image_url: { url: 'data:image/png;base64,' + fs.readFileSync(fp).toString('base64') } };
const res = await fetch('https://tokenrhythm.studio/v1/chat/completions', {
  method: 'POST',
  headers: { 'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json' },
  body: JSON.stringify({ model: 'qwen3.7-flash', messages: [{ role: 'user', content: [{ type: 'text', text: prompt }, dataUrl] }], max_tokens: 4000 }),
});
const j = await res.json();
const msg = j.choices?.[0]?.message || {};
const out = (msg.content || '').trim() || (msg.reasoning_content || '').trim();
console.log(out.slice(0, 4000));
