extends Control

@onready var visual_display = preload("res://assets/scenes/VisualDisplay.tscn")
@onready var scene_entry = preload("res://assets/scenes/gui/SceneEntry.tscn")
@onready var scene_list = $"VBoxContainer/TabContainer/Scene Select/MarginContainer3/VBoxContainer/ScrollContainer/SceneList"
@onready var online = preload("res://assets/sprites/Online.tres")
@onready var offline = preload("res://assets/sprites/Offline.tres")
@onready var server_status = $HBoxContainer/StatusTexture
@onready var port_error = $"VBoxContainer/TabContainer/Settings/VBoxContainer/Server Settings/Server Settings/PortError"
@onready var variance_error = $"VBoxContainer/TabContainer/Settings/VBoxContainer/Visualization Settings/Visualization Settings/VarianceError"
@onready var delay_error = $"VBoxContainer/TabContainer/Settings/VBoxContainer/Visualization Settings/Visualization Settings/DelayError"

var visual_display_instance: VisualDisplay

var bin_path = ProjectSettings.globalize_path("res://bin/lt_server")
var server_pid: int
var platform: String

var port = 3000
var input_mode = false
var scene_change_delay = 20.0
var scene_change_variance = 5.0
var shuffle_mode = false

var scenes = {}
var checkbox_state = {}

var dir: DirAccess = null
var last_files = null

var p_accum = 0
var scene_idx = 0

func _process(delta: float) -> void:
	var files = dir.get_files()
	if last_files != files:
		_reset_scene_entries()
	last_files = files
	
	p_accum += 1
	if p_accum == 50:
		clean_zombie()
		p_accum = 0
		if is_server_running():
			server_status.texture = online
		else:
			server_status.texture = offline
			restart_server()


func _ready() -> void:
	var instance = visual_display.instantiate()
	add_child(instance)
	visual_display_instance = instance
	
	platform = OS.get_name()
	# TODO: Windows support
	if platform != "Linux":
		printerr("Unsupported Platform: " + platform)
		OS.kill(OS.get_process_id())
	
	if is_server_running():
		server_status.texture = online
	else:
		restart_server()
	_reset_scene_entries()
	_on_scene_switcher_timeout()


func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		kill_server()
		get_tree().quit()


func is_server_running() -> bool:
	var args = ["lt_server"]
	var output = []
	
	match platform:
		"Linux":
			OS.execute("pgrep", args, output)
		"Windows":
			# TODO
			pass
			
	return str(output[0]).replace("\n", "").is_valid_int()


func kill_server():
	var output = []
	match platform:
		"Linux":
			OS.execute("killall", ["lt_server"], output)
		"Windows":
			# TODO
			pass


func launch_bin():
	var output = []
	var args = []
	match platform:
		"Linux":
			args = ["-p", port, "-H"]
			if input_mode:
				args.append("-I")
		"Windows":
			# TODO
			pass
	server_pid = OS.create_process(bin_path, args)


func restart_server():
	kill_server()
	clean_zombie()
	launch_bin()


func clean_zombie():
	var output = []
	OS.execute("pgrep", ["lt_server"], output)
	var pids = str(output[0]).split("\n")
	for pid in pids:
		OS.is_process_running(pid.to_int())

func _on_entry_toggled(scene, toggled_on):
	checkbox_state[scene] = toggled_on
	if not toggled_on:
		scenes.erase(scene)
	else:
		# check hashmap for scene
		if scenes.has(scene):
			pass
		else:
			var new_scene = load("res://assets/scenes/vis_scenes/" + scene)
			scenes[scene] = new_scene


func _reset_scene_entries():
	for scene_entry in scene_list.get_children():
		scene_entry.queue_free()
	dir = DirAccess.open("res://assets/scenes/vis_scenes/")
	dir.list_dir_begin()
	while (true):
		var file = dir.get_next()
		if file == "":
			break
		elif file.get_extension() == "tscn":
			var new_scene_entry: SceneEntry = scene_entry.instantiate()
			new_scene_entry.get_label().text = file
			new_scene_entry.connect("SceneEntryToggled", _on_entry_toggled)
			scene_list.add_child(new_scene_entry)
			
			if checkbox_state.has(file):
				new_scene_entry.checkbox.button_pressed = checkbox_state[file]
			else:
				checkbox_state[file] = true
				new_scene_entry.checkbox.button_pressed = checkbox_state[file]
			
			# check hashmap for scene
			if scenes.has(file):
				pass
			else:
				var new_scene = load("res://assets/scenes/vis_scenes/" + file)
				scenes[file] = new_scene


func _shuffle_mode_toggled(toggled_on: bool) -> void:
	shuffle_mode = toggled_on


func _on_scene_change_delay_text_changed(new_text: String) -> void:
	if new_text.is_valid_float():
		delay_error.text = ""
		scene_change_delay = new_text.to_float()
	else:
		delay_error.text = "INVALID"


func _on_scene_change_var_text_changed(new_text: String) -> void:
	if new_text.is_valid_float():
		variance_error.text = ""
		scene_change_variance = new_text.to_float()
	else:
		variance_error.text = "INVALID"


func _on_port_text_changed(new_text: String) -> void:
	if new_text.is_valid_int():
		port_error.text = ""
		port = new_text.to_int()
		restart_server()
		OscClient.port = port
	else:
		port_error.text = "INVALID"


func _input_mode_toggled(toggled_on: bool) -> void:
	input_mode = toggled_on
	restart_server()


func _on_scene_switcher_timeout() -> void:
	if visual_display_instance != null:
		var new_time =  scene_change_delay + randf_range(-scene_change_variance, scene_change_variance)
		new_time = 0.01 if new_time <= 0 else new_time
		$SceneSwitcher.wait_time = new_time
		$SceneSwitcher.start()
		
		var scene_keys = scenes.keys()
		if len(scene_keys) <= 0:
			return
			
		
		for child in visual_display_instance.scene_layer.get_children():
			child.queue_free()
		
		if shuffle_mode:
			scene_idx = randi() % len(scene_keys)
		else:
			scene_idx = 0 if scene_idx >= len(scene_keys) - 1 else scene_idx + 1
		
		var new_scene = scenes[scene_keys[scene_idx]].instantiate()
		visual_display_instance.scene_layer.add_child(new_scene)
