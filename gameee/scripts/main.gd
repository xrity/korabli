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
		for sign in mainPlayerName.to_utf8_buffer():
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
func spawn_entity(idServ, posxServ, posyServ, name_entityServ):
	if idServ == mainPlayerId:
		return

	var player = player_temp.instantiate()
	player.position = Vector2(posxServ, posyServ)
	add_child(player)
	
	player.setPlayerName(name_entityServ)
	
	player.idp = idServ
	players[idServ] = player
		
		
#2		
func move_self(posxServ, posyServ):
	players[mainPlayerId].position.x = posxServ
	players[mainPlayerId].position.y = posyServ	

#3			
func move_entity(idServ, posxServ, posyServ, armAngleServ):
	if idServ == mainPlayerId:
		return
	players[idServ].update_position(Vector2(posxServ, posyServ))
	players[idServ].update_arm_angle(armAngleServ, 0.05)

#4
func attack_entity(idattServ, idgetServ, armAngleServ, hpgetServ):
	if idgetServ != 0:
		players[idgetServ].hp -= hpgetServ
		
	if idattServ == mainPlayerId:
		return
	else:
		players[idattServ].attack_process()
		players[idattServ].update_arm_angle(armAngleServ, 0.5)
	
