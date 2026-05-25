extends Camera3D

## World point the camera orbits (terrain centre for 128×128 grid).
@export var orbit_target := Vector3(64.0, 12.0, 64.0)
@export var min_distance := 15.0
@export var max_distance := 250.0
@export var rotate_speed := 0.005
@export var zoom_speed := 4.0

var _yaw := 0.0
var _pitch := 0.6
var _distance := 50.0

func _ready() -> void:
	var offset := global_position - orbit_target
	_distance = clampf(offset.length(), min_distance, max_distance)
	if _distance > 0.01:
		_pitch = asin(clampf(offset.y / _distance, -1.0, 1.0))
		_yaw = atan2(offset.x, offset.z)
	_apply_orbit()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion and Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT):
		_yaw -= event.relative.x * rotate_speed
		_pitch = clampf(_pitch - event.relative.y * rotate_speed, 0.1, 1.4)
		_apply_orbit()
	elif event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_distance = maxf(min_distance, _distance - zoom_speed)
			_apply_orbit()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_distance = minf(max_distance, _distance + zoom_speed)
			_apply_orbit()

func set_orbit_target(target: Vector3, keep_height: bool = false) -> void:
	if keep_height:
		orbit_target = Vector3(target.x, orbit_target.y, target.z)
	else:
		orbit_target = target
	_apply_orbit()


## FR-CIV-UX-005 era / overview camera presets.
func apply_preset(preset: String) -> void:
	match preset:
		"close":
			_distance = 28.0
			_pitch = 0.55
			_yaw = 0.4
		"orbit":
			_distance = 55.0
			_pitch = 0.6
			_yaw = 0.0
		"wide":
			_distance = 120.0
			_pitch = 0.75
			_yaw = -0.5
		_:
			_distance = 50.0
			_pitch = 0.6
	_apply_orbit()


func _apply_orbit() -> void:
	var offset := Vector3(
		_distance * cos(_pitch) * sin(_yaw),
		_distance * sin(_pitch),
		_distance * cos(_pitch) * cos(_yaw),
	)
	global_position = orbit_target + offset
	look_at(orbit_target)
