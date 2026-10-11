// check_palette_applied.mjs — 检查渲染图是否应用了立绘染色
// 原理: 染色生效时躯干中心像素≈立绘对应位置颜色(#53646f 蓝灰);
//       染色未生效时躯干≈基础材质色(CHARACTERS.zhenghai shirt 或 DS 版 0x4a545c)
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
function sample(img, fx, fy) {
  const x = Math.min(img.w-1, Math.max(0, Math.round(fx*(img.w-1))));
  const y = Math.min(img.h-1, Math.max(0, Math.round(fy*(img.h-1))));
  const i=(y*img.w+x)*4;
  return {r:img.px[i],g:img.px[i+1],b:img.px[i+2],a:img.px[i+3]};
}
const shots = path.join(__dirname, 'shots');
for (const f of ['render_zhenghai_front.png','render_ds_zhenghai_front.png']) {
  const fp = path.join(shots, f);
  if (!fs.existsSync(fp)) { console.log(f, 'MISSING'); continue; }
  const img = readPNG(fp);
  console.log(`\n=== ${f} (${img.w}x${img.h}) ===`);
  const probes = [
    ['躯干中心(胸)', 0.5, 0.42],
    ['躯干(腹)', 0.5, 0.52],
    ['左上臂', 0.42, 0.44],
    ['左前臂', 0.40, 0.55],
    ['左大腿', 0.42, 0.68],
    ['头部', 0.5, 0.24],
  ];
  for (const [name, fx, fy] of probes) {
    const c = sample(img, fx, fy);
    console.log(`${name.padEnd(12)} → rgba(${c.r},${c.g},${c.b},${c.a})  #${c.r.toString(16).padStart(2,'0')}${c.g.toString(16).padStart(2,'0')}${c.b.toString(16).padStart(2,'0')}`);
  }
}
console.log('\n判定依据:');
console.log('  立绘躯干色 ≈ #53646f (蓝灰) — 立绘 probe 实测');
console.log('  基础材质 zhenghai shirt ≈ 0x4a5a52/#4a5a52 (军绿灰) 原版 / 0x4a545c 蓝灰 DS版');
console.log('  若躯干≈#4a5a52 或 #4a545c → 染色未生效(基础材质)');
console.log('  若躯干≈#53646f → 染色生效');
