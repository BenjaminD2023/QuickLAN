#!/usr/bin/env python3
"""Verify that the DMG contains the exact production helper embedded by the app."""
import hashlib,json,os,pathlib,plistlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
if os.environ.get('CI')!='true':raise SystemExit('Disposable native CI only')
bundle=ROOT/'src-tauri/target/release/bundle'
images=list((bundle/'dmg').glob('*.dmg'))
assert len(images)==1
subprocess.run(['/usr/bin/hdiutil','verify',str(images[0])],check=True,capture_output=True)
with tempfile.TemporaryDirectory(prefix='quicklan-dmg-') as tmp:
 mount=pathlib.Path(tmp)/'mount';mount.mkdir()
 attached=False
 try:
  subprocess.run(['/usr/bin/hdiutil','attach','-readonly','-nobrowse','-mountpoint',str(mount),str(images[0])],check=True,capture_output=True)
  attached=True
  apps=list(mount.glob('*.app'));assert len(apps)==1
  app=apps[0];helper=app/'Contents/Resources/resources/engine/quicklan-engine'
  expected=hashlib.sha256((ROOT/'src-tauri/resources/engine/quicklan-engine').read_bytes()).hexdigest()
  assert helper.is_file() and hashlib.sha256(helper.read_bytes()).hexdigest()==expected, 'Bundled helper digest changed or resource path is wrong'
  assert subprocess.check_output([str(helper),'--version'],text=True).strip()=='quicklan-engine-0.2.0-easytier-2.6.4'
  probe=subprocess.run([str(helper),'--stdio-lab'],input=b'',capture_output=True,timeout=5)
  assert probe.returncode==2 and not probe.stdout
  info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
  assert info['CFBundleShortVersionString']==json.loads((ROOT/'package.json').read_text())['version']
  subprocess.run(['/usr/bin/codesign','--verify','--deep','--strict',str(app)],capture_output=True,check=True)
  # Exercise the helper directly from the mounted installer through authenticated
  # IPC. This tests packaged resources without needing interactive UI automation.
  subprocess.run(['/usr/bin/sudo','--preserve-env=CI',str(ROOT/'target/debug/examples/engine_smoke'),str(helper)],check=True,timeout=180)
 finally:
  if attached:subprocess.run(['/usr/bin/hdiutil','detach',str(mount)],check=True,capture_output=True)
report={'dmg_sha256':hashlib.sha256(images[0].read_bytes()).hexdigest(),'helper_sha256':expected,
 'checks':['DMG verified and mounted read-only','Embedded helper digest and production mode verified','Bundle signature integrity verified','Packaged native helper creates and cleans up real adapter'],
 'limitations':['Ad-hoc signature integrity is not Apple Developer ID signing or notarization','Does not automate interactive macOS elevation approval or physical cross-device traffic']}
(bundle/'macos-smoke.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
