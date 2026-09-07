#!/usr/bin/env python3
"""Generate component inventory and retain bundled dependency license texts."""
import argparse,hashlib,json,pathlib,subprocess
parser=argparse.ArgumentParser()
targets=parser.add_mutually_exclusive_group()
targets.add_argument("--engine", action="store_true")
targets.add_argument("--android", action="store_true")
args=parser.parse_args()
ROOT=pathlib.Path(__file__).resolve().parents[1]
cache=ROOT/'.cache';cache.mkdir(exist_ok=True)
kind='android' if args.android else 'engine' if args.engine else 'native'
manifest='android/native/Cargo.toml' if args.android else 'engine/Cargo.toml' if args.engine else 'src-tauri/Cargo.toml'
metadata=cache/f'{kind}-metadata.json'
result=subprocess.run(['cargo','metadata','--locked','--format-version','1','--manifest-path',str(ROOT/manifest)],capture_output=True,check=True)
metadata.write_bytes(result.stdout)
rust=json.loads(metadata.read_text())['packages'];npm={} if args.engine else json.loads((ROOT/'package-lock.json').read_text())['packages']
components=[];notices=[];missing=[]
supplemental=ROOT/'licenses/supplemental'
sources=json.loads((supplemental/'sources.json').read_text()) if (supplemental/'sources.json').exists() else []
def append(name,version,license_expression,root,ecosystem):
 if license_expression:license_expression=license_expression.replace('/', ' OR ')
 components.append({'type':'library','name':name,'version':version,'purl':f'pkg:{ecosystem}/{name}@{version}',**({'licenses':[{'expression':license_expression}]} if license_expression else {})})
 paths=[p for p in root.iterdir() if p.is_file() and not p.is_symlink() and any(p.name.upper().startswith(k) for k in ('LICENSE','LICENCE','COPYING','NOTICE'))]
 extra=[s for s in sources if f'{ecosystem}:{name}@{version}' in s['packages']]
 if not paths and not extra:missing.append(f'{ecosystem}:{name}@{version}')
 notices.append(f'\n{"="*72}\n{name} {version}\nDeclared license: {license_expression or "unspecified"}\n')
 for path in sorted(paths):
  if path.stat().st_size<500_000:notices.append(f'\n--- {path.name} ---\n'+path.read_text(errors='replace')+'\n')
 for source in extra:
  path=supplemental/source['file']
  if hashlib.sha256(path.read_bytes()).hexdigest()!=source['sha256']:raise RuntimeError('Supplemental license digest mismatch')
  notices.append(f"\n--- {source['url']} ---\n"+path.read_text(errors='replace')+'\n')
for p in sorted(rust,key=lambda p:(p['name'],p['version'])):
 if p.get('source'):append(p['name'],p['version'],p.get('license'),pathlib.Path(p['manifest_path']).parent,'cargo')
for key,spec in sorted(npm.items()):
 if not key or spec.get('dev') or spec.get('optional'):continue
 directory=ROOT/key
 if not directory.is_dir():continue
 manifest=json.loads((directory/'package.json').read_text())
 license_expression=manifest.get('license')
 if not isinstance(license_expression,str):license_expression=None
 append(manifest['name'],spec['version'],license_expression,directory,'npm')
bom={'bomFormat':'CycloneDX','specVersion':'1.5','version':1,'metadata':{'component':{'type':'application','name':'QuickLAN','version':json.loads((ROOT/'package.json').read_text())['version']}},'components':components}
prefix='android-' if args.android else 'engine-' if args.engine else ''
(ROOT/f'docs/evidence/{prefix}dependencies.cdx.json').write_text(json.dumps(bom,indent=2)+'\n')
notice_name='ANDROID_DEPENDENCY_LICENSES.txt' if args.android else 'ENGINE_DEPENDENCY_LICENSES.txt' if args.engine else 'DEPENDENCY_LICENSES.txt'
(ROOT/'licenses'/notice_name).write_text(f'QuickLAN {kind} dependency notices.\nIncludes target-conditional dependencies; not every listed component is linked on this host.\n'+''.join(notices))
report={'components':len(components),'missing_license_texts':missing,'limitation':'Metadata license expressions are not legal review. Resolve missing texts and inspect platform-only bundled resources before public redistribution.'}
(ROOT/f'docs/evidence/{prefix}license-inventory.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
