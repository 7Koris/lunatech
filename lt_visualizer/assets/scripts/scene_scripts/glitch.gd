extends Node2D

@onready var sprite2d = $Sprite2D
@onready var colorrect = $ColorRect
@onready var c2 = $ColorRect2
var sum = 0
func _process(delta: float) -> void:
	sprite2d.rotation += 0.1 * delta * 100 * OscClient.high_range_rms * 50
	colorrect.material.set("shader_parameter/shake_power", (OscClient.high_range_rms) / 2)
	colorrect.material.set("shader_parameter/mu", (OscClient.high_range_rms) * 1000/2)
	sum += delta + OscClient.high_range_rms * delta * 100
	c2.material.set("shader_parameter/iTime", sum)
	c2.material.set("shader_parameter/mu", OscClient.high_range_rms * 1000 / 2)
