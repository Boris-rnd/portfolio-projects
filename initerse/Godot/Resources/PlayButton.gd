extends Button

@export var scene = "game"

func _on_pressed():
	get_tree().change_scene_to_file("res://Resources/"+scene+".tscn")
