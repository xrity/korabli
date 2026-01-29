extends Node2D

@onready var player_temp = preload('res://players/playerTemp.tscn')
@onready var server: Node = $server
@export var playerName = ''
var is_spawned = false
var players = {}
var mainPlayerId = null

func _ready() -> void:
	pass
	
func _physics_process(delta: float) -> void:
	if Input.is_action_just_pressed("q") and not is_spawned:
		var data = {
			"req": 0,
			"name": playerName
		}
		server.send(data)
		is_spawned = true

func spawn_self(id, posx, posy):
	var player = player_temp.instantiate()
	add_child(player)
	mainPlayerId = id
	player.idp = id
	players[id] = player
	player.position = Vector2(posx, posy)

func spawn_entity(id, posx, posy):
	var player = player_temp.instantiate()
	add_child(player)
	player.idp = id
	players[id] = player
	player.position = Vector2(posx, posy)
			
func move_entity(id, posx, posy):
	players[id].position.x = posx
	players[id].position.y = posy
	
func move_self(apr):
	if apr:
		pass
