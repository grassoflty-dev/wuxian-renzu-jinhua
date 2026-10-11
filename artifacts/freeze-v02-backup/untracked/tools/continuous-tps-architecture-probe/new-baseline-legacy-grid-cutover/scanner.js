import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '../../..');
const exactFiles = [
  ['config','server-rs/tauri.conf.json','entry'], ['campaignMain','server-rs/src/campaign_main.rs','entry'], ['store','server-rs/src/campaign_store.rs','formal'],
  ['campaign','server-rs/src/campaign.rs','formal'], ['state','server-rs/src/state.rs','formal'], ['main','server-rs/src/main.rs','formal'],
  ['app','server-rs/release-ui/js/app.js','formal'], ['boot','server-rs/release-ui/js/renderer-boot.js','formal'], ['tps','server-rs/release-ui/js/tps-renderer.js','formal']
];
const patterns = [/\bbase_grid\b/gi, /\btiles\b/gi, /\bpx\b/gi, /\bpy\b/gi, /\bzone2d\b/gi];
const classify = (key, field) => key === 'app' || key === 'tps' ? 'formal_view_projection' : field === 'entry' ? 'uncertain' : 'formal_runtime_authority';
export function scanFormal() {
  const findings = [];
  for (const [key, rel, reachability] of exactFiles) {
    const p = path.join(root, rel); const lines = fs.readFileSync(p, 'utf8').split(/\r?\n/);
    lines.forEach((text, index) => patterns.forEach(re => { re.lastIndex = 0; if (re.test(text)) findings.push({ id: `LEGACY-${key.toUpperCase()}-${index + 1}`, path: rel, line: index + 1, symbol: text.trim().slice(0, 160), field: re.source.replace(/\\b/g,''), direction: /map|view|json|serialize|tiles|px|py/.test(text) ? 'read_or_project' : 'read_or_write', owner: key === 'app' || key === 'tps' ? 'Babylon/UI projection' : 'Rust legacy campaign', classification: classify(key, reachability), severity: key === 'app' || key === 'tps' || key === 'campaign' || key === 'store' ? 'blocker' : 'high', suggestion: 'replace with one-way V2→V3 adapter and metre WorldView' }); }));
  }
  return { entry: { frontendDist: './release-ui', root: 'server-rs/release-ui/index.html', chain: ['vendor/babylon.js','js/tps-renderer.js','js/renderer-boot.js','js/app.js'] }, findings, graph: [
    { from: 'tauri.frontendDist', to: 'release-ui/index.html' }, { from: 'campaign_main::campaign_view/action/save', to: 'CampaignStore' }, { from: 'CampaignStore', to: 'CampaignState/map_for/base_grid' }, { from: 'CampaignStore.view', to: 'app.js renderState' }, { from: 'app.js map', to: 'TpsRenderer.setData' }, { from: 'zone_enter_inner', to: 'zone2d output' }
  ] };
}
