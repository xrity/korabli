extends Node2D

@onready var player_temp = preload('res://players/playerTemp.tscn')
@onready var camera_scene = preload("res://players/camera.tscn")
@onready var server: Node = $server



@export var mainPlayerName = 'asd'
@export var is_Online = false
var mainPlayerId = null

var players = {}
	
func _ready() -> void:
	pass
	
func _physics_process(delta: float) -> void:
	if Input.is_action_just_pressed("connect"):
		if is_Online:
			var msg = StreamPeerBuffer.new()
			msg.put_u8(0)
			for sign in mainPlayerName.to_utf8_buffer():
				msg.put_u8(sign)
			server.send(msg)
		elif not is_Online:
			spawn_self(1, 100, 0, 0)


#0
func spawn_self(idServ, hpServ, posxServ, posyServ):
	var player = player_temp.instantiate()
	var camera = camera_scene.instantiate()
	add_child(player)
	player.add_child(camera)
	mainPlayerId = idServ
	
	player.hp = hpServ
	player.position = Vector2(posxServ, posyServ)
	player.idp = idServ
	players[idServ] = player
	
	player.setPlayerName(mainPlayerName)
	player.update_weapon(0)
	
	
#1
func spawn_entity(idServ, hpServ, name_entityServ, posxServ, posyServ):
	if idServ == mainPlayerId:
		return

	var player = player_temp.instantiate()
	add_child(player)
	
	player.hp = hpServ
	player.position = Vector2(posxServ, posyServ)
	player.idp = idServ
	players[idServ] = player
	player.setPlayerName(name_entityServ)
	
		
#2			
func game_state_process(
		tickServer, idServ, hpServ,
		angleServ, posxServ, posyServ, 
		is_attackingServ, is_dodgeServ,
		is_movingServ, weaponServ
	):
	if idServ != mainPlayerId:
		players[idServ].hp = hpServ
		players[idServ].is_moving = is_movingServ
		players[idServ].update_position(Vector2(posxServ, posyServ))
		players[idServ].update_arm_angle(angleServ)
		if players[idServ].weapon_indx != weaponServ:
			players[idServ].update_weapon(weaponServ)
		if is_attackingServ:
			players[idServ].attack()
		if is_dodgeServ:
			players[idServ].dash()
	else:
		players[idServ].hp = hpServ
		players[idServ].move_aprove_from_server(tickServer, posxServ, posyServ)
