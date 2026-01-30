use tokio::net::UdpSocket;
use serde_json::{json, Value};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc; // Каналы!
use tokio::time::{self, Duration};
use std::net::SocketAddr;

// Внутренние команды сервера (от сети к логике)
enum GameCommand {
    Connect { addr: SocketAddr, name: String },
    MoveRequest { addr: SocketAddr, target_x: f64, target_y: f64 },
}

struct Player {
    id: u32,
    name: String,
    pos: (f64, f64), // x, y
    last_processed_pos: (f64, f64), // Для валидации
    speed: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let socket = Arc::new(UdpSocket::bind("10.121.217.53:9001").await?);
    println!("UDP server listening on 9001");

    // Создаем канал: sender (tx) клонируем для сетевого цикла, receiver (rx) отдаем в тики
    let (tx, rx) = mpsc::channel::<GameCommand>(100);

    // Запускаем игровой цикл в отдельном таске
    let socket_clone = socket.clone();
    tokio::spawn(async move {
        game_tick_loop(rx, socket_clone).await;
    });

    // --- СЕТЕВОЙ ЦИКЛ (Только прием и парсинг) ---
    let mut buf = [0u8; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];

        // Парсим JSON (лучше вынести в отдельную функцию, чтобы не паниковать)
        if let Ok(json) = serde_json::from_slice::<Value>(data) {
            let req = json["req"].as_u64().unwrap_or(999);

            match req {
                0 => { // Join
                    if let Some(name) = json["name"].as_str() {
                        let _ = tx.send(GameCommand::Connect { 
                            addr, 
                            name: name.to_string() 
                        }).await;
                    }
                },
                2 => { // Move Request
                    let x = json["posx"].as_f64().unwrap_or(0.0);
                    let y = json["posy"].as_f64().unwrap_or(0.0);
                    let _ = tx.send(GameCommand::MoveRequest { 
                        addr, 
                        target_x: x, 
                        target_y: y 
                    }).await;
                },
                _ => {}
            }
        }
    }
}

// --- ИГРОВОЙ ЦИКЛ (Server Authoritative) ---
async fn game_tick_loop(mut rx: mpsc::Receiver<GameCommand>, socket: Arc<UdpSocket>) {
    // HashMap теперь живет ТОЛЬКО здесь. Мьютексы не нужны!
    let mut players: HashMap<SocketAddr, Player> = HashMap::new();
    let mut next_id = 1;
    
    // Тикрейт 32 раза в секунду (~31.25 мс)
    let tick_duration = Duration::from_millis(31); 
    let mut interval = time::interval(tick_duration);

    loop {
        interval.tick().await; // Ждем следующего тика// 1. ОБРАБОТКА ВСЕХ ВХОДЯЩИХ СОБЫТИЙ
        // Мы вычитываем всё, что накопилось в канале за время сна
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                GameCommand::Connect { addr, name } => {
                    if !players.contains_key(&addr) {
                        let player = Player {
                            id: next_id,
                            name: name.clone(),
                            pos: (50.0, 50.0),
                            last_processed_pos: (50.0, 50.0),
                            speed: 1000.0, // пикселей/единиц в секунду
                        };
                        
                        // Ответ клиенту (инит)
                        let resp = json!({ "data": { "id": next_id, "posx": 50.0, "posy": 50.0 }, "req": 0});
                        let _ = socket.send_to(resp.to_string().as_bytes(), addr).await;

                        players.insert(addr, player);
                        println!("Игрок {} (ID {}) подключился", name, next_id);
                        next_id += 1;


                        let snapshot: Vec<Value> = players.values().map(|p| {
                            json!({
                                "id": p.id,
                                "posx": p.pos.0,
                                "posy": p.pos.1
                            })
                        }).collect();

                        let broadcast_packet = json!({ "req": 1, "data": snapshot }).to_string();
                        let bytes = broadcast_packet.as_bytes();

                        for addr in players.keys() {
                            let _ = socket.send_to(bytes, *addr).await;
                        }
                        
                        // Тут можно сделать бродкаст о новом игроке всем остальным
                    }
                },
                GameCommand::MoveRequest { addr, target_x, target_y } => {
                    if let Some(player) = players.get_mut(&addr) {
                        // --- ВАЛИДАЦИЯ (Purple Box из схемы) ---
                        // Расчет дистанции
                        let dx = target_x - player.pos.0;
                        let dy = target_y - player.pos.1;
                        let dist = (dx*dx + dy*dy).sqrt();

                        // Максимально допустимая дистанция за время 1-го кадра (или времени с последнего апдейта)
                        // Допустим, клиент шлет апдейты часто, берем запас
                        let max_dist = player.speed * (0.034965 + 0.0015625); // 0.1с - примерный лаг + дельта

                        // println!("{}", max_dist);
                        // println!("{}", dist);


                        if dist <= max_dist {
                            // Valid move
                            player.pos = (target_x, target_y);
                            
                            // Ответ клиенту: Approve
                            println!("move aprove");
                            let resp = json!({"data": { "apr": true }, "req": 2});
                            let _ = socket.send_to(resp.to_string().as_bytes(), addr).await;
                            println!("player moved");
                        } else {
                            // Cheater or Lag! Teleport back
                            println!("Cheater detected or Lag! {} moved too fast", player.name);
                            let resp = json!({"data": {  
                                "apr": false, 
                                "fix_x": player.pos.0, 
                                "fix_y": player.pos.1 
                            }, "req": 2});
                            let _ = socket.send_to(resp.to_string().as_bytes(), addr).await;
                        }
                    }
                }
            }
        }

        // 2. РАССЫЛКА МИРА (SNAPSHOT)
        // В реальной игре мы не шлем полный JSON каждый тик (это дорого), 
        // но для примера соберем данные всех и разошлем.
        

        let snapshot: Vec<Value> = players.values().map(|p| {
            json!({
                "id": p.id,
                "posx": p.pos.0,
                "posy": p.pos.1
            })
        }).collect();

        let broadcast_packet = json!({ "req": 3, "data": snapshot }).to_string();
        let bytes = broadcast_packet.as_bytes();




        for addr in players.keys() {
            let _ = socket.send_to(bytes, *addr).await;
        }
    }
}