class_name WorldGrid extends TileMap

var WORLD = {}

@export
var IMAGE_SIZE = 32

enum Tower {
	Empty,
	Electron,
	StringCreator,
	VerticalRoad,
	CornerRoad,
	Generator1,
}
var tower_textures = [
	preload("res://Resources/Empty.png"),
	preload("res://Resources/Electron.png"),
	preload("res://Resources/string creator.png"),
	preload("res://Resources/Vertical Road.png"),
	preload("res://Resources/Corner Road.png"),
	preload("res://Resources/1 generator.png"),
]

func get_tower_texture(tower: Tower) -> Texture2D:
	var text = tower_textures[tower]
	if tower == Tower.Empty:pass
		#text.draw_rect()
	return text


func set_tower(pos: Vector2i, tower: Tower):
	WORLD[pos] = tower
	var sprite = Sprite2D.new()
	sprite.position = pos*IMAGE_SIZE+Vector2i(IMAGE_SIZE/2, IMAGE_SIZE/2)
	sprite.texture = tower_textures[tower]
	
	$".".add_child(sprite)


func get_tower(pos: Vector2i) -> Tower:
	var row = WORLD.get(pos.y, {})
	return row.get(pos.x, Tower.Empty)


func _ready():
	var tileset = TileSet.new()
	var tile_id = 0
	var source = TileSetAtlasSource.new()
	var atlas = PackedByteArray()	
	for texture in tower_textures:
		if texture.get_image().data.width != IMAGE_SIZE or texture.get_image().data.height != IMAGE_SIZE:
			print("Not gonna work !!")
		atlas.append_array(texture.get_image().data.data)

	source.texture = ImageTexture.create_from_image(Image.create_from_data(IMAGE_SIZE*tower_textures.size(), IMAGE_SIZE, false, Image.FORMAT_RGB8, atlas))
	
	tileset.tile_size = Vector2i(IMAGE_SIZE,IMAGE_SIZE)
	for i in tower_textures:
		source.create_tile(Vector2i(tile_id,0))
		tile_id += 1
	
	tileset.add_source(source)

	$".".tile_set = tileset

	$".".set_cell(0, Vector2i(2, 3), 0)
	$".".set_cell(0, Vector2i(5, 6), 1)
	$".".set_cell(0, Vector2i(5, 7), 2)
	#var w = get_viewport().size.x
	#var h = get_viewport().size.y
	#var img = Image.create(w, h, false, Image.FORMAT_RGBA4444)
	#$".".texture = ImageTexture.create_from_image(img)
	set_tower(Vector2i(5,5), Tower.Electron)

func _process(_delta):
	pass
