use crate::events::InputEvent;
use crate::player::Player;
use std::sync::mpsc;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use std::time::Duration;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

pub fn audio_player(
    player: &Arc<Mutex<Player>>,
    tx: mpsc::Sender<InputEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Did not find default output device");
    let config = device.default_output_config().unwrap();

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;

    let player_clone = player.clone();
    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &player_clone.clone(), &tx.clone());
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

fn write_data(
    output: &mut [f32],
    channels: usize,
    player: &Arc<Mutex<Player>>,
    tx: &mpsc::Sender<InputEvent>,
) {
    let mut time_b32 = player.lock().unwrap().current_time_b32();
    for frame in output.chunks_mut(channels) {
        #[allow(clippy::cast_possible_truncation)]
        let sample = player.lock().unwrap().next().unwrap() as f32;
        let next_time_b32 = player.lock().unwrap().current_time_b32();
        if next_time_b32 != time_b32 {
            time_b32 = next_time_b32;
            tx.send(InputEvent::PlayerBeatChange(time_b32)).unwrap();
        }
        for s in frame.iter_mut() {
            *s = sample;
        }
    }
}
