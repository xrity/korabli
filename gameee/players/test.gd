extends CharacterBody2D

@onready var anim: AnimationPlayer = $anim
@onready var anim_dash: AnimationPlayer = $anim_dash

@onready var weapons_list = [
	preload("res://weapon/weapon_0.tscn"),
	preload("res://weapon/weapon_1.tscn")
]

@onready var arm_node: Node2D = $armNode
@onready var angle_node: Node2D = $angle_node


var speed = 100
var weapons = [0, 1]
var current_weapon = null
var weapon_change = 0

var weapon_indx = null

var direction = Vector2(0, 0)
var dir_keyboard = 0
var dir_mouse = 0

var is_attacking = false
var is_dashing = false
var is_moving = false


func _physics_process(delta: float) -> void:
	angle_node.look_at(get_global_mouse_position())
	var angle = angle_node.rotation
	angle = wrapf(angle, 0.0, TAU)
	var sectors = 8
	var sector_size = TAU / sectors
	dir_mouse = int((angle + sector_size / 2) / sector_size) % sectors
	arm_node.rotation_degrees = angle_node.rotation_degrees
	
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
		
	frames_process()
	move_and_slide()
	animation_process(direction.x, direction.y)
	
func animation_process(dirx, diry):
	if direction:
		anim.play("walk_" + str(direction_process(dirx,diry)))
	else:
		anim.play("idle_" + str(dir_keyboard))
		
func direction_process(dirx, diry):
	if dirx == 0 and diry == 0:
		return dir_keyboard

	var x = sign(dirx)
	var y = sign(diry)

	var key = Vector2i(x, y)

	var map = {
		Vector2i(1, 0): 0,
		Vector2i(1, 1): 1,
		Vector2i(0, 1): 2,
		Vector2i(-1, 1): 3,
		Vector2i(-1, 0): 4,
		Vector2i(-1, -1): 5,
		Vector2i(0, -1): 6,
		Vector2i(1, -1): 7
	}

	dir_keyboard = map.get(key, dir_keyboard)
	return dir_keyboard

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
		
func frames_process():
	var dir_body = circular_mid(dir_keyboard, dir_mouse)
	var dir_asdasf = circular_mid(dir_keyboard, dir_body)
	
	 
	
	$legf.frame_coords.y = dir_keyboard
	$legfb.frame_coords.y = dir_keyboard
	$bodyu.frame_coords.y = dir_keyboard
	$body.frame_coords.y = dir_asdasf
	$head.frame_coords.y = dir_body

func circular_mid(a: int, b: int) -> int:
	var d := (b - a) % 8
	if d < 0:
		d += 8

	if d > 4:
		d -= 8

	var m := a + int(round(d / 2.0))
	m = m % 8
	if m < 0:
		m += 8

	return m
