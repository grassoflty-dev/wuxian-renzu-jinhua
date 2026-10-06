"""Strict, cache-independent PE/ICO/PNG checks using Python's standard library.
Only supports this candidate's bounded non-interlaced RGBA8 PNG frames.
Never launches the executable. No source/Cargo cache or Pillow is required.
"""
import hashlib, struct, zlib
H=lambda data:hashlib.sha256(data).hexdigest()

def png_rgba(data):
 assert data[:8]==b'\x89PNG\r\n\x1a\n','PNG signature'
 pos=8;idat=[];ihdr=None;ended=False
 while pos<len(data):
  assert pos+12<=len(data),'PNG chunk header'
  n=struct.unpack_from('>I',data,pos)[0];kind=data[pos+4:pos+8]
  assert n<=len(data)-pos-12,'PNG chunk length'
  payload=data[pos+8:pos+8+n]
  assert zlib.crc32(kind+payload)&0xffffffff==struct.unpack_from('>I',data,pos+8+n)[0],'PNG CRC'
  if kind==b'IHDR':assert ihdr is None and n==13;ihdr=struct.unpack('>IIBBBBB',payload)
  elif kind==b'IDAT':idat.append(payload)
  elif kind==b'IEND':assert n==0 and pos+n+12==len(data);ended=True
  pos+=n+12
 assert ended and ihdr is not None
 width,height,depth,color,compress,filt,interlace=ihdr
 assert 0<width<=1254 and 0<height<=1254 and (depth,color,compress,filt,interlace)==(8,6,0,0,0),ihdr
 stride=width*4;raw=zlib.decompress(b''.join(idat));assert len(raw)==height*(stride+1)
 result=bytearray();previous=bytearray(stride)
 def paeth(a,b,c):
  p=a+b-c;pa,pb,pc=abs(p-a),abs(p-b),abs(p-c)
  return a if pa<=pb and pa<=pc else b if pb<=pc else c
 for y in range(height):
  typ=raw[y*(stride+1)];row=bytearray(raw[y*(stride+1)+1:(y+1)*(stride+1)]);assert typ<=4
  for x in range(stride):
   a=row[x-4] if x>=4 else 0;b=previous[x];c=previous[x-4] if x>=4 else 0
   add=0 if typ==0 else a if typ==1 else b if typ==2 else (a+b)//2 if typ==3 else paeth(a,b,c)
   row[x]=(row[x]+add)&255
  result.extend(row);previous=row
 return width,height,bytes(result)

def parse_ico(data):
 assert len(data)>=6;reserved,typ,count=struct.unpack_from('<HHH',data)
 assert (reserved,typ,count)==(0,1,7),'expected seven ICO frames'
 frames=[];end=6+count*16
 for i in range(count):
  w,h,colors,reserved,planes,bits,size,offset=struct.unpack_from('<BBBBHHII',data,6+16*i)
  w=w or 256;h=h or 256
  assert colors==reserved==0 and planes in (0,1) and bits==32
  assert offset==end and size>0 and offset+size<=len(data),'ICO noncontiguous/overlapping frame'
  payload=data[offset:offset+size];pw,ph,rgba=png_rgba(payload);assert (w,h)==(pw,ph)
  frames.append({'index':i,'width':w,'height':h,'planes':planes,'bitCount':bits,'sizeBytes':size,'icoOffset':offset,'sha256':H(payload),'rgbaSha256':H(rgba),'payload':payload,'rgba':rgba})
  end=offset+size
 assert end==len(data) and [(f['width'],f['height']) for f in frames]==[(x,x) for x in [256,128,64,48,32,24,16]],'ICO order must choose 256px for Tauri entries()[0]'
 return frames

def parse_pe(pe):
 u16=lambda o:struct.unpack_from('<H',pe,o)[0]
 u32=lambda o:struct.unpack_from('<I',pe,o)[0]
 assert pe[:2]==b'MZ';ph=u32(0x3c);assert pe[ph:ph+4]==b'PE\0\0'
 assert u16(ph+4)==0x8664;opt=ph+24;assert u16(opt)==0x20b and u16(opt+68)==2
 sections=[];sh=opt+u16(ph+20)
 for i in range(u16(ph+6)):
  off=sh+i*40;vs,rva,rs,raw=struct.unpack_from('<IIII',pe,off+8);assert raw+rs<=len(pe)
  sections.append({'name':pe[off:off+8].rstrip(b'\0').decode(),'rva':rva,'virtualSize':vs,'rawOffset':raw,'rawSize':rs})
 def rvaoff(rva,size=1):
  for s in sections:
   rel=rva-s['rva']
   if 0<=rel and rel+size<=s['rawSize']:return s['rawOffset']+rel
  raise AssertionError(('unmapped RVA',rva,size))
 imports=[];idir=u32(opt+120)
 if idir:
  off=rvaoff(idir)
  for i in range(1024):
   fields=struct.unpack_from('<IIIII',pe,off+i*20)
   if not any(fields):break
   start=rvaoff(fields[3]);stop=pe.find(b'\0',start);assert stop>=start
   imports.append(pe[start:stop].decode('ascii'))
  else:raise AssertionError('unterminated import directory')
 assert not any(s.lower().startswith(('vcruntime','msvcp','api-ms-win-crt','msvcr')) or s.lower()=='ucrtbase.dll' for s in imports)
 rrva=u32(opt+128);rsize=u32(opt+132);rb=rvaoff(rrva,rsize);resources=[];visited=set()
 def bound(offset,size):assert 0<=offset<=rsize-size;return rb+offset
 def walk(offset,path=()):
  assert len(path)<=2 and offset not in visited;visited.add(offset)
  pos=bound(offset,16);named,ids=struct.unpack_from('<HH',pe,pos+12)
  for i in range(named+ids):
   pos=bound(offset+16+i*8,8);name,child=struct.unpack_from('<II',pe,pos)
   if name&0x80000000:
    so=name&0x7fffffff;length=u16(bound(so,2));name=pe[bound(so+2,length*2):rb+so+2+length*2].decode('utf-16le')
   if child&0x80000000:walk(child&0x7fffffff,path+(name,))
   else:
    assert len(path)==2
    rva,size,codepage,reserved=struct.unpack_from('<IIII',pe,bound(child,16));assert reserved==0
    off=rvaoff(rva,size);resources.append({'path':list(path+(name,)),'sizeBytes':size,'sha256':H(pe[off:off+size]),'fileOffset':off,'codepage':codepage})
 walk(0)
 assert len({tuple(r['path']) for r in resources})==len(resources)
 return {'resources':resources,'sections':sections,'imports':imports}
