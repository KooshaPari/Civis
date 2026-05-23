extends Node3D

@onready var civis_client = CivisClient
@onready var terrain_mesh: MeshInstance3D = $Terrain/TerrainMesh
@onready var civilians_root: Node3D = $Civilians
@onready var ui = $UI

var current_tool := "Inspect"
var current_material := 0
var current_speed := 1
var terrain_heights: PackedFloat32Array = PackedFloat32Array()
var terrain_biomes: PackedByteArray = PackedByteArray()
var civilian_nodes: Dictionary = {}

func _ready() -> void:
	civis_client.connect("http://127.0.0.1:9090")
	terrain_heights = civis_client.fetch_terrain()
	terrain_biomes = civis_client.fetch_terrain_biomes()
	_build_terrain_mesh()
	_bind_ui()
	_update_snapshot()
	$Timer.start()

func _build_terrain_mesh() -> void:
	if terrain_heights.is_empty():
		return
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for x in range(127):
		for z in range(127):
			_add_terrain_quad(st, x, z)
	terrain_mesh.mesh = st.commit()
	terrain_mesh.rotation_degrees = Vector3(-90, 0, 0)

func _add_terrain_quad(st: SurfaceTool, x: int, z: int) -> void:
	var h00 := _terrain_height(x, z)
	var h10 := _terrain_height(x + 1, z)
	var h01 := _terrain_height(x, z + 1)
	var h11 := _terrain_height(x + 1, z + 1)
	var c00 := _terrain_color(x, z)
	var c10 := _terrain_color(x + 1, z)
	var c01 := _terrain_color(x, z + 1)
	var c11 := _terrain_color(x + 1, z + 1)
	_emit_vertex(st, Vector3(x, h00, z), c00)
	_emit_vertex(st, Vector3(x + 1, h10, z), c10)
	_emit_vertex(st, Vector3(x, h01, z + 1), c01)
	_emit_vertex(st, Vector3(x + 1, h10, z), c10)
	_emit_vertex(st, Vector3(x + 1, h11, z + 1), c11)
	_emit_vertex(st, Vector3(x, h01, z + 1), c01)

func _emit_vertex(st: SurfaceTool, pos: Vector3, color: Color) -> void:
	st.set_color(color)
	st.add_vertex(pos)

func _terrain_height(x: int, z: int) -> float:
	var idx := z * 128 + x
	if idx < 0 or idx >= terrain_heights.size():
		return 0.0
	return terrain_heights[idx] * 24.0

func _terrain_color(x: int, z: int) -> Color:
	var idx := z * 128 + x
	var biome := int(terrain_biomes[idx]) if idx >= 0 and idx < terrain_biomes.size() else 0
	match biome:
		0: return Color(0.25, 0.45, 0.20)
		1: return Color(0.84, 0.76, 0.46)
		2: return Color(0.20, 0.35, 0.48)
		3: return Color(0.56, 0.55, 0.42)
		4: return Color(0.14, 0.22, 0.14)
		5: return Color(0.45, 0.40, 0.30)
		_: return Color(0.55, 0.68, 0.45)

func _bind_ui() -> void:
	for button_name in ["Place Voxel", "Spawn Civilian", "Damage", "Inspect", "Camera"]:
		var button := ui.get_node("BottomBar/HBoxContainer/%s" % button_name) as Button
		button.pressed.connect(_on_tool_pressed.bind(button_name))
	ui.get_node("BottomBar/HBoxContainer/Speed").item_selected.connect(_on_speed_selected)
	ui.get_node("BottomBar/HBoxContainer/Material").value_changed.connect(_on_material_changed)
	ui.get_node("BottomBar/HBoxContainer/TickLabel").text = "Tick: 0"
	ui.get_node("BottomBar/HBoxContainer/PopulationLabel").text = "Population: 0"

func _process(_delta: float) -> void:
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		_handle_click()

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
		civis_client.post_place_voxel(int(pos.x), int(pos.y), int(pos.z), current_material)
	elif current_tool == "Spawn Civilian":
		civis_client.post_spawn_civilian(pos.x, pos.z, 0)

func _on_timer_timeout() -> void:
	_update_snapshot()

func _update_snapshot() -> void:
	var snapshot := civis_client.latest_snapshot()
	if snapshot.is_empty():
		return
	ui.get_node("BottomBar/HBoxContainer/TickLabel").text = "Tick: %s" % snapshot.get("tick", 0)
	ui.get_node("BottomBar/HBoxContainer/PopulationLabel").text = "Population: %s" % snapshot.get("population", 0)
	_sync_civilians(snapshot.get("civ_pins", []))

func _sync_civilians(pins: Array) -> void:
	var seen := {}
	for pin in pins:
		var idx := int(pin.get("idx", 0))
		seen[idx] = true
		var node := civilian_nodes.get(idx)
		if node == null:
			node = MeshInstance3D.new()
			node.mesh = BoxMesh.new()
			node.scale = Vector3(0.4, 1.4, 0.4)
			civilians_root.add_child(node)
			civilian_nodes[idx] = node
		node.position = Vector3(float(pin.get("x", 0.0)) * 128.0, 1.0, float(pin.get("y", 0.0)) * 128.0)
	for idx in civilian_nodes.keys():
		if not seen.has(idx):
			civilian_nodes[idx].queue_free()
			civilian_nodes.erase(idx)

func set_current_tool(tool: String) -> void:
	current_tool = tool

func _on_tool_pressed(tool: String) -> void:
	current_tool = tool

func _on_speed_selected(index: int) -> void:
	var speeds := [0, 1, 2, 4, 8]
	current_speed = speeds[index]

func _on_material_changed(value: float) -> void:
	current_material = int(value)
