class_name EraTimelapse
extends RefCounted
## Matches `clients/godot-ref/rust/src/ux.rs` (`DEFAULT_ERA_LENGTH_TICKS`).

const DEFAULT_ERA_LENGTH_TICKS := 5000

static func era_at_tick(tick: int, era_length_ticks: int = DEFAULT_ERA_LENGTH_TICKS) -> int:
	var period := maxi(era_length_ticks, 1)
	return int(tick / period)

static func era_label(tick: int, era_length_ticks: int = DEFAULT_ERA_LENGTH_TICKS) -> String:
	return "Era %d (tick %d)" % [era_at_tick(tick, era_length_ticks), tick]
