// Offline regression test: no real credentials, network, generation or output writes.
// Run: node --experimental-vm-modules tools/test-local-media-guards.mjs
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const source = fs.readFileSync(new URL('./img_gen.mjs', import.meta.url), 'utf8');
for (const argv of [[], ['prompt', 'never-created.png'], ['--help']]) {
  let effects = 0;
  const deny = () => { effects++; throw new Error('FORBIDDEN_SIDE_EFFECT'); };
  const context = vm.createContext({
    process: { argv: ['node', 'img_gen.mjs', ...argv], env: { ALLOW_CLOUD: '1' }, exit: deny },
    fetch: deny, console: { log: deny, error: deny }, Buffer,
  });
  const module = new vm.SourceTextModule(source, { context });
  const mocks = {
    'node:fs': { readFileSync: deny, mkdirSync: deny, writeFileSync: deny },
    'node:os': { homedir: deny },
    'node:path': { join: deny, dirname: deny },
  };
  await module.link((name) => {
    assert.ok(Object.hasOwn(mocks, name), `Unexpected import: ${name}`);
    return new vm.SyntheticModule(['default'], function () {
      this.setExport('default', mocks[name]);
    }, { context });
  });
  await assert.rejects(module.evaluate(), { code: 'LOCAL_MEDIA_ONLY' });
  assert.equal(effects, 0, 'JS guard must precede credentials, network and writes');
}
console.log('PASS JavaScript: 3 argument variants, zero side effects');

// Use the existing small assets interpreter, never import model dependencies.
const python = fileURLToPath(new URL('./.venv-assets/Scripts/python.exe', import.meta.url));
const result = spawnSync(python, ['-I', '-B', '-c', String.raw`
import ast, builtins, io, pathlib, socket, sys, urllib.request
path = pathlib.Path('tools/design/gen_wan.py')
source = path.read_text(encoding='utf-8')
code = compile(source, str(path), 'exec')
# Verify the guard is the first executable statement after the historical docstring.
tree = ast.parse(source)
assert isinstance(tree.body[1], ast.Raise)
effects = []
def deny(*args, **kwargs):
    effects.append('forbidden')
    raise AssertionError('FORBIDDEN_SIDE_EFFECT')
builtins.open = deny
io.open = deny
socket.socket = deny
socket.create_connection = deny
urllib.request.urlopen = deny
for name in ('__main__', 'historical_import'):
    for argv in ([], ['out.png', 'prompt'], ['--help']):
        sys.argv = [str(path)] + argv
        try:
            exec(code, {'__name__': name})
        except SystemExit as error:
            assert str(error).startswith('LOCAL_MEDIA_ONLY:')
        else:
            raise AssertionError('Cloud entry did not fail closed')
assert effects == [], effects
print('PASS Python: CLI/import x 3 argument variants, zero side effects')
`], { cwd: root, stdio: 'inherit', windowsHide: true });
if (result.error) throw result.error;
assert.equal(result.status, 0, 'Python offline guard test failed');
console.log('PASS all cloud-entry guards; no network or media generation performed');
