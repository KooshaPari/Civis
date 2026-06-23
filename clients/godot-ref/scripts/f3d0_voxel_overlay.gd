class_name F3d0VoxelOverlay
extends Node3D
## Renders coarse markers for F3D0 `VoxelDelta` chunks (live path; not full 16³ mesh).

const CHUNK_EDGE := 16
const MAX_OVERLAYS := 64

var _chunk_nodes: Dictionary = {}

func apply_voxel_delta_frame(frame: Variant) -> void:
	if typeof(frame) != TYPE_DICTIONARY:
		return
	var body: Dictionary = frame
	if frame.has("VoxelDelta"):
		body = frame["VoxelDelta"]
	var deltas: Array = body.get("deltas", [])
	if deltas.is_empty():
		return
	var shown := 0
	for delta in deltas:
		if shown >= MAX_OVERLAYS:
			break
		if typeof(delta) != TYPE_DICTIONARY:
			continue
		var event: Dictionary = delta.get("event", {})
		var chunk_raw: int = int(event.get("chunk_id", 0))
		if chunk_raw == 0:
			continue
		_upsert_chunk_marker(chunk_raw)
		shown += 1

func _upsert_chunk_marker(chunk_raw: int) -> void:
	var node: MeshInstance3D = _chunk_nodes.get(chunk_raw)
	if node == null:
		node = MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(CHUNK_EDGE * 0.9, CHUNK_EDGE * 0.35, CHUNK_EDGE * 0.9)
		node.mesh = box
		var mat := StandardMaterial3D.new()
		mat.albedo_color = Color(0.95, 0.55, 0.2, 0.45)
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		node.material_override = mat
		add_child(node)
		_chunk_nodes[chunk_raw] = node
	var grid := _chunk_grid_xyz(chunk_raw)
	node.position = Vector3(
		float(grid.x) * CHUNK_EDGE + CHUNK_EDGE * 0.5,
		float(grid.y) * CHUNK_EDGE + CHUNK_EDGE * 0.2,
		float(grid.z) * CHUNK_EDGE + CHUNK_EDGE * 0.5,
	)

func _chunk_grid_xyz(chunk_raw: int) -> Vector3i:
	var raw := chunk_raw
	var cx := int((raw >> 40) & 0x00FFFFFF)
	var cy := int((raw >> 16) & 0x00FFFFFF)
	var cz := int(raw & 0x0000FFFF)
	if cx & 0x00800000:
		cx |= ~0x00FFFFFF
	if cy & 0x00800000:
		cy |= ~0x00FFFFFF
	if cz & 0x00008000:
		cz |= ~0x0000FFFF
	return Vector3i(cx, cy, cz)
