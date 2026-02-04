extends Node

@onready var lobby: Node2D = $".."

var udp := PacketPeerUDP.new()
var server_address := "10.10.135.240"
var server_port := 9001

var packet_buffer = []

var tick_server = 0
var tick_client = 0
var tick_server_got = false

func _physics_process(delta: float) -> void:
	if tick_server_got:
		tick_client = wrap(tick_client+1, 0, 255)

func _ready():
	var err = udp.set_dest_address(server_address, server_port)
	if err == OK:
		print("Connected")

func _process(delta: float) -> void:
	while udp.get_available_packet_count() > 0:
		var packet = udp.get_packet()
		packet_buffer.append(packet)
	
	if packet_buffer:
		for packet in packet_buffer:
			data_process(packet)
		packet_buffer.clear()

func send(msg):
	var packet = msg.data_array
	udp.put_packet(packet)
		
func data_process(packet):
	var bytes = PackedByteArray(packet)
	var buffer = StreamPeerBuffer.new()
	buffer.data_array = bytes
	
	if not buffer.get_available_bytes():
		return
	
	var req = buffer.get_u8()
	if req == 0:
		tick_server = buffer.get_u8()
		tick_client = tick_server
		tick_server_got = true
		var id = buffer.get_u8()
		var hp = buffer.get_u32()
		var posx = buffer.get_float()
		var posy = buffer.get_float()
		lobby.spawn_self(id, hp, posx, posy)
	if req == 1:
		var id = buffer.get_u8()
		var hp = buffer.get_u32()
		var posx = buffer.get_float()
		var posy = buffer.get_float()
		var name_len = buffer.get_u8()
		var pname = buffer.get_utf8_string(name_len)
		lobby.spawn_entity(id, hp, pname, posx, posy)
	if req == 2:
		tick_server = buffer.get_u8()
		var tick_diff = tick_client - tick_server
		if abs(tick_diff) > 10:
			tick_client = tick_server
			
		var players_count = buffer.get_u8()
		for i in range(players_count):
			var id = buffer.get_u8()
			var hp = buffer.get_u32()
			var angle = buffer.get_u8()
			var posx = buffer.get_float()
			var posy = buffer.get_float()
			var is_attacking = buffer.get_u8()
			var is_dash = buffer.get_u8()
			var is_moving = buffer.get_u8()
			var weapon = buffer.get_u8()
			lobby.game_state_process(
				tick_server, id, hp, 
				angle, posx, posy, 
				is_attacking, is_dash, 
				is_moving, weapon
			)
