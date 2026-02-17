#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherState {
    Clear,
    LightFog,
    HeavyFog,
}

impl WeatherState {
    pub fn base_fog_density(&self) -> f32 {
        match self {
            WeatherState::Clear => 0.01,
            WeatherState::LightFog => 0.03,
            WeatherState::HeavyFog => 0.08,
        }
    }
}

pub struct WorldState {
    pub time_of_day: f32,
    day_night_cycle_duration: f32,
    pub weather: WeatherState,
    target_weather: WeatherState,
    weather_transition_progress: f32,
    weather_transition_duration: f32,
}

impl WorldState {
    pub fn new(cycle_duration_seconds: f32) -> Self {
        Self {
            time_of_day: 0.5,
            day_night_cycle_duration: cycle_duration_seconds,
            weather: WeatherState::Clear,
            target_weather: WeatherState::Clear,
            weather_transition_progress: 1.0,
            weather_transition_duration: 30.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let time_increment = dt / self.day_night_cycle_duration;
        self.time_of_day += time_increment;
        
        if self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
        }

        if self.weather_transition_progress < 1.0 {
            self.weather_transition_progress += dt / self.weather_transition_duration;
            
            if self.weather_transition_progress >= 1.0 {
                self.weather_transition_progress = 1.0;
                self.weather = self.target_weather;
            }
        }
    }

    pub fn get_ambient_light(&self) -> f32 {
        let t = self.time_of_day;
        
        if t < 0.25 {
            0.2 + (t / 0.25) * 0.3
        } else if t < 0.5 {
            0.5 + ((t - 0.25) / 0.25) * 0.5
        } else if t < 0.75 {
            1.0 - ((t - 0.5) / 0.25) * 0.5
        } else {
            0.5 - ((t - 0.75) / 0.25) * 0.3
        }
    }

    pub fn get_fog_density(&self) -> f32 {
        if self.weather_transition_progress >= 1.0 {
            self.weather.base_fog_density()
        } else {
            let current_density = self.weather.base_fog_density();
            let target_density = self.target_weather.base_fog_density();
            
            current_density + (target_density - current_density) * self.weather_transition_progress
        }
    }

    pub fn set_weather(&mut self, new_weather: WeatherState) {
        if new_weather != self.target_weather {
            self.target_weather = new_weather;
            self.weather_transition_progress = 0.0;
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new(1200.0)
    }
}
