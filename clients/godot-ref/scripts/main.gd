extends Node3D

## Vertical scale for normalized `[0, 1]` heights from civ-watch (tweak in Inspector).
@export_range(1.0, 64.0, 1.0) var terrain_height_exaggeration := 24.0

## `server` = civ-server WebSocket + civ-watch terrain. `watch` = civ-watch HTTP only.
@export_enum("server", "watch") var attach_mode := "server"

@export var civ_server_ws := "ws://127.0.0.1:3000/ws?tick_format=binary"
@export var civ_watch_http := "http://127.0.0.1:9090"

## When true (ADR-009), hide world-mutation tools.
@export var spectator_mode := true

const TERRAIN_GRID_SIZE := 128
const WORLD_SCALE := 1_000_000
const MUTATION_TOOLS := ["Place Voxel", "Spawn Civilian", "Damage"]

@export_enum("civilian", "vehicle", "airport") var spawn_kind := "civilian"
@export_range(1, 32, 1) var damage_radius := 8

const JOB_COLORS := {
	"farmer": Color(0.49, 0.85, 0.34),
	"warrior": Color(1.0, 0.42, 0.42),
	"scholar": Color(0.44, 0.73, 1.0),
	"trader": Color(1.0, 0.82, 0.4),
	"priest": Color(0.75, 0.52, 0.99),
	"admin": Color(0.72, 0.75, 0.8),
	"unemployed": Color(0.72, 0.75, 0.8),
}

const BUILDING_COLORS := {
	"Residential": Color(0.72, 0.68, 0.55),
	"Commercial": Color(0.55, 0.72, 0.95),
	"Industrial": Color(0.62, 0.58, 0.52),
	"Civic": Color(0.85, 0.78, 0.45),
}

var _civis_http: CivisClient
@onready var terrain_mesh: MeshInstance3D = $Terrain/TerrainMesh
@onready var civilians_root: Node3D = $Civilians
@onready var buildings_root: Node3D = $Buildings
@onready var ui = $UI

var current_tool := "Inspect"
var current_material := 0
var current_speed := 1
var terrain_heights: PackedFloat32Array = PackedFloat32Array()
var terrain_biomes: PackedByteArray = PackedByteArray()
var civilian_nodes: Dictionary = {}
var building_nodes: Dictionary = {}
var spawn_count := 0
var _ws_client: CivisWsClient

func _ready() -> void:
	_civis_http = CivisClient.new()
	add_child(_civis_http)

	_ws_client = CivisWsClient.new()
	_ws_client.name = "CivisWsClient"
	add_child(_ws_client)
	_ws_client.snapshot_received.connect(_on_ws_snapshot)
	_ws_client.connection_changed.connect(_on_ws_connection)

	_civis_http.connect(civ_watch_http)
	_load_terrain()
	_build_terrain_mesh()
	_bind_ui()
	ui.get_node("Minimap").setup(terrain_heights, terrain_biomes, $Camera3D, terrain_height_exaggeration)

	if attach_mode == "server":
		_ws_client.connect_server(civ_server_ws)
		$Timer.stop()
	else:
		_update_snapshot_watch()
		$Timer.start()

func _load_terrain() -> void:
	terrain_heights = _civis_http.fetch_terrain()
	terrain_biomes = _civis_http.fetch_terrain_biomes()

func _build_terrain_mesh() -> void:
	if terrain_heights.is_empty():
		return
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for x in range(TERRAIN_GRID_SIZE - 1):
		for z in range(TERRAIN_GRID_SIZE - 1):
			_add_terrain_quad(st, x, z)
	terrain_mesh.mesh = st.commit()
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = 0.95
	terrain_mesh.material_override = mat
	terrain_mesh.rotation_degrees = Vector3(-90, 0, 0)

func _add_terrain_quad(st: SurfaceTool, x: int, z: int) -> void:
	var h00 := _terrain_height(x, z)
	var h10 := _terrain_height(x + 1, z)
	var h01 := _terrain_height(x, z + 1)
	var h11 := _terrain_height(x + 1, z + 1)
	_emit_vertex(st, Vector3(x, h00, z), _terrain_color(x, z))
	_emit_vertex(st, Vector3(x + 1, h10, z), _terrain_color(x + 1, z))
	_emit_vertex(st, Vector3(x, h01, z + 1), _terrain_color(x, z + 1))
	_emit_vertex(st, Vector3(x + 1, h10, z), _terrain_color(x + 1, z))
	_emit_vertex(st, Vector3(x + 1, h11, z + 1), _terrain_color(x + 1, z + 1))
	_emit_vertex(st, Vector3(x, h01, z + 1), _terrain_color(x, z + 1))

func _emit_vertex(st: SurfaceTool, pos: Vector3, color: Color) -> void:
	st.set_color(color)
	st.add_vertex(pos)

func _terrain_height(x: int, z: int) -> float:
	var idx := z * TERRAIN_GRID_SIZE + x
	if idx < 0 or idx >= terrain_heights.size():
		return 0.0
	return terrain_heights[idx] * terrain_height_exaggeration

func _terrain_color(x: int, z: int) -> Color:
	var idx := z * TERRAIN_GRID_SIZE + x
	if idx < 0:
		return CivisClient.biome_color(0)
	if not terrain_biomes.is_empty() and idx < terrain_biomes.size():
		return CivisClient.biome_color(int(terrain_biomes[idx]))
	if idx < terrain_heights.size():
		return CivisClient.height_color(terrain_heights[idx])
	return CivisClient.biome_color(0)

func _bind_ui() -> void:
	for button_name in ["Place Voxel", "Spawn Civilian", "Damage", "Inspect", "Camera"]:
		var button := ui.get_node("BottomBar/HBoxContainer/%s" % button_name) as Button
		if spectator_mode and button_name in MUTATION_TOOLS:
			button.visible = false
			button.disabled = true
			continue
		button.pressed.connect(_on_tool_pressed.bind(button_name))
	var speed := ui.get_node("BottomBar/HBoxContainer/Speed") as OptionButton
	speed.clear()
	for label in ["Pause (0×)", "1×", "2×", "4×", "8×"]:
		speed.add_item(label)
	speed.select(1)
	speed.item_selected.connect(_on_speed_selected)
	var attach_label := "civ-server WS" if attach_mode == "server" else "civ-watch HTTP"
	var mode_label := "Spectator" if spectator_mode else "Authoring"
	var hint := "%s — %s. Terrain from civ-watch." % [mode_label, attach_label]
	ui.get_node("BottomBar").tooltip_text = hint
	if spectator_mode:
		ui.get_node("BottomBar/HBoxContainer/Material").visible = false
	else:
		ui.get_node("BottomBar/HBoxContainer/SpawnCountLabel").visible = true
	ui.get_node("BottomBar/HBoxContainer/Material").value_changed.connect(_on_material_changed)
	var cam: Camera3D = $Camera3D
	ui.get_node("BottomBar/HBoxContainer/CameraOverview").pressed.connect(
		func(): cam.apply_preset("wide")
	)
	ui.get_node("BottomBar/HBoxContainer/CameraClose").pressed.connect(
		func(): cam.apply_preset("close")
	)
	_apply_speed(current_speed)

func _apply_speed(speed: int) -> void:
	if attach_mode == "server":
		_ws_client.set_speed(speed)
	else:
		_civis_http.post_speed(speed)

func _process(_delta: float) -> void:
	if spectator_mode:
		return
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) and not _clicking_minimap():
		_handle_click()

func _clicking_minimap() -> bool:
	var minimap := ui.get_node("Minimap") as Control
	return minimap.get_rect().has_point(minimap.get_local_mouse_position())

func _handle_click() -> void:
	var cam: Camera3D = $Camera3D
	var from := cam.project_ray_origin(get_viewport().get_mouse_position())
	var to := from + cam.project_ray_normal(get_viewport().get_mouse_position()) * 1000.0
	var query := PhysicsRayQueryParameters3D.create(from, to)
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	if hit.is_empty():
		return
	var pos: Vector3 = hit.position
	if current_tool == "Place Voxel":
		if attach_mode == "server":
			_ws_client.place_voxel(int(pos.x), int(pos.y), int(pos.z), current_material)
		else:
			_civis_http.post_place_voxel(int(pos.x), int(pos.y), int(pos.z), current_material)
	elif current_tool == "Spawn Civilian":
		var norm_x := pos.x / float(TERRAIN_GRID_SIZE)
		var norm_y := pos.z / float(TERRAIN_GRID_SIZE)
		if attach_mode == "server":
			_ws_client.spawn_entity(spawn_kind, norm_x, norm_y, 0)
		else:
			_civis_http.post_spawn_entity(spawn_kind, norm_x, norm_y, 0)
		spawn_count += 1
		ui.get_node("BottomBar/HBoxContainer/SpawnCountLabel").text = "Spawns: %d" % spawn_count
	elif current_tool == "Damage":
		var wx := int(pos.x) * WORLD_SCALE
		var wy := int(maxf(0.0, pos.y)) * WORLD_SCALE
		var wz := int(pos.z) * WORLD_SCALE
		if attach_mode == "server":
			_ws_client.apply_damage(wx, wy, wz, damage_radius)
		else:
			_civis_http.post_damage(wx, wy, wz, damage_radius)

func _on_timer_timeout() -> void:
	_update_snapshot_watch()

func _update_snapshot_watch() -> void:
	var snapshot := _civis_http.latest_snapshot()
	if snapshot.is_empty():
		return
	_apply_snapshot(snapshot)

func _on_ws_snapshot(snapshot: Dictionary) -> void:
	_apply_snapshot(snapshot)

func _on_ws_connection(state: String) -> void:
	if state != "live":
		ui.get_node("BottomBar/HBoxContainer/TickLabel").text = "Conn: %s" % state

func _apply_snapshot(snapshot: Dictionary) -> void:
	var tick := int(snapshot.get("tick", 0))
	ui.get_node("BottomBar/HBoxContainer/TickLabel").text = "Tick: %s" % tick
	ui.get_node("BottomBar/HBoxContainer/PopulationLabel").text = "Population: %s" % snapshot.get("population", 0)
	ui.get_node("BottomBar/HBoxContainer/EraLabel").text = EraTimelapse.era_label(tick)
	_sync_civilians(snapshot.get("civ_pins", []))
	_sync_buildings(snapshot.get("buildings", []))

func _sync_civilians(pins: Array) -> void:
	var seen := {}
	for pin in pins:
		var idx := int(pin.get("idx", 0))
		seen[idx] = true
		var node: MeshInstance3D = civilian_nodes.get(idx)
		if node == null:
			node = MeshInstance3D.new()
			node.mesh = BoxMesh.new()
			node.scale = Vector3(0.4, 1.4, 0.4)
			civilians_root.add_child(node)
			civilian_nodes[idx] = node
		var job := str(pin.get("job", "unemployed")).to_lower()
		var mat := StandardMaterial3D.new()
		mat.albedo_color = JOB_COLORS.get(job, JOB_COLORS["unemployed"])
		node.material_override = mat
		node.position = Vector3(
			float(pin.get("x", 0.0)) * float(TERRAIN_GRID_SIZE),
			1.0,
			float(pin.get("y", 0.0)) * float(TERRAIN_GRID_SIZE),
		)
	for idx in civilian_nodes.keys():
		if not seen.has(idx):
			civilian_nodes[idx].queue_free()
			civilian_nodes.erase(idx)

func _sync_buildings(pins: Array) -> void:
	var seen := {}
	for pin in pins:
		var id := int(pin.get("id", 0))
		seen[id] = true
		var node: MeshInstance3D = building_nodes.get(id)
		if node == null:
			node = MeshInstance3D.new()
			node.mesh = BoxMesh.new()
			node.scale = Vector3(1.2, 2.0, 1.2)
			buildings_root.add_child(node)
			building_nodes[id] = node
		var kind := str(pin.get("kind", "Residential"))
		var mat := StandardMaterial3D.new()
		mat.albedo_color = BUILDING_COLORS.get(kind, BUILDING_COLORS["Residential"])
		node.material_override = mat
		node.position = Vector3(
			float(pin.get("x", 0.0)) * float(TERRAIN_GRID_SIZE),
			1.5,
			float(pin.get("y", 0.0)) * float(TERRAIN_GRID_SIZE),
		)
	for id in building_nodes.keys():
		if not seen.has(id):
			building_nodes[id].queue_free()
			building_nodes.erase(id)

func _on_tool_pressed(tool: String) -> void:
	current_tool = tool

func _on_speed_selected(index: int) -> void:
	var speeds := [0, 1, 2, 4, 8]
	current_speed = speeds[index]
	_apply_speed(current_speed)

func _on_material_changed(value: float) -> void:
	current_material = int(value)
