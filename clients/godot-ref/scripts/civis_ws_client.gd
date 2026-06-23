class_name CivisWsClient
extends Node
## JSON-RPC WebSocket client for `civ-server` (FR-CIV-UX / ADR-009).
## Terrain still comes from civ-watch HTTP via `CivisClient`.

signal snapshot_received(snapshot: Dictionary)
signal connection_changed(state: String)
signal f3d0_frame_received(kind: String, tick: int, frame: Variant)

## WebSocket URL for civ-server. Override via CIV_SERVER_WS env var or the Inspector.
@export var ws_url := "ws://127.0.0.1:3000/ws?tick_format=binary"
@export var reconnect_delay_sec := 3.0
@export var snapshot_throttle_ms := 250

var _ws: WebSocketPeer
var _rpc_id := 1
var _pending: Dictionary = {}
var _connected := false
var _last_snapshot_request_ms := 0
var _reconnect_at_ms := 0

func connect_server(url: String = "") -> void:
	if not url.is_empty():
		ws_url = url
	_close_ws()
	_ws = WebSocketPeer.new()
	var err := _ws.connect_to_url(ws_url)
	if err != OK:
		connection_changed.emit("disconnected")
		_schedule_reconnect()
		return
	connection_changed.emit("reconnecting")

func disconnect_server() -> void:
	_close_ws()
	connection_changed.emit("disconnected")

func set_speed(multiplier: int) -> void:
	_call_rpc("sim.set_speed", {"multiplier": multiplier})

func request_snapshot() -> void:
	_call_rpc("sim.snapshot", {})

func spawn_civilian(x: float, y: float, faction: int = 0) -> void:
	_call_rpc("sim.spawn_civilian", {"x": x, "y": y, "faction": faction})

func spawn_entity(kind: String, x: float, y: float, faction: int = 0) -> void:
	if kind == "civilian":
		spawn_civilian(x, y, faction)
	else:
		_call_rpc("sim.spawn_entity", {"kind": kind, "x": x, "y": y, "faction": faction})

func place_voxel(x: int, y: int, z: int, material: int) -> void:
	_call_rpc("sim.place_voxel", {"x": x, "y": y, "z": z, "material": material})

func apply_damage(x: int, y: int, z: int, radius: int, energy: int = 1000) -> void:
	_call_rpc("sim.damage", {"x": x, "y": y, "z": z, "radius": radius, "energy": energy})

func _close_ws() -> void:
	if _ws != null:
		_ws.close()
	_ws = null
	_pending.clear()
	_connected = false

func _schedule_reconnect() -> void:
	_reconnect_at_ms = Time.get_ticks_msec() + int(reconnect_delay_sec * 1000.0)

func _process(_delta: float) -> void:
	if _ws == null:
		if _reconnect_at_ms > 0 and Time.get_ticks_msec() >= _reconnect_at_ms:
			_reconnect_at_ms = 0
			connect_server(ws_url)
		return

	_ws.poll()
	var state := _ws.get_ready_state()
	match state:
		WebSocketPeer.STATE_OPEN:
			if not _connected:
				_connected = true
				connection_changed.emit("live")
				_on_open()
		WebSocketPeer.STATE_CLOSING, WebSocketPeer.STATE_CLOSED:
			if _connected or state == WebSocketPeer.STATE_CLOSING:
				_connected = false
				connection_changed.emit("disconnected")
				_close_ws()
				_schedule_reconnect()

	while _ws.get_available_packet_count() > 0:
		var packet := _ws.get_packet()
		_handle_packet(packet)

func _on_open() -> void:
	_call_rpc("health", {})
	request_snapshot()
	set_speed(1)

func _call_rpc(method: String, params: Dictionary) -> void:
	if _ws == null or _ws.get_ready_state() != WebSocketPeer.STATE_OPEN:
		return
	var id := _rpc_id
	_rpc_id += 1
	_pending[id] = method
	var body := {
		"jsonrpc": "2.0",
		"id": id,
		"method": method,
		"params": params,
	}
	_ws.send_text(JSON.stringify(body))

func _handle_packet(packet: PackedByteArray) -> void:
	if ClassDB.class_exists("CivisWsFrame"):
		var decoded: Dictionary = CivisWsFrame.decode_ws_packet(packet)
		if not decoded.get("ok", false):
			return
		var kind: String = str(decoded.get("kind", ""))
		var tick: int = int(decoded.get("tick", 0))
		if kind == "RpcJson":
			var parsed = JSON.parse_string(str(decoded.get("json", "")))
			if typeof(parsed) == TYPE_DICTIONARY:
				_handle_rpc_response(parsed)
			return
		var frame = JSON.parse_string(str(decoded.get("json", "")))
		f3d0_frame_received.emit(kind, tick, frame)
		_maybe_refresh_snapshot()
		return

	if packet.size() >= 4:
		var magic := packet.slice(0, 4).get_string_from_ascii()
		if magic == "F3D0":
			_maybe_refresh_snapshot()
			return

	var text := packet.get_string_from_utf8()
	var parsed = JSON.parse_string(text)
	if typeof(parsed) != TYPE_DICTIONARY:
		return
	var msg: Dictionary = parsed
	if msg.has("id") and (msg.has("result") or msg.has("error")):
		_handle_rpc_response(msg)
	elif msg.has("VoxelDelta") or msg.has("BuildingDiff") or msg.has("AgentAppearance"):
		f3d0_frame_received.emit("legacy", 0, msg)
		_maybe_refresh_snapshot()

func _handle_rpc_response(msg: Dictionary) -> void:
	var id: int = int(msg.get("id", -1))
	var method: String = _pending.get(id, "")
	_pending.erase(id)
	if msg.has("error"):
		push_warning("civ-server RPC %s failed: %s" % [method, str(msg.get("error"))])
		return
	var result = msg.get("result")
	if method == "sim.snapshot" and typeof(result) == TYPE_DICTIONARY:
		snapshot_received.emit(_normalize_snapshot(result))
	elif method in ["sim.spawn_civilian", "sim.spawn_entity", "sim.place_voxel", "sim.damage"]:
		request_snapshot()
	elif method == "health":
		pass

func _normalize_snapshot(result: Dictionary) -> Dictionary:
	var out := {
		"tick": int(result.get("tick", 0)),
		"population": int(result.get("population", 0)),
		"voxel_dirty_count": 0,
		"civ_pins": [],
		"buildings": [],
	}
	var pins = result.get("civ_pins", [])
	if typeof(pins) == TYPE_ARRAY:
		out["civ_pins"] = pins
	var buildings = result.get("buildings", [])
	if typeof(buildings) == TYPE_ARRAY:
		out["buildings"] = buildings
	var military = result.get("military_units", [])
	if typeof(military) == TYPE_ARRAY:
		out["military_units"] = military
	if result.has("is_day"):
		out["is_day"] = result.get("is_day")
	return out

func _maybe_refresh_snapshot() -> void:
	var now := Time.get_ticks_msec()
	if now - _last_snapshot_request_ms < snapshot_throttle_ms:
		return
	_last_snapshot_request_ms = now
	request_snapshot()
