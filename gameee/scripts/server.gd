extends Node

@onready var map: Node2D = $".."
var udp := PacketPeerUDP.new()
var server_address := "10.10.135.240"
var server_port := 9001
var is_connected = false

func _ready():
	var err = udp.set_dest_address(server_address, server_port)
	if err == OK:
		print("UDP готов к работе")
		is_connected = true

func _physics_process(delta: float) -> void:
	if udp.get_available_packet_count() > 0:
		var packet = udp.get_packet()
		var msg_string = packet.get_string_from_utf8()
		var json = JSON.new()
		var error = json.parse(msg_string)
		if error == OK:
			var msg = json.get_data()
			data_process(msg)
		else:
			print("Ошибка парсинга JSON: ", msg_string)

func send(data):
	var json_string = JSON.stringify(data)
	var packet_data = json_string.to_utf8_buffer()
	
	var err = udp.put_packet(packet_data)
	if err != OK:
		print("Ошибка отправки пакета: ", err)
	else:
		print("send: ", json_string)
		
func data_process(msg):
	print("req ", msg.req)
	print("data ", msg.data)
	print()
	
	if msg.req == 0:
		map.spawn_self(int(msg.data.id), msg.data.posx, msg.data.posy)
	if msg.req == 1:
		for dp in msg.data:
			map.spawn_entity(int(dp.id), dp.posx, dp.posy)
	if msg.req == 2:
		if msg.data.apr:
			map.move_self(msg.data.apr)
		else:
			#speed * (0.03125 + 0.0015625)
			map.move_self(msg.data.apr, msg.data.fix_x, msg.data.fix_y)
	if msg.req == 3:
		for dp in msg.data:
			map.move_entity(int(dp.id), dp.posx, dp.posy)
	
		
