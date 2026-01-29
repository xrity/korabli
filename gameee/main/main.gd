extends Node2D

@onready var player_temp = preload('res://players/playerTemp.tscn')
@onready var server: Node = $server
var players = {}
@onready var mainPlayerId = null
@export var playerName = ''
var is_spawned = false

func _ready() -> void:
	pass
	
func _physics_process(delta: float) -> void:
	if server.is_connected and not is_spawned:
		print("spawn")
		var data = {
			"req": 0,
			"name": playerName
		}
		server.send(data)
		is_spawned = true

func spawn(id, posx, posy):
	var player = player_temp.instantiate()
	add_child(player)
	
	mainPlayerId = id
	player.idp = id
	players[id] = player
	player.position = Vector2(posx, posy)
		
