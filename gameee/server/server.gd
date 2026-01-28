extends Node

var ws := WebSocketPeer.new()
var data = ''
var n = 1
@onready var player: CharacterBody2D = $player
@onready var player_2: CharacterBody2D = $player2


func _ready():
	var err = ws.connect_to_url("ws://10.10.135.240:9001")
	if err != OK:
		print("Ошибка подключения:", err)
	else:
		print("Подключаемся...")

func _process(_delta):
	ws.poll()
	
	match ws.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			while ws.get_available_packet_count() > 0:
				var msg = ws.get_packet().get_string_from_utf8()
				data_process(msg)
				

		WebSocketPeer.STATE_CLOSED:
			print("Соединение закрыто")

func data_process(msg):
	msg = msg.split(' ')
	print(msg)
	if msg[0] == "1":
		player.move(float(msg[1]), float(msg[2]))
	if msg[0] == "2":
		player_2.move(float(msg[1]), float(msg[2]))
	

func send_message(p, x, y):
	match ws.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			var data = str(p) + ' ' + str(x) + ' ' + str(y)
			ws.send_text(data)
