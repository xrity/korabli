extends CharacterBody2D

@onready var server: Node = $"../server"
@onready var map: Node2D = $".."
@onready var anim: AnimationPlayer = $anim
@onready var attack_collider: CollisionShape2D = $attackCollider
@onready var namelabel: Label = $namelabel


var speed = 100
var idp = null
var pname = null
var direction = Vector2(0, 0)
var dir_buffer = 'down'
var is_attacking = false



func _physics_process(delta: float) -> void:
	if idp == map.mainPlayerId:
		if direction:
			var msg = StreamPeerBuffer.new()
			msg.put_u8(2)
			msg.put_32(position.x)
			msg.put_32(position.y)
			
			server.send(msg)

			
func _process(delta: float) -> void:
	if idp == map.mainPlayerId:
		direction = Input.get_vector("a", "d", "w", "s")
		velocity = speed * direction
		move_and_slide()
		
		if Input.is_action_just_pressed("k") and not is_attacking:
			attack_process(direction)
		
	animation_process(direction.x, direction.y)
	
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
		
func attack_process(dir):
	attack_collider.disabled = false
