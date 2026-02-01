use tokio::net::UdpSocket;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc; // Каналы!
use tokio::time::{self, Duration};
use std::net::SocketAddr;

// Внутренние команды сервера (от сети к логике)
enum GameCommand {
    Connect { addr: SocketAddr, name: String },
    MoveRequest { addr: SocketAddr, target_x: i32, target_y: i32 },
}

struct Player {
    id: u8,
    name: String,
    pos: (i32, i32), // (x, y)
    speed: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:9001").await?);
    println!("UDP server listening on 9001");

    // making channel to comunicate between network cycle and game cycle
    let (tx, rx) = mpsc::channel::<GameCommand>(100);

    // starting another process for game cycle
    let socket_clone = socket.clone();
    tokio::spawn(async move {
        game_tick_loop(rx, socket_clone).await;
    });

    // --- network cycle only data recieve and parse ---
    let mut buf = [0u8; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        let req = data[0];
        
        println!("{data:?}");

        match req {
            0 => { // Join request
                let name = String::from_utf8_lossy(&data[1..len]).to_string();
                    let _ = tx.send(GameCommand::Connect { 
                        addr,
                        name: name.to_string() 
                    }).await;
                }
            2 => { // Move Request
                let x = &data[1..5];
                let x = i32::from_le_bytes(x.try_into().expect("error x"));
                let y = &data[5..9];
                let y = i32::from_le_bytes(y.try_into().expect("error y"));
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

// --- Tick cycle (game cycle) ---
async fn game_tick_loop(mut rx: mpsc::Receiver<GameCommand>, socket: Arc<UdpSocket>) {
    let mut players: HashMap<SocketAddr, Player> = HashMap::new();
    let mut next_id = 1;
    
    // tickrate 32 per sec (~31.25 ms)
    let tick_duration = Duration::from_millis(31); 
    let mut interval = time::interval(tick_duration);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);

    loop {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                GameCommand::Connect { addr, name } => {
                    if !players.contains_key(&addr) {
                        let player = Player {
                            id: next_id,
                            name: name.clone(),
                            pos: (50, 50),
                            speed: 1000.0,
                        };
                        println!("player connected: {}", player.name);
                        
                        // response to connect
                        let mut resp = Vec::new();

                        resp.push(0u8);
                        resp.extend_from_slice(&next_id.to_le_bytes());
                        resp.extend_from_slice(&player.pos.0.to_le_bytes());
                        resp.extend_from_slice(&player.pos.1.to_le_bytes());

                        socket.send_to(&resp, addr).await.unwrap();

                        players.insert(addr, player);
                        println!("Игрок {} (ID {}) подключился", name, next_id);
                        next_id += 1;

                        resp[0] = 1;
                        resp.push(players.get(&addr).unwrap().name.len() as u8);
                        resp.extend_from_slice(&players.get(&addr).unwrap().name.as_bytes());
                        
                
                        let keys: Vec<SocketAddr> = players.keys().filter(|&&p| p != addr).cloned().collect();

                        for i in keys{
                            socket.send_to(&resp, &i).await.unwrap();
                            let mut resp1: Vec<u8> = vec![1u8];

                            let p = players.get(&i).unwrap();

                            resp1.push(p.id);
                            let x = p.pos.0.to_le_bytes();
                            let y = p.pos.1.to_le_bytes();
                            resp1.extend_from_slice(&x);
                            resp1.extend_from_slice(&y);

                            let name  = &p.name;
                            resp1.push(name.len() as u8);
                            resp1.extend_from_slice(name.as_bytes());

                            socket.send_to(&resp1, addr).await.unwrap();
                            
                            
                        }

                        
                    }
                },
                GameCommand::MoveRequest { addr, target_x, target_y } => {
                    if let Some(player) = players.get_mut(&addr) {
                        let dx = target_x - player.pos.0;
                        let dy = target_y - player.pos.1;
                        let dist = ((dx*dx + dy*dy) as f64).sqrt();

                        let max_dist = player.speed * (0.034965 + 0.0015625);


                        if dist <= max_dist {
                            // Valid move
                            player.pos = (target_x, target_y);
                            
                            // movement aprove send
                            let mut resp = vec![];
                            resp.push(2u8);
                            resp.push(1u8);
                            socket.send_to(&resp, addr).await.unwrap();
                            
                        } else {
                            println!("Cheater detected or Lag! {} moved too fast", player.name);
                            // let _ = socket.send_to(resp.to_string().as_bytes(), addr).await;
                        }
                    }
                }
            }
        }

        send_new_positions(&players, &socket).await;
        interval.tick().await;
    }
}


async fn send_new_positions(players: &HashMap<SocketAddr ,Player>, socket: &Arc<UdpSocket>){
    let mut data = Vec::with_capacity(2 + (players.keys().len() * 8));
    data.push(3u8);
    data.push(players.len() as u8);

    for player in players.values(){
        data.extend_from_slice(&player.id.to_le_bytes());
        data.extend_from_slice(&player.pos.0.to_le_bytes());
        data.extend_from_slice(&player.pos.1.to_le_bytes());
    }
    let addrs: Vec<SocketAddr> = players.keys().cloned().collect();
    let socket_clone = socket.clone();
    let shared_data = Arc::new(data);
    tokio::spawn(async move {
        for addr in addrs{
            let s = socket_clone.clone();
            let d = shared_data.clone();

            tokio::spawn(async move {
                s.send_to(&d, addr).await.unwrap();
            });
        }
    });
    
}
