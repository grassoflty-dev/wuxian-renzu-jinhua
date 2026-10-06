#!/usr/bin/env python3
"""Read-only source-archive hash and Git tree verification. No Git checkout needed."""
if not __debug__:
 raise RuntimeError("Verification requires normal Python; do not use -O or -OO")

import hashlib,json,sys,zipfile
from pathlib import Path,PurePosixPath

def git_hash(kind,data):return hashlib.sha1(kind.encode()+b' '+str(len(data)).encode()+b'\0'+data).hexdigest()
def tree_hash(items):
 files={};dirs={}
 for path,mode,oid in items:
  first,sep,rest=path.partition('/')
  if sep:dirs.setdefault(first,[]).append((rest,mode,oid))
  else:files[first]=(mode,oid)
 entries=[(name,mode,oid,False) for name,(mode,oid) in files.items()]+[(name,'40000',tree_hash(rows),True) for name,rows in dirs.items()]
 raw=b''.join(mode.encode()+b' '+name.encode()+b'\0'+bytes.fromhex(oid) for name,mode,oid,isdir in sorted(entries,key=lambda e:(e[0]+('/' if e[3] else '')).encode()))
 return git_hash('tree',raw)
EXPECTED_COMMIT='1d8aff05f35b24c49355cf64aa58cd7eba561571'
EXPECTED_TREE='1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a'
EXPECTED_ARCHIVE_SHA256='2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033'
p=Path(sys.argv[1]);items=[];normalized=[]
with zipfile.ZipFile(p) as z:
 manifest=json.loads(z.read('SOURCE-MANIFEST.json'));assert manifest['sourceCommit']==EXPECTED_COMMIT and manifest['sourceTree']==EXPECTED_TREE;assert len(manifest['entries'])==1906 and manifest['entryCount']==1906 and manifest['projectFileCount']==757;expected={x['path'] for x in manifest['entries']}|{'SOURCE-MANIFEST.json'}
 assert len(z.namelist())==len(expected) and set(z.namelist())==expected,'missing, extra or duplicate archive paths'
 for row in manifest['entries']:
  name=row['path'];parts=PurePosixPath(name);assert not parts.is_absolute() and '..' not in parts.parts
  data=z.read(name);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],name
  if name.startswith('project/'):
   path=name[len('project/'):];canonical=data
   # Exact raw design docs and binary files preserve bytes. The verified source
   # export's only worktree/Git normalization differences are PowerShell CRLF.
   if path.endswith('.ps1'):canonical=data.replace(b'\r\n',b'\n')
   if canonical!=data:normalized.append(path)
   mode='100755' if (z.getinfo(name).external_attr>>16)&0o111 else '100644'
   items.append((path,mode,git_hash('blob',canonical)))
 assert len(items)==757
 computed=tree_hash(items);assert computed==manifest['sourceTree'],(computed,manifest['sourceTree'])
digest=hashlib.sha256()
with p.open('rb') as f:
 for chunk in iter(lambda:f.read(4*1024*1024),b''):digest.update(chunk)
assert digest.hexdigest()==EXPECTED_ARCHIVE_SHA256,'unexpected source ZIP hash'
result={'result':'pass','sourceCommitDeclared':manifest['sourceCommit'],'sourceTreeIndependentlyComputed':computed,'projectFiles':len(items),'archiveEntriesVerified':len(manifest['entries']),'normalizedPowerShellFiles':normalized,'archiveBytes':p.stat().st_size,'archiveSha256':digest.hexdigest(),'qualification':'Content tree independently verified; commit object/history intentionally absent from source archive.'}
print(json.dumps(result,indent=2))
