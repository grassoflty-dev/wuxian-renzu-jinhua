// probe_portrait.mjs — 探测立绘指定 UV 位置颜色
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
const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
const img = readPNG(portrait);
// alpha bbox
let bx0=img.w,by0=img.h,bx1=-1,by1=-1;
for(let y=0;y<img.h;y++)for(let x=0;x<img.w;x++){const a=img.px[(y*img.w+x)*4+3];if(a>=80){if(x<bx0)bx0=x;if(x>bx1)bx1=x;if(y<by0)by0=y;if(y>by1)by1=y;}}
const fw=bx1-bx0,fh=by1-by0;
console.log(`bbox: [${bx0},${by0}]-[${bx1},${by1}] ${fw}x${fh}`);
function sample(u,v){
  const x=Math.min(img.w-1,Math.max(0,Math.round(bx0+u*fw)));
  const y=Math.min(img.h-1,Math.max(0,Math.round(by0+v*fh)));
  const i=(y*img.w+x)*4;
  return {r:img.px[i],g:img.px[i+1],b:img.px[i+2],a:img.px[i+3]};
}
// Probe several positions
const probes=[
  ['torso center (chest)',0.5,0.35],
  ['torso center (abs)',0.5,0.45],
  ['L arm outer (v=0.30)',0.05,0.30],
  ['L arm (v=0.40)',0.08,0.40],
  ['L arm (v=0.50)',0.06,0.50],
  ['L forearm/hand (v=0.60)',0.08,0.60],
  ['R arm outer (v=0.30)',0.95,0.30],
  ['R arm (v=0.40)',0.92,0.40],
  ['L leg upper (v=0.60)',0.30,0.60],
  ['L leg mid (v=0.75)',0.28,0.75],
  ['L leg boot (v=0.90)',0.24,0.90],
  ['R leg upper (v=0.60)',0.70,0.60],
  ['face skin (v=0.10)',0.5,0.10],
  ['hair (v=0.03)',0.5,0.03],
  ['L shoulder (v=0.25)',0.12,0.25],
  ['R shoulder (v=0.25)',0.88,0.25],
];
for(const[name,u,v]of probes){const c=sample(u,v);console.log(`${name.padEnd(30)} u=${u.toFixed(2)} v=${v.toFixed(2)} → rgba(${c.r},${c.g},${c.b},${c.a})  #${c.r.toString(16).padStart(2,'0')}${c.g.toString(16).padStart(2,'0')}${c.b.toString(16).padStart(2,'0')}`);}
// Also find the skin-colored area near waist for arms
console.log('\n--- scanning L arm region at v=0.50 (waist) ---');
for(let u=0;u<=0.25;u+=0.02){const c=sample(u,0.50);console.log(`u=${u.toFixed(2)} → rgba(${c.r},${c.g},${c.b},${c.a})`);}
console.log('\n--- scanning L arm region at v=0.30 (chest) ---');
for(let u=0;u<=0.25;u+=0.02){const c=sample(u,0.30);console.log(`u=${u.toFixed(2)} → rgba(${c.r},${c.g},${c.b},${c.a})`);}
console.log('\n--- scanning v=0.65 (forearm height) across x ---');
for(let u=0;u<=1;u+=0.04){const c=sample(u,0.65);if(c.a>=80)console.log(`u=${u.toFixed(2)} → rgba(${c.r},${c.g},${c.b})`);}
