extends Node2D

@onready var player_temp = preload('res://players/playerTemp.tscn')
@onready var server: Node = $server


@export var mainPlayerName = 'asd'
var mainPlayerId = null

var is_spawned = false
var players = {}
	
func _ready() -> void:
	pass
	
func _physics_process(delta: float) -> void:
	if Input.is_action_just_pressed("q") and not is_spawned:
		var msg = StreamPeerBuffer.new()
		msg.put_u8(0)
		for sign in "zh".to_utf8_buffer():
			msg.put_u8(sign)
		server.send(msg)
		
		is_spawned = true


#0
func spawn_self(id, posx, posy):
	var player = player_temp.instantiate()
	player.position = Vector2(posx, posy)
	add_child(player)
	
	player.setPlayerName(mainPlayerName)
	
	mainPlayerId = id
	player.idp = id
	players[id] = player
	

#1
func spawn_entity(id, posx, posy, name_entity):
	if id == mainPlayerId:
		return

	var player = player_temp.instantiate()
	player.position = Vector2(posx, posy)
	add_child(player)
	
	player.setPlayerName(name_entity)
	
	player.idp = id
	players[id] = player
		
		
#2		
func move_self(posx, posy):
	players[mainPlayerId].position.x = posx
	players[mainPlayerId].position.y = posy		

#3			
func move_entity(id, posx, posy):
	if id == mainPlayerId:
		return
		
	var dirx = posx - players[id].position.x
	var diry = posy - players[id].position.y 
	
	players[id].direction = Vector2(dirx, diry)
	
	var lerpindx = get_process_delta_time() * 5
	
	if abs(dirx) > 3:
		players[id].position.x = lerpf(players[id].position.x, posx, lerpindx)
	else:
		players[id].position.x = posx
	
	if abs(diry) > 3:
		players[id].position.y = lerpf(players[id].position.y, posy, lerpindx)
	else:
		players[id].position.y = posy
	
