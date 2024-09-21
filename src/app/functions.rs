#![allow(non_snake_case)]

#[path = "./constants.rs"]
mod constants;
use constants::*;

use stoppable_thread::SimpleAtomicBool;

use std::path::Path;
use std::io::{BufRead, BufReader, Seek};
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::sleep;
use std::time::Duration;
use enigo::{Enigo, Key, Keyboard, Settings};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};
use regex::Regex;
use windows_volume_control::{AudioController, CoinitMode};

static STATE: AtomicU8 = AtomicU8::new(States::NOT_IN_GAME);
fn getState() -> u8 { return STATE.load(Ordering::Relaxed); }
fn setState(newState: u8) { STATE.store(newState, Ordering::Relaxed); }

fn playOrPauseMedia() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let _ = enigo.key(Key::MediaPlayPause, enigo::Direction::Click);
}

fn updateVolume(prevState: u8, audio_controller: &mut AudioController, process_name: &mut String, volumes: [u8; 3]) {
    let volume = volumes[getState() as usize] as f32 / 100.0;
    let prevVolume = volumes[prevState as usize] as f32 / 100.0;
    println!("Setting volume from {} to {}", prevVolume, volume);

    unsafe {
        let selectedSession = audio_controller.get_session_by_name(process_name.clone());
            
        // Si se pretende pasar de un estado con volumen 0 a un estado con volumen mayor, se reanuda la música.
        if prevVolume == 0.0 && volume > 0.0 { playOrPauseMedia(); }
        
        for i in 1..11 {
            selectedSession.unwrap().setVolume(prevVolume + (volume - prevVolume) * (i as f32) / 10.0);
            sleep(Duration::from_millis(100));
        }

        // Si el volumen objetivo es 0 después de estar en un estado con volumen mayor que 0, se pausa la música.
        if volume == 0.0 && prevVolume > 0.0 { playOrPauseMedia(); }
    }
}

fn analyzeText(line: &str) -> u8 {
    let re = Regex::new(r"^\[(?P<date>[^\]]+)\]\[(?P<code>[^\]]+)\](?P<name>[^\:]+):\s*(?P<text>.+)$").unwrap();
    if let Some(captures) = re.captures(line) {
        // let date = &captures["date"];
        // let code = &captures["code"];
        let name = &captures["name"];
        let text = &captures["text"];

        // println!("Log date: {}", date);
        // println!("Log code: {}", code);
        // println!("Log name: {}", name);
        // println!("Text: {}", text);

        // println!("{}", line);

        if name == "LogShooterGameState" {
            if text.contains("Match Ended") {
                println!("Match ended.");
                return States::NOT_IN_GAME;
            }
            else if text.contains("AShooterGameState::OnRoundEnded") {
                println!("Round ended.");
                return States::IN_GAME_PREPARING;
            }
            else if text.contains("Gameplay started at local time") && !text.contains("0.000000") {
                println!("Round started.");
                return States::IN_GAME_PLAYING;
            }
        }
        else if name == "LogGameFlowStateManager" {
            if text.contains("Reconcile called with state: TransitionToInGame and new state: InGame. Changing state") {
                println!("Match started.");
                return States::IN_GAME_PREPARING;
            }
        }
    }
    
    return getState(); // No se ha producido ningún cambio, se mantiene el estado actual.
}

fn watchFile(should_stop: &SimpleAtomicBool, audio_controller: &mut AudioController, process_name: &mut String, volumes: [u8; 3]) -> Result<()> {
    let binding = std::env::var("LOCALAPPDATA").unwrap() + "\\VALORANT\\Saved\\Logs\\ShooterGame.log";
    // let binding = "D:\\Users\\Saulete\\Downloads\\test.txt";
    let path = Path::new(&binding);
    let mut f = std::fs::File::open(&path)?;
    let mut pos = std::fs::metadata(&path)?.len();

    let (tx, _) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // No se puede comprobar actualizaciones en el archivo por culpa de cómo funciona Windows; por tanto,
    // se comprueba cada X segundos si el archivo ha sido modificado.
    while !should_stop.get() {
        if std::fs::metadata(&path)?.len() != pos {
            f.seek(std::io::SeekFrom::Start(pos))?;
            pos = std::fs::metadata(&path)?.len();

            let reader = BufReader::new(&f);
            for line in reader.lines() {
                let text = line.unwrap();
                if !text.is_empty() {
                    let newState = analyzeText(&text);
                    if newState != getState() { // Si el estado ha cambiado, se actualiza.
                        let prevState = getState();
                        setState(newState);
                        updateVolume(prevState, audio_controller, process_name, volumes);
                    }
                }
            }
        }

        //println!("Current state: {}", getState());
        sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn main_function(should_stop: &SimpleAtomicBool, process_name: String, volumes: [u8; 3]) {
    // No se pueden pasar entre hilos el controlador de audio por lo que se inicializa aquí
    let mut audio_controller = unsafe { AudioController::init(Some(CoinitMode::ApartmentThreaded)) };
    unsafe {
        audio_controller.GetSessions();
        audio_controller.GetDefaultAudioEnpointVolumeControl();
        audio_controller.GetAllProcessSessions();
    }

    updateVolume(States::NOT_IN_GAME, &mut audio_controller, &mut process_name.clone(), volumes); // Se establece el volumen inicial
    watchFile(should_stop, &mut audio_controller, &mut process_name.clone(), volumes).unwrap();
}

// Simulamos una partida de prueba: Entro en una partida, empiezo a jugar, muero, me reviven, empieza una nueva ronda y termina la partida por surrender.
pub fn simulate_match(process_name: String, volumes: [u8; 3]) {
    let mut audio_controller = unsafe { AudioController::init(Some(CoinitMode::ApartmentThreaded)) };
    unsafe {
        audio_controller.GetSessions();
        audio_controller.GetDefaultAudioEnpointVolumeControl();
        audio_controller.GetAllProcessSessions();
    }

    let simulationStates = [
        States::IN_GAME_PREPARING,
        States::IN_GAME_PLAYING,
        States::IN_GAME_PREPARING,
        States::NOT_IN_GAME
    ];

    for state in simulationStates.iter() {
        let prevState = getState();
        setState(*state);
        updateVolume(prevState, &mut audio_controller, &mut process_name.clone(), volumes);
        sleep(Duration::from_secs(1));
    }
}