#!/usr/bin/env python3
"""Build-only adapter: real library/tests + production content gates, without native shell.
Does not alter reference source, production Cargo.toml/build.rs, or assertions.
Usage: prepare_headless.py SOURCE_ROOT OUTPUT_ROOT
"""
import hashlib, json, pathlib, shutil, sys
source, output = map(lambda p: pathlib.Path(p).resolve(), sys.argv[1:3])
if output.exists():
    raise SystemExit(f'refusing to overwrite existing harness: {output}')
output.mkdir(parents=True)
for name in ['apps','assets','content','design','docs','governance','tools']:
    (output/name).symlink_to(source/name, target_is_directory=True)
shutil.copytree(source/'server-rs',output/'server-rs',ignore=shutil.ignore_patterns('target'))
crate=output/'server-rs'
manifest=(crate/'Cargo.toml').read_text()
manifest=manifest.replace('edition = "2021"','edition = "2021"\nautobins = false')
for dep in ['tauri-build = { version = "2", features = [] }\n','tauri = { version = "2", features = [] }\n']:
    if manifest.count(dep)!=1: raise SystemExit(f'expected exact dependency: {dep}')
    manifest=manifest.replace(dep,'')
(crate/'Cargo.toml').write_text(manifest)
original=(crate/'build.rs').read_text()
start=original.index('    let identity_module = manifest_dir.join("build_support/bundle_identity.rs");')
end=original.index('    let mut input_json = BTreeMap::new();',start)
removed=original[start:end]
if removed.count('write_native_bundle_identity(&out_dir, &bundle_identity)?;')!=1: raise SystemExit('unexpected identity gate')
adapted=original[:start]+'    // HEADLESS TEST HARNESS ONLY: native Web dist identity gate not exercised.\n\n'+original[end:]
if adapted.count('    tauri_build::build();')!=1: raise SystemExit('unexpected Tauri gate')
adapted=adapted.replace('    tauri_build::build();','    // HEADLESS TEST HARNESS ONLY: no Tauri OS build or executable.')
(crate/'build.rs').write_text(adapted)
report={'scope':'Headless real-source library/integration tests. NOT native build or release validation.','source':str(source),'output':str(output),'originalBuildSha256':hashlib.sha256(original.encode()).hexdigest(),'excluded':['Tauri dependency/OS build','native Web dist identity gate (no native target exists)'],'preserved':['every game source/test byte','locked production scene SHA256 validation','manifest admission checks','all runtime assertions']}
(output/'HARNESS-SCOPE.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
