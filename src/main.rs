extern crate sdl2;
use rand::Rng;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn main() {
    // Initialisation du contexte SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Création de la fenêtre SDL2
    let window = video_subsystem.window("rust-sdl2 demo", 1000, 1000)
        .position_centered()
        .build()
        .unwrap();
        
    // Création du canvas pour dessiner sur la fenêtre
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    
    // Initialisation de l'écouteur d'événements SDL2
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    // Initialisation des voies de circulation
    let mut lanes = Vec::<Lane>::new();
    lanes.push(Lane { name: String::from("south"), start: (550, 1000), length: 400, direction: VehiclesSpawingFrom::Up, state: LaneLight::Red, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("east"), start: (0, 550), length: 400, direction: VehiclesSpawingFrom::Left, state: LaneLight::Red, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("north"), start: (450, 0), length: 400, direction: VehiclesSpawingFrom::Down, state: LaneLight::Red, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("west"), start: (1000, 450), length: 400, direction: VehiclesSpawingFrom::Right, state: LaneLight::Red, vehicles: Vec::<Vehicle>::new() });
    
    let mut road_intersection= Vec::<Vehicle>::new();
    
    // Initialisation des feux de circulation
    let mut intersection_lights = Lights {current_lane_index: 0, last_change_time: Instant::now(), change_interval: Duration::new(2, 0)};
    
    // Ajout des voies de sortie
    lanes.push(Lane { name: String::from("exit_north"), start: (550, 400), length: 400, direction: VehiclesSpawingFrom::Up, state: LaneLight::Green, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("exit_west"), start: (600, 550), length: 400, direction: VehiclesSpawingFrom::Left, state: LaneLight::Green, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("exit_south"), start: (450, 600), length: 400, direction: VehiclesSpawingFrom::Down, state: LaneLight::Green, vehicles: Vec::<Vehicle>::new() });
    lanes.push(Lane { name: String::from("exit_east"), start: (400, 450), length: 400, direction: VehiclesSpawingFrom::Right, state: LaneLight::Green, vehicles: Vec::<Vehicle>::new() });
    
    'running: loop {
        // Réinitialiser le canvas
        canvas.set_draw_color(Color::RGB(0,0,0));
        canvas.clear();
        
        // Écoute des événements clavier
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        // Spawn un véhicule selon la touche pressée
                        Keycode::Up => spawn_vehicle(lanes.iter_mut().find(|lane| lane.name == "south").expect("no south lane")),
                        Keycode::Down => spawn_vehicle(lanes.iter_mut().find(|lane| lane.name == "north").expect("no north lane")),
                        Keycode::Right => spawn_vehicle(lanes.iter_mut().find(|lane| lane.name == "east").expect("no east lane")),
                        Keycode::Left => spawn_vehicle(lanes.iter_mut().find(|lane| lane.name == "west").expect("no west lane")),
                        Keycode::R => {
                            let mut rng = rand::thread_rng();
                            let options  = vec![
                                "south",
                                "north",
                                "west",
                                "east"];
                            let random_lane = options.choose(&mut rng).unwrap().to_string();
                            spawn_vehicle(lanes.iter_mut().find(|lane| lane.name == random_lane).expect("no west lane"))
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Dessiner les voies et les véhicules
        draw_lanes(&mut canvas, &lanes);
        draw_vehicles(&mut canvas, &road_intersection);
        
        // Déplacer les véhicules dans les voies et l'intersection
        move_lanes(&mut lanes, &mut road_intersection);
        move_intersection(&mut road_intersection, &mut lanes);
        
        // Changer les feux de circulation
        change_lights(&mut intersection_lights, &mut lanes, &road_intersection);        
        
        // Présenter le canvas
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[derive(PartialEq)]
pub enum LaneLight { Green, Red }

#[derive(Clone, Debug)]
pub enum VehiclesSpawingFrom { Up, Down, Right, Left }

#[derive(Clone)]
pub enum Behavior { Straight, TurnRight, TurnLeft }

const VEHICLE_SIZE: i32 = 40;
const VEHICLE_SPEED: i32 = 3;
const SECURITY_DISTANCE: i32 = VEHICLE_SIZE*2;

#[derive(Clone)]
pub struct Vehicle{
    pub position: (i32, i32),
    pub origin: VehiclesSpawingFrom,
    pub behavior: Behavior,
    pub intersection_progress : i32,
}

pub struct Lane{
    pub name: String,
    pub start: (i32, i32),
    pub length: u32,
    pub direction: VehiclesSpawingFrom,
    pub state : LaneLight,
    pub vehicles : Vec<Vehicle>,
}

pub struct Lights {
    current_lane_index : usize,
    last_change_time : Instant,
    change_interval: Duration,
}

// Fonction pour spawn un véhicule dans une voie
pub fn spawn_vehicle(lane: &mut Lane) {    
    if !lane.vehicles.is_empty(){
        let last_vehicle = &lane.vehicles[lane.vehicles.len()-1];        
        if are_too_close(&last_vehicle.position, &lane.start) {
            return
        }
    }    
    let mut rng = rand::thread_rng();
    let options = [Behavior::Straight, Behavior::TurnRight, Behavior::TurnLeft];
    let random_behavior = options.choose(&mut rng).unwrap().clone();
    lane.vehicles.push(Vehicle { position: lane.start, origin: lane.direction.clone(), behavior: random_behavior, intersection_progress: 0 });
    println!("vehicle spawned {:?}, total vehicle: {}", lane.direction, lane.vehicles.len());
}

// Fonction pour déplacer les véhicules dans les voies
pub fn move_lanes(lanes: &mut Vec<Lane>, road_intersection: &mut Vec<Vehicle>){
    for lane in lanes.iter_mut() {  
        let positions: Vec<_> = lane.vehicles.iter().map(|car| car.position).collect();
        let mut indices_to_remove = Vec::new();    
        for (i, car) in lane.vehicles.iter_mut().enumerate() {
            
            let end_of_lane: (i32, i32);
            let next_position: (i32, i32);
            match lane.direction {
                VehiclesSpawingFrom::Down => {
                    next_position = (car.position.0, car.position.1 + VEHICLE_SPEED);
                    end_of_lane = (lane.start.0, lane.start.1 + lane.length as i32 + SECURITY_DISTANCE - VEHICLE_SIZE/2);
                },
                VehiclesSpawingFrom::Up => {
                    next_position = (car.position.0, car.position.1 - VEHICLE_SPEED);
                    end_of_lane = (lane.start.0, lane.start.1 - lane.length as i32 - SECURITY_DISTANCE + VEHICLE_SIZE/2);
                },
                VehiclesSpawingFrom::Right => {
                    next_position = (car.position.0 - VEHICLE_SPEED, car.position.1);
                    end_of_lane = (lane.start.0 - lane.length as i32 - SECURITY_DISTANCE + VEHICLE_SIZE/2, lane.start.1);
                },
                VehiclesSpawingFrom::Left => { 
                    next_position = (car.position.0 + VEHICLE_SPEED, car.position.1);
                    end_of_lane = (lane.start.0 + lane.length as i32 + SECURITY_DISTANCE - VEHICLE_SIZE/2, lane.start.1);
                },
            }
            
            // Vérifier si le véhicule précédent n'est pas trop proche
            if i > 0 {
                if are_too_close(&next_position, &positions[i - 1]) {
                    continue;
                }
            }
            
            // Vérifier si c'est la fin de la voie et si le feu est rouge            
            if are_too_close(&next_position, &end_of_lane) && lane.state == LaneLight::Red{
                continue;
            }
            
            if are_too_close(&next_position, &end_of_lane) && lane.state == LaneLight::Green{
                // Copier le véhicule dans l'intersection, si ce n'est pas une voie de sortie
                if &lane.name[0..4] != "exit"{
                    road_intersection.push(car.clone());
                    
                }
                indices_to_remove.push(i);                
                continue;
            }     
            car.position = next_position;
        }
        for &i in indices_to_remove.iter().rev() {
            lane.vehicles.remove(i);
        }
     }    
}

// Fonction pour déplacer les véhicules dans l'intersection
pub fn move_intersection(intersection: &mut Vec<Vehicle>, lanes: &mut Vec<Lane>){
    let mut indices_to_remove = Vec::new();
    for (i, car) in intersection.iter_mut().enumerate() {   
        let origin_lane_indice: usize;         
        let next_position: (i32, i32);            
        match car.origin {
            VehiclesSpawingFrom::Down => {
                next_position = (car.position.0, car.position.1 + VEHICLE_SPEED);
                origin_lane_indice = 2;
            },
            VehiclesSpawingFrom::Up => {
                next_position = (car.position.0, car.position.1 - VEHICLE_SPEED);
                origin_lane_indice = 0;
            },
            VehiclesSpawingFrom::Right => {
                next_position = (car.position.0 - VEHICLE_SPEED, car.position.1);
                origin_lane_indice = 3;
            },
            VehiclesSpawingFrom::Left => { 
                next_position = (car.position.0 + VEHICLE_SPEED, car.position.1);
                origin_lane_indice = 1;
            },
        }
        car.position = next_position;
        car.intersection_progress += VEHICLE_SPEED;
        match car.behavior {
            Behavior::Straight => {
                if car.intersection_progress > 200 {
                    indices_to_remove.push(i);
                    lanes[4 + origin_lane_indice].vehicles.push(car.clone());
                }
            },
            Behavior::TurnLeft => {
                if car.intersection_progress > 69 {
                    indices_to_remove.push(i);
                    let mut exit_lane_indice = 4 + origin_lane_indice +1;
                    if exit_lane_indice > 7 {
                        exit_lane_indice = exit_lane_indice - 4;
                    }
                    lanes[exit_lane_indice].vehicles.push(car.clone());                  
                }
            },
            Behavior::TurnRight => {
                if car.intersection_progress > 169 {
                    indices_to_remove.push(i);
                    let mut exit_lane_indice = 4 + origin_lane_indice +3;
                    if exit_lane_indice > 7 {
                        exit_lane_indice = exit_lane_indice - 4;
                    }
                    lanes[exit_lane_indice].vehicles.push(car.clone());               
                }
            },
        }
    }
    for &i in indices_to_remove.iter().rev() {
        intersection.remove(i);
    }
}

// Fonction pour vérifier si deux véhicules sont trop proches l'un de l'autre
pub fn are_too_close(car1_pos :&(i32, i32), car2_pos :&(i32, i32)) -> bool {
    let x_too_close = (car1_pos.0 - car2_pos.0).abs() < SECURITY_DISTANCE;
    let y_too_close = (car1_pos.1 - car2_pos.1).abs() < SECURITY_DISTANCE;
    x_too_close && y_too_close 
}

// Fonction pour changer les feux de circulation
pub fn change_lights(l: &mut Lights, lanes: &mut Vec<Lane>, road_intersection: &Vec<Vehicle>){
    if l.last_change_time.elapsed() >= l.change_interval {
        lanes[l.current_lane_index].state = LaneLight::Red;
        if road_intersection.is_empty() {
            l.current_lane_index = (l.current_lane_index + 1) % 4; // pour ne changer que les 4 premières lignes 
            lanes[l.current_lane_index].state = LaneLight::Green;
            l.last_change_time = Instant::now();
        }        
    }
}

// Fonction pour dessiner les voies
pub fn draw_lanes<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>, lanes: &Vec<Lane>){
        for lane in lanes.iter() {
            // dessiner les voies
            if &lane.name[0..4] == "exit"{
                canvas.set_draw_color(Color::RGB(255, 255, 255));
            } else {
                match lane.state {
                    LaneLight::Green => canvas.set_draw_color(Color::RGB(0, 255, 0)),
                    LaneLight::Red => canvas.set_draw_color(Color::RGB(255, 0, 0)),
                }
            }
 
            let mut x = lane.start.0;
            let mut y = lane.start.1;
            let mut width = 1;
            let mut height = 1;
            match lane.direction {
                VehiclesSpawingFrom::Down => height = lane.length,
                VehiclesSpawingFrom::Up => {
                    height = lane.length;
                    y = y-height as i32;
                },
                VehiclesSpawingFrom::Right => {
                    width = lane.length;
                    x = x-width as i32;
                },
                VehiclesSpawingFrom::Left => width = lane.length,
            }
            let rectangle = Rect::new(x, y, width, height);
            canvas.fill_rect(rectangle).unwrap();
            // dessiner les véhicules
            draw_vehicles(canvas, &lane.vehicles);
        }
}

// Fonction pour dessiner les véhicules
pub fn draw_vehicles<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>, vehicles: &Vec<Vehicle>){
    
    for car in vehicles.iter(){
        match car.behavior {
            Behavior::Straight => canvas.set_draw_color(Color::RGB(0, 0, 255)),
            Behavior::TurnLeft => canvas.set_draw_color(Color::RGB(200, 255, 0)),
            Behavior::TurnRight => canvas.set_draw_color(Color::RGB(200, 0, 255)),            
        }
        let x = car.position.0 - VEHICLE_SIZE / 2;
        let y = car.position.1 - VEHICLE_SIZE / 2;
        let rectangle = Rect::new(x, y, VEHICLE_SIZE as u32, VEHICLE_SIZE as u32);
        canvas.fill_rect(rectangle).unwrap();
    }
}
