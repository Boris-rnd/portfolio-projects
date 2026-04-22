extends HBoxContainer

func _ready():
	for i in  $"../../../WorldGrid".tower_textures.size()-1:
		i += 1 # Skip Empty texture
		var texture = $"../../../WorldGrid".tower_textures[i]
		var tower_button = TextureButton.new()
		tower_button.texture_normal = texture
		var button_pressed = func but_clicked():
			$".".clicked(tower_button, i)
		tower_button.connect("pressed", button_pressed)
		add_child(tower_button)

func clicked(button,tower):
	$"../../Building".current_tower = tower
	$"../../Building".size.x = button.texture_normal.get_width()
	$"../../Building".size.y = button.texture_normal.get_height()
