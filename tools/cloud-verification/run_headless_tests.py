#!/usr/bin/env python3
"""Run every compiled real-source test in isolation with a per-test deadline.
Timeouts are reported separately from assertion failures, never converted to passes. Pass binaries JSON
from cargo test --lib --tests --no-run --message-format=json as argv[1].
argv[2] is the report directory; argv[3] is the headless server-rs directory.
"""
import concurrent.futures, json, os, pathlib, re, signal, subprocess, sys, tempfile, time
artifacts=pathlib.Path(sys.argv[1]).resolve(); out=pathlib.Path(sys.argv[2]).resolve(); out.mkdir(parents=True,exist_ok=True)
crate=pathlib.Path(sys.argv[3]).resolve()
rows=[]
for line in artifacts.read_text().splitlines():
    try: data=json.loads(line)
    except ValueError: continue
    if data.get('reason')=='compiler-artifact' and data.get('profile',{}).get('test') and data.get('executable'):
        exe=data['executable']; target=data['target']['name']
        listing=subprocess.run([exe,'--list'],capture_output=True,text=True,check=True).stdout
        for test in listing.splitlines():
            if test.endswith(': test'): rows.append((exe,target,test[:-6]))
(out/'inventory.json').write_text(json.dumps([{'target':r[1],'test':r[2]} for r in rows],indent=2)+'\n')
def run(row):
    exe,target,test=row; started=time.monotonic(); deadline=600 if target in {'production_scene_bootstrap','gh_first_three_scenes'} else 60; result={'target':target,'test':test,'deadlineSeconds':deadline}
    with tempfile.TemporaryDirectory(prefix='game-headless-test-') as temporary:
        env = dict(os.environ, TMPDIR=temporary, TEMP=temporary, TMP=temporary)
        proc = subprocess.Popen([exe,'--exact',test,'--nocapture','--test-threads=1'],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
            cwd=crate, env=env, start_new_session=True)
        try:
            output, _ = proc.communicate(timeout=deadline)
            result.update(status='passed' if proc.returncode==0 else 'failed',exitCode=proc.returncode)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            output, _ = proc.communicate()
            result.update(status='timed_out')
    result['seconds']=round(time.monotonic()-started,3)
    if result['status']!='passed':
        name=re.sub(r'[^a-zA-Z0-9._-]','_',target+'__'+test)[:220]+'.log'
        (out/name).write_text(output); result['log']=name
    return result
results=[]
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    for row in pool.map(run,rows):
        results.append(row)
        if row['status']!='passed': print(json.dumps(row),flush=True)
        (out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
summary={kind:sum(r['status']==kind for r in results) for kind in ['passed','failed','timed_out']}
summary.update(total=len(results),execution='One subprocess per exact unchanged test; four processes maximum; isolated temporary directory per test;600s production route/three-scene tests,60s others; no skipped tests or modified assertions')
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n'); print(json.dumps(summary),flush=True)
sys.exit(bool(summary['failed'] or summary['timed_out']))
