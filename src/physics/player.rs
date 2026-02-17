use macroquad::prelude::*;

pub struct Player {
    pub id: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: f32,
    pub is_grounded: bool,
    pub stamina: f32,
    pub is_sprinting: bool,
    pub current_speed_multiplier: f32,
    pub time_since_last_sprint: f32,
    pub time_since_last_jump: f32,
}

impl Player {
    pub fn new(id: u8, position: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            rotation: 0.0,
            is_grounded: false,
            stamina: 100.0,
            is_sprinting: false,
            current_speed_multiplier: 1.0,
            time_since_last_sprint: 999.0,
            time_since_last_jump: 999.0,
        }
    }
}

pub struct PlayerController {
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub sprint_multiplier: f32,
    pub stamina_drain_rate: f32,
    pub stamina_regen_rate: f32,
    pub sprint_acceleration_time: f32,
    pub stamina_regen_delay: f32,
    pub auto_jump_delay: f32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 8.0,
            gravity: 20.0,
            sprint_multiplier: 2.5,
            stamina_drain_rate: 20.0,
            stamina_regen_rate: 33.333,
            sprint_acceleration_time: 0.5,
            stamina_regen_delay: 1.0,
            auto_jump_delay: 0.2,
        }
    }

    pub fn update(&self, player: &mut Player, input: &crate::input::controls::InputState, dt: f32) {
        self.update_sprint_state(player, input, dt);
        self.update_stamina(player, dt);
        
        let mut move_dir = Vec3::ZERO;
        
        if input.move_forward {
            move_dir.z += 1.0;
        }
        if input.move_back {
            move_dir.z -= 1.0;
        }
        if input.move_left {
            move_dir.x += 1.0;
        }
        if input.move_right {
            move_dir.x -= 1.0;
        }
        
        if move_dir.length() > 0.0 {
            move_dir = move_dir.normalize();
            
            let sin_rot = player.rotation.sin();
            let cos_rot = player.rotation.cos();
            
            let forward = vec3(sin_rot, 0.0, cos_rot);
            let right = vec3(cos_rot, 0.0, -sin_rot);
            
            let movement = forward * move_dir.z + right * move_dir.x;
            
            let effective_speed = self.move_speed * player.current_speed_multiplier;
            player.velocity.x = movement.x * effective_speed;
            player.velocity.z = movement.z * effective_speed;
        } else {
            player.velocity.x = 0.0;
            player.velocity.z = 0.0;
        }
        
        player.time_since_last_jump += dt;
        
        if input.jump && player.is_grounded && player.time_since_last_jump >= self.auto_jump_delay {
            player.velocity.y = self.jump_force;
            player.is_grounded = false;
            player.time_since_last_jump = 0.0;
        }
        
        self.apply_gravity(player, dt);
        
        player.position += player.velocity * dt;
        
        self.handle_ground_collision(player);
    }

    pub fn apply_gravity(&self, player: &mut Player, dt: f32) {
        if !player.is_grounded {
            player.velocity.y -= self.gravity * dt;
        }
    }
    
    pub fn update_sprint_state(&self, player: &mut Player, input: &crate::input::controls::InputState, dt: f32) {
        let sprint_lockout_threshold = 30.0;
        let can_sprint = player.stamina >= sprint_lockout_threshold;
        
        let wants_to_sprint = input.sprint && (input.move_forward || input.move_back || input.move_left || input.move_right);
        
        let was_sprinting = player.is_sprinting;
        
        if wants_to_sprint && can_sprint {
            player.is_sprinting = true;
            player.time_since_last_sprint = 0.0;
        } else {
            player.is_sprinting = false;
            if was_sprinting {
                player.time_since_last_sprint = 0.0;
            } else {
                player.time_since_last_sprint += dt;
            }
        }
        
        let target_multiplier = if player.is_sprinting {
            self.sprint_multiplier
        } else {
            1.0
        };
        
        let acceleration_rate = (self.sprint_multiplier - 1.0) / self.sprint_acceleration_time;
        
        if player.current_speed_multiplier < target_multiplier {
            player.current_speed_multiplier += acceleration_rate * dt;
            if player.current_speed_multiplier > target_multiplier {
                player.current_speed_multiplier = target_multiplier;
            }
        } else if player.current_speed_multiplier > target_multiplier {
            player.current_speed_multiplier -= acceleration_rate * dt;
            if player.current_speed_multiplier < target_multiplier {
                player.current_speed_multiplier = target_multiplier;
            }
        }
    }
    
    pub fn update_stamina(&self, player: &mut Player, dt: f32) {
        if player.is_sprinting {
            player.stamina -= self.stamina_drain_rate * dt;
            if player.stamina < 0.0 {
                player.stamina = 0.0;
            }
        } else if player.time_since_last_sprint >= self.stamina_regen_delay {
            player.stamina += self.stamina_regen_rate * dt;
            if player.stamina > 100.0 {
                player.stamina = 100.0;
            }
        }
    }
    
    fn handle_ground_collision(&self, player: &mut Player) {
        let ground_level = 0.0;
        let _player_height = 1.8;
        
        if player.position.y <= ground_level {
            player.position.y = ground_level;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        } else if player.position.y > ground_level + 0.1 {
            player.is_grounded = false;
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::input::controls::InputState;

    #[test]
    fn test_vertical_surface_collision() {
        let mut player = Player::new(0, vec3(0.0, 1.0, 0.0));
        player.is_grounded = true;
        player.velocity = vec3(5.0, 0.0, 0.0);
        
        let wall_x = 10.0;
        let player_radius = 0.5;
        
        player.position.x = wall_x - player_radius - 0.01;
        
        let initial_x = player.position.x;
        player.position.x += player.velocity.x * 0.016;
        
        if player.position.x + player_radius > wall_x {
            player.position.x = wall_x - player_radius;
            player.velocity.x = 0.0;
        }
        
        assert!(player.position.x <= wall_x - player_radius, 
            "Player should not pass through vertical wall");
        assert_eq!(player.velocity.x, 0.0, 
            "Horizontal velocity should be zero after wall collision");
        assert!(player.position.x >= initial_x, 
            "Player should not move backward from wall collision");
    }

    #[test]
    fn test_horizontal_surface_collision_floor() {
        let mut player = Player::new(0, vec3(0.0, 0.1, 0.0));
        player.velocity.y = -10.0;
        player.is_grounded = false;
        
        let ground_level = 0.0;
        
        let dt = 0.016;
        player.position.y += player.velocity.y * dt;
        
        if player.position.y <= ground_level {
            player.position.y = ground_level;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        }
        
        assert_eq!(player.position.y, 0.0, 
            "Player should be exactly at ground level after collision");
        assert_eq!(player.velocity.y, 0.0, 
            "Vertical velocity should be zero after ground collision");
        assert!(player.is_grounded, 
            "Player should be grounded after floor collision");
    }

    #[test]
    fn test_horizontal_surface_collision_ceiling() {
        let mut player = Player::new(0, vec3(0.0, 1.0, 0.0));
        player.velocity.y = 10.0;
        player.is_grounded = false;
        
        let ceiling_y = 5.0;
        let player_height = 1.8;
        
        player.position.y = ceiling_y - player_height + 0.5;
        
        let dt = 0.016;
        player.position.y += player.velocity.y * dt;
        
        if player.position.y + player_height > ceiling_y {
            player.position.y = ceiling_y - player_height;
            player.velocity.y = 0.0;
        }
        
        assert!(player.position.y + player_height <= ceiling_y, 
            "Player should not pass through ceiling");
        assert_eq!(player.velocity.y, 0.0, 
            "Vertical velocity should be zero after ceiling collision");
    }

    #[test]
    fn test_corner_collision_ground_and_wall() {
        let mut player = Player::new(0, vec3(9.45, 0.05, 0.0));
        player.velocity = vec3(5.0, -10.0, 0.0);
        player.is_grounded = false;
        
        let wall_x = 10.0;
        let player_radius = 0.5;
        let ground_y = 0.0;
        
        let dt = 0.016;
        
        player.position.x += player.velocity.x * dt;
        player.position.y += player.velocity.y * dt;
        
        if player.position.x + player_radius > wall_x {
            player.position.x = wall_x - player_radius;
            player.velocity.x = 0.0;
        }
        
        if player.position.y <= ground_y {
            player.position.y = ground_y;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        }
        
        assert!(player.position.x <= wall_x - player_radius, 
            "Player should not pass through wall in corner collision");
        assert!((player.position.y - ground_y).abs() < 0.001, 
            "Player should be at ground level in corner collision, got: {}", player.position.y);
        assert_eq!(player.velocity.x, 0.0, 
            "Horizontal velocity should be zero after corner collision");
        assert_eq!(player.velocity.y, 0.0, 
            "Vertical velocity should be zero after corner collision");
        assert!(player.is_grounded, 
            "Player should be grounded after corner collision");
    }

    #[test]
    fn test_corner_collision_sliding_along_wall() {
        let mut player = Player::new(0, vec3(0.0, 1.0, 0.0));
        player.velocity = vec3(5.0, 0.0, 3.0);
        player.is_grounded = true;
        
        let wall_x = 10.0;
        let player_radius = 0.5;
        
        player.position.x = wall_x - player_radius - 0.01;
        
        let dt = 0.016;
        let initial_z = player.position.z;
        
        player.position.x += player.velocity.x * dt;
        player.position.z += player.velocity.z * dt;
        
        if player.position.x + player_radius > wall_x {
            player.position.x = wall_x - player_radius;
            player.velocity.x = 0.0;
        }
        
        assert!(player.position.x <= wall_x - player_radius, 
            "Player should not pass through wall");
        assert_eq!(player.velocity.x, 0.0, 
            "Horizontal X velocity should be zero after wall collision");
        assert!(player.position.z > initial_z, 
            "Player should continue moving along Z axis (sliding along wall)");
        assert_eq!(player.velocity.z, 3.0, 
            "Z velocity should be preserved when sliding along wall");
    }

    #[test]
    fn test_sprint_activation_deactivation() {
        let controller = PlayerController::new();
        let mut player = Player::new(0, vec3(0.0, 0.0, 0.0));
        player.stamina = 100.0;
        player.is_grounded = true;
        player.current_speed_multiplier = 1.0;
        
        let mut input = InputState::new();
        input.sprint = true;
        input.move_forward = true;
        
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(player.is_sprinting, "Sprint should activate when sprint key is pressed with movement");
        
        input.sprint = false;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(!player.is_sprinting, "Sprint should deactivate when sprint key is released");
        
        input.sprint = true;
        input.move_forward = false;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(!player.is_sprinting, "Sprint should not activate without movement input");
        
        input.move_forward = true;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(player.is_sprinting, "Sprint should reactivate with both sprint key and movement");
    }

    #[test]
    fn test_stamina_lockout_boundary() {
        let controller = PlayerController::new();
        let mut player = Player::new(0, vec3(0.0, 0.0, 0.0));
        player.is_grounded = true;
        player.current_speed_multiplier = 1.0;
        
        let mut input = InputState::new();
        input.sprint = true;
        input.move_forward = true;
        
        player.stamina = 29.0;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(!player.is_sprinting, "Sprint should be locked at 29% stamina (below 30% threshold)");
        
        player.stamina = 30.0;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(player.is_sprinting, "Sprint should be allowed at exactly 30% stamina (at threshold)");
        
        player.stamina = 31.0;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(player.is_sprinting, "Sprint should be allowed at 31% stamina (above threshold)");
        
        player.stamina = 0.0;
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(!player.is_sprinting, "Sprint should be locked at 0% stamina");
    }

    #[test]
    fn test_sprint_in_air() {
        let controller = PlayerController::new();
        let mut player = Player::new(0, vec3(0.0, 5.0, 0.0));
        player.stamina = 100.0;
        player.is_grounded = false;
        player.current_speed_multiplier = 1.0;
        
        let mut input = InputState::new();
        input.sprint = true;
        input.move_forward = true;
        
        controller.update_sprint_state(&mut player, &input, 0.016);
        assert!(player.is_sprinting, "Sprint state should be allowed in air");
        
        let initial_multiplier = player.current_speed_multiplier;
        for _ in 0..10 {
            controller.update_sprint_state(&mut player, &input, 0.016);
        }
        assert!(player.current_speed_multiplier > initial_multiplier, 
            "Speed multiplier should increase even in air");
        
        controller.update_stamina(&mut player, 0.016);
        assert!(player.stamina < 100.0, "Stamina should drain while sprinting in air");
    }

    #[test]
    fn test_stamina_regeneration_interruption() {
        let controller = PlayerController::new();
        let mut player = Player::new(0, vec3(0.0, 0.0, 0.0));
        player.stamina = 50.0;
        player.is_sprinting = false;
        player.is_grounded = true;
        player.time_since_last_sprint = 999.0;
        
        let mut input = InputState::new();
        input.sprint = false;
        input.move_forward = true;
        
        for _ in 0..10 {
            controller.update_stamina(&mut player, 0.1);
        }
        let stamina_after_regen = player.stamina;
        assert!(stamina_after_regen > 50.0, "Stamina should regenerate when not sprinting");
        
        input.sprint = true;
        controller.update_sprint_state(&mut player, &input, 0.016);
        player.is_sprinting = true;
        
        for _ in 0..10 {
            controller.update_stamina(&mut player, 0.1);
        }
        assert!(player.stamina < stamina_after_regen, 
            "Stamina should immediately start draining when sprint is activated");
        
        input.sprint = false;
        controller.update_sprint_state(&mut player, &input, 0.016);
        player.is_sprinting = false;
        
        let stamina_before_resume = player.stamina;
        
        player.time_since_last_sprint = controller.stamina_regen_delay + 0.1;
        
        for _ in 0..10 {
            controller.update_stamina(&mut player, 0.1);
        }
        assert!(player.stamina > stamina_before_resume, 
            "Stamina should resume regeneration after delay when sprint is deactivated");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use crate::input::controls::InputState;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_property_1_movement_input_response(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            rotation in 0.0f32..std::f32::consts::TAU,
            dt in 0.001f32..0.1,
            move_forward in any::<bool>(),
            move_back in any::<bool>(),
            move_left in any::<bool>(),
            move_right in any::<bool>()
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.rotation = rotation;
            player.is_grounded = true;
            
            let mut input = InputState::new();
            input.move_forward = move_forward;
            input.move_back = move_back;
            input.move_left = move_left;
            input.move_right = move_right;
            input.jump = false;
            
            let initial_pos = player.position;
            
            controller.update(&mut player, &input, dt);
            
            let forward_back_cancel = move_forward && move_back;
            let left_right_cancel = move_left && move_right;
            let has_effective_input = (move_forward || move_back) && !forward_back_cancel 
                                   || (move_left || move_right) && !left_right_cancel;
            
            if has_effective_input {
                let horizontal_displacement = vec2(
                    player.position.x - initial_pos.x,
                    player.position.z - initial_pos.z
                ).length();
                
                let expected_speed = controller.move_speed * player.current_speed_multiplier * dt;
                
                prop_assert!(
                    (horizontal_displacement - expected_speed).abs() < 0.01,
                    "Player should move at consistent speed. Expected: {}, Got: {}",
                    expected_speed,
                    horizontal_displacement
                );
            } else {
                prop_assert_eq!(
                    player.velocity.x, 0.0,
                    "Player horizontal velocity X should be zero with no effective input"
                );
                prop_assert_eq!(
                    player.velocity.z, 0.0,
                    "Player horizontal velocity Z should be zero with no effective input"
                );
            }
        }

        #[test]
        fn test_property_2_jump_velocity_application(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            rotation in 0.0f32..std::f32::consts::TAU,
            dt in 0.001f32..0.1
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.rotation = rotation;
            player.is_grounded = true;
            player.velocity.y = 0.0;
            
            let mut input = InputState::new();
            input.jump = true;
            
            controller.update(&mut player, &input, dt);
            
            let expected_velocity_after_gravity = controller.jump_force - controller.gravity * dt;
            
            prop_assert!(
                player.velocity.y > 0.0,
                "Jump should result in positive upward velocity. Got: {}",
                player.velocity.y
            );
            
            prop_assert!(
                (player.velocity.y - expected_velocity_after_gravity).abs() < 0.1,
                "Jump velocity after gravity should be jump_force - gravity*dt. Expected: {}, Got: {}",
                expected_velocity_after_gravity,
                player.velocity.y
            );
            
            prop_assert!(
                !player.is_grounded,
                "Player should not be grounded after jumping"
            );
        }

        #[test]
        fn test_property_3_gravity_application(
            pos_x in -1000.0f32..1000.0,
            pos_y in 1.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            initial_velocity_y in -50.0f32..50.0,
            dt in 0.001f32..0.1
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.velocity.y = initial_velocity_y;
            player.is_grounded = false;
            
            let velocity_before = player.velocity.y;
            
            controller.apply_gravity(&mut player, dt);
            
            let velocity_after = player.velocity.y;
            let expected_velocity = velocity_before - controller.gravity * dt;
            
            prop_assert!(
                (velocity_after - expected_velocity).abs() < 0.001,
                "Gravity should decrease vertical velocity by gravity * dt. Expected: {}, Got: {}",
                expected_velocity,
                velocity_after
            );
            
            prop_assert!(
                velocity_after < velocity_before,
                "Vertical velocity should decrease when gravity is applied. Before: {}, After: {}",
                velocity_before,
                velocity_after
            );
        }

        #[test]
        fn test_property_42_sprint_acceleration(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            initial_stamina in 30.0f32..100.0,
            elapsed_time in 0.0f32..1.0
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.stamina = initial_stamina;
            player.current_speed_multiplier = 1.0;
            player.is_grounded = true;
            
            let mut input = InputState::new();
            input.sprint = true;
            input.move_forward = true;
            
            let num_steps = 100;
            let dt = elapsed_time / num_steps as f32;
            
            for _ in 0..num_steps {
                controller.update_sprint_state(&mut player, &input, dt);
            }
            
            let expected_progress = (elapsed_time / controller.sprint_acceleration_time).min(1.0);
            let expected_multiplier = 1.0 + (controller.sprint_multiplier - 1.0) * expected_progress;
            
            prop_assert!(
                (player.current_speed_multiplier - expected_multiplier).abs() < 0.05,
                "Sprint acceleration should be smooth over {} seconds. Expected: {}, Got: {} after {} seconds",
                controller.sprint_acceleration_time,
                expected_multiplier,
                player.current_speed_multiplier,
                elapsed_time
            );
            
            prop_assert!(
                player.current_speed_multiplier >= 1.0 && player.current_speed_multiplier <= controller.sprint_multiplier,
                "Speed multiplier should be between 1.0 and sprint_multiplier. Got: {}",
                player.current_speed_multiplier
            );
        }

        #[test]
        fn test_property_43_stamina_depletion_rate(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            sprint_duration in 0.1f32..5.0
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.stamina = 100.0;
            player.is_sprinting = true;
            
            let initial_stamina = player.stamina;
            
            let num_steps = 100;
            let dt = sprint_duration / num_steps as f32;
            
            for _ in 0..num_steps {
                controller.update_stamina(&mut player, dt);
            }
            
            let expected_stamina_loss = controller.stamina_drain_rate * sprint_duration;
            let actual_stamina_loss = initial_stamina - player.stamina;
            
            prop_assert!(
                (actual_stamina_loss - expected_stamina_loss).abs() < 1.0,
                "Stamina depletion should be stamina_drain_rate * time. Expected loss: {}, Got: {}",
                expected_stamina_loss,
                actual_stamina_loss
            );
            
            let full_depletion_time = 100.0 / controller.stamina_drain_rate;
            prop_assert!(
                (full_depletion_time - 5.0).abs() < 0.5,
                "Full stamina depletion should take approximately 5 seconds. Got: {} seconds",
                full_depletion_time
            );
        }

        #[test]
        fn test_property_44_stamina_regeneration_rate(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            initial_stamina in 0.0f32..70.0,
            regen_duration in 0.1f32..3.0
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.stamina = initial_stamina;
            player.is_sprinting = false;
            
            let stamina_before = player.stamina;
            
            let num_steps = 100;
            let dt = regen_duration / num_steps as f32;
            
            for _ in 0..num_steps {
                controller.update_stamina(&mut player, dt);
            }
            
            let expected_stamina_gain = controller.stamina_regen_rate * regen_duration;
            let actual_stamina_gain = player.stamina - stamina_before;
            let capped_expected_gain = expected_stamina_gain.min(100.0 - stamina_before);
            
            prop_assert!(
                (actual_stamina_gain - capped_expected_gain).abs() < 1.0,
                "Stamina regeneration should be stamina_regen_rate * time. Expected gain: {}, Got: {}",
                capped_expected_gain,
                actual_stamina_gain
            );
            
            let full_regen_time = 100.0 / controller.stamina_regen_rate;
            prop_assert!(
                (full_regen_time - 3.0).abs() < 0.5,
                "Full stamina regeneration should take approximately 3 seconds. Got: {} seconds",
                full_regen_time
            );
        }

        #[test]
        fn test_property_45_sprint_lockout_threshold(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            stamina in 0.0f32..100.0,
            dt in 0.001f32..0.1
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.stamina = stamina;
            player.current_speed_multiplier = 1.0;
            player.is_grounded = true;
            
            let mut input = InputState::new();
            input.sprint = true;
            input.move_forward = true;
            
            controller.update_sprint_state(&mut player, &input, dt);
            
            let sprint_lockout_threshold = 30.0;
            
            if stamina < sprint_lockout_threshold {
                prop_assert!(
                    !player.is_sprinting,
                    "Player should not be able to sprint with stamina {} below threshold {}",
                    stamina,
                    sprint_lockout_threshold
                );
                
                prop_assert!(
                    player.current_speed_multiplier <= 1.0 + 0.01,
                    "Speed multiplier should not increase with stamina below threshold. Got: {}",
                    player.current_speed_multiplier
                );
            } else {
                prop_assert!(
                    player.is_sprinting,
                    "Player should be able to sprint with stamina {} at or above threshold {}",
                    stamina,
                    sprint_lockout_threshold
                );
            }
        }

        #[test]
        fn test_property_46_stamina_bounds(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            initial_stamina in 0.0f32..100.0,
            is_sprinting in any::<bool>(),
            duration in 0.1f32..10.0
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.stamina = initial_stamina;
            player.is_sprinting = is_sprinting;
            
            let num_steps = 100;
            let dt = duration / num_steps as f32;
            
            for _ in 0..num_steps {
                controller.update_stamina(&mut player, dt);
                
                prop_assert!(
                    player.stamina >= 0.0 && player.stamina <= 100.0,
                    "Stamina should always be within [0.0, 100.0]. Got: {}",
                    player.stamina
                );
            }
        }

        #[test]
        fn test_property_47_sprint_speed_consistency(
            pos_x in -1000.0f32..1000.0,
            pos_y in 0.0f32..100.0,
            pos_z in -1000.0f32..1000.0,
            rotation in 0.0f32..std::f32::consts::TAU,
            dt in 0.001f32..0.1
        ) {
            let controller = PlayerController::new();
            let mut player = Player::new(0, vec3(pos_x, pos_y, pos_z));
            player.rotation = rotation;
            player.stamina = 100.0;
            player.current_speed_multiplier = controller.sprint_multiplier;
            player.is_sprinting = true;
            player.is_grounded = true;
            
            let mut input = InputState::new();
            input.sprint = true;
            input.move_forward = true;
            
            let initial_pos = player.position;
            
            controller.update(&mut player, &input, dt);
            
            let horizontal_displacement = vec2(
                player.position.x - initial_pos.x,
                player.position.z - initial_pos.z
            ).length();
            
            let expected_speed = controller.move_speed * controller.sprint_multiplier * dt;
            
            prop_assert!(
                (horizontal_displacement - expected_speed).abs() < 0.01,
                "Sprint speed should equal base_move_speed * sprint_multiplier. Expected: {}, Got: {}",
                expected_speed,
                horizontal_displacement
            );
        }
    }
}
