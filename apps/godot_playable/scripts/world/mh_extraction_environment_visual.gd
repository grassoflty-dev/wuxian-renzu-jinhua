extends RefCounted

const P = preload("res://scripts/bridge/protocol.gd")
const ProjectionScript = preload("res://scripts/world/projection.gd")
const SCENE_PATH: String = "res://data/scenes/mh_extraction.json"
const SCENE_BYTES: int = 6215
const SCENE_SHA: String = "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa"
const MAP_PATH: String = "res://data/mh_extraction_environment.internal-use.json"
const MAP_BYTES: int = 11198
const MAP_SHA: String = "bbbf7bbea4c5847dabd625092779851ada01ef12a7b2808e4b930faea9245c8b"
const SPEC: Dictionary = {
  "schema": "wuxian.mh-extraction-environment.internal-use/v1",
  "registeredSourceReuse": true,
  "approval": "INTERNAL_REGISTERED_SOURCE_REUSE",
  "roomUse": "MAIN_EXPLICIT_EXTRACTION_ONLY",
  "GodotDisplay": "NOT_RUN",
  "publicRelease": "NOT_ESTABLISHED",
  "worldId": "mist_harbor",
  "sceneId": "mh_extraction",
  "source": {
    "scenePath": "res://data/scenes/mh_extraction.json",
    "sceneBytes": 6215,
    "sceneSha256": "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa",
    "compiledMirrorPath": "content/scenes/compiled/mh_extraction.json",
    "compiledMirrorBytes": 6215,
    "compiledMirrorSha256": "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa",
    "schemaVersion": 1,
    "boundsM": {
      "x": 0,
      "z": 0,
      "width": 24,
      "depth": 16
    },
    "walkable": {
      "id": "mh_extraction_walkable_area",
      "polygon": [
        [
          0.5,
          0.5
        ],
        [
          23.5,
          0.5
        ],
        [
          23.5,
          15.5
        ],
        [
          0.5,
          15.5
        ]
      ]
    },
    "walls": [
      {
        "id": "extraction_wall_north",
        "polygon": [
          [
            0,
            0
          ],
          [
            24,
            0
          ],
          [
            24,
            0.5
          ],
          [
            0,
            0.5
          ]
        ],
        "displayHeightM": 0.5
      },
      {
        "id": "extraction_wall_south",
        "polygon": [
          [
            0,
            15.5
          ],
          [
            24,
            15.5
          ],
          [
            24,
            16
          ],
          [
            0,
            16
          ]
        ],
        "displayHeightM": 0.18
      },
      {
        "id": "extraction_wall_west_north",
        "polygon": [
          [
            0,
            0.5
          ],
          [
            0.5,
            0.5
          ],
          [
            0.5,
            6
          ],
          [
            0,
            6
          ]
        ],
        "displayHeightM": 0.5
      },
      {
        "id": "extraction_wall_west_south",
        "polygon": [
          [
            0,
            9.5
          ],
          [
            0.5,
            9.5
          ],
          [
            0.5,
            15.5
          ],
          [
            0,
            15.5
          ]
        ],
        "displayHeightM": 0.5
      },
      {
        "id": "extraction_wall_east",
        "polygon": [
          [
            23.5,
            0.5
          ],
          [
            24,
            0.5
          ],
          [
            24,
            15.5
          ],
          [
            23.5,
            15.5
          ]
        ],
        "displayHeightM": 0.18
      }
    ],
    "sprites": [
      {
        "id": "approved_extraction_stone_water_module",
        "assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
        "layer": "visual.floor",
        "position": [
          12,
          0,
          8
        ]
      },
      {
        "id": "extraction_outer_water_edge",
        "assetId": "runtime2d.world.mistharbor.water_edge.v1",
        "layer": "visual.props_back",
        "position": [
          12,
          0,
          14.5
        ]
      },
      {
        "id": "extraction_fog_bank",
        "assetId": "runtime2d.world.mistharbor.fog_bank.v1",
        "layer": "visual.props_back",
        "position": [
          18,
          0,
          3
        ]
      }
    ],
    "vfxMarkers": [
      {
        "id": "extraction_ambient_fog",
        "assetId": "runtime2d.world.mistharbor.fog_vfx.v1",
        "position": [
          10,
          0,
          4.5
        ]
      }
    ]
  },
  "references": {
    "stoneDefinition": "res://data/mh_fog_pier_stone_material.internal-use.json",
    "stoneDefinitionBytes": 1517,
    "stoneDefinitionSha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a",
    "fogDefinition": "res://data/mh_fog_pier_layered_fog.internal-use.json",
    "fogDefinitionBytes": 2270,
    "fogDefinitionSha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
  },
  "stone": {
    "assetId": "runtime2d.world.mistharbor.water_edge_stone.v1",
    "path": "res://assets/props/mist_harbor/fog_pier_stone_source.png",
    "bytes": 1983603,
    "sha256": "e823434b3fbc420fc14270200f81e0a459e594597304b1323d336e532a924f9d",
    "canvas": [
      1228,
      1254
    ],
    "format": "RGBA8",
    "sourceRelative": "batch07/02_runtime2d.world.mistharbor.water_edge_stone.v1.png",
    "sourceSha256": "dc09452bf3f55c57cca1a93bb09f169ef441af90321dff47d91c4be7e62d3a83",
    "sourcePixelEdgeCorners": [
      [
        820,
        240
      ],
      [
        1032,
        342
      ],
      [
        392,
        627
      ],
      [
        180,
        525
      ]
    ],
    "uvConvention": "sourcePixelEdge / standaloneCanvasSize"
  },
  "fog": {
    "assetId": "runtime2d.world.mistharbor.fog_bank.v1",
    "path": "res://assets/props/mist_harbor/fog_pier_fog_bank_source.png",
    "bytes": 684960,
    "sha256": "9bbbfd9783f70a8b1f6ffeaabd44e568074f4c20f1f7500d7ef4f0e534edeac2",
    "canvas": [
      1254,
      1236
    ],
    "format": "RGBA8",
    "cropXYWH": [
      27,
      370,
      1201,
      498
    ],
    "uvConvention": "sourcePixelEdge / standaloneCanvasSize",
    "sampling": "linear",
    "repeat": false,
    "blend": "normal-alpha"
  },
  "player": {
    "mapPath": "res://data/cenyao_idle.internal-use.json",
    "mapBytes": 3925,
    "mapSha256": "b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881",
    "canvas": [
      336,
      560
    ],
    "footAnchor": [
      168,
      551
    ],
    "nominalHeightPxAt48Ppm": 104,
    "frames": [
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_south_west.png",
        "bytes": 158163,
        "sha256": "3d3e34d0a5d388de522fd6a94fd2769efbf8137aea1ae7c0b69881c5416520be"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_east.png",
        "bytes": 109177,
        "sha256": "fd38db1daec829c0a7c2adffcdfd5c605f54b3611bb515599cd8deeea1f47402"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_west.png",
        "bytes": 121863,
        "sha256": "fc8e0b132c48f594a49a20bd39955d731f02077502e33131499608cc3bd7a70c"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_south.png",
        "bytes": 158435,
        "sha256": "b6cff22dabb303f0a9c95ba24a41fb3c2f5159ff191adefacf0c6dad38ffe875"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_north_east.png",
        "bytes": 144761,
        "sha256": "19b8e318bf5216d6075f64ac986ef7c3c3299241816b472843aac24bbf4a7676"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_south_east.png",
        "bytes": 158547,
        "sha256": "bfe0107fe1efdcd73b90d57b01283a22658960b5847bb5e88b401dae9857be56"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_north.png",
        "bytes": 151244,
        "sha256": "6608ed34685f28f3380849d7fe5e5319078e8feb1fbd66cbcd4cb3782a1a8221"
      },
      {
        "path": "res://assets/characters/cenyao/idle/cenyao_idle_north_west.png",
        "bytes": 143105,
        "sha256": "8dc527b975b1da5ed7b36a54cf4ad260cdcd9d7d3ccba24660308cf971c3164e"
      }
    ]
  },
  "ground": {
    "rowId": "approved_extraction_stone_water_module",
    "layer": "visual.floor",
    "position": [
      12,
      0,
      8
    ],
    "walkableId": "mh_extraction_walkable_area",
    "walkablePolygon": [
      [
        0.5,
        0.5
      ],
      [
        23.5,
        0.5
      ],
      [
        23.5,
        15.5
      ],
      [
        0.5,
        15.5
      ]
    ],
    "groundY": 0,
    "graphicCellM": [
      8,
      8
    ],
    "grid": [
      3,
      2
    ],
    "patchCount": 6,
    "edgeClip": "clip_world_cell_and_uv_fraction_equally_no_stretch",
    "uvConvention": "A + (B-A)*u + (D-A)*v; edge / standalone canvas",
    "tint": [
      0.36,
      0.45,
      0.5,
      1
    ],
    "seamlessTileability": "NOT_ESTABLISHED"
  },
  "walls": {
    "geometry": "exact_existing_collision_footprints_only",
    "visualHeightIsPhysical": false,
    "northAndWestHeightM": 0.5,
    "southAndEastHeightM": 0.18,
    "faceRgb": [
      45,
      68,
      76
    ],
    "capRgb": [
      82,
      104,
      113
    ],
    "contactInsideFootprintM": 0.05,
    "contactColor": [
      0.025,
      0.032,
      0.035,
      0.1
    ],
    "westApertureZ": [
      6,
      9.5
    ],
    "eastWallContinuous": true
  },
  "bank": {
    "rowId": "extraction_fog_bank",
    "assetId": "runtime2d.world.mistharbor.fog_bank.v1",
    "layer": "visual.props_back",
    "position": [
      18,
      0,
      3
    ],
    "cropXYWH": [
      27,
      370,
      1201,
      498
    ],
    "widthPerPpm": 9,
    "centerOffsetPerPpm": [
      0,
      0
    ],
    "modulate": [
      0.86,
      0.92,
      0.95,
      0.35
    ],
    "temporal": "STATIC",
    "drawLayer": "behind_all_bodies"
  },
  "air": {
    "kind": "static_marker_associated_air",
    "markerId": "extraction_ambient_fog",
    "assetId": "runtime2d.world.mistharbor.fog_vfx.v1",
    "center": [
      10,
      0,
      4.5
    ],
    "wispCentersX": [
      9.8,
      10,
      10.2
    ],
    "w": 0.12,
    "topHeightsM": [
      0.16,
      0.28,
      0.2
    ],
    "outerVerticesRelativeXYZ": [
      [
        -1,
        0.03,
        1
      ],
      [
        -0.7,
        0.1,
        0.7
      ],
      [
        -0.35,
        0.55,
        0.35
      ],
      [
        -0.15,
        1,
        0.15
      ],
      [
        0.25,
        0.9,
        -0.25
      ],
      [
        0.7,
        0.45,
        -0.7
      ],
      [
        1,
        0.08,
        -1
      ],
      [
        0.5,
        0.03,
        -0.5
      ]
    ],
    "innerScale": 0.6,
    "innerHeightScale": 0.85,
    "outerColor": [
      138,
      166,
      172,
      0.025
    ],
    "innerColor": [
      138,
      166,
      172,
      0.012
    ],
    "polygonCap": 6,
    "verticesPerPolygonCap": 8,
    "temporal": "STATIC",
    "drawLayer": "behind_all_bodies"
  },
  "photos": [
    {
      "file": "01-room-arrival.png",
      "player": [
        3,
        0,
        8
      ],
      "origin": [
        12,
        0,
        8
      ],
      "ppm": 36,
      "center": [
        800,
        500
      ],
      "reducedMotion": false,
      "bounds": [
        80,
        122,
        1520,
        860
      ]
    },
    {
      "file": "02-room-center.png",
      "player": [
        12,
        0,
        8
      ],
      "origin": [
        12,
        0,
        8
      ],
      "ppm": 36,
      "center": [
        800,
        500
      ],
      "reducedMotion": false,
      "bounds": [
        80,
        122,
        1520,
        860
      ]
    },
    {
      "file": "03-back-bank-contact.png",
      "player": [
        17,
        0,
        5.5
      ],
      "origin": [
        18,
        0,
        3
      ],
      "ppm": 96,
      "center": [
        800,
        620
      ],
      "reducedMotion": false,
      "bounds": [
        368,
        284,
        1376,
        799.130724
      ]
    },
    {
      "file": "04-ambient-air-and-paving.png",
      "player": [
        9,
        0,
        6
      ],
      "origin": [
        10,
        0,
        4.5
      ],
      "ppm": 128,
      "center": [
        1000,
        650
      ],
      "reducedMotion": false,
      "bounds": [
        596.8,
        409.12381,
        1070.4,
        726.8
      ]
    },
    {
      "file": "05-east-pad-wall-readability.png",
      "player": [
        20,
        0,
        8
      ],
      "origin": [
        20,
        0,
        8
      ],
      "ppm": 80,
      "center": [
        900,
        680
      ],
      "reducedMotion": false,
      "bounds": [
        336,
        94,
        1500,
        920
      ]
    },
    {
      "file": "06-room-center-reduced.png",
      "player": [
        12,
        0,
        8
      ],
      "origin": [
        12,
        0,
        8
      ],
      "ppm": 36,
      "center": [
        800,
        500
      ],
      "reducedMotion": true,
      "bounds": [
        80,
        122,
        1520,
        860
      ]
    }
  ],
  "exclusions": [
    "extraction_outer_water_edge remains unconsumed",
    "no water polygon, shoreline, ship, tower, exit apparatus, wraith or Tidebound bitmap is admitted by this map"
  ],
  "sourcePins": [
    {
      "path": "apps/godot_playable/scripts/world/renderer.gd",
      "bytes": 126029,
      "sha256": "ac2d8e47f0790b2e052e86560edefc91fe5508fa48bcf23fa09e7e9c8676aca9"
    },
    {
      "path": "apps/godot_playable/data/scenes/mh_extraction.json",
      "bytes": 6215,
      "sha256": "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa"
    },
    {
      "path": "content/scenes/compiled/mh_extraction.json",
      "bytes": 6215,
      "sha256": "3476501f29ea441c51caeab61633b929ef7e7abc3c887691a7557098986a47aa"
    },
    {
      "path": "apps/godot_playable/scripts/world/projection.gd",
      "bytes": 3301,
      "sha256": "45f518f6dddebaebedd6bc1b5eb66476ebd3f039779cb353cc6124fbc5c9c82e"
    },
    {
      "path": "apps/godot_playable/scripts/bridge/protocol.gd",
      "bytes": 7675,
      "sha256": "9558f4192c4738b475c6c03046a1db1cd4074a8caf689008cccab013404b696b"
    },
    {
      "path": "apps/godot_playable/scripts/world/presentation.gd",
      "bytes": 7353,
      "sha256": "659c9ccf61f4be94eaee10394e08b57accf7556f12f568809e8d61ae7a43e087"
    },
    {
      "path": "apps/godot_playable/scripts/world/cenyao_idle_visual.gd",
      "bytes": 3875,
      "sha256": "52ead3447f264a4a6328b82142345f3fd3566599b95f4dadc15fa80d8a217705"
    },
    {
      "path": "apps/godot_playable/data/cenyao_idle.internal-use.json",
      "bytes": 3925,
      "sha256": "b063efd36046834c41ebed05c12ffb8729926f7d4edff0e15d36dccdbb1da881"
    },
    {
      "path": "apps/godot_playable/data/mh_fog_pier_stone_material.internal-use.json",
      "bytes": 1517,
      "sha256": "c56ad7549a49809341e2aea6efcbe7fbe0477eb51026e776c4a49fa03077585a"
    },
    {
      "path": "apps/godot_playable/data/mh_fog_pier_layered_fog.internal-use.json",
      "bytes": 2270,
      "sha256": "027980b44e5f92aa5e7759611eeffe69d6ae59be90a4ae4e0858a96123d2bee0"
    },
    {
      "path": "apps/godot_playable/scripts/world/mh_breakwater_environment_visual.gd",
      "bytes": 24257,
      "sha256": "22d678e7f7fddb9f68b94717a8fc41b277ad4fa4475a020661da9a11f0325485"
    },
    {
      "path": "apps/godot_playable/data/mh_breakwater_environment.internal-use.json",
      "bytes": 13530,
      "sha256": "9fc84205b01f672aea6b2af19e4667aeb29060b1395e42d54758fd39f52b1ada"
    },
    {
      "path": "apps/godot_playable/project.godot",
      "bytes": 556,
      "sha256": "9f6c26d26037b0f61bc07f36db40f451f544e7f5542f49ca6d53055f5b5758a3"
    }
  ]
}
var _attempted: bool = false
var _loaded: bool = false
var _ready: bool = false
var _key: String = ""
var _source: Dictionary = {}
var _textures: Dictionary = {}
var prepare_error: String = ""

func _same(a: Variant,b: Variant) -> bool:
	if typeof(a) in [TYPE_INT,TYPE_FLOAT] or typeof(b) in [TYPE_INT,TYPE_FLOAT]:
		return P.number(a,-1.0e12,1.0e12) and P.number(b,-1.0e12,1.0e12) and float(a) == float(b)
	if typeof(a) != typeof(b):
		return false
	if a is Dictionary:
		if a.size() != b.size():
			return false
		for k: Variant in b:
			if not a.has(k) or not _same(a[k],b[k]):
				return false
		return true
	if a is Array:
		if a.size() != b.size():
			return false
		for i: int in range(b.size()):
			if not _same(a[i],b[i]):
				return false
		return true
	return a == b

func _bytes(path: String,limit: int) -> PackedByteArray:
	var f: FileAccess = FileAccess.open(path,FileAccess.READ)
	if f == null:
		return PackedByteArray()
	var n: int = f.get_length()
	if n <= 0 or n > limit:
		f.close()
		return PackedByteArray()
	var bytes: PackedByteArray = f.get_buffer(n)
	f.close()
	return bytes

func _sha(bytes: PackedByteArray) -> String:
	var h: HashingContext = HashingContext.new()
	if h.start(HashingContext.HASH_SHA256) != OK or h.update(bytes) != OK:
		return ""
	return h.finish().hex_encode()

func _basic(s: Variant,d: Variant,k: Variant,h: Variant) -> bool:
	return s is Dictionary and d is Dictionary and typeof(k) == TYPE_STRING and typeof(h) == TYPE_STRING and P.snapshot(s) and s.get("entryToken") == null and s.get("worldId") == "mist_harbor" and s.get("sceneId") == "mh_extraction" and P.counter(s.get("worldEpoch"),1,4294967295) and d.get("worldId") == "mist_harbor" and d.get("sceneId") == "mh_extraction" and k == "mist_harbor:mh_extraction:"+str(s.worldEpoch) and h == SCENE_SHA

func _source_shape(d: Dictionary) -> bool:
	if not _same(d.get("schemaVersion"),SPEC.source.schemaVersion) or not _same(d.get("boundsM"),SPEC.source.boundsM):
		return false
	var walls_value: Variant = d.get("collision")
	if not walls_value is Array or walls_value.size() != SPEC.source.walls.size():
		return false
	for i: int in range(SPEC.source.walls.size()):
		var wall_value: Variant = walls_value[i]
		if not wall_value is Dictionary:
			return false
		var expected: Dictionary = {"id":SPEC.source.walls[i].id,"polygon":SPEC.source.walls[i].polygon}
		if not _same(wall_value,expected):
			return false
	var walk_value: Variant = d.get("walkablePolygons")
	if not walk_value is Array or walk_value.size() != 1 or not walk_value[0] is Dictionary or not _same(walk_value[0],SPEC.source.walkable):
		return false
	var presentation_value: Variant = d.get("presentation")
	if not presentation_value is Dictionary or not _same(presentation_value.get("sprites"),SPEC.source.sprites):
		return false
	return _same(d.get("vfxMarkers"),SPEC.source.vfxMarkers)

func _decode(spec: Dictionary) -> ImageTexture:
	var raw: PackedByteArray = _bytes(str(spec.path),int(spec.bytes))
	if raw.size() != int(spec.bytes) or _sha(raw) != spec.sha256:
		return null
	var image: Image = Image.new()
	if image.load_png_from_buffer(raw) != OK or image.get_width() != int(spec.canvas[0]) or image.get_height() != int(spec.canvas[1]) or image.get_format() != Image.FORMAT_RGBA8:
		return null
	return ImageTexture.create_from_image(image)

func prepare(s: Variant,d: Variant,k: Variant,h: Variant) -> bool:
	_ready = false
	_key = ""
	prepare_error = "context"
	if not _basic(s,d,k,h):
		return false
	if not _attempted:
		_attempted = true
		prepare_error = "map seal"
		var map_bytes: PackedByteArray = _bytes(MAP_PATH,MAP_BYTES)
		if map_bytes.size() != MAP_BYTES or _sha(map_bytes) != MAP_SHA or map_bytes.get_string_from_utf8().to_utf8_buffer() != map_bytes:
			return false
		var parsed_map: Variant = JSON.parse_string(map_bytes.get_string_from_utf8())
		if not parsed_map is Dictionary or not _same(parsed_map,SPEC):
			return false
		prepare_error = "scene seal"
		var raw: PackedByteArray = _bytes(SCENE_PATH,SCENE_BYTES)
		if raw.size() != SCENE_BYTES or _sha(raw) != SCENE_SHA or raw.get_string_from_utf8().to_utf8_buffer() != raw:
			return false
		var source_value: Variant = JSON.parse_string(raw.get_string_from_utf8())
		if not source_value is Dictionary:
			return false
		_source = source_value
		if not _source_shape(_source):
			return false
		for reference: Dictionary in [SPEC.references,SPEC.player]:
			var paths: Array = []
			if reference.has("stoneDefinition"):
				paths = [{"path":reference.stoneDefinition,"bytes":reference.stoneDefinitionBytes,"sha256":reference.stoneDefinitionSha256},{"path":reference.fogDefinition,"bytes":reference.fogDefinitionBytes,"sha256":reference.fogDefinitionSha256}]
			else:
				paths = [{"path":reference.mapPath,"bytes":reference.mapBytes,"sha256":reference.mapSha256}]
			for pin: Dictionary in paths:
				var data: PackedByteArray = _bytes(str(pin.path),int(pin.bytes))
				if data.size() != int(pin.bytes) or _sha(data) != pin.sha256:
					prepare_error = "upstream map pin"
					return false
		prepare_error = "binary PNG"
		var stone: ImageTexture = _decode(SPEC.stone)
		var fog: ImageTexture = _decode(SPEC.fog)
		if stone == null or fog == null:
			return false
		_textures = {"stone":stone,"fog":fog}
		_loaded = true
	if not _loaded or not _same(d,_source):
		prepare_error = "current scene"
		return false
	_key = k
	_ready = true
	prepare_error = ""
	return true

func _finite2(p: Vector2) -> bool:
	return is_finite(p.x) and is_finite(p.y)
func _finite3(p: Vector3) -> bool:
	return is_finite(p.x) and is_finite(p.y) and is_finite(p.z)

func matches_context(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant) -> bool:
	if not _ready or not _basic(s,d,k,h) or k != _key or not _same(d,_source):
		return false
	if not is_instance_valid(c) or not c is CanvasItem or c.is_queued_for_deletion() or not c.is_inside_tree():
		return false
	if not _same(c.get("snapshot"),s) or not _same(c.get("scene"),d) or c.get("current_key") != k or c.get("scene_hash") != h:
		return false
	if not is_instance_valid(camera) or camera.get_script() != ProjectionScript or c.get("camera") != camera or not viewport is Rect2 or c.get("viewport_rect") != viewport:
		return false
	var origin_value: Variant = camera.get("origin")
	var center_value: Variant = camera.get("screen_center")
	var size_value: Variant = camera.get("viewport")
	if not origin_value is Vector3 or not center_value is Vector2 or not size_value is Vector2:
		return false
	if not _finite3(origin_value) or not _finite2(center_value) or not _finite2(size_value) or typeof(camera.get("initialized")) != TYPE_BOOL or not camera.get("initialized") or not P.number(camera.get("ppm"),0.001,256):
		return false
	return _finite2(viewport.position) and _finite2(viewport.size) and viewport.size.x > 0 and viewport.size.y > 0 and viewport.position == Vector2.ZERO and viewport.size == size_value

func _project(camera: Variant,point: Vector3) -> Vector2:
	var value: Variant = camera.call("project",point)
	return value if value is Vector2 else Vector2(INF,INF)

func _poly(camera: Variant,world: Array[Vector3]) -> PackedVector2Array:
	var out: PackedVector2Array = PackedVector2Array()
	for p: Vector3 in world:
		if not _finite3(p):
			return PackedVector2Array()
		var q: Vector2 = _project(camera,p)
		if not _finite2(q):
			return PackedVector2Array()
		out.append(q)
	var area: float = 0.0
	for i: int in range(out.size()):
		area += out[i].cross(out[(i+1)%out.size()])
	if out.size() < 3 or absf(area) < 0.000001:
		return PackedVector2Array()
	return out

func _bounds(points: PackedVector2Array) -> Rect2:
	var lo: Vector2 = Vector2(INF,INF)
	var hi: Vector2 = Vector2(-INF,-INF)
	for p: Vector2 in points:
		lo = lo.min(p)
		hi = hi.max(p)
	return Rect2(lo,hi-lo)

func _uv(u: float,v: float) -> Vector2:
	return (Vector2(820,240)+Vector2(212,102)*u+Vector2(-640,285)*v)/Vector2(1228,1254)

func draw_ground(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant) -> bool:
	if not matches_context(c,s,d,k,h,camera,viewport):
		return false
	var plans: Array[Dictionary] = []
	for iz: int in range(2):
		for ix: int in range(3):
			var x: float = 0.5+float(ix)*8.0
			var z: float = 0.5+float(iz)*8.0
			var width: float = minf(8.0,23.5-x)
			var depth: float = minf(8.0,15.5-z)
			var points: PackedVector2Array = _poly(camera,[Vector3(x,0,z),Vector3(x+width,0,z),Vector3(x+width,0,z+depth),Vector3(x,0,z+depth)])
			var uv: PackedVector2Array = PackedVector2Array([_uv(0,0),_uv(width/8.0,0),_uv(width/8.0,depth/8.0),_uv(0,depth/8.0)])
			if points.size() != 4:
				return false
			for value: Vector2 in uv:
				if not _finite2(value) or value.x < 0 or value.y < 0 or value.x > 1 or value.y > 1:
					return false
			plans.append({"points":points,"uv":uv})
	for plan: Dictionary in plans:
		c.draw_polygon(plan.points,PackedColorArray([Color(.36,.45,.50,1)]),plan.uv,_textures.stone)
	_field_draw(c,s,d,k,h,camera,viewport)
	return true

# BEGIN Extraction ground field
const FIELD_XZ = [[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]
const FIELD_RGB = [0.12,0.16,0.18]
const FIELD_ALPHA = [0.025,0.010]
const FIELD_AXIS = [0.0,1.0,0.5,15.0]

func _field_bind(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,p: Variant,v: Variant) -> bool:
	if not c is CanvasItem or not is_instance_valid(c) or not s is Dictionary or not d is Dictionary or not p is RefCounted or not is_instance_valid(p) or not v is Rect2:
		return false
	if typeof(k)!=TYPE_STRING or typeof(h)!=TYPE_STRING or not s.has("entryToken") or s.get("entryToken")!=null or not _loaded or not _ready or not matches_context(c,s,d,k,h,p,v):
		return false
	var canvas: CanvasItem = c
	var rect: Rect2 = v
	return canvas.is_visible_in_tree() and rect.position==Vector2.ZERO and _finite2(rect.end) and canvas.get_viewport_rect()==rect and canvas.get_global_transform()==Transform2D.IDENTITY and canvas.get_global_transform_with_canvas()==Transform2D.IDENTITY

func _field_capture(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,p: Variant,v: Variant) -> Dictionary:
	if not _field_bind(c,s,d,k,h,p,v): return {}
	var snapshot_data: Dictionary = s
	var scene_data: Dictionary = d
	var canvas: CanvasItem = c
	var camera: RefCounted = p
	return {"snapshot":snapshot_data.duplicate(true),"scene":scene_data.duplicate(true),
		"id":canvas.get_instance_id(),"key":k,"hash":h,"camera":camera,"viewport":v,
		"origin":camera.get("origin"),"center":camera.get("screen_center"),"ppm":camera.get("ppm"),
		"initialized":camera.get("initialized"),"size":camera.get("viewport"),
		"global":canvas.get_global_transform(),"with_canvas":canvas.get_global_transform_with_canvas()}

func _field_same(m: Dictionary,c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,p: Variant,v: Variant) -> bool:
	if m.is_empty() or not _field_bind(c,s,d,k,h,p,v): return false
	var canvas: CanvasItem = c
	var camera: RefCounted = p
	return (_same(s,m.snapshot) and _same(d,m.scene)
		and _same(canvas.get("snapshot"),m.snapshot) and _same(canvas.get("scene"),m.scene)
		and canvas.get_instance_id()==m.id and k==m.key and h==m.hash and canvas.get("scene_hash")==m.hash
		and camera==m.camera and v==m.viewport and camera.get("origin")==m.origin
		and camera.get("screen_center")==m.center and camera.get("ppm")==m.ppm
		and camera.get("initialized")==m.initialized and camera.get("viewport")==m.size
		and canvas.get_global_transform()==m.global and canvas.get_global_transform_with_canvas()==m.with_canvas)

func _field_color(x: float,z: float) -> Color:
	var t: float = (float(FIELD_AXIS[0])*x+float(FIELD_AXIS[1])*z-float(FIELD_AXIS[2]))/float(FIELD_AXIS[3])
	return Color(float(FIELD_RGB[0]),float(FIELD_RGB[1]),float(FIELD_RGB[2]),lerpf(float(FIELD_ALPHA[0]),float(FIELD_ALPHA[1]),t))

func _field_color_ok(color: Color) -> bool:
	for component: float in [color.r,color.g,color.b,color.a]:
		if not is_finite(component) or component<0.0 or component>1.0: return false
	var low: float = Color(0,0,0,minf(float(FIELD_ALPHA[0]),float(FIELD_ALPHA[1]))).a
	var high: float = Color(0,0,0,maxf(float(FIELD_ALPHA[0]),float(FIELD_ALPHA[1]))).a
	return color.a>=low and color.a<=high

func _field_quad(points: PackedVector2Array) -> bool:
	if points.size()!=4: return false
	for point: Vector2 in points:
		if not _finite2(point): return false
	var area2: float = 0.0
	for i: int in range(1,3):
		var term: float = (points[i]-points[0]).cross(points[i+1]-points[0])
		if not is_finite(term): return false
		area2+=term
	if not is_finite(area2) or area2==0.0: return false
	var low: Vector2 = points[0]
	var high: Vector2 = points[0]
	for i: int in range(4):
		var edge: Vector2 = points[(i+1)%4]-points[i]
		var next: Vector2 = points[(i+2)%4]-points[(i+1)%4]
		var turn: float = edge.cross(next)
		if not _finite2(edge) or edge==Vector2.ZERO or not is_finite(turn) or turn==0.0 or (turn>0.0)!=(area2>0.0): return false
		low=low.min(points[i])
		high=high.max(points[i])
	return _finite2(high-low) and high.x>low.x and high.y>low.y

func _field_visible(points: PackedVector2Array,rect: Rect2) -> int:
	var corners: PackedVector2Array = PackedVector2Array([rect.position,Vector2(rect.end.x,rect.position.y),rect.end,Vector2(rect.position.x,rect.end.y)])
	var axes: Array[Vector2] = [Vector2.RIGHT,Vector2.DOWN]
	for i: int in range(4):
		var edge: Vector2 = points[(i+1)%4]-points[i]
		axes.append(Vector2(-edge.y,edge.x))
	var separated: bool = false
	for axis: Vector2 in axes:
		if not _finite2(axis) or axis==Vector2.ZERO: return -1
		var plo: float = INF
		var phi: float = -INF
		var rlo: float = INF
		var rhi: float = -INF
		for point: Vector2 in points:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value): return -1
			plo=minf(plo,value)
			phi=maxf(phi,value)
		for point: Vector2 in corners:
			var value: float = (point-points[0]).dot(axis)
			if not is_finite(value): return -1
			rlo=minf(rlo,value)
			rhi=maxf(rhi,value)
		if not is_finite(plo) or not is_finite(phi) or not is_finite(rlo) or not is_finite(rhi) or plo>=phi or rlo>=rhi: return -1
		if phi<=rlo or rhi<=plo: separated=true
	return 0 if separated else 1

func _field_draw(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,p: Variant,v: Variant) -> bool:
	var capture: Dictionary = _field_capture(c,s,d,k,h,p,v)
	if capture.is_empty() or not _same(FIELD_XZ,_source.walkablePolygons[0].polygon): return false
	var points: PackedVector2Array = PackedVector2Array()
	var colors: PackedColorArray = PackedColorArray()
	for pair: Array in FIELD_XZ:
		var x: float = float(pair[0])
		var z: float = float(pair[1])
		var point: Vector2 = _project(p,Vector3(x,0.0,z))
		var color: Color = _field_color(x,z)
		if not _finite2(point) or not _field_color_ok(color): return false
		points.append(point)
		colors.append(color)
	if colors.size()!=4 or not _field_quad(points): return false
	var visibility: int = _field_visible(points,v)
	if visibility!=1: return false
	if not _field_same(capture,c,s,d,k,h,p,v): return false
	var canvas: CanvasItem = c
	canvas.draw_polygon(points,colors)
	return true
# END Extraction ground field

func draw_wall(c: Variant,wall: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant) -> bool:
	if not matches_context(c,s,d,k,h,camera,viewport) or not wall is Dictionary:
		return false
	var plans: Dictionary = {}
	for expected: Dictionary in SPEC.source.walls:
		var polygon: Array = expected.polygon
		var height: float = float(expected.displayHeightM)
		var base: Array[Vector3] = []
		var top: Array[Vector3] = []
		for pair: Array in polygon:
			base.append(Vector3(float(pair[0]),0,float(pair[1])))
			top.append(Vector3(float(pair[0]),height,float(pair[1])))
		var pieces: Array[PackedVector2Array] = []
		for i: int in range(4):
			var j: int = (i+1)%4
			var points: PackedVector2Array = _poly(camera,[base[i],base[j],top[j],top[i]])
			if points.size() != 4:
				return false
			pieces.append(points)
		var cap: PackedVector2Array = _poly(camera,top)
		var contact: PackedVector2Array = _poly(camera,base)
		if cap.size() != 4 or contact.size() != 4:
			return false
		plans[expected.id] = {"source":{"id":expected.id,"polygon":polygon},"faces":pieces,"cap":cap,"contact":contact}
	var id_value: Variant = wall.get("id")
	if typeof(id_value) != TYPE_STRING or not plans.has(id_value) or not _same(wall,plans[id_value].source):
		return false
	var plan: Dictionary = plans[id_value]
	c.draw_colored_polygon(plan.contact,Color(.025,.032,.035,.10))
	for face: PackedVector2Array in plan.faces:
		c.draw_colored_polygon(face,Color("#2d444c"))
	c.draw_colored_polygon(plan.cap,Color("#526871"))
	return true

func _bank_rect(camera: Variant) -> Rect2:
	var center: Vector2 = _project(camera,Vector3(18,0,3))
	var width: float = 9.0*float(camera.get("ppm"))
	var size: Vector2 = Vector2(width,width*498.0/1201.0)
	return Rect2(center-size*0.5,size)
func _bank_visible(rect: Rect2,viewport: Rect2) -> bool:
	return _finite2(rect.position) and _finite2(rect.size) and rect.size.x > 0 and rect.size.y > 0 and rect.intersects(viewport)

func draw_bank(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant) -> Array[String]:
	var ids: Array[String] = []
	if not matches_context(c,s,d,k,h,camera,viewport):
		return ids
	var rect: Rect2 = _bank_rect(camera)
	if not _bank_visible(rect,viewport):
		return ids
	c.draw_texture_rect_region(_textures.fog,rect,Rect2(27,370,1201,498),Color(.86,.92,.95,.35),false,true)
	ids.append("extraction_fog_bank")
	return ids

func legacy_bank_consumed(c: Variant,row: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant,ids: Variant) -> bool:
	if not matches_context(c,s,d,k,h,camera,viewport) or not row is Dictionary or not ids is Array or ids.count("extraction_fog_bank") != 1:
		return false
	return _same(row,SPEC.source.sprites[2]) and _bank_visible(_bank_rect(camera),viewport)

func _wisp(center: Vector3,w: float,h: float,camera: Variant) -> PackedVector2Array:
	var relative: Array[Vector3] = [Vector3(-w,.03,w),Vector3(-.7*w,.10,.7*w),Vector3(-.35*w,.55*h,.35*w),Vector3(-.15*w,h,.15*w),Vector3(.25*w,.90*h,-.25*w),Vector3(.7*w,.45*h,-.7*w),Vector3(w,.08,-w),Vector3(.5*w,.03,-.5*w)]
	var world: Array[Vector3] = []
	for q: Vector3 in relative:
		var point: Vector3 = center+q
		if not _finite3(point) or point.x < 9.65 or point.x > 10.35 or point.z < 4.3 or point.z > 4.7 or point.y < 0 or point.y > .30:
			return PackedVector2Array()
		world.append(point)
	return _poly(camera,world)

func draw_air(c: Variant,s: Variant,d: Variant,k: Variant,h: Variant,camera: Variant,viewport: Variant) -> int:
	if not matches_context(c,s,d,k,h,camera,viewport):
		return 0
	var plans: Array[Dictionary] = []
	for i: int in range(3):
		var center: Vector3 = Vector3(float(SPEC.air.wispCentersX[i]),0,4.5)
		var height: float = float(SPEC.air.topHeightsM[i])
		for inner: bool in [false,true]:
			var width: float = .12*.6 if inner else .12
			var actual_height: float = height*.85 if inner else height
			var points: PackedVector2Array = _wisp(center,width,actual_height,camera)
			if points.size() != 8:
				return 0
			plans.append({"points":points,"color":Color(138.0/255.0,166.0/255.0,172.0/255.0,.012 if inner else .025)})
	var count: int = 0
	for plan: Dictionary in plans:
		if _bounds(plan.points).intersects(viewport):
			c.draw_colored_polygon(plan.points,plan.color)
			count += 1
	return count

