use std::time::Instant;
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
    MoveRequest { addr: SocketAddr, target_x: i32, target_y: i32, direction: u8 },
    AttackRequest { addr: SocketAddr, target: u8, direction: u8,},
}

struct Player {
    id: u8,
    name: String,
    health: i64,
    pos: (i32, i32), // (x, y)
    speed: f64,
    direction: u8,
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

        match req {
            0 => { // Join request
                let name = String::from_utf8_lossy(&data[1..len]).to_string();
                    let _ = tx.try_send(GameCommand::Connect { 
                        addr,
                        name: name.to_string() 
                    });
                }
            2 => { // Move Request
                let x = &data[1..5];
                let x = i32::from_le_bytes(x.try_into().expect("error x"));
                let y = &data[5..9];
                let y = i32::from_le_bytes(y.try_into().expect("error y"));
                let direction: u8 = data[9];
                println!("{direction}");
                let _ = tx.try_send(GameCommand::MoveRequest { 
                    addr, 
                    target_x: x, 
                    target_y: y,
                    direction: direction, 
                });
            },
            4 => { //Attack request
                let target: u8 = data[1];
                let direction: u8 = data[2];

                let _ = tx.try_send(GameCommand::AttackRequest { 
                    addr,
                    target, 
                    direction, 
                });
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
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        let mut attack_request_buffer: Vec<(u8, Option<u8>, u8)> = Vec::new();

        interval.tick().await;
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                GameCommand::Connect { addr, name } => {
                    if !players.contains_key(&addr) {
                        let player = Player {
                            id: next_id,
                            health: 100,
                            name: name.clone(),
                            pos: (50, 50),
                            speed: 1000.0,
                            direction: 0,
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
                GameCommand::MoveRequest { addr, target_x, target_y , direction} => {
                    if let Some(player) = players.get_mut(&addr) {
                        let dx = target_x - player.pos.0;
                        let dy = target_y - player.pos.1;
                        let dist = ((dx*dx + dy*dy) as f64).sqrt();

                        let max_dist = player.speed * (0.034965 + 0.0015625);
                        player.direction = direction;


                        if dist <= max_dist {
                            // Valid move
                            player.pos = (target_x, target_y);
                            
                            let socket_clone = socket.clone();
                            tokio::spawn(async move {
                                let mut resp = vec![];
                                resp.push(2u8);
                                resp.push(1u8);
                                socket_clone.send_to(&resp, addr).await.unwrap();
                            });
                        } else {
                            println!("Cheater detected or Lag! {} moved too fast", player.name);
                            // let _ = socket.send_to(resp.to_string().as_bytes(), addr).await;
                        }
                    }
                },
                GameCommand::AttackRequest { addr, target, direction } => {
                    let mut dist: f64 = 0.0;
                    for player in players.values(){
                        if player.id == target{
                            let dx = players.get(&addr).unwrap().pos.0 - player.pos.0;
                            let dy = players.get(&addr).unwrap().pos.1 - player.pos.1;
                            attack_request_buffer.push((players.get(&addr).unwrap().id, Some((player.id)), direction));
                            dist = ((dx * dx + dy * dy) as f64).sqrt();
                        }
                    }

                    println!("Attack requested, distance {}", dist);
                }
            }
        }

        send_new_positions(&players, socket.clone());
        if attack_request_buffer.len() > 0{
            send_all_attacks(&players, socket.clone(), attack_request_buffer);  
        }
    }
}

fn send_all_attacks(players: &HashMap<SocketAddr, Player>, socket: Arc<UdpSocket>, attack_request_buffer : Vec<(u8, Option<u8>, u8)>){
    let mut data: Vec<u8> = Vec::new();
    data.push(4u8);
    data.push(attack_request_buffer.len() as u8);
    for i in attack_request_buffer{
        match i.1 {
            Some(player) => {
                data.push(i.0);
                data.push(player);
                let mut hp = 0;
                for j in players.values(){
                    if j.id == player{
                        hp = j.health
                    }
                }
                data.push(i.2);
                data.extend_from_slice(&hp.to_le_bytes());
            }
            None => {
                data.push(i.0);
                data.push(0u8);
                data.push(i.2);
                data.extend_from_slice(&0i32.to_le_bytes());
            }
        }
    }

    let addrs: Vec<SocketAddr> = players.keys().cloned().collect();

    tokio::spawn(async move {
        for addr in addrs{
            let _ = socket.send_to(&data, addr).await;
        }
    });
}

fn send_new_positions(players: &HashMap<SocketAddr, Player>, socket: Arc<UdpSocket>) {
    if players.is_empty() { return; }

    let mut data = Vec::with_capacity(6 + (players.len() * 9));
    data.push(3u8);
    data.push(players.len() as u8);

    for player in players.values() {
        data.push(player.id);
        data.extend_from_slice(&player.pos.0.to_le_bytes());
        data.extend_from_slice(&player.pos.1.to_le_bytes());
        data.push(player.direction);
    }

    let shared_data = Arc::new(data);
    let addrs: Vec<SocketAddr> = players.keys().cloned().collect();

    tokio::spawn(async move {
        for addr in addrs {
            let _ = socket.send_to(&shared_data, addr).await;
        }
    });
}
