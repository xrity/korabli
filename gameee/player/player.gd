extends CharacterBody2D

var speed = 1000

func _physics_process(_delta: float) -> void:
	var direction := Input.get_vector("a", "d", "w", "s")
	
	velocity = direction * speed
	
	move_and_slide()
