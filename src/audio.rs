use bevy::prelude::*;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::Arc;
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
    _stream: Arc<OutputStream>,
    stream_handle: Arc<OutputStreamHandle>,
}

unsafe impl Send for AudioSystem {}
unsafe impl Sync for AudioSystem {}

#[derive(Resource)]
struct FootstepTimer {
    timer: Timer,
    base_interval: f32,
    is_left_foot: bool,
}

impl Default for FootstepTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.4, TimerMode::Repeating),
            base_interval: 0.4,
            is_left_foot: true,
        }
    }
}

fn setup_audio(mut commands: Commands) {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    
    commands.insert_resource(AudioSystem {
        _stream: Arc::new(stream),
        stream_handle: Arc::new(stream_handle),
    });
    
    commands.insert_resource(FootstepTimer::default());
}

fn handle_footsteps(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut timer_res: ResMut<FootstepTimer>,
    audio: Res<AudioSystem>,
    player_query: Query<(&crate::player::PlayerSpeed, &Transform), With<crate::player::Player>>,
) {
    let Ok((player_speed, transform)) = player_query.get_single() else {
        return;
    };

    let is_grounded = transform.translation.y <= 1.01;

    if keyboard.just_pressed(KeyCode::Space) && is_grounded {
        play_jump_sound(&audio.stream_handle);
    }

    let is_moving = keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::KeyS)
        || keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD);

    if !is_moving || !is_grounded {
        timer_res.timer.reset();
        return;
    }

    let speed_factor = (player_speed.current / 8.0).max(0.3);
    let interval = timer_res.base_interval / speed_factor;
    timer_res.timer.set_duration(Duration::from_secs_f32(interval));

    timer_res.timer.tick(time.delta());

    if timer_res.timer.just_finished() {
        play_footstep_sound(&audio.stream_handle, timer_res.is_left_foot);
        timer_res.is_left_foot = !timer_res.is_left_foot;
    }
}

fn play_footstep_sound(stream_handle: &OutputStreamHandle, is_left: bool) {
    let sound = generate_footstep_sound(is_left);
    
    if let Ok(sink) = Sink::try_new(stream_handle) {
        sink.append(sound);
        sink.detach();
    }
}

fn play_jump_sound(stream_handle: &OutputStreamHandle) {
    let sound = generate_jump_sound();
    
    if let Ok(sink) = Sink::try_new(stream_handle) {
        sink.append(sound);
        sink.detach();
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
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            (self.samples.len() / 2) as f32 / self.sample_rate as f32,
        ))
    }
}

fn generate_footstep_sound(is_left: bool) -> FootstepSound {
    let sample_rate = 44100;
    let attack = 0.005;
    let decay = if is_left { 0.08 } else { 0.06 };
    let duration = attack + decay;
    let num_samples = (sample_rate as f32 * duration) as usize;
    
    let lpf_cutoff = if is_left { 800.0 } else { 650.0 };
    let gain = if is_left { 0.8 } else { 0.6 };
    let pan = if is_left { 0.45 } else { 0.55 };
    
    let mut samples = Vec::with_capacity(num_samples * 2);
    
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let mut lpf_state = 0.0;
    let lpf_alpha = 1.0 - (-2.0 * std::f32::consts::PI * lpf_cutoff / sample_rate as f32).exp();
    
    let mut hpf_state = 0.0;
    let hpf_cutoff = 100.0;
    let hpf_alpha = 1.0 - (-2.0 * std::f32::consts::PI * hpf_cutoff / sample_rate as f32).exp();
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        
        let envelope = if t < attack {
            t / attack
        } else {
            let decay_t = (t - attack) / decay;
            (1.0 - decay_t).max(0.0)
        };
        
        let white_noise = rng.r#gen::<f32>() * 2.0 - 1.0;
        
        lpf_state += lpf_alpha * (white_noise - lpf_state);
        
        let hpf_input = lpf_state;
        hpf_state += hpf_alpha * (hpf_input - hpf_state);
        let filtered = hpf_input - hpf_state;
        
        let sample = filtered * envelope * gain * 0.3;
        
        let left = sample * (1.0 - pan);
        let right = sample * pan;
        
        samples.push(left);
        samples.push(right);
    }
    
    FootstepSound {
        sample_rate,
        samples,
        current_sample: 0,
    }
}

fn generate_jump_sound() -> FootstepSound {
    let sample_rate = 44100;
    let duration = 0.15;
    let num_samples = (sample_rate as f32 * duration) as usize;
    
    let mut samples = Vec::with_capacity(num_samples * 2);
    
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let mut lpf_state = 0.0;
    let lpf_alpha = 1.0 - (-2.0 * std::f32::consts::PI * 1200.0 / sample_rate as f32).exp();
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        
        let freq = 200.0 + (1.0 - t / duration) * 400.0;
        let tone = (2.0 * std::f32::consts::PI * freq * t).sin();
        
        let white_noise = rng.r#gen::<f32>() * 2.0 - 1.0;
        
        lpf_state += lpf_alpha * (white_noise - lpf_state);
        
        let envelope = (1.0 - t / duration).powf(1.5);
        
        let sample = (tone * 0.3 + lpf_state * 0.7) * envelope * 0.25;
        
        samples.push(sample);
        samples.push(sample);
    }
    
    FootstepSound {
        sample_rate,
        samples,
        current_sample: 0,
    }
}
