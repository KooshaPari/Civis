extends Node3D

## Vertical scale for normalized `[0, 1]` heights from civ-watch (tweak in Inspector).
@export_range(1.0, 64.0, 1.0) var terrain_height_exaggeration := 24.0

## When true (ADR-009), hide world-mutation tools.
@export var spectator_mode := true

const TERRAIN_GRID_SIZE := 128
const CIVILIAN_FOOT_OFFSET := 0.55
const BUILDING_FOOT_OFFSET := 1.0
const MILITARY_FOOT_OFFSET := 0.6
const SPAWN_BURST_FOOT_OFFSET := 1.1
const WORLD_SCALE := 1_000_000
const MUTATION_TOOLS := ["Place Voxel", "Spawn Civilian", "Damage"]
const SPAWN_DRAG_MIN_CELLS := 4.0

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

const MILITARY_COLORS := {
	"Soldier": Color(0.9, 0.35, 0.35),
	"Archer": Color(0.45, 0.75, 0.95),
	"Vehicle": Color(0.75, 0.75, 0.8),
	"Scout": Color(0.95, 0.85, 0.4),
}

var _simulation_host: SimulationHost
@onready var terrain_mesh: MeshInstance3D = $Terrain/TerrainMesh
@onready var civilians_root: Node3D = $Civilians
@onready var buildings_root: Node3D = $Buildings
@onready var military_root: Node3D = $Military
@onready var voxel_overlay: F3d0VoxelOverlay = $VoxelOverlays
@onready var ui = $UI

var current_tool := "Inspect"
var current_material := 0
var current_speed := 1
var spawn_kind := "civilian"
var terrain_heights: PackedFloat32Array = PackedFloat32Array()
var terrain_biomes: PackedByteArray = PackedByteArray()
var civilian_nodes: Dictionary = {}
var building_nodes: Dictionary = {}
var military_nodes: Dictionary = {}
var spawn_count := 0
var _spawn_drag_active := false
var _spawn_drag_start := Vector2(-1.0, -1.0)
var _drag_preview: MeshInstance3D

func _ready() -> void:
	_simulation_host = SimulationHost.new()
	_simulation_host.name = "SimulationHost"
	add_child(_simulation_host)
	_load_terrain()
	_build_terrain_mesh()
	_bind_ui()
	ui.get_node("Minimap").setup(terrain_heights, terrain_biomes, $Camera3D, terrain_height_exaggeration)

func _load_terrain() -> void:
	var terrain := _simulation_host.get_terrain()
	terrain_heights = terrain.get("heights", PackedFloat32Array())
	terrain_biomes = terrain.get("biomes", PackedByteArray())

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

func _world_y_at_norm(norm_x: float, norm_y: float, foot_offset: float) -> float:
	var gx := clampi(int(norm_x * float(TERRAIN_GRID_SIZE)), 0, TERRAIN_GRID_SIZE - 1)
	var gz := clampi(int(norm_y * float(TERRAIN_GRID_SIZE)), 0, TERRAIN_GRID_SIZE - 1)
	return _terrain_height(gx, gz) + foot_offset

func _terrain_color(x: int, z: int) -> Color:
	var idx := z * TERRAIN_GRID_SIZE + x
	if idx < 0:
		return SimulationHost.biome_color(0)
	if not terrain_biomes.is_empty() and idx < terrain_biomes.size():
		return SimulationHost.biome_color(int(terrain_biomes[idx]))
	if idx < terrain_heights.size():
		return SimulationHost.height_color(terrain_heights[idx])
	return SimulationHost.biome_color(0)

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
	var spawn_kind_ui := ui.get_node("BottomBar/HBoxContainer/SpawnKind") as OptionButton
	spawn_kind_ui.clear()
	for label in ["civilian", "vehicle", "airport", "port", "hangar"]:
		spawn_kind_ui.add_item(label)
	spawn_kind_ui.select(0)
	spawn_kind_ui.item_selected.connect(_on_spawn_kind_selected)
	var mode_label := "Spectator" if spectator_mode else "Authoring"
	var hint := "%s — standalone in-process simulation." % mode_label
	ui.get_node("BottomBar").tooltip_text = hint
	if spectator_mode:
		ui.get_node("BottomBar/HBoxContainer/Material").visible = false
		ui.get_node("BottomBar/HBoxContainer/SpawnKind").visible = false
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
	ui.get_node("BottomBar/HBoxContainer/CameraOrbit").pressed.connect(
		func(): cam.apply_preset("orbit")
	)
	_apply_speed(current_speed)

func _apply_speed(speed: int) -> void:
	var timer := $Timer as Timer
	if speed <= 0:
		timer.stop()
		return
	timer.wait_time = 0.1 / float(speed)
	if timer.is_stopped():
		timer.start()

func _process(_delta: float) -> void:
	if spectator_mode:
		return
	if _spawn_drag_active:
		_update_drag_preview()
		return
	if current_tool == "Spawn Civilian":
		return
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) and not _clicking_minimap():
		_handle_mutation_click()

func _input(event: InputEvent) -> void:
	if spectator_mode:
		return
	if not event is InputEventMouseButton:
		return
	var mb := event as InputEventMouseButton
	if mb.button_index != MOUSE_BUTTON_LEFT or _clicking_minimap():
		return
	if current_tool != "Spawn Civilian":
		return
	if mb.pressed:
		var norm := _ray_hit_norm()
		if norm.x < 0.0:
			return
		if spawn_kind in ["vehicle", "airport", "port", "hangar"]:
			_spawn_drag_active = true
			_spawn_drag_start = norm
			_ensure_drag_preview()
		else:
			_spawn_at_norm(norm.x, norm.y)
	else:
		if not _spawn_drag_active:
			return
		_spawn_drag_active = false
		var end := _ray_hit_norm()
		_clear_drag_preview()
		if end.x < 0.0:
			if _spawn_drag_start.x >= 0.0:
				_spawn_at_norm(_spawn_drag_start.x, _spawn_drag_start.y)
			return
		_spawn_convoy_along_drag(_spawn_drag_start, end)

func _clicking_minimap() -> bool:
	var minimap := ui.get_node("Minimap") as Control
	return minimap.get_rect().has_point(minimap.get_local_mouse_position())

func _ray_hit_norm() -> Vector2:
	var cam: Camera3D = $Camera3D
	var from := cam.project_ray_origin(get_viewport().get_mouse_position())
	var to := from + cam.project_ray_normal(get_viewport().get_mouse_position()) * 1000.0
	var query := PhysicsRayQueryParameters3D.create(from, to)
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	if hit.is_empty():
		return Vector2(-1.0, -1.0)
	var pos: Vector3 = hit.position
	return Vector2(
		clampf(pos.x / float(TERRAIN_GRID_SIZE), 0.0, 1.0),
		clampf(pos.z / float(TERRAIN_GRID_SIZE), 0.0, 1.0),
	)

func _spawn_convoy_along_drag(start: Vector2, end: Vector2) -> void:
	var dist_cells := start.distance_to(end) * float(TERRAIN_GRID_SIZE)
	if dist_cells < SPAWN_DRAG_MIN_CELLS:
		if end.x >= 0.0:
			_spawn_at_norm(end.x, end.y)
		return
	var steps := mini(int(dist_cells / 8.0), 31)
	if steps <= 0:
		_spawn_at_norm(end.x, end.y)
		return
	for i in range(steps + 1):
		var t := float(i) / float(steps)
		var nx := lerpf(start.x, end.x, t)
		var ny := lerpf(start.y, end.y, t)
		_spawn_at_norm(nx, ny)

func _spawn_at_norm(norm_x: float, norm_y: float) -> void:
	_simulation_host.tick()
	spawn_count += 1
	ui.get_node("BottomBar/HBoxContainer/SpawnCountLabel").text = "Spawns: %d" % spawn_count
	_spawn_burst_at_norm(norm_x, norm_y, _spawn_burst_color_for_kind(spawn_kind))

func _ensure_drag_preview() -> void:
	if _drag_preview != null:
		return
	_drag_preview = MeshInstance3D.new()
	var mesh := CylinderMesh.new()
	mesh.top_radius = 0.35
	mesh.bottom_radius = 0.5
	mesh.height = 0.2
	_drag_preview.mesh = mesh
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.9, 0.85, 0.35, 0.55)
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	_drag_preview.material_override = mat
	military_root.add_child(_drag_preview)

func _update_drag_preview() -> void:
	if _drag_preview == null or _spawn_drag_start.x < 0.0:
		return
	var end := _ray_hit_norm()
	if end.x < 0.0:
		_drag_preview.visible = false
		return
	_drag_preview.visible = true
	var sx := _spawn_drag_start.x * float(TERRAIN_GRID_SIZE)
	var sz := _spawn_drag_start.y * float(TERRAIN_GRID_SIZE)
	var ex := end.x * float(TERRAIN_GRID_SIZE)
	var ez := end.y * float(TERRAIN_GRID_SIZE)
	_drag_preview.position = Vector3(ex, 1.2, ez)
	var dir := Vector3(ex - sx, 0.0, ez - sz)
	if dir.length_squared() > 0.01:
		_drag_preview.look_at(_drag_preview.position + dir.normalized(), Vector3.UP)

func _clear_drag_preview() -> void:
	if _drag_preview != null:
		_drag_preview.visible = false

func _handle_mutation_click() -> void:
	var cam: Camera3D = $Camera3D
	var from := cam.project_ray_origin(get_viewport().get_mouse_position())
	var to := from + cam.project_ray_normal(get_viewport().get_mouse_position()) * 1000.0
	var query := PhysicsRayQueryParameters3D.create(from, to)
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	if hit.is_empty():
		return
	var pos: Vector3 = hit.position
	if current_tool == "Place Voxel":
		_simulation_host.tick()
	elif current_tool == "Damage":
		var wx := int(pos.x) * WORLD_SCALE
		var wy := int(maxf(0.0, pos.y)) * WORLD_SCALE
		var wz := int(pos.z) * WORLD_SCALE
		_simulation_host.tick()
		SpawnBurst.emit_at(self, pos + Vector3(0, 0.6, 0), Color(1.0, 0.3, 0.3))

func _on_timer_timeout() -> void:
	_simulation_host.tick()
	var snapshot := _simulation_host.snapshot()
	if snapshot.is_empty():
		return
	_apply_snapshot(snapshot)

func _apply_snapshot(snapshot: Dictionary) -> void:
	var tick := int(snapshot.get("tick", 0))
	ui.get_node("BottomBar/HBoxContainer/TickLabel").text = "Tick: %s" % tick
	ui.get_node("BottomBar/HBoxContainer/PopulationLabel").text = "Population: %s" % snapshot.get("population", 0)
	ui.get_node("BottomBar/HBoxContainer/EraLabel").text = EraTimelapse.era_label(tick)
	_apply_day_night(bool(snapshot.get("is_day", true)))
	_sync_civilians(snapshot.get("civ_pins", []))
	_sync_buildings(snapshot.get("buildings", []))
	_sync_military(snapshot.get("military_units", []))


func _apply_day_night(is_day: bool) -> void:
	var sun: DirectionalLight3D = $DirectionalLight3D
	sun.light_energy = 1.15 if is_day else 0.32
	var env: Environment = $WorldEnvironment.environment
	if env:
		env.ambient_light_energy = 0.55 if is_day else 0.18

func _sync_civilians(pins: Array) -> void:
	var seen := {}
	for pin in pins:
		var idx := int(pin.get("idx", 0))
		seen[idx] = true
		var node: MeshInstance3D = civilian_nodes.get(idx)
		if node == null:
			node = MeshInstance3D.new()
			var cap := CapsuleMesh.new()
			cap.radius = 0.22
			cap.height = 1.05
			node.mesh = cap
			civilians_root.add_child(node)
			civilian_nodes[idx] = node
		var job := str(pin.get("job", "unemployed")).to_lower()
		var mat := StandardMaterial3D.new()
		mat.albedo_color = JOB_COLORS.get(job, JOB_COLORS["unemployed"])
		node.material_override = mat
		var norm_x := float(pin.get("x", 0.0))
		var norm_y := float(pin.get("y", 0.0))
		node.position = Vector3(
			norm_x * float(TERRAIN_GRID_SIZE),
			_world_y_at_norm(norm_x, norm_y, CIVILIAN_FOOT_OFFSET),
			norm_y * float(TERRAIN_GRID_SIZE),
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
		var norm_x := float(pin.get("x", 0.0))
		var norm_y := float(pin.get("y", 0.0))
		node.position = Vector3(
			norm_x * float(TERRAIN_GRID_SIZE),
			_world_y_at_norm(norm_x, norm_y, BUILDING_FOOT_OFFSET),
			norm_y * float(TERRAIN_GRID_SIZE),
		)
	for id in building_nodes.keys():
		if not seen.has(id):
			building_nodes[id].queue_free()
			building_nodes.erase(id)

func _sync_military(units: Array) -> void:
	var seen := {}
	for unit in units:
		var id := int(unit.get("id", 0))
		seen[id] = true
		var node: MeshInstance3D = military_nodes.get(id)
		if node == null:
			node = MeshInstance3D.new()
			var cone := CylinderMesh.new()
			cone.top_radius = 0.05
			cone.bottom_radius = 0.35
			cone.height = 1.2
			node.mesh = cone
			military_root.add_child(node)
			military_nodes[id] = node
		var unit_type := str(unit.get("unit_type", "Soldier"))
		var mat := StandardMaterial3D.new()
		mat.albedo_color = MILITARY_COLORS.get(unit_type, MILITARY_COLORS["Soldier"])
		node.material_override = mat
		var norm_x := float(unit.get("x", 0.0))
		var norm_y := float(unit.get("y", 0.0))
		node.position = Vector3(
			norm_x * float(TERRAIN_GRID_SIZE),
			_world_y_at_norm(norm_x, norm_y, MILITARY_FOOT_OFFSET),
			norm_y * float(TERRAIN_GRID_SIZE),
		)
	for id in military_nodes.keys():
		if not seen.has(id):
			military_nodes[id].queue_free()
			military_nodes.erase(id)

func _on_tool_pressed(tool: String) -> void:
	current_tool = tool

func _on_spawn_kind_selected(index: int) -> void:
	spawn_kind = ["civilian", "vehicle", "airport", "port", "hangar"][index]

func _on_speed_selected(index: int) -> void:
	var speeds := [0, 1, 2, 4, 8]
	current_speed = speeds[index]
	_apply_speed(current_speed)

func _on_material_changed(value: float) -> void:
	current_material = int(value)

func _spawn_burst_color_for_kind(kind: String) -> Color:
	match kind:
		"vehicle":
			return Color(0.75, 0.75, 0.82)
		"airport":
			return Color(0.55, 0.72, 0.95)
		"port":
			return Color(0.45, 0.82, 0.72)
		"hangar":
			return Color(0.72, 0.58, 0.48)
		_:
			return Color(0.49, 0.85, 0.34)

func _spawn_burst_at_norm(norm_x: float, norm_y: float, color: Color) -> void:
	var gx := norm_x * float(TERRAIN_GRID_SIZE)
	var gz := norm_y * float(TERRAIN_GRID_SIZE)
	var gy := _world_y_at_norm(norm_x, norm_y, SPAWN_BURST_FOOT_OFFSET)
	SpawnBurst.emit_at(civilians_root, Vector3(gx, gy, gz), color)
