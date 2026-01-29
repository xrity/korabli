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

func _process(_delta):
	if udp.get_available_packet_count() > 0:
		var packet = udp.get_packet()
		var msg_string = packet.get_string_from_utf8()
		
		var json = JSON.new()
		var error = json.parse(msg_string)
		if error == OK:
			var data = json.get_data()
			data_process(data)
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
		
func data_process(data):
	print(data)
	if data.req == 0:
		map.spawn_me(int(data.id), data.posx, data.posy)
	if data.req == 1:
		pass
	if data.req == 2:
		map.spawn_entity(int(data.id), data.posx, data.posy)
		
