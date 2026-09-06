#!/usr/bin/env python3
"""Generate component inventory and retain bundled dependency license texts."""
import hashlib,json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
cache=ROOT/'.cache';cache.mkdir(exist_ok=True)
metadata=cache/'native-metadata.json'
if not metadata.exists() or not metadata.stat().st_size:
 result=subprocess.run(['cargo','metadata','--locked','--format-version','1','--manifest-path',str(ROOT/'src-tauri/Cargo.toml')],capture_output=True,check=True)
 metadata.write_bytes(result.stdout)
rust=json.loads(metadata.read_text())['packages'];npm=json.loads((ROOT/'package-lock.json').read_text())['packages']
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
bom={'bomFormat':'CycloneDX','specVersion':'1.5','version':1,'metadata':{'component':{'type':'application','name':'QuickLAN','version':'0.1.0'}},'components':components}
(ROOT/'docs/evidence/dependencies.cdx.json').write_text(json.dumps(bom,indent=2)+'\n')
(ROOT/'licenses/DEPENDENCY_LICENSES.txt').write_text('QuickLAN engineering desktop dependency notices.\nIncludes target-conditional dependencies; not every listed component is linked on this host.\n'+''.join(notices))
report={'components':len(components),'missing_license_texts':missing,'limitation':'Metadata license expressions are not legal review. Resolve missing texts and inspect platform-only bundled resources before public redistribution.'}
(ROOT/'docs/evidence/license-inventory.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
