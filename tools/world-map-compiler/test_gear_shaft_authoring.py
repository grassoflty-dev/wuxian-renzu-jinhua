"""Exact authored development-tuned lift/deck contract; no enemy rebalance."""
import hashlib,json,tempfile,unittest
from pathlib import Path
from compile_maps import compile_directory,compile_map,load_admitted_assets,MapError
ROOT=Path(__file__).resolve().parents[2]
class GearShaftAuthoringTests(unittest.TestCase):
 def test_repeatable_authored_main_route_and_earned_shortcut(self):
  with tempfile.TemporaryDirectory() as tmp:
   a,b=Path(tmp)/'a',Path(tmp)/'b'
   args=[ROOT/'design/maps',ROOT/'governance/assets/RUNTIME_ASSET_MANIFEST.json',ROOT/'governance/assets/AI_ASSET_RELEASE_MANIFEST.json']
   tail=[ROOT/'server-rs/data/world_progression_v1.json',ROOT/'content/enemies/entity-types.json']
   scenes=compile_directory(*args,a,*tail);compile_directory(*args,b,*tail)
   self.assertEqual(len(scenes),30)
   for p in a.glob('*.json'):
    self.assertEqual(p.read_bytes(),(b/p.name).read_bytes());self.assertEqual(p.read_bytes(),(ROOT/'content/scenes/compiled'/p.name).read_bytes())
   s=next(s for s in scenes if s['sceneId']=='cw_gear_shaft')
   self.assertEqual(s['movingSupports'],[dict(id='cw_gear_main_lift',polygon=[[10.,6.],[15.,6.],[15.,10.],[10.,10.]],lowerM=0.,upperM=2.,travelMs=4000,endpointHoldMs=2000)])
   self.assertEqual(s['standingDecks'],[dict(id='cw_gear_upper_dock',polygon=[[14.,6.],[18.5,6.],[18.5,10.],[14.,10.]],heightM=2.),dict(id='cw_gear_furnace_landing',polygon=[[19.5,6.],[24.,6.],[24.,10.],[19.5,10.]],heightM=2.)])
   t={t['id']:t for t in s['logic']['traversal']};self.assertEqual(set(t),{'cw_gear_upper_gap','cw_gear_earned_air_step'})
   self.assertEqual(t['cw_gear_upper_gap'],dict(id='cw_gear_upper_gap',**{'from':[17.8,2.,8.]},to=[20.2,2.,8.],rangeM=1.2,cooldownMs=350,requiredCapabilities=[],fromHeightRangeM=[1.9,2.1]))
   self.assertEqual(t['cw_gear_earned_air_step'],dict(id='cw_gear_earned_air_step',**{'from':[11.,0.,8.]},to=[16.,2.,8.],rangeM=1.2,cooldownMs=350,requiredCapabilities=['mobility.air_step_i'],fromHeightRangeM=[0.,.1]))
   self.assertEqual({t['id']:t['heightRangeM'] for t in s['transitions']},{'cw_gear_return_to_boiler':[0.,.1],'cw_gear_to_furnace_heart':[1.9,2.1]})
   self.assertEqual(s['interactions'],[]);self.assertEqual(s['hazards'],[])
   self.assertEqual([(p['id'],p['position']) for p in s['checkpoints']],[('cw_gear_shaft_checkpoint',[4.,0.,8.]),('cw_gear_upper_dock_checkpoint',[16.,2.,8.])])
   self.assertEqual([(p['id'],p['entityType'],p['position']) for p in s['spawns'] if p['kind']=='enemy'],[
     ('cw_gear_shaft_forged_guard_01','enemy.clockworks.forged_guard',[6.5,0.,8.]),('cw_gear_shaft_forged_guard_02','enemy.clockworks.forged_guard',[9.5,0.,8.]),
     ('cw_gear_shaft_pressure_drone_01','enemy.clockworks.pressure_drone',[11.,0.,4.]),('cw_gear_shaft_pressure_drone_02','enemy.clockworks.pressure_drone',[16.,0.,12.]),('cw_gear_shaft_pressure_drone_03','enemy.clockworks.pressure_drone',[20.,0.,5.])])
 def test_missing_height_band_and_unsupported_shortcut_fail(self):
  base=json.loads((ROOT/'design/maps/clockworks/cw_gear_shaft.tmj').read_text());admitted=load_admitted_assets(ROOT/'governance/assets/RUNTIME_ASSET_MANIFEST.json',ROOT/'governance/assets/AI_ASSET_RELEASE_MANIFEST.json');entities=set(json.loads((ROOT/'content/enemies/entity-types.json').read_text())['entityTypes'])
  for label in ('height','endpoint'):
   d=json.loads(json.dumps(base))
   if label=='height':
    layer=next(x for x in d['layers'] if x['name']=='logic.transition');layer['objects'][1]['properties']=[p for p in layer['objects'][1]['properties'] if p['name']!='heightMinM']
   else:
    layer=next(x for x in d['layers'] if x['name']=='logic.traversal');next(p for p in layer['objects'][1]['properties'] if p['name']=='toHeightM')['value']=1.5
   with self.assertRaises(MapError):compile_map(d,admitted,entities)
