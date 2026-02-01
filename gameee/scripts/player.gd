extends CharacterBody2D

@onready var server: Node = $"../server"
@onready var map: Node2D = $".."
@onready var anim: AnimationPlayer = $anim

@onready var namelabel: Label = $namelabel
@onready var hplabel: Label = $hplabel

@onready var arm: Sprite2D = $arm
@onready var attack_area: Area2D = $arm/attack_area
@onready var anim_arm: AnimationPlayer = $anim_arm
@onready var arm_collider: CollisionShape2D = $arm/attack_area/arm_collider


var targepos = Vector2(0, 0)


var speed = 100
var idp = null
var pname = null
var direction = Vector2(0, 0)
var dir_buffer = 'down'
var is_attacking = false
var hp = 100

var move_buffer = []
var attack_buffer = []


func _physics_process(delta: float) -> void:
	if idp == map.mainPlayerId:
		if direction:
			var angle_255 = int(remap(arm.rotation_degrees, 0, 360, 0, 255))
			var msg = StreamPeerBuffer.new()
			msg.put_u8(2)
			msg.put_32(position.x)
			msg.put_32(position.y)
			msg.put_u8(angle_255)
			
			server.send(msg)

func _process(delta: float) -> void:
	if idp == map.mainPlayerId:
		arm.look_at(get_global_mouse_position())
		arm.rotation_degrees = wrapf(arm.rotation_degrees, 0, 360)
		direction = Input.get_vector("a", "d", "w", "s")
		velocity = speed * direction
		move_and_slide()
		
		if Input.is_action_just_pressed("k") and not is_attacking:
			attack_process()
			
	else:
		move_interpolation()
	
	hplabel.text = str(hp)
	animation_process(direction.x, direction.y)
	

func _ready():
	pass

func update_arm_angle(new_arm_angle, indx):
	var new_arm_angle_360 = remap(new_arm_angle, 0, 255, 0, 360)
	
	if abs(arm.rotation_degrees - new_arm_angle_360) > 2.5:
		arm.rotation_degrees = lerpf(arm.rotation_degrees, new_arm_angle_360, indx)
	else:
		arm.rotation_degrees = new_arm_angle_360

func move_interpolation():
	if len(move_buffer) > 0:
		targepos = move_buffer[-1]
		move_buffer.clear()
		
	if abs(targepos.x - position.x) > 2.5:
		position.x = lerpf(position.x, targepos.x, 0.05)
	else:
		position.x = targepos.x
	
	if abs(targepos.y - position.y) > 2.5:
		position.y = lerpf(position.y, targepos.y, 0.05)
	else:
		position.y = targepos.y
		
	direction.x = targepos.x - position.x
	direction.y = targepos.y - position.y

func update_position(new_pos: Vector2):
	move_buffer.append(new_pos)

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

func attack_process():
	is_attacking = true
	arm_collider.disabled = false
	anim_arm.play("attack")
	
	await anim_arm.animation_finished
	arm_collider.disabled = true
	is_attacking = false


func _on_attack_area_area_entered(area: Area2D) -> void:
	if area.name == "hitbox_area" and is_attacking:
		if "idp" in area.get_parent() and area.get_parent().idp != self.idp:
			var msg = StreamPeerBuffer.new()
			var angle_255 = int(remap(arm.rotation_degrees, 0, 360, 0, 255))
			msg.put_u8(4)
			msg.put_u8(area.get_parent().idp)
			msg.put_u8(angle_255)
			
			server.send(msg)
	#elif is_attacking:
		#var msg = StreamPeerBuffer.new()
		#var angle_255 = int(remap(arm.rotation_degrees, 0, 360, 0, 255))
		#msg.put_u8(4)
		#msg.put_u8(angle_255)
