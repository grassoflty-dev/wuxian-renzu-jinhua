// desc 截图视觉质检：把 4 张 desc PNG 发给 qwen3.7-flash 视觉模型判图
import fs from 'node:fs';
import path from 'node:path';

const DIR = 'C:/Users/PUBLIC_USER/Desktop/itwillclaude/games/wuxian-horror-ch1/tools/shots';
const CRED = 'C:/Users/PUBLIC_USER/.dsh/.credentials.yaml';
const cred = fs.readFileSync(CRED, 'utf8');
const key = (cred.match(/TOKENRHYTHM_API_KEY:\s*(\S+)/) || [])[1];
const images = [1,2,3,4].map(n => ({
  name: `desc_${n}.png`,
  data: fs.readFileSync(path.join(DIR, `desc_${n}.png`)).toString('base64'),
}));

const prompt = `你是 3D 游戏画面质检员。下面会给 4 张来自《无限轮回》游戏生化蜂巢世界 3D 体素地图的截图（desc_1~desc_4），它们应分别呈现不同视角/距离：1高远俯视鸟瞰地图、2拉近正对近景、3侧向视角、4贴近地面/墙壁看贴图。对每张图请输出：图名 / 是否非黑屏非空白(yes|no) / 是否能看到 3D 体素地图与建筑墙体地板(yes|no) / 简要中文描述画面内容 / 这4张之间视角是否互不相同(distinct|similar)。逐张单独判断，最后给一行总结。`;

const content = [{ type: 'text', text: prompt }];
for (const im of images) {
  content.push({ type: 'text', text: '图 ' + im.name + ':' });
  content.push({ type: 'image_url', image_url: { url: 'data:image/png;base64,' + im.data } });
}

const sleep = ms => new Promise(r => setTimeout(r, ms));
async function call() {
  for (let attempt = 0; attempt < 6; attempt++) {
    try {
      const res = await fetch('https://tokenrhythm.studio/v1/chat/completions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + key },
        body: JSON.stringify({
          model: 'qwen3.7-flash',
          messages: [{ role: 'user', content }],
          max_tokens: 4000,
        }),
      });
      if (res.status === 429) { await sleep(15000); continue; }
      if (!res.ok) { throw new Error('HTTP ' + res.status + ': ' + (await res.text()).slice(0, 300)); }
      const j = await res.json();
      const text = j?.choices?.[0]?.message?.content || j?.choices?.[0]?.message?.reasoning_content || JSON.stringify(j);
      console.log(text);
      return;
    } catch (e) {
      if (attempt === 5) throw e;
      await sleep(15000);
    }
  }
}
call().catch(e => { console.error('ERR', e); process.exit(1); });
