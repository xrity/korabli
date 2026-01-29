extends CharacterBody2D

@onready var server: Node = $"../server"
@onready var map: Node2D = $".."

var speed = 100
var idp = null

func _physics_process(delta: float) -> void:
	if idp == map.mainPlayerId:
		var direction := Input.get_vector("a", "d", "w", "s")
		var data = {
			"req": 1,
			"id": idp,
			"dirx": direction[0],
			"diry": direction[1]
		}
		#print(data)
		#server.send(data)
		velocity = speed * direction
		move_and_slide()
	
