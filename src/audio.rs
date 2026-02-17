use bevy::prelude::*;
use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio)
            .add_systems(Update, handle_footsteps);
    }
}

#[derive(Resource)]
pub struct AudioSystem {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
}

#[derive(Resource)]
struct FootstepTimer {
    timer: Timer,
    base_interval: f32,
}

impl Default for FootstepTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.4, TimerMode::Repeating),
            base_interval: 0.4,
        }
    }
}

fn setup_audio(mut commands: Commands) {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    commands.insert_resource(AudioSystem {
        _stream: stream,
        sink: Arc::new(Mutex::new(sink)),
    });
    
    commands.insert_resource(FootstepTimer::default());
}

fn handle_footsteps(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut timer_res: ResMut<FootstepTimer>,
    audio: Res<AudioSystem>,
    player_query: Query<&crate::player::PlayerSpeed, With<crate::player::Player>>,
) {
    let Ok(player_speed) = player_query.get_single() else {
        return;
    };

    let is_moving = keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::KeyS)
        || keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD);

    if !is_moving {
        timer_res.timer.reset();
        return;
    }

    let speed_factor = (player_speed.current / 8.0).max(0.3);
    let interval = timer_res.base_interval / speed_factor;
    timer_res.timer.set_duration(Duration::from_secs_f32(interval));

    timer_res.timer.tick(time.delta());

    if timer_res.timer.just_finished() {
        play_footstep_sound(&audio.sink, speed_factor);
    }
}

fn play_footstep_sound(sink: &Arc<Mutex<Sink>>, speed_factor: f32) {
    let sound = generate_footstep_sound(speed_factor);
    
    if let Ok(sink) = sink.lock() {
        sink.append(sound);
    }
}

struct FootstepSound {
    sample_rate: u32,
    samples: Vec<f32>,
    current_sample: usize,
}

impl Iterator for FootstepSound {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.samples.len() {
            None
        } else {
            let sample = self.samples[self.current_sample];
            self.current_sample += 1;
            Some(sample)
        }
    }
}

impl Source for FootstepSound {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.current_sample)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / self.sample_rate as f32,
        ))
    }
}

fn generate_footstep_sound(speed_factor: f32) -> FootstepSound {
    let sample_rate = 44100;
    let duration = 0.08;
    let num_samples = (sample_rate as f32 * duration) as usize;
    
    let mut samples = Vec::with_capacity(num_samples);
    
    let base_freq = 80.0 + (speed_factor - 1.0) * 30.0;
    let noise_amount = 0.7;
    
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let envelope = (1.0 - (t / duration)).powf(2.0);
        
        let tone = (2.0 * std::f32::consts::PI * base_freq * t).sin() * 0.3;
        let noise = (rng.gen::<f32>() * 2.0 - 1.0) * noise_amount;
        
        let sample = (tone + noise) * envelope * 0.15;
        samples.push(sample);
    }
    
    FootstepSound {
        sample_rate,
        samples,
        current_sample: 0,
    }
}
