extends CharacterBody2D

@onready var server: Node = $"../server"
@onready var lobby: Node2D = $".."
@onready var anim: AnimationPlayer = $anim

@onready var weapons_list = [
	preload("res://weapon/weapon_0.tscn"),
	preload("res://weapon/weapon_1.tscn")
]

@onready var namelabel: Label = $namelabel
@onready var hplabel: Label = $hplabel

@onready var arm_node: Node2D = $armNode

@onready var anim_dash: AnimationPlayer = $anim_dash


var hp = 100
var speed = 100
var weapons = [0, 1]
var current_weapon = null
var weapon_change = 0

var weapon_indx = null

var idp = null
var pname = null

var direction = Vector2(0, 0)
var targepos = Vector2(0, 0)
var dir_buffer = 'down'

var is_attacking = false
var is_dashing = false
var is_moving = false

var move_buffer_client = {}
var move_buffer_server  = []
var attack_buffer = []


func _physics_process(delta: float) -> void:
	if idp == lobby.mainPlayerId:
		if lobby.is_Online:
			send_data()
			move_buffer_client[server.tick_client] = Vector2(position.x, position.y)
		
		if not is_attacking:
			arm_node.look_at(get_global_mouse_position())
			arm_node.rotation_degrees = wrapf(arm_node.rotation_degrees, 0, 360)
			if arm_node.rotation_degrees < 270 and arm_node.rotation_degrees > 90:
				arm_node.get_child(0).get_child(0).scale.y = -1
			else:
				arm_node.get_child(0).get_child(0).scale.y = 1
		
		if not is_dashing:
			direction = Input.get_vector("left", "right", "up", "down")
			velocity = speed * direction
		
		if direction:
			is_moving = true
		else:
			is_moving = false
		
		if Input.is_action_just_pressed("attack") and not is_attacking:
			attack()
			
		if Input.is_action_just_pressed("change weapon") and not is_attacking:
			change_weapon()
				
		if Input.is_action_just_pressed("dash") and not is_dashing and not is_attacking:
			dash()
			
		move_and_slide()
			
	else:
		move_interpolation()
	
	hplabel.text = str(hp)
	animation_process(direction.x, direction.y)
	
func move_aprove_from_server(tick_serv, newposx, newposy):
	if tick_serv in move_buffer_client:
		var history_pos = move_buffer_client[tick_serv]
		var server_pos = Vector2(newposx, newposy)

		if history_pos.distance_to(server_pos) > 10.0:
			position = server_pos

	
	var keys_to_remove = []
	for t in move_buffer_client.keys():
		var diff = t - tick_serv
		if diff <= 0 and diff > -128: 
			keys_to_remove.append(t)
		elif diff > 128:
			keys_to_remove.append(t)

	for k in keys_to_remove:
		move_buffer_client.erase(k)

func update_arm_angle(new_arm_angle):
	var new_arm_angle_360 = remap(new_arm_angle, 0, 255, 0, 360)
	arm_node.rotation_degrees = new_arm_angle_360	
	if arm_node.rotation_degrees < 270 and arm_node.rotation_degrees > 90:
		arm_node.get_child(0).get_child(0).scale.y = -1
	else:
		arm_node.get_child(0).get_child(0).scale.y = 1

func move_interpolation():
	if len(move_buffer_server) > 0:
		targepos = move_buffer_server[0]
	
	if abs(targepos.x - position.x) > 200:
		position.x = targepos.x
	elif abs(targepos.x - position.x) > 2.5:
		position.x = lerpf(position.x, targepos.x, 0.1)
	else:
		position.x = targepos.x
	
	if abs(targepos.y - position.y) > 200:
		position.y = targepos.y
	elif abs(targepos.y - position.y) > 2.5:
		position.y = lerpf(position.y, targepos.y, 0.1)
	else:
		position.y = targepos.y
		
	direction.x = targepos.x - position.x
	direction.y = targepos.y - position.y
	move_buffer_server.clear()

func update_position(new_pos: Vector2):
	move_buffer_server.append(new_pos)

func setPlayerName(nameinput):
	pname = nameinput
	namelabel.text = nameinput
	
func animation_process(dirx, diry):
	if direction:
		anim.play("run_" + direction_process(dirx,diry))
	else:
		anim.play("idle_" + dir_buffer)
		
func direction_process(dirx, diry):
	if dirx > 0:
		dir_buffer = "right"
		return "right"
	elif dirx < 0:
		dir_buffer = "left"
		return "left"
	elif diry > 0:
		dir_buffer = "down"
		return "down"
	elif diry < 0:
		dir_buffer = "up"
		return "up"
	else: return dir_buffer

func change_weapon():
	weapon_change = (weapon_change + 1)%2
	update_weapon(weapons[weapon_change])
	
func update_weapon(wp):
	weapon_indx = wp
	if current_weapon:
		current_weapon.queue_free()
	
	current_weapon = weapons_list[wp].instantiate()
	arm_node.add_child(current_weapon)
	current_weapon.anim_attack.play("get")
		
func dash():
	is_dashing = true
	anim_dash.play("dash")
	
	velocity = direction * 300
	
	await anim_dash.animation_finished
	is_dashing = false

func attack():
	if not current_weapon:
		return
	is_attacking = true
	if is_dashing:
		current_weapon.anim_attack.play("attack_dash")
	elif not is_moving:
		current_weapon.anim_attack.play("attack_idle")
	else:
		current_weapon.anim_attack.play("attack_run")
		
	await current_weapon.anim_attack.animation_finished
	is_attacking = false
		
func send_data():
	var angle_255 = int(remap(arm_node.rotation_degrees, 0, 360, 0, 255))
	var msg = StreamPeerBuffer.new()
	msg.put_u8(2)
	
	msg.put_u8(angle_255)
	msg.put_8(roundi(direction.x))
	msg.put_8(roundi(direction.y))
	
	if is_dashing:
		msg.put_u8(1)
	else:
		msg.put_u8(0)
		
	msg.put_u8(weapon_change)
	
	if is_attacking:
		msg.put_u8(1)
		msg.put_u8(len(attack_buffer))
		msg.put_u8(server.tick_client)
		if len(attack_buffer):
			for i in attack_buffer:
				msg.put_u8(i)
	else:
		msg.put_u8(0)
		
		
	server.send(msg)
	attack_buffer.clear()
