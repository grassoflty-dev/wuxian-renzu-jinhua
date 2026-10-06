#!/usr/bin/env python3
"""Read-only checks of exact preview.5 bytes; never executes the EXE.
Python3 and Brotli (or a Linux/macOS system decoder) are required. No source extraction or network.
"""
if not __debug__:
 raise RuntimeError("Verification requires normal Python; do not use -O or -OO")

from pathlib import Path
import argparse,hashlib,json,mmap,struct,zipfile
from brotli_decoder import get_decoder
from icon_parser import parse_pe,parse_ico,png_rgba
H=lambda b:hashlib.sha256(b).hexdigest()
ROOT=Path(__file__).resolve().parent
SHA='1d8aff05f35b24c49355cf64aa58cd7eba561571'
TREE='1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a'
EXE_SHA='e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da'
ZIP_SHA='2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033'
def file_hash(path):
 digest=hashlib.sha256()
 with path.open('rb') as f:
  for chunk in iter(lambda:f.read(4*1024*1024),b''):digest.update(chunk)
 return digest.hexdigest()
def check_icons(pe,resources,ico,provenance):
 frames=parse_ico(ico);conv=provenance['conversion']
 assert len(frames)==7 and len(conv['frames'])==7
 assert H(ico)==conv['sha256'] and len(ico)==conv['bytes']
 for frame,ref in zip(frames,conv['frames']):
  assert [frame['width'],frame['height']]==ref['size']
  assert (frame['sha256'],frame['rgbaSha256'],frame['sizeBytes'],frame['icoOffset'])==(ref['sha256'],ref['rgbaSha256'],ref['bytes'],ref['offset'])
 icons=[r for r in resources if r['path'][0]==3];groups=[r for r in resources if r['path'][0]==14]
 assert len(icons)==7 and len(groups)==1
 by_id={(r['path'][1],r['path'][2]):r for r in icons};seen=set()
 group=groups[0];data=pe[group['fileOffset']:group['fileOffset']+group['sizeBytes']]
 assert struct.unpack_from('<HHH',data)==(0,1,7) and len(data)==104
 for i,frame in enumerate(frames):
  w,h,colors,reserved,planes,bits,size,rid=struct.unpack_from('<BBBBHHIH',data,6+14*i);w=w or 256;h=h or 256
  assert colors==reserved==0
  key=(rid,group['path'][2]);assert key in by_id and key not in seen;seen.add(key)
  row=by_id[key];payload=pe[row['fileOffset']:row['fileOffset']+row['sizeBytes']]
  assert (w,h,planes,bits,size)==(frame['width'],frame['height'],frame['planes'],frame['bitCount'],frame['sizeBytes'])
  assert payload==frame['payload'] and png_rgba(payload)[:2]==(w,h)
 assert seen==set(by_id)
 assert pe.find(frames[0]['rgba'])>=0
 return frames

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('exe',type=Path);parser.add_argument('source_zip',type=Path);args=parser.parse_args()
 assert args.exe.stat().st_size==255988736 and file_hash(args.exe)==EXE_SHA,'unexpected EXE'
 assert args.source_zip.stat().st_size==428215921 and file_hash(args.source_zip)==ZIP_SHA,'unexpected source ZIP'
 p=json.loads((ROOT/'PUBLIC-NATIVE-EVIDENCE.json').read_text());side=(ROOT/'bundle-identity.json').read_bytes();sj=json.loads(side)
 assert p['sourceSha']==SHA and p['sourceTree']==TREE and p['sha256']==EXE_SHA
 assert H(side)==p['nativeSidecarSha256'];identity=p['nativeBuildIdentity']
 assert identity==sj['buildIdentity'] and identity['gitSha']==SHA and identity['builtAtUtc']=='2026-10-04T13:20:41.716Z'
 assert all(identity[k] is False for k in ['hasTrackedDiff','hasUntrackedFiles','sourceTreeDirty'])
 expected={x['path']:(x['sizeBytes'],x['sha256']) for x in sj['files']};expected['bundle-identity.json']=(len(side),H(side));assert len(expected)==163
 decoder=get_decoder()
 with zipfile.ZipFile(args.source_zip) as z,args.exe.open('rb') as f,mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as pe:
  parsed=parse_pe(pe);assert parsed['imports']==p['imports'] and parsed['resources']==p['resources'] and parsed['sections']==p['sections']
  def extract(row):
   pos=row['fileOffset'];size=row['sizeBytes'];assert 0<=pos<=len(pe)-size
   data=pe[pos:pos+size];assert H(data)==row['sha256'];return data
  for row in p['resources']:extract(row)
  ico=z.read('project/server-rs/icons/icon.ico');prov=json.loads(z.read('project/server-rs/icons/ICON_PROVENANCE.json'))
  original=z.read('project/server-rs/icons/source/chatgpt-portal-emblem-20261003.png')
  assert H(original)==prov['conversion']['sourceSha256'] and len(original)==prov['conversion']['sourceBytes']
  assert png_rgba(original)[:2]==tuple(prov['conversion']['sourceSize'])
  frames=check_icons(pe,parsed['resources'],ico,prov)
  assert extract(p['nativeIconRGBA'])==frames[0]['rgba']
  seen_scenes=set()
  for row in p['nativeScenes']:
   assert extract(row)==z.read('project/content/scenes/compiled/'+row['name']);seen_scenes.add(row['name'])
  assert len(seen_scenes)==30
  seen=set();decoded_side=None
  for row in p['embeddedWebAssets']:
   raw=extract(row)
   if row['encoding']=='brotli':
    raw=decoder(raw,row['decodedBytes'])
   else:assert row['encoding']=='raw'
   assert len(raw)==row['decodedBytes'] and H(raw)==row['decodedSha256']
   for path in row['paths']:
    assert expected[path]==(len(raw),H(raw)) and pe.find(('/'+path).encode())>=0;seen.add(path)
    if path=='bundle-identity.json':decoded_side=raw
  assert seen==set(expected) and decoded_side==side and len(p['embeddedWebAssets'])==163
  assert pe.find(p['nativeBuildIdentityJsonExact'].encode())>=0 and json.loads(p['nativeBuildIdentityJsonExact'])==identity
  assert pe.find(p['nativeSidecarSha256'].encode())>=0
 print(json.dumps({'result':'pass','sourceCommit':SHA,'sourceTree':TREE,'exeSha256':EXE_SHA,'sourceZipSha256':ZIP_SHA,'iconFrames':7,'defaultWindowIcon':[256,256],'nativeScenes':30,'webFiles':163,'WindowsExecuted':False,'qualification':'Positive closure recheck; not Windows runtime, performance, legal or byte-reproducible-build certification.'},indent=2))
if __name__=='__main__':main()
