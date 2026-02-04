use std::time::Instant;
use glam::Vec2;
use tokio::net::UdpSocket;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc; // Каналы!
use tokio::time::{self, Duration};
use std::net::SocketAddr;
use std::collections::VecDeque;

// Внутренние команды сервера (от сети к логике)
enum GameCommand {
    Connect { addr: SocketAddr, name: String },
    TickRequest { addr: SocketAddr, x_direction: i8, y_direction: i8, direction: u8, is_attacking: bool, is_dodging: bool, current_weapon: u8, count: u8, ids: Option<Vec<u8>>, client_tick: u8 },
}

struct Wall {
    a: Vec2,
    b: Vec2,
}

impl Wall {
    // Находит ближайшую точку на отрезке к данной точке P
    fn closest_point(&self, p: Vec2) -> Vec2 {
        let ab = self.b - self.a;
        let t = ((p - self.a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
        self.a + ab * t
    }
}

struct PositionSnapshot {
    tick: u8,
    pos: (f32, f32),
}

struct Player {
    id: u8,
    name: String,
    health: u32,
    pos: (f32, f32), // (x, y)
    speed: u16,
    direction: u8,
    weapon: u8,
    gun: u8,
    current_weapon: u8,
    is_attacking: u8,
    is_dodging: u8,
    can_move: bool,
    is_moving: u8,
    history: VecDeque<PositionSnapshot>,
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
            2 => { // Tick request
                let direction = data[1];
                let x_direction = data[2] as i8;
                let y_direction = data[3] as i8;
                let is_dodging: bool = (0 != data[4]);
                let current_weapon = data[5];
                let is_attacking: bool = (0 != data[6]);
                let mut count: u8 = 0;
                let mut client_tick: u8 = 0;
                if is_attacking{
                    count = data[7];
                    client_tick = data[8];
                }
                match is_attacking {
                    true => {
                        let mut ids: Vec<u8> = Vec::with_capacity(count.try_into().unwrap());
                        for i in 9..(count+9){
                            ids.push(data[i as usize]);
                        }
                        let _ = tx.try_send(GameCommand::TickRequest { 
                            addr, 
                            x_direction,
                            y_direction,
                            direction,
                            is_attacking,
                            is_dodging,
                            current_weapon,
                            count,
                            ids: Some(ids), 
                            client_tick,
                        });

                    },
                    false => {
                        let _ = tx.try_send(GameCommand::TickRequest { 
                            addr, 
                            x_direction,
                            y_direction,
                            direction,
                            is_attacking,
                            is_dodging,
                            current_weapon,
                            count,
                            ids: None, 
                            client_tick,
                        });
                    },
                }
               
            },
            _ => {}  
        }
}
}

// --- Tick cycle (game cycle) ---

async fn game_tick_loop(mut rx: mpsc::Receiver<GameCommand>, socket: Arc<UdpSocket>) {
    let mut players: HashMap<SocketAddr, Player> = HashMap::new();
    let mut next_id = 1;
    let mut tick_counter: u8 = 0;

    let walls = vec![
        // Wall { a: Vec2::new(0.0, 0.0), b: Vec2::new(1000.0, 0.0) },    // Верхняя стена
        // Wall { a: Vec2::new(0.0, 0.0), b: Vec2::new(0.0, 1000.0) },    // Левая стена
        Wall { a: Vec2::new(0.0, 0.0), b: Vec2::new(100.0, 0.0) }, // Наклонная преграда
    ];
    
    // tickrate 32 per sec (~31.25 ms)
    let tick_duration = Duration::from_millis(31); 
    let mut interval = time::interval(tick_duration);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    const DELTA: f32 = 0.03125;

    loop {

        interval.tick().await;
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                GameCommand::Connect { addr, name } => {
                    if !players.contains_key(&addr) {
                        let player = Player {
                            id: next_id,
                            health: 100,
                            name: name.clone(),
                            pos: (50.0, 50.0),
                            speed: 100,
                            direction: 0,
                            gun: 0,
                            weapon: 0,
                            current_weapon: 1,
                            is_attacking: 0,                            
                            is_dodging: 0,
                            is_moving: 0,
                            can_move: true,
                            history: VecDeque::new(),
                        };                        
                        // response to connect
                        let mut resp = Vec::new();

                        resp.push(0u8);
                        resp.push(tick_counter);
                        resp.extend_from_slice(&next_id.to_le_bytes());
                        resp.extend_from_slice(&player.health.to_le_bytes());
                        resp.extend_from_slice(&player.pos.0.to_le_bytes());
                        resp.extend_from_slice(&player.pos.1.to_le_bytes());

                        socket.send_to(&resp, addr).await.unwrap();
                        resp.remove(1);

                        players.insert(addr, player);
                        println!("Player {} (ID {}) connected", name, next_id);
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
                            resp1.extend_from_slice(&p.health.to_le_bytes());
                            resp1.extend_from_slice(&p.pos.0.to_le_bytes());
                            resp1.extend_from_slice(&p.pos.1.to_le_bytes());
                            let name  = &p.name;
                            resp1.push(name.len() as u8);
                            resp1.extend_from_slice(name.as_bytes());


                            socket.send_to(&resp1, addr).await.unwrap();
                        }
                    }
                },
                GameCommand::TickRequest { addr, x_direction, y_direction, direction, is_attacking, is_dodging, current_weapon, count, ids, client_tick} => {

                    let (attacker_pos, attacker_id, attacker_dir) = {
                        if let Some(player) = players.get_mut(&addr) {
                            player.current_weapon = current_weapon;

                            let move_direction = Vec2::new(x_direction as f32, y_direction as f32).normalize_or_zero();

                            if move_direction.length_squared() > 0.0{
                                player.is_moving = 1;
                            } else {
                                player.is_moving = 0;
                            }
                            
                            // 1. Конвертируем текущую позицию в Vec2
                            let current_pos = Vec2::new(player.pos.0, player.pos.1);
                            
                            // 2. Рассчитываем желаемую скорость (Velocity)
                            let velocity = move_direction * player.speed as f32 * DELTA;

                            // 3. Вызываем Move and Slide
                            // Радиус 16.0 подойдет, если спрайт игрока примерно 32x32
                            let new_pos = move_and_slide(current_pos, velocity, &walls, 16.0);

                            // 4. Сохраняем результат обратно в структуру
                            player.pos = (new_pos.x, new_pos.y);
                            
                            // Обновляем остальные данные
                            player.direction = direction;
                            player.is_attacking = is_attacking as u8;
                            player.is_dodging = is_dodging as u8;
                            player.current_weapon = current_weapon;

                            (new_pos, player.id, player.direction)
                        } else{
                            return
                        }
                    };
                    // if is_attacking{
                    //     if ids.as_ref().unwrap().len() >= 1{
                    //         let player = players.get(&addr).unwrap();
                    //         for victim in players.values_mut(){
                    //             if ids.as_ref().unwrap().contains(&victim.id){
                    //                 let victims_past_pos = victim.history.iter()
                    //                     .find(|s| s.tick == client_tick)
                    //                     .map(|s| s.pos)
                    //                     .unwrap_or(victim.pos);

                    //                 if check_collision(Vec2::new(player.pos.0, player.pos.1), Vec2::new(victims_past_pos.0, victims_past_pos.1), 100.0 , 16.0, player.direction){
                    //                     println!("player {}, got hit by {}", victim.id, player.id);
                    //                     victim.health -= 100;
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    if is_attacking {
                        if let Some(target_ids) = ids.as_ref() {
                            // Теперь мы можем свободно использовать values_mut(), 
                            // так как предыдущая ссылка на игрока уже удалена
                            for victim in players.values_mut() {
                                if victim.id == attacker_id { continue; }

                                if target_ids.contains(&victim.id) {
                                    let v_past_pos = victim.history.iter()
                                        .find(|s| s.tick == client_tick)
                                        .map(|s| Vec2::new(s.pos.0, s.pos.1))
                                        .unwrap_or(Vec2::new(victim.pos.0, victim.pos.1));

                                    if check_collision(attacker_pos, v_past_pos, 50.0, 16.0, attacker_dir) {
                                        // Безопасное вычитание HP
                                        victim.health = victim.health.saturating_sub(20);
                                        println!("Игрок {} получил урон! HP: {}", victim.id, victim.health);
                                    }
                                }
                            }
                        }
                    }

                    // if let Some(player) = players.get_mut(&addr) {
                    //     player.pos.0 += move_direction.x * player.speed as f32 * 0.03125;
                    //     player.pos.1 += move_direction.y * player.speed as f32 * 0.03125;
                    //     player.direction = direction;
                    //     player.is_attacking = is_attacking as u8;
                    //     player.is_dodging = is_dodging as u8;
                    //     player.current_weapon = current_weapon;
                    // }
                },
            }
        }

        tick_counter = tick_counter.wrapping_add(1);
        send_game_state(&players, socket.clone(), tick_counter);
        for player in players.values_mut() {
            player.history.push_back(PositionSnapshot {
                tick: tick_counter,
                pos: player.pos,
            });
            
            // Храним последние 20-30 тиков (около 1 секунды)
            if player.history.len() > 32 {
                player.history.pop_front();
            }
        }
    }
}

fn send_game_state(players: &HashMap<SocketAddr, Player>, socket: Arc<UdpSocket>, tick_counter: u8){
    if players.is_empty() { return };

    let mut data: Vec<u8> = Vec::with_capacity(3 + players.len() * 20);

    data.push(2u8);
    data.push(tick_counter);
    data.push(players.len() as u8);

    for player in players.values(){
        data.push(player.id);
        data.extend_from_slice(&player.health.to_le_bytes());
        data.push(player.direction);
        data.extend_from_slice(&player.pos.0.to_le_bytes());
        data.extend_from_slice(&player.pos.1.to_le_bytes());
        data.push(player.is_attacking);
        data.push(player.is_dodging);
        data.push(player.is_moving);
        if player.current_weapon == 0{
            data.push(player.weapon);
        }
        else {
            data.push(player.gun);
        }
        
    }

    let data = Arc::new(data);
    let addrs: Vec<SocketAddr> = players.keys().cloned().collect();

    tokio::spawn(async move {
        for addr in addrs{
            let _ = socket.send_to(&data, addr).await;
        }
    });
}

fn move_and_slide(mut pos: Vec2, mut velocity: Vec2, walls: &[Wall], radius: f32) -> Vec2 {
    let mut time_left = 1.0; // Доля времени тика
    
    // Делаем до 4-х попыток скольжения (на случай углов)
    for _ in 0..4 {
        if velocity.length_squared() < 0.001 { break; }
        
        let target = pos + velocity * time_left;
        let mut collision: Option<(Vec2, Vec2)> = None; // (точка коллизии, нормаль)

        // Ищем ближайшее столкновение
        for wall in walls {
            let cp = wall.closest_point(target);
            let dist_vec = target - cp;
            let dist = dist_vec.length();

            if dist < radius {
                let normal = dist_vec.normalize_or_zero();
                collision = Some((cp, normal));
                break; 
            }
        }

        if let Some((_cp, normal)) = collision {
            // 1. Выталкиваем из стены (как в Godot)
            let overlap = radius - (target - _cp).length();
            pos = target + normal * overlap;

            // 2. Считаем вектор скольжения (проекция скорости на стену)
            // Формула: v = v - dot(v, n) * n
            velocity = velocity - normal * velocity.dot(normal);
            
            // Уменьшаем оставшееся время, чтобы не двигаться бесконечно
            time_left *= 0.5; 
        } else {
            // Если столкновений нет — просто перемещаемся
            pos = target;
            break;
        }
    }
    pos
}


fn check_collision(
    attacker_pos: Vec2, 
    victim_pos: Vec2, 
    attack_range: f32, 
    victim_radius: f32, 
    direction_byte: u8
) -> bool {
    // 1. Сначала проверяем дистанцию (самая быстрая проверка)
    let dist_sq = attacker_pos.distance_squared(victim_pos);
    let total_range = attack_range + victim_radius;

    println!("{}, {}, {}, {}, {}", attacker_pos, victim_pos, attack_range, victim_radius, dist_sq);

    if dist_sq > total_range * total_range {
        return false; // Слишком далеко
    }

    // 2. Проверяем направление
    // Переводим 0..255 в радианы
    let angle_rad = (direction_byte as f32 / 255.0) * 2.0 * std::f32::consts::PI;
    
    // Создаем вектор взгляда атакующего
    let look_dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
    
    // Создаем вектор от атакующего к жертве
    let to_victim = (victim_pos - attacker_pos).normalize_or_zero();
    
    // Считаем скалярное произведение
    let dot = look_dir.dot(to_victim);
    println!("{}, {}, {}, {}, {}", attacker_pos, victim_pos, attack_range, victim_radius, dist_sq);

    // 0.5 означает угол 60 градусов от центральной линии (общий сектор 120°)
    // 0.707 означает 45 градусов (общий сектор 90°)
    dot > 0.5
}