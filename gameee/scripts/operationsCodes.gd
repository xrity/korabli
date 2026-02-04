extends Node

"""
0 - join
sent
	name count u8
	name chars u8
	
get 
	tick u8
	id u8
	hp u64

1 - join player
get
	id u8
	hp u64
	name count u8
	name chars u8

2 - every tick
sent
	angle u8
	dirx i32
	diry i32
	is_attack u8
		if true
			count u8
			ids u8
	is_dodge u8
	current_weapon u8

get
	tick u8
	players count u8
	id u8
	hp u16
	angle u8
	posx i32
	posy i32	
	is_attack u8
	is_dodge u8
	weapon u8
	
	
"""
