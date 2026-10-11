// ascii_map.mjs — 把渲染图降采样成 ASCII 密度图, 直观看到画面布局
import fs from 'node:fs';
import zlib from 'node:zlib';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
function readPNG(filepath) {
  const buf = fs.readFileSync(filepath);
  if (buf.slice(0,8).toString('hex')!=='89504e470d0a1a0a') throw new Error('not PNG');
  let pos=8,w=0,h=0,idat=Buffer.alloc(0);
  while(pos<buf.length){const len=buf.readUInt32BE(pos);pos+=4;const t=buf.slice(pos,pos+4).toString('ascii');pos+=4;const d=buf.slice(pos,pos+len);pos+=len;pos+=4;
    if(t==='IHDR'){w=d.readUInt32BE(0);h=d.readUInt32BE(4);}
    else if(t==='IDAT')idat=Buffer.concat([idat,d]);
    else if(t==='IEND')break;}
  const raw=zlib.inflateSync(idat);const bpp=4,stride=w*bpp,px=Buffer.alloc(stride*h);
  let rp=0,wp=0;const prev=Buffer.alloc(stride);
  for(let y=0;y<h;y++){const f=raw[rp++];const cur=Buffer.alloc(stride);
    for(let x=0;x<stride;x++){const fx=raw[rp++];let a=x>=bpp?cur[x-bpp]:0,b=y>0?prev[x]:0,c=(x>=bpp&&y>0)?prev[x-bpp]:0,out;
      switch(f){case 0:out=fx;break;case 1:out=(fx+a)&0xff;break;case 2:out=(fx+b)&0xff;break;
      case 3:out=(fx+((a+b)>>1))&0xff;break;case 4:{const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);out=(fx+(pa<=pb&&pa<=pc?a:pb<=pc?b:c))&0xff;break;}
      default:out=fx;}cur[x]=out;}
    cur.copy(px,wp);wp+=stride;cur.copy(prev);}
  return {w,h,px};
}
const f = process.argv[2] || 'render_zhenghai_front.png';
const img = readPNG(path.join(__dirname, 'shots', f));
// 降采样到 100x40 字符画, '#'=有内容(不透明且非纯白), '.'=空
const CW = 100, CH = 40;
const cellW = img.w / CW, cellH = img.h / CH;
let map = '';
for (let cy = 0; cy < CH; cy++) {
  let row = '';
  for (let cx = 0; cx < CW; cx++) {
    let count = 0, total = 0;
    const x0 = Math.floor(cx * cellW), x1 = Math.min(img.w, Math.floor((cx + 1) * cellW));
    const y0 = Math.floor(cy * cellH), y1 = Math.min(img.h, Math.floor((cy + 1) * cellH));
    for (let y = y0; y < y1; y += 2) for (let x = x0; x < x1; x += 2) {
      const i = (y * img.w + x) * 4;
      if (img.px[i + 3] > 60) { count++; total++; }
    }
    row += count > 0 ? '#' : '.';
  }
  map += row + '\n';
}
console.log(`${f} ${img.w}x${img.h}`);
console.log(map);
