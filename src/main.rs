use tokio::net::UdpSocket;
use serde_json::Value;
use serde_json::json;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};


#[derive(Debug)]
struct Player {
    id: u32,
    name: String,
    posx: f64,
    posy: f64,
    speed: f64,
}

type Players = Arc<Mutex<HashMap<std::net::SocketAddr, Player>>>;


#[tokio::main]
async fn main() -> Result<()> {
    let socket = UdpSocket::bind("158.195.205.45:9001").await?;
    println!("UDP server listening on 9001");

    let players: Players = Arc::new(Mutex::new(HashMap::new()));
    let player_id_counter = Arc::new(AtomicU32::new(1));

    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        let text = String::from_utf8_lossy(data);

        match serde_json::from_str::<Value>(&text) {
            Ok(json) => {
                println!("Received from {}: {}", addr, json);
                let req: u64 = json["req"].as_u64().unwrap();

                match req {
                    0 => {
                        add_player( json["name"].as_str().unwrap(), addr, &players, &player_id_counter, &socket).await;
                        println!("{:?}", players);
                    },
                    2 => {
                        move_player(&players, addr, json, &socket).await;
                    }
                    _ => println!("Unexpected request"),
                }


            }
            Err(e) => eprintln!("Invalid JSON from {}: {}", addr, e),
        }
    }
}

async fn add_player(name: &str, addr: std::net::SocketAddr, players: &Players, player_id_counter: &Arc<AtomicU32>, socket: &UdpSocket){
    let mut map = players.lock().unwrap();
    if !map.contains_key(&addr){
        let id = player_id_counter.fetch_add(1, Ordering::SeqCst);
        let player = Player {
        id,
        name: name.to_string(),
        posx: 50.0,
        posy: 50.0,
        speed: 100.0,
        };

        
        
        let addrs: Vec<std::net::SocketAddr> = {
            map.keys().cloned().collect()
        };

        for ad in &addrs {
            let data = json!({
                "req" : 1,
                "id" : map.get(&ad).map(|p| p.id),
                "posx" : map.get(&ad).map(|p| p.posx),
                "posy" : map.get(&ad).map(|p| p.posy),
            });

            let msg_str = data.to_string();
            let msg_bytes = msg_str.as_bytes();

            if let Err(e) = socket.send_to(msg_bytes, addr).await {
                eprintln!("Не удалось отправить игроку {}: {}", ad, e);
            }
        }

        map.insert(addr, player);
        
        let id = map.get(&addr).map(|p| p.id);
        let posx = map.get(&addr).map(|p| p.posx);
        let posy = map.get(&addr).map(|p| p.posy);

        let data = json!({
            "req" : 0,
            "id" : id,
            "posx" : posx,
            "posy" : posy 
        });

        println!("{:?}", data);
        socket.send_to(data.to_string().as_bytes(), addr).await.unwrap();

        let data = json!({
            "req" : 1,
            "id" : id,
            "posx" : posx,
            "posy" : posy,
        });

        let msg_str = data.to_string();
        let msg_bytes = msg_str.as_bytes();

        // Захватываем мьютекс только для того, чтобы собрать адреса
        

        // Проходим по всем адресам и отправляем данные
        for ad in addrs {
            if let Err(e) = socket.send_to(msg_bytes, ad).await {
                eprintln!("Не удалось отправить игроку {}: {}", ad, e);
            }
        }
    }
    

}

async fn move_player(players: &Players, addr: std::net::SocketAddr, json: Value, socket: &UdpSocket){
    let mut map = players.lock().unwrap();
    let data = json!({
            "req" : 2,
            "apr" : true,
        });

    println!("{:?}", data);
    
    if let Some(player) = map.get_mut(&addr) {
        player.posx = json["posx"].as_f64().unwrap_or(0.0);
        player.posy = json["posy"].as_f64().unwrap_or(0.0);
        println!("Координаты игрока {} обновлены у игрока!", player.name);
    }

    socket.send_to(data.to_string().as_bytes(), addr).await.unwrap();

    let addrs: Vec<std::net::SocketAddr> = {
        map.keys().filter(|&&p| p != addr).cloned().collect()
    };

    for ad in &addrs {
        let data = json!({
            "req" : 3,
            "id" : map.get(&addr).map(|p| p.id),
            "posx" : map.get(&addr).map(|p| p.posx),
            "posy" : map.get(&addr).map(|p| p.posy),
        });

        let msg_str = data.to_string();
        let msg_bytes = msg_str.as_bytes();

        if let Err(e) = socket.send_to(msg_bytes, ad).await {
            eprintln!("Не удалось отправить игроку {}: {}", ad, e);
        }
        println!("Координаты игрока обновлены у всех игроков!");
    }
}

async fn broadcast(socket: &UdpSocket, players: &Players, message: &serde_json::Value) {
    let msg_str = message.to_string();
    let msg_bytes = msg_str.as_bytes();

    // Захватываем мьютекс только для того, чтобы собрать адреса
    let addrs: Vec<std::net::SocketAddr> = {
        let map = players.lock().unwrap();
        map.keys().cloned().collect()
    };

    // Проходим по всем адресам и отправляем данные
    for addr in addrs {
        if let Err(e) = socket.send_to(msg_bytes, addr).await {
            eprintln!("Не удалось отправить игроку {}: {}", addr, e);
        }
    }
}