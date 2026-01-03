use std::{
    thread,
    time::{Duration, Instant, SystemTime},
};

use average::{Estimate, Kurtosis, Quantile, Variance, concatenate};

use enigo::{Enigo, Keyboard, Settings};
use midir::MidiInput;

fn main() {
    let mut enigo = Enigo::new(&Settings::default()).expect("failed to initialize Enigo");
    let mut midi_input =
        MidiInput::new("midi2input").expect("failed to initialize midir::MidiInput");

    let port = loop {
        let ports = midi_input.ports();
        for port in ports.iter() {
            println!("{}", port.id());
        }
        if ports.is_empty() {
            println!("no MIDI ports found, connect a device and try again");
            thread::sleep(Duration::from_secs(1));
            continue;
        }
        println!("Found port: {}", ports[0].id());
        break ports.into_iter().next().unwrap();
    };

    struct State {
        last_micros: u64,
        last_rt: SystemTime,
        estimator: Kurtosis,
    }

    let mut state = State {
        last_micros: 0,
        last_rt: SystemTime::now(),
        estimator: Kurtosis::new(),
    };

    let connection = midi_input
        .connect(
            &port,
            "midi2input_port",
            |micros, data, state| {
                let current_time = SystemTime::now();
                let rt_delta = current_time.duration_since(state.last_rt).unwrap();
                let micros_delta = Duration::from_micros(micros - state.last_micros);
                state.last_micros = micros;
                state.last_rt = current_time;
                let difference = rt_delta.as_secs_f64() - micros_delta.as_secs_f64();
                state.estimator.add(difference);
                // println!("At {micros}us, received {data:?}");
                // println!("rt delta: {rt_delta:?}, micros delta: {micros_delta:?}, difference: {difference:?}");

                println!(
                    "mean: {}, error: {}",
                    state.estimator.mean(),
                    state.estimator.error_mean()
                );
                println!("{:?}", state.estimator);
            },
            state,
        )
        .expect("failed to connect");

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
