// check_palette_applied2.mjs — 先扫描渲染图人物包围盒, 再在人物区域内采样躯干/头/腿颜色
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
const shots = path.join(__dirname, 'shots');
for (const f of ['render_zhenghai_front.png','render_ds_zhenghai_front.png']) {
  const fp = path.join(shots, f);
  if (!fs.existsSync(fp)) { console.log(f, 'MISSING'); continue; }
  const img = readPNG(fp);
  // 扫描不透明包围盒
  let x0=img.w,y0=img.h,x1=-1,y1=-1,count=0;
  for(let y=0;y<img.h;y++)for(let x=0;x<img.w;x++){
    const i=(y*img.w+x)*4;
    if(img.px[i+3]>60){ if(x<x0)x0=x; if(x>x1)x1=x; if(y<y0)y0=y; if(y>y1)y1=y; count++; }
  }
  console.log(`\n=== ${f} === 人物bbox: [${x0},${y0}]-[${x1},${y1}] w=${x1-x0} h=${y1-y0} 不透明像素=${count}`);
  const bw=x1-x0, bh=y1-y0;
  const sp = (fx, fy) => { // fx/fy 为人物bbox内归一化坐标
    const x=Math.round(x0+fx*bw), y=Math.round(y0+fy*bh);
    const i=(y*img.w+x)*4;
    return {r:img.px[i],g:img.px[i+1],b:img.px[i+2],a:img.px[i+3]};
  };
  const probes = [
    ['头部(额)', 0.5, 0.06],
    ['脸', 0.5, 0.10],
    ['躯干胸', 0.5, 0.30],
    ['躯干腹/腰', 0.5, 0.42],
    ['左上臂', 0.36, 0.32],
    ['左前臂', 0.36, 0.45],
    ['左大腿', 0.40, 0.62],
    ['左小腿', 0.40, 0.78],
    ['鞋', 0.42, 0.94],
  ];
  for (const [name, fx, fy] of probes) {
    const c = sp(fx, fy);
    console.log(`${name.padEnd(10)} → rgba(${c.r},${c.g},${c.b},${c.a})  #${c.r.toString(16).padStart(2,'0')}${c.g.toString(16).padStart(2,'0')}${c.b.toString(16).padStart(2,'0')}`);
  }
}
