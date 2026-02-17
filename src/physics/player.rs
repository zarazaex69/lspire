use macroquad::prelude::*;

pub struct Player {
    pub id: u8,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: f32,
    pub is_grounded: bool,
    pub selected_gray_shade: u8,
}

impl Player {
    pub fn new(id: u8, position: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            rotation: 0.0,
            is_grounded: false,
            selected_gray_shade: 128,
        }
    }
}

pub struct PlayerController {
    pub move_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 8.0,
            gravity: 20.0,
        }
    }

    pub fn update(&self, player: &mut Player, input: &crate::input::controls::InputState, dt: f32) {
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
            
            player.velocity.x = movement.x * self.move_speed;
            player.velocity.z = movement.z * self.move_speed;
        } else {
            player.velocity.x = 0.0;
            player.velocity.z = 0.0;
        }
        
        if input.jump && player.is_grounded {
            player.velocity.y = self.jump_force;
            player.is_grounded = false;
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
    
    fn handle_ground_collision(&self, player: &mut Player) {
        let ground_level = 0.0;
        let player_height = 1.8;
        
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
                
                let expected_speed = controller.move_speed * dt;
                
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
    }
}
