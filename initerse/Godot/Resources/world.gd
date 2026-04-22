extends Node2D

const TILE_POS = 32

func _input(event):
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var tile_pos = event.position
		tile_pos.x -= int(event.position.x)%TILE_POS
		tile_pos.y -= int(event.position.y)%TILE_POS
		print(tile_pos)
		
