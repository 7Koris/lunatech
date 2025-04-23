extends Node

@export var scenes: Array[PackedScene] = []

var current_scene: Node
var scene_idx = 0

const SWITCH_SCENE_TIME = 3
var timer: Timer

func _ready() -> void:
	timer = Timer.new()
	add_child(timer)
	timer.timeout.connect(_on_timeout)
	timer.start(SWITCH_SCENE_TIME)
	current_scene = scenes[0].instantiate()
	add_child(current_scene)

func _on_timeout():
	current_scene.queue_free()
	scene_idx = 0 if scene_idx >= len(scenes) - 1 else scene_idx + 1
	current_scene = scenes[scene_idx].instantiate()
	add_child(current_scene)
