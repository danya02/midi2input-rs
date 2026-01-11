use std::{
    thread,
    time::{Duration, Instant, SystemTime},
};

use average::{Estimate, Kurtosis, Quantile, Variance, concatenate};

use enigo::{Enigo, Keyboard, Settings};
use midir::{MidiInput, MidiInputConnection};

fn main() {
    // let mut enigo = Enigo::new(&Settings::default()).expect("failed to initialize Enigo");
    let midi_input = MidiInput::new("midi2input").expect("failed to initialize midir::MidiInput");

    let ports = midi_input.ports();
    let connections = ports.iter().map(setup_port).collect::<Vec<_>>();
    drop(midi_input);

    loop {
        thread::sleep(Duration::from_secs(5));
    }

    // let bpm = 90.;
    // let beats_to_delay = 4.;
    // let delay = (60000. / bpm) * beats_to_delay;
    // loop {
    //     enigo
    //         .key(enigo::Key::Space, enigo::Direction::Click)
    //         .unwrap();
    //     thread::sleep(Duration::from_secs_f64(delay / 1000.));
    // }
    // enigo.text("hello world!").unwrap();
}

struct State {
    pressed_keys: [bool; 256],
    is_pressing_input: bool,
    enigo: Enigo,
    estimator: Kurtosis,
    last_event_midi: u64,
    last_event_time: Instant,
}

fn setup_port(port: &midir::MidiInputPort) -> MidiInputConnection<State> {
    let midi_input = MidiInput::new(&format!("midi2input-{}", port.id()))
        .expect("failed to initialize midir::MidiInput");
    let id = port.id();

    let state = State {
        pressed_keys: [false; 256],
        is_pressing_input: false,
        enigo: Enigo::new(&Settings::default()).unwrap(),
        estimator: Kurtosis::new(),
        last_event_midi: 0,
        last_event_time: Instant::now(),
    };

    enum Modes {
        AnyKeyPressed,
        ClickOnEach,
    }

    let mode = Modes::ClickOnEach;

    let conn = midi_input
        .connect(
            port,
            &format!("midi2input-port-{}", port.id()),
            move |when, what, state| {
                let now = Instant::now();
                let prev_midi = state.last_event_midi;
                let prev_time = state.last_event_time;
                state.last_event_midi = when;
                state.last_event_time = now;

                let elapsed_midi = when - prev_midi;
                let elapsed_time = (now - prev_time).as_micros() as u64;
                let elapsed_midi = elapsed_midi as f64;
                let elapsed_time = elapsed_time as f64;
                let error = elapsed_midi - elapsed_time;
                state.estimator.add(error);

                println!("Port {id} recv at {when}us: {what:?}");
                if what.len() == 3 {
                    // maybe key message
                    let (channel, key, velocity) = (what[0], what[1], what[2]);
                    if channel == 177 {
                        // pedal -- use to display estimator
                        println!("Mean: {}", state.estimator.mean());
                        println!("Error: {}", state.estimator.error_mean());
                        println!(
                            "Population variance: {}",
                            state.estimator.population_variance()
                        );
                        println!("Sample variance: {}", state.estimator.sample_variance());
                        println!("Skewedness: {}", state.estimator.skewness());
                        println!("Kurtosis: {}", state.estimator.kurtosis());
                    } else if (144u8..=(144 + 16)).contains(&channel) {
                        // let channel = channel - 144;
                        match mode {
                            Modes::AnyKeyPressed => {
                                if velocity > 0 {
                                    state.pressed_keys[key as usize] = true;
                                } else {
                                    state.pressed_keys[key as usize] = false;
                                }

                                // If any keys are pressed
                                if state.pressed_keys.iter().any(|x| *x) {
                                    if !state.is_pressing_input {
                                        // println!("Pressing input");
                                        state.is_pressing_input = true;
                                        state
                                            .enigo
                                            .key(enigo::Key::Space, enigo::Direction::Press)
                                            .expect("should be able to press key");
                                    }
                                } else {
                                    if state.is_pressing_input {
                                        // println!("Releasing input");
                                        state.is_pressing_input = false;
                                        state
                                            .enigo
                                            .key(enigo::Key::Space, enigo::Direction::Release)
                                            .expect("should be able to release key");
                                    }
                                }
                            }
                            Modes::ClickOnEach => {
                                if velocity > 0 {
                                    state
                                        .enigo
                                        .key(enigo::Key::Space, enigo::Direction::Click)
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            },
            state,
        )
        .unwrap();
    conn
}
