extends RefCounted


static func evaluate(owns_collision: Callable) -> Array[String]:
	var failures: Array[String] = []
	if not owns_collision.is_valid():
		failures.append("invalid_callable")
		return failures

	var cases: Array[Dictionary] = []
	var north: Dictionary = _wall("sentinel_arena_wall_north", [[0, 0], [24, 0], [24, 0.5], [0, 0.5]])
	var south: Dictionary = _wall("sentinel_arena_wall_south", [[0, 15.5], [24, 15.5], [24, 16], [0, 16]])
	_append_case(cases, "sentinel_arena_wall_north", north, true)
	_append_case(cases, "sentinel_arena_wall_south", south, true)
	_append_case(cases, "sentinel_arena_wall_west_north", _wall("sentinel_arena_wall_west_north", [[0, 0.5], [0.5, 0.5], [0.5, 6.5], [0, 6.5]]), true)
	_append_case(cases, "sentinel_arena_wall_west_south", _wall("sentinel_arena_wall_west_south", [[0, 9.5], [0.5, 9.5], [0.5, 15.5], [0, 15.5]]), true)
	_append_case(cases, "sentinel_arena_wall_east_north", _wall("sentinel_arena_wall_east_north", [[23.5, 0.5], [24, 0.5], [24, 6.5], [23.5, 6.5]]), true)
	_append_case(cases, "sentinel_arena_wall_east_south", _wall("sentinel_arena_wall_east_south", [[23.5, 9.5], [24, 9.5], [24, 15.5], [23.5, 15.5]]), true)

	_append_case(cases, "sentinel_arena_wall_north_float", _wall("sentinel_arena_wall_north", [[0.0, 0.0], [24.0, 0.0], [24.0, 0.5], [0.0, 0.5]]), true)
	_append_case(cases, "sentinel_arena_wall_south_float", _wall("sentinel_arena_wall_south", [[0.0, 15.5], [24.0, 15.5], [24.0, 16.0], [0.0, 16.0]]), true)
	_append_case(cases, "sentinel_arena_wall_west_north_float", _wall("sentinel_arena_wall_west_north", [[0.0, 0.5], [0.5, 0.5], [0.5, 6.5], [0.0, 6.5]]), true)
	_append_case(cases, "sentinel_arena_wall_west_south_float", _wall("sentinel_arena_wall_west_south", [[0.0, 9.5], [0.5, 9.5], [0.5, 15.5], [0.0, 15.5]]), true)
	_append_case(cases, "sentinel_arena_wall_east_north_float", _wall("sentinel_arena_wall_east_north", [[23.5, 0.5], [24.0, 0.5], [24.0, 6.5], [23.5, 6.5]]), true)
	_append_case(cases, "sentinel_arena_wall_east_south_float", _wall("sentinel_arena_wall_east_south", [[23.5, 9.5], [24.0, 9.5], [24.0, 15.5], [23.5, 15.5]]), true)

	_append_case(cases, "sentinel_arena_pylon_collision_left", _wall("sentinel_arena_pylon_collision_left", [[7, 5], [8, 5], [8, 7], [7, 7]]), false)
	_append_case(cases, "sentinel_arena_pylon_collision_right", _wall("sentinel_arena_pylon_collision_right", [[16, 5], [17, 5], [17, 7], [16, 7]]), false)

	_append_case(cases, "root_null", null, false)
	_append_case(cases, "root_boolean", true, false)
	_append_case(cases, "root_number", 17, false)
	_append_case(cases, "root_string", "sentinel_arena_wall_north", false)
	_append_case(cases, "root_array", [], false)
	_append_case(cases, "root_dictionary", {}, false)

	var missing_id: Dictionary = north.duplicate(true)
	missing_id.erase("id")
	_append_case(cases, "north_missing_id", missing_id, false)
	var missing_polygon: Dictionary = north.duplicate(true)
	missing_polygon.erase("polygon")
	_append_case(cases, "north_missing_polygon", missing_polygon, false)
	var extra_field: Dictionary = north.duplicate(true)
	extra_field["extra"] = 0
	_append_case(cases, "north_extra_field", extra_field, false)

	var null_id: Dictionary = north.duplicate(true)
	null_id["id"] = null
	_append_case(cases, "north_null_id", null_id, false)
	var numeric_id: Dictionary = north.duplicate(true)
	numeric_id["id"] = 7
	_append_case(cases, "north_numeric_id", numeric_id, false)
	var foreign_id: Dictionary = north.duplicate(true)
	foreign_id["id"] = "gh_sentinel_arena_wall_north"
	_append_case(cases, "north_foreign_id", foreign_id, false)

	var null_polygon: Dictionary = north.duplicate(true)
	null_polygon["polygon"] = null
	_append_case(cases, "north_null_polygon", null_polygon, false)
	var string_polygon: Dictionary = north.duplicate(true)
	string_polygon["polygon"] = "polygon"
	_append_case(cases, "north_string_polygon", string_polygon, false)
	var dictionary_polygon: Dictionary = north.duplicate(true)
	dictionary_polygon["polygon"] = {}
	_append_case(cases, "north_dictionary_polygon", dictionary_polygon, false)

	var south_polygon: Array = south["polygon"].duplicate(true)
	var north_id_south_polygon: Dictionary = north.duplicate(true)
	north_id_south_polygon["polygon"] = south_polygon
	_append_case(cases, "north_id_south_polygon", north_id_south_polygon, false)
	var triangle: Dictionary = north.duplicate(true)
	triangle["polygon"] = [[0, 0], [24, 0], [24, 0.5]]
	_append_case(cases, "north_triangle", triangle, false)
	var repeated_vertex: Dictionary = north.duplicate(true)
	repeated_vertex["polygon"] = [[0, 0], [24, 0], [24, 0.5], [0, 0.5], [0, 0]]
	_append_case(cases, "north_repeated_vertex", repeated_vertex, false)
	var reversed_winding: Dictionary = north.duplicate(true)
	reversed_winding["polygon"] = [[0, 0.5], [24, 0.5], [24, 0], [0, 0]]
	_append_case(cases, "north_reversed_winding", reversed_winding, false)
	var perturbed_coordinate: Dictionary = north.duplicate(true)
	perturbed_coordinate["polygon"] = [[0, 0], [24, 0], [24, 0.5001], [0, 0.5]]
	_append_case(cases, "north_perturbed_coordinate", perturbed_coordinate, false)
	var boolean_coordinate: Dictionary = north.duplicate(true)
	boolean_coordinate["polygon"] = [[true, 0], [24, 0], [24, 0.5], [0, 0.5]]
	_append_case(cases, "north_boolean_coordinate", boolean_coordinate, false)
	var string_coordinate: Dictionary = north.duplicate(true)
	string_coordinate["polygon"] = [["0", 0], [24, 0], [24, 0.5], [0, 0.5]]
	_append_case(cases, "north_string_coordinate", string_coordinate, false)
	var null_coordinate: Dictionary = north.duplicate(true)
	null_coordinate["polygon"] = [[null, 0], [24, 0], [24, 0.5], [0, 0.5]]
	_append_case(cases, "north_null_coordinate", null_coordinate, false)

	for case_value: Variant in cases:
		if typeof(case_value) != TYPE_DICTIONARY:
			failures.append("invalid_fixture_definition")
			continue
		var case_record: Dictionary = case_value
		var id_value: Variant = case_record.get("case_id")
		var fixture_value: Variant = case_record.get("fixture")
		var expected_value: Variant = case_record.get("expected")
		if typeof(id_value) != TYPE_STRING or typeof(expected_value) != TYPE_BOOL:
			failures.append("invalid_fixture_definition")
			continue
		var fixture_id: String = id_value
		var expected: bool = expected_value
		var actual_value: Variant = owns_collision.call(fixture_value)
		if typeof(actual_value) != TYPE_BOOL:
			failures.append(fixture_id)
		elif actual_value != expected:
			failures.append(fixture_id)

	return failures


static func _wall(wall_id: String, polygon: Array) -> Dictionary:
	return {"id": wall_id, "polygon": polygon.duplicate(true)}


static func _append_case(cases: Array[Dictionary], case_id: String, fixture: Variant, expected: bool) -> void:
	var isolated_fixture: Variant = fixture
	if typeof(fixture) == TYPE_ARRAY:
		var fixture_array: Array = fixture
		isolated_fixture = fixture_array.duplicate(true)
	elif typeof(fixture) == TYPE_DICTIONARY:
		var fixture_dictionary: Dictionary = fixture
		isolated_fixture = fixture_dictionary.duplicate(true)
	cases.append({"case_id": case_id, "fixture": isolated_fixture, "expected": expected})
