extends CharacterBody2D

var speed = 1000

@onready var node_2d: Node2D = $".."

func _physics_process(_delta: float) -> void:
	if node_2d.n == 1:
		var direction := Input.get_vector("a", "d", "w", "s")
		if direction:
			node_2d.send_message(node_2d.n, direction[0], direction[1])
	
func move(x, y):
	velocity = Vector2(x * speed, y * speed)
	move_and_slide()
