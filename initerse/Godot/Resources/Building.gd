extends TextureButton

@onready var world = $"../../WorldGrid"
@onready var Tower = world.Tower
@onready var IMAGE_SIZE = $"../../WorldGrid".IMAGE_SIZE
@onready var _current = world.Tower.Empty
var current_tower : int :
	get:
		return _current
	set(value):
		_current = value
		$".".texture_normal = world.get_tower_texture(current_tower)

func _process(_delta):
	var mouse_pos = get_viewport().get_mouse_position()
	mouse_pos -= Vector2(IMAGE_SIZE/2,IMAGE_SIZE/2)
	var cell = (mouse_pos/IMAGE_SIZE).round()
	$".".set_position(cell*IMAGE_SIZE)
	if $".".button_pressed:
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT):
			_current = Tower.Empty
			$".".texture_normal = null
		var tower_coords = Vector2i(cell)
		world.set_tower(tower_coords, current_tower)
	
	if Input.is_key_pressed(KEY_R):
		print("R is pressed")


func _on_pressed():
	pass
