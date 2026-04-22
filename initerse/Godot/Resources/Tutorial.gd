extends Control

var tutorials = [
	{
		"title": "Tutorial: The beginning !",
		"description": "You've just arrived, in this game you will play... God ! 
You will have to automate the creation of particles (starting from strings
and then continue creating the universe, life and much more !
Your first machine is a string creator, it creates strings from nothing...
Or maybe virtual particles, but no one knows how it works",
	},
	{
		"title": "Tutorial: The beginning 2 !",
		"description": "Now go ahead ",
	},
	{
		"title": "Tutorial: The beginning anddddda 3 !",
		"description": "Now go ahead ",
	},
]
var current_tutorial = 0

func _ready():
	$Description.text = tutorials[current_tutorial].description
	$Title.text = tutorials[current_tutorial].title

func _on_continue_pressed():
	current_tutorial += 1
	if current_tutorial >= len(tutorials):
		$".".hide()
	else:
		$Description.text = tutorials[current_tutorial].description
		$Title.text = tutorials[current_tutorial].title
