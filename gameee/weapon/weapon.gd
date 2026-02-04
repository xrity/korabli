extends Node2D

@onready var anim_attack: AnimationPlayer = $anim_attack
@onready var hit_box: Area2D = $hit_box


func _on_hit_box_area_entered(area: Area2D) -> void:
	var player = get_parent().get_parent()
	if area.name == "hitbox_area" and player.is_attacking:
		if "idp" in area.get_parent() and area.get_parent().idp != player.idp:
			if not area.get_parent().idp in player.attack_buffer:
				player.attack_buffer.append(area.get_parent().idp)
