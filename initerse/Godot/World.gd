class_name World

enum Tower {
	Empty,
	Electron,
	StringCreator
}

var inner = {}

func set_tower(pos: Vector2i, value: Tower):
	var row = inner.get(pos.y, {})
	row[pos.x] = value
	inner[pos.y] = row

func get_tower(pos: Vector2i) -> Tower:
	var row = inner.get(pos.y, {})
	return row.get(pos.x, Tower.Empty)

