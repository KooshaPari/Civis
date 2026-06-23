extends Panel

const GRID := 128

var _heights: PackedFloat32Array
var _biomes: PackedByteArray
var _camera: Camera3D
var _height_scale := 24.0

@onready var _view: TextureRect = $TextureRect
@onready var _dot: ColorRect = $Dot


func setup(
	heights: PackedFloat32Array,
	biomes: PackedByteArray,
	camera: Camera3D,
	height_exaggeration: float = 24.0,
) -> void:
	_heights = heights
	_biomes = biomes
	_camera = camera
	_height_scale = height_exaggeration
	_build_texture()
	_update_dot()


func _build_texture() -> void:
	var img := Image.create(GRID, GRID, false, Image.FORMAT_RGBA8)
	if _heights.is_empty():
		img.fill(Color(0.12, 0.14, 0.16))
		for x in range(0, GRID, 16):
			for z in range(0, GRID, 16):
				img.set_pixel(x, z, Color(0.35, 0.4, 0.45))
	else:
		for z in GRID:
			for x in GRID:
				var idx := z * GRID + x
				var color: Color
				if not _biomes.is_empty() and idx < _biomes.size():
					color = SimulationHost.biome_color(int(_biomes[idx]))
				elif idx < _heights.size():
					color = SimulationHost.height_color(_heights[idx])
				else:
					color = Color.GRAY
				img.set_pixel(x, z, color)
	_view.texture = ImageTexture.create_from_image(img)


func _update_dot() -> void:
	if _camera == null:
		_dot.visible = false
		return
	_dot.visible = true
	_dot.position = Vector2(_camera.orbit_target.x - 2.0, _camera.orbit_target.z - 2.0)


func _gui_input(event: InputEvent) -> void:
	if not (event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT):
		return
	if _camera == null:
		return
	var local := get_local_mouse_position()
	if local.x < 0.0 or local.y < 0.0 or local.x >= GRID or local.y >= GRID:
		return
	var x := int(local.x)
	var z := int(local.y)
	var y: float = _camera.orbit_target.y
	if not _heights.is_empty():
		var idx := z * GRID + x
		if idx >= 0 and idx < _heights.size():
			y = _heights[idx] * _height_scale
	_camera.set_orbit_target(Vector3(float(x), y, float(z)))
	accept_event()
	_update_dot()
