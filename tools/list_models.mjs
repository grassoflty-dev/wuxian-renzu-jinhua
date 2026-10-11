import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
const key = fs.readFileSync(path.join(os.homedir(), '.dsh', '.credentials.yaml'), 'utf8').match(/TOKENRHYTHM_API_KEY:\s*(\S+)/)[1];
const res = await fetch('https://tokenrhythm.studio/v1/models', { headers: { 'Authorization': 'Bearer ' + key } });
const d = await res.json();
for (const m of d.data) {
  console.log(m.id.padEnd(34) + ' vision=' + m.supports_vision + ' imgGen=' + (m.responses_capabilities?.imageGeneration ?? false) + ' ctx=' + (m.context_length||'-'));
}