import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { Container, Sprite } from 'pixi.js';
import { returnStationArchitecture, projectArchitectureFace, isReturnStationRearWall } from '../dist/renderer/ReturnStationArchitecture.js';
import { projectWorldPoint } from '../dist/renderer/CameraModel.js';
import { WorldRenderer } from '../dist/renderer/WorldRenderer.js';

const faces = () => returnStationArchitecture('return_station','rs_core_room');
const camera = { width:1280,height:720,origin:{xM:4,yM:0,zM:8},pixelsPerMeter:48 };

test('architecture is exact-scene, deterministic and confined to the authored north wall',async()=>{
  assert.deepEqual(returnStationArchitecture('grey_hive','rs_core_room'),[]);
  assert.deepEqual(returnStationArchitecture('return_station','other'),[]);
  assert.deepEqual(faces(),faces());
  const scene=JSON.parse(await readFile(new URL('../../../content/scenes/compiled/rs_core_room.json',import.meta.url),'utf8'));
  const wall=scene.collision.find(c=>c.id==='north_bulkhead').polygon;
  assert.ok(faces().length>100);
  for(const face of faces()) for(const p of face.points) {
    assert.ok(Object.values(p).every(Number.isFinite));
    assert.ok(p.xM>=wall[0][0]&&p.xM<=wall[1][0]);
    assert.ok(p.zM>=wall[0][1]&&p.zM<=wall[2][1]);
    assert.ok(p.yM>=0);
  }
});

test('geometry uses unchanged world projection at 720p/1080p and camera movement',()=>{
  for(const c of [camera,{...camera,width:1920,height:1080,pixelsPerMeter:72},{...camera,origin:{xM:9,yM:0,zM:5}}]) {
    const before=structuredClone(c);
    for(const face of faces()) assert.deepEqual(projectArchitectureFace(face,c),face.points.flatMap(p=>{const q=projectWorldPoint(p,c);return[q.x,q.y];}));
    assert.deepEqual(c,before);
  }
});

test('actual Pixi architecture stays behind actors, has no input, reuses and releases its graphic',()=>{
  const r=Object.create(WorldRenderer.prototype);
  r.layerContainers=new Map(['L2_BACK_PROPS','L3_ACTORS'].map(k=>[k,new Container()]));
  r.renderReturnStationArchitecture('return_station','rs_core_room',camera,true);
  const g=r.returnStationArchitectureGraphic;
  assert.equal(g.parent,r.layerContainers.get('L2_BACK_PROPS'));
  assert.match(g.label,/temporary_visual/);assert.equal(g.eventMode,'none');
  assert.equal(g.zIndex,-8000);
  r.renderReturnStationArchitecture('return_station','rs_core_room',{...camera,width:1920},true);
  assert.equal(r.returnStationArchitectureGraphic,g);
  r.renderReturnStationArchitecture('grey_hive','gh_entry_maintenance',camera,true);
  assert.equal(r.returnStationArchitectureGraphic,null);assert.equal(g.destroyed,true);
  r.renderReturnStationArchitecture('return_station','rs_core_room',camera,false);
  assert.equal(r.returnStationArchitectureGraphic,null);
  r.renderReturnStationArchitecture('return_station','rs_core_room',camera,true);
  const again=r.returnStationArchitectureGraphic;r.clearFacilityFloor();
  assert.equal(again.destroyed,true);assert.equal(r.returnStationArchitectureGraphic,null);
});

test('only exact return-station north wall panels move behind ring and actors; reuse restores ordinary depth',()=>{
  const r=Object.create(WorldRenderer.prototype);
  r.layerContainers=new Map(['L2_BACK_PROPS','L3_ACTORS','L5_FRONT_PROPS'].map(k=>[k,new Container()]));
  const sprite=new Sprite();const key='scene:approved_facility_walls:panel:0:0.75';
  const asset='runtime2d.grey_hive.wall_tiles.v1';
  assert.equal(isReturnStationRearWall('return_station','rs_core_room',key,asset,'L2_BACK_PROPS'),true);
  r.applySceneSpriteDepth('return_station','rs_core_room',key,asset,'L2_BACK_PROPS',sprite,70);
  assert.equal(sprite.parent,r.layerContainers.get('L2_BACK_PROPS'));assert.equal(sprite.zIndex,-9000);
  r.renderReturnStationArchitecture('return_station','rs_core_room',camera,true);
  assert.ok(sprite.zIndex<r.returnStationArchitectureGraphic.zIndex);
  for(const [world,scene,id,art,layer] of [
    ['grey_hive','gh_entry_maintenance',key,asset,'L2_BACK_PROPS'],
    ['return_station','other',key,asset,'L2_BACK_PROPS'],
    ['return_station','rs_core_room','scene:approved_facility_walls:panel:1:0.75',asset,'L2_BACK_PROPS'],
    ['return_station','rs_core_room','actor:player','runtime2d.actor.cenyao.base.v1','L3_ACTORS'],
    ['return_station','rs_core_room','scene:mission_terminal_art','runtime2d.world.returnstation.terminal.v1','L2_BACK_PROPS'],
    ['return_station','rs_core_room',key,'other','L2_BACK_PROPS'],
  ]) {
    r.applySceneSpriteDepth(world,scene,id,art,layer,sprite,70);
    assert.equal(sprite.parent,r.layerContainers.get('L3_ACTORS'));assert.equal(sprite.zIndex,70);
  }
  r.clearFacilityFloor();sprite.destroy();
});

test('authoritative room bytes, including collision, interactions and navigation, are unchanged',async()=>{
  const bytes=await readFile(new URL('../../../content/scenes/compiled/rs_core_room.json',import.meta.url));
  assert.equal(createHash('sha256').update(bytes).digest('hex'),'927f8dc466403837b081027bafb04c3c66a6ca17ee71bf5ce0a7d891e4fb3ad9');
});
