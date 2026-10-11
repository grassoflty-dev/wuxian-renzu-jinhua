// locate_character.mjs — 找渲染图中人物密集区域(按列/行直方图)
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
// 列直方图(每 50px 一桶)和行直方图(每 20px 一桶)
const colBuckets = new Map(), rowBuckets = new Map();
for(let y=0;y<img.h;y++)for(let x=0;x<img.w;x++){
  const i=(y*img.w+x)*4;
  if(img.px[i+3]>60){
    const cb=Math.floor(x/50)*50, rb=Math.floor(y/20)*20;
    colBuckets.set(cb, (colBuckets.get(cb)||0)+1);
    rowBuckets.set(rb, (rowBuckets.get(rb)||0)+1);
  }
}
console.log(`${f} ${img.w}x${img.h}`);
console.log('列直方图(x桶:像素数) top10:');
[...colBuckets.entries()].sort((a,b)=>b[1]-a[1]).slice(0,10).forEach(([k,v])=>console.log(`  x=${k}~${k+50}: ${v}`));
console.log('行直方图(y桶:像素数) top12:');
[...rowBuckets.entries()].sort((a,b)=>b[1]-a[1]).slice(0,12).forEach(([k,v])=>console.log(`  y=${k}~${k+20}: ${v}`));
