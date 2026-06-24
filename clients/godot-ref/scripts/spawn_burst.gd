class_name SpawnBurst
extends RefCounted

## Short faction-colored burst at spawn/damage (FR-CIV-L5, mirrors web `spawnBurst`).

static func emit_at(parent: Node, world_pos: Vector3, color: Color = Color(0.49, 0.85, 0.34)) -> void:
	if parent == null:
		return
	var particles := CPUParticles3D.new()
	particles.one_shot = true
	particles.explosiveness = 1.0
	particles.amount = 12
	particles.lifetime = 0.42
	particles.direction = Vector3(0, 1, 0)
	particles.spread = 55.0
	particles.initial_velocity_min = 1.8
	particles.initial_velocity_max = 4.8
	particles.gravity = Vector3(0, -5.5, 0)
	var sphere := SphereMesh.new()
	sphere.radius = 0.12
	sphere.height = 0.24
	particles.draw_pass_1 = sphere
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.emission_enabled = true
	mat.emission = color
	mat.emission_energy_multiplier = 1.4
	particles.material_override = mat
	particles.position = world_pos
	particles.emitting = true
	parent.add_child(particles)
	var tree := parent.get_tree()
	if tree:
		tree.create_timer(0.65).timeout.connect(particles.queue_free)
