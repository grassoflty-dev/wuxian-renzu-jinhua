import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
const key = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
const sizes = ['1024x1024','768x768','1280x720','720x1280','1024x768','512x768','768x1024','1328x1328'];
for (const s of sizes) {
  const res = await fetch('https://tokenrhythm.studio/v1/images/generations', {
    method: 'POST',
    headers: { 'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: 'wan2.7-image', prompt: 'a red cube', n: 1, size: s })
  });
  const t = await res.text();
  console.log(s + ' -> ' + res.status + ' :: ' + t.slice(0, 160).replace(/\n/g,' '));
}