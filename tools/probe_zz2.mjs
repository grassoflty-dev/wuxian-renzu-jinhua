// probe_zz2.mjs — 郑吒立绘精细标定：中线颜色过渡 + 手臂/腿柱精确范围 + 关键行分段
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
const portrait = path.resolve(__dirname,'..','server-rs','ui','assets','img','pc_zhengzha.png');
const img = readPNG(portrait);
let bx0=img.w,by0=img.h,bx1=-1,by1=-1;
for(let y=0;y<img.h;y++)for(let x=0;x<img.w;x++){const a=img.px[(y*img.w+x)*4+3];if(a>=80){if(x<bx0)bx0=x;if(x>bx1)bx1=x;if(y<by0)by0=y;if(y>by1)by1=y;}}
const fw=bx1-bx0,fh=by1-by0;
console.log(`bbox [${bx0},${by0}]-[${bx1},${by1}] ${fw}x${fh}`);
const at=(u,v)=>{const x=Math.round(bx0+u*fw),y=Math.round(by0+v*fh);const i=(y*img.w+x)*4;return{r:img.px[i],g:img.px[i+1],b:img.px[i+2],a:img.px[i+3]};};
const kind=(c)=>{
  if(c.a<80)return 'bg';
  const{r,g,b}=c;
  if(r>150&&g>120&&b>90&&r>g&&g>=b)return 'skin';
  if(r<110&&g<120&&b<130&&b>=g-10&&Math.abs(r-g)<40)return 'darkcloth'; // 深布/发(蓝灰暗)
  if(r<60&&g<60&&b<60)return 'black';
  return `other(${r},${g},${b})`;
};
// 1) 中线扫描 v=0..0.30（头发→脸→下巴→领口→T恤）
console.log('\n== center column v 0.00→0.32 step0.01 ==');
let prevK='';
for(let v=0;v<=0.321;v+=0.01){const c=at(0.5,v);const k=kind(c);if(k!==prevK){console.log(`v=${v.toFixed(2)} y=${Math.round(by0+v*fh)} → ${k} rgb(${c.r},${c.g},${c.b})`);prevK=k;}}
// 2) 左臂柱 u=0.065 扫描 v=0.20→0.62（袖口=shirt→skin 交界；手底=skin→bg）
console.log('\n== left arm column u=0.065, v 0.20→0.64 ==');
prevK='';
for(let v=0.20;v<=0.641;v+=0.01){const c=at(0.065,v);const k=kind(c);if(k!==prevK){console.log(`v=${v.toFixed(2)} y=${Math.round(by0+v*fh)} → ${k} rgb(${c.r},${c.g},${c.b})`);prevK=k;}}
// 3) 关键行分段（更多 v，含 0.26-0.58 找袖口/腰带/裤档）
console.log('\n== row segments ==');
function segs(vv){
  const y=Math.round(by0+vv*fh);const cols=[];let inSeg=false,segL=0;
  for(let x=bx0;x<=bx1;x++){const a=img.px[(y*img.w+x)*4+3];const op=a>=80;
    if(op&&!inSeg){inSeg=true;segL=x;}else if(!op&&inSeg){inSeg=false;cols.push([segL,x-1]);}}
  if(inSeg)cols.push([segL,bx1]);
  return cols.map(([l,r])=>`[${((l-bx0)/fw).toFixed(3)},${((r-bx0)/fw).toFixed(3)}]w=${((r-l)/fw).toFixed(3)}`).join(' | ');
}
for(const vv of [0.24,0.26,0.28,0.32,0.36,0.40,0.44,0.46,0.48,0.50,0.52,0.54,0.56,0.58,0.60,0.64,0.70,0.76,0.86,0.94,0.97]){
  console.log(`v=${vv.toFixed(2)} y=${Math.round(by0+vv*fh)}: ${segs(vv)}`);
}
// 4) 腰带亮色(金属扣)扫描：v 0.48-0.54 找亮像素(>180)
console.log('\n== bright buckle pixels v 0.47→0.55 ==');
for(let v=0.47;v<=0.551;v+=0.01){let n=0,us=[];
  for(let u=0.30;u<=0.70;u+=0.005){const c=at(u,v);if(c.a>80&&c.r>150&&c.g>150&&c.b>150){n++;us.push(u.toFixed(2));}}
  if(n)console.log(`v=${v.toFixed(2)} bright=${n} u≈${us[0]}..${us[us.length-1]}`);
}
// 5) 大腿口袋/绑带：左腿柱 u=0.29 扫描 v 0.56→0.80 找比裤色亮的带状
console.log('\n== left leg column u=0.29 v 0.56→0.84 (找口袋/绑带亮度突变) ==');
let lastC=null;
for(let v=0.56;v<=0.841;v+=0.01){const c=at(0.29,v);
  const lum=(c.r*0.299+c.g*0.587+c.b*0.114);
  const mark=(lastC&&Math.abs(lum-lastC)>18)?' <-- jump':'';
  console.log(`v=${v.toFixed(2)} rgb(${c.r},${c.g},${c.b}) lum=${lum.toFixed(0)}${mark}`);
  lastC=lum;}
