extends Node

"""
0 - join self
1 - join entity
2 - move self
3 - move entity
4 - disconnect

if data.req == 0:
		print("spawn me ", data.id)
		map.spawn_self(int(data.id), data.posx, data.posy)
	if data.req == 1:
		print("spawn ent ", data.id)
		map.spawn_entity(int(data.id), data.posx, data.posy)
	if data.req == 2:
		if data.apr:
			print("move me ", data.apr)
			map.move_self(data.apr)
		else:
			#speed * (0.03125 + 0.0015625)
			print("decline")
			map.move_self(data.apr, data.posx, data.posy)
	if data.req == 3:
		print("move ent ", data.posx, ' ', data.posy)
		map.move_entity(int(data.id), data.posx, data.posy)
		
		
"""
