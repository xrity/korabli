extends Node

var ws := WebSocketPeer.new()

func _ready():
	var err = ws.connect_to_url("ws://127.0.0.1:9001")
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
				print("От сервера:", msg)

		WebSocketPeer.STATE_CLOSED:
			print("Соединение закрыто")

func send_message():
	var data = {
		"event": "hello",
		"data": "Привет из Godot!"
	}
	ws.send_text(JSON.stringify(data))
