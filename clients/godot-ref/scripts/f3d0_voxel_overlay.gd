class_name F3d0VoxelOverlay
extends Node3D
## F3D0 VoxelDelta: 16³ mesh when dense `voxels` present; else chunk markers.

const CHUNK_EDGE := 16
const CHUNK_VOXELS := 4096
const MAX_OVERLAYS := 64

var _chunk_nodes: Dictionary = {}

func apply_voxel_delta_frame(frame: Variant) -> void:
	if typeof(frame) != TYPE_DICTIONARY:
		return
	var body: Dictionary = frame.get("VoxelDelta", frame)
	var deltas: Array = body.get("deltas", [])
	var shown := 0
	for delta in deltas:
		if shown >= MAX_OVERLAYS or typeof(delta) != TYPE_DICTIONARY:
			continue
		var chunk_raw: int = int(delta.get("event", {}).get("chunk_id", 0))
		if chunk_raw == 0:
			continue
		var voxels: Array = delta.get("voxels", [])
		if voxels.size() == CHUNK_VOXELS and ClassDB.class_exists("CivisWsFrame"):
			_upsert_chunk_mesh(chunk_raw, voxels)
		else:
			_upsert_chunk_marker(chunk_raw)
		shown += 1

func _upsert_chunk_mesh(chunk_raw: int, voxels: Array) -> void:
	var node: MeshInstance3D = _ensure_node(chunk_raw)
	var ids := PackedInt32Array()
	ids.resize(CHUNK_VOXELS)
	for i in CHUNK_VOXELS:
		ids[i] = int(voxels[i])
	var built: Dictionary = CivisWsFrame.mesh_voxel_chunk(ids)
	if not built.get("ok", false):
		_upsert_chunk_marker(chunk_raw)
		return
	var mesh := ArrayMesh.new()
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = built["vertices"]
	arrays[Mesh.ARRAY_NORMAL] = built["normals"]
	arrays[Mesh.ARRAY_INDEX] = built["indices"]
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	node.mesh = mesh
	node.position = _chunk_origin(chunk_raw)

func _upsert_chunk_marker(chunk_raw: int) -> void:
	var node: MeshInstance3D = _ensure_node(chunk_raw)
	var box := BoxMesh.new()
	box.size = Vector3(CHUNK_EDGE * 0.9, CHUNK_EDGE * 0.35, CHUNK_EDGE * 0.9)
	node.mesh = box
	node.position = _chunk_origin(chunk_raw) + Vector3(CHUNK_EDGE * 0.5, CHUNK_EDGE * 0.2, CHUNK_EDGE * 0.5)

func _ensure_node(chunk_raw: int) -> MeshInstance3D:
	var node: MeshInstance3D = _chunk_nodes.get(chunk_raw)
	if node == null:
		node = MeshInstance3D.new()
		add_child(node)
		_chunk_nodes[chunk_raw] = node
	return node

func _chunk_origin(chunk_raw: int) -> Vector3:
	var g := _chunk_grid_xyz(chunk_raw)
	return Vector3(float(g.x) * CHUNK_EDGE, float(g.y) * CHUNK_EDGE, float(g.z) * CHUNK_EDGE)

func _chunk_grid_xyz(chunk_raw: int) -> Vector3i:
	var cx := int((chunk_raw >> 40) & 0xFFFFFF)
	var cy := int((chunk_raw >> 16) & 0xFFFFFF)
	var cz := int(chunk_raw & 0xFFFF)
	if cx & 0x800000: cx |= ~0xFFFFFF
	if cy & 0x800000: cy |= ~0xFFFFFF
	if cz & 0x8000: cz |= ~0xFFFF
	return Vector3i(cx, cy, cz)
