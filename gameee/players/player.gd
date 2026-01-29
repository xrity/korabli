extends CharacterBody2D

@onready var server: Node = $"../server"
@onready var map: Node2D = $".."

var speed = 100
var idp = null
var direction = Vector2(0, 0)

func _physics_process(delta: float) -> void:
	if idp == map.mainPlayerId:
		if direction:
			var data = {
				"req": 2,
				"id": idp,
				"posx": position.x,
				"posy": position.y
			}
			server.send(data)
			
func _process(delta: float) -> void:
	if idp == map.mainPlayerId:
		direction = Input.get_vector("a", "d", "w", "s")
		velocity = speed * direction
		move_and_slide()
	
		
	
