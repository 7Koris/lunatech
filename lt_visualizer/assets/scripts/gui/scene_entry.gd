extends MarginContainer

class_name SceneEntry
signal SceneEntryToggled

@onready var checkbox: CheckBox = $Hbox/MarginContainer/CheckBox
@onready var label: Label = $Hbox/Label
@onready var color = $Hbox/MarginContainer/Color

func _ready() -> void:
	color.color = Color.from_hsv(randf(), 1.0, 1.0)

func get_checkbox() -> CheckBox:
	return $Hbox/MarginContainer/CheckBox
	
func get_label() -> Label:
	return $Hbox/Label

func _on_check_box_toggled(toggled_on: bool) -> void:
	emit_signal("SceneEntryToggled", label.text, toggled_on)
