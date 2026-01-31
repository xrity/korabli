extends Node

@onready var map: Node2D = $".."
var udp := PacketPeerUDP.new()
var server_address := "10.10.135.240"
var server_port := 9001
var is_connected = false
var move_buffer = {}

func _ready():
	var err = udp.set_dest_address(server_address, server_port)
	if err == OK:
		print("UDP готов к работе")
		is_connected = true

func _process(delta: float) -> void:
	while udp.get_available_packet_count() > 0:
		var packet = udp.get_packet()
		data_process(packet)
	
	for i in move_buffer.keys():
		map.move_entity(i, move_buffer[i][0], move_buffer[i][1])

func _physics_process(delta: float) -> void:
	pass

func send(msg):
	var packet = msg.data_array
	udp.put_packet(packet)
		
func data_process(packet):
	var bytes = PackedByteArray(packet)
	var buffer = StreamPeerBuffer.new()
	buffer.data_array = bytes
	
	var req = buffer.get_u8()
	if req == 0:
		var id = buffer.get_u8()
		var posx = buffer.get_32()
		var posy = buffer.get_32()
		map.spawn_self(id, posx, posy)
	if req == 1:
		var id = buffer.get_u8()
		var posx = buffer.get_32()
		var posy = buffer.get_32()
		var name_len = buffer.get_u8()
		var pname = buffer.get_utf8_string(name_len)
		move_buffer[id] = [posx, posy]
		map.spawn_entity(id, posx, posy, pname)
		
	if req == 2:
		var apr = buffer.get_u8()
		if !apr:
			var posx = buffer.get_32()
			var posy = buffer.get_32()
			map.move_self(posx, posy)
	if req == 3:
		var players_count = buffer.get_u8()
		for i in players_count:
			var id = buffer.get_u8()
			var posx = buffer.get_32()
			var posy = buffer.get_32()
			move_buffer[id] = [posx, posy]
	
		
