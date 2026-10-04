import test from 'node:test';
import assert from 'node:assert/strict';
import { CoreUiPresenter } from '../dist/ui/CoreUiPresenter.js';
import { AccessibilityPreferenceStore } from '../dist/ui/AccessibilityPreferences.js';
class Element {
  children=[]; dataset={}; attributes={}; listeners={}; hidden=false; disabled=false; value=''; tabIndex=0;
  append(...items){this.children.push(...items);} replaceChildren(...items){this.children=items;}
  setAttribute(k,v){this.attributes[k]=v;} addEventListener(k,f){(this.listeners[k]??=[]).push(f);}
  focus(){document.activeElement=this;} contains(n){return n===this||this.children.some(c=>c.contains(n));}
  fire(type,event={}){for(const f of this.listeners[type]??[])f(event);}
}
function harness(){
  const oldDocument=globalThis.document,oldElement=globalThis.HTMLElement;
  globalThis.HTMLElement=Element;globalThis.document={activeElement:null,createElement:()=>new Element()};
  const root=new Element();root.hidden=true;
  const nodes=Object.fromEntries(['title','subtitle','content','close'].map(k=>[k,new Element()]));
  const tab=new Element();tab.dataset.corePanel='settings';root.append(nodes.title,nodes.subtitle,nodes.close,tab,nodes.content);
  root.querySelector=s=>nodes[s.replace('#core-panel-','')];
  const descendants=n=>[n,...n.children.flatMap(descendants)];
  root.querySelectorAll=s=>s==='[data-core-panel]'?[tab]:descendants(root).filter(n=>n===nodes.close||n===tab||n.id?.startsWith('settings-'));
  const store=new AccessibilityPreferenceStore(()=>null,{dataset:{}});store.restore();
  const presenter=new CoreUiPresenter(root,store);const trigger=new Element();presenter.open(trigger,'settings');
  return{root,nodes,tab,store,presenter,trigger,find:id=>descendants(root).find(n=>n.id===id),restore(){globalThis.document=oldDocument;globalThis.HTMLElement=oldElement;}};
}
test('actual settings presenter applies both controls and preserves focus across snapshots and reopen',()=>{
  const h=harness();try{
    const text=h.find('settings-textScalePercent'),ui=h.find('settings-uiScalePercent');
    assert.deepEqual(text.children.map(x=>x.value),['100','115','130']);assert.deepEqual(ui.children.map(x=>x.value),['100','110','125']);
    text.focus();text.value='130';text.fire('change');assert.equal(document.activeElement,text);assert.equal(h.store.getScale().textScalePercent,130);
    h.presenter.apply({worldEpoch:9,worldId:'grey_hive',sceneId:'gh_entry_maintenance'});
    assert.equal(document.activeElement,text);assert.equal(h.find(text.id),text);
    ui.value='125';ui.fire('change');h.presenter.close();assert.equal(document.activeElement,h.trigger);
    h.presenter.open(h.trigger,'settings');assert.equal(h.find(text.id).value,'130');assert.equal(h.find(ui.id).value,'125');
    const last=h.find(ui.id);last.focus();let prevented=false;h.root.fire('keydown',{key:'Tab',shiftKey:false,preventDefault(){prevented=true;}});
    assert.ok(prevented);assert.equal(document.activeElement,h.nodes.close);
  }finally{h.restore();}
});
