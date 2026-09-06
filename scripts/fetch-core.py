#!/usr/bin/env python3
"""Fetch verified, pinned upstream artifacts for developer integration only."""
import argparse, hashlib, json, os, pathlib, platform, tempfile, urllib.request, zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
MANIFEST=json.loads((ROOT/'upstream/easytier.lock.json').read_text())
TARGETS=['macos-aarch64','macos-x86_64','windows-x86_64','linux-x86_64']
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
def fetch(target):
 name=f'easytier-{target}-{MANIFEST["tag"]}.zip';spec=MANIFEST['artifacts'][name]
 cache=ROOT/'.cache';cache.mkdir(exist_ok=True)
 if cache.is_symlink():raise RuntimeError('Refusing symlink cache directory')
 archive=cache/name
 if archive.exists():
  if archive.is_symlink() or digest(archive)!=spec['sha256']:raise RuntimeError('Cached archive digest mismatch; inspect/remove it explicitly')
 else:
  with tempfile.NamedTemporaryFile(dir=cache,delete=False) as f:
   temporary=pathlib.Path(f.name)
   try:
    with urllib.request.urlopen(spec['url'],timeout=60) as response:
     count=0
     while block:=response.read(1024*1024):
      count+=len(block)
      if count>200_000_000:raise RuntimeError('Unexpected archive size')
      f.write(block)
    f.flush();os.fsync(f.fileno())
    if digest(temporary)!=spec['sha256']:raise RuntimeError('Downloaded archive digest mismatch')
    temporary.replace(archive)
   finally:temporary.unlink(missing_ok=True)
 output=cache/f'easytier-{target}';output.mkdir(exist_ok=True)
 if output.is_symlink():raise RuntimeError('Refusing symlink output directory')
 suffix='.exe' if target.startswith('windows') else ''
 records=[]
 with zipfile.ZipFile(archive) as z:
  for binary in ['easytier-core','easytier-cli']:
   member=f'easytier-{target}/{binary}{suffix}';info=z.getinfo(member)
   if info.file_size>100_000_000 or (info.external_attr>>16)&0o170000==0o120000:raise RuntimeError('Unexpected archive member')
   data=z.read(member);path=output/(binary+suffix)
   if path.is_symlink():raise RuntimeError('Refusing executable symlink')
   if path.exists() and path.read_bytes()!=data:raise RuntimeError('Existing executable differs; inspect it explicitly')
   if not path.exists():
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o700)
    with os.fdopen(fd,'wb') as f:f.write(data)
   path.chmod(0o700)
   records.append({'binary':binary+suffix,'sha256':hashlib.sha256(data).hexdigest()})
 record={'target':target,'archive':name,'archive_sha256':digest(archive),'binaries':records,'executed':False}
 (ROOT/f'docs/evidence/artifact-{target}.json').write_text(json.dumps(record,indent=2)+'\n')
 print(f'{target}: archive verified; extracted core and CLI. Not approved for privileged execution.')
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--target',choices=TARGETS,default='macos-aarch64' if platform.system()=='Darwin' and platform.machine()=='arm64' else None);parser.add_argument('--all',action='store_true');args=parser.parse_args()
 if not args.all and not args.target:parser.error('Choose an explicit supported artifact target')
 for target in TARGETS if args.all else [args.target]:fetch(target)
