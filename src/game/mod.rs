use rand::Rng;
use raylib::prelude::*;

fn get_tile_color(x: i32, y: i32) -> Color {
    // A simple hash to get a consistent color for every coordinate
    let seed = (x * 12345) ^ (y * 67890);
    let r = (seed % 255).abs() as u8;
    let g = ((seed >> 8) % 255).abs() as u8;
    let b = ((seed >> 16) % 255).abs() as u8;
    Color::new(r, g, b, 255)
}

fn draw_tile(
    d2: &mut RaylibDrawHandle,
    gx: i32,
    gy: i32,
    tile_height: f32,
    tile_width: f32,
    fill: Option<Color>,
) {
    // 3. ISOMETRIC MATH
    let screen_x = (gx as f32 - gy as f32) * (tile_width / 2.0);
    let screen_y = (gx as f32 + gy as f32) * (tile_height / 2.0);

    print!("{} {}\n", gx - gy, gx + gy);

    // let color = get_tile_color(gx, gy);
    let color = if let Some(color) = fill {
        color
    } else {
        get_tile_color(gx, gy)
    };

    // Draw the tile (using your existing triangle logic)
    let top = Vector2::new(screen_x, screen_y);
    let right = Vector2::new(screen_x + tile_width / 2.0, screen_y + tile_height / 2.0);
    let bottom = Vector2::new(screen_x, screen_y + tile_height);
    let left = Vector2::new(screen_x - tile_width / 2.0, screen_y + tile_height / 2.0);

    d2.draw_triangle(top, left, bottom, color);
    d2.draw_triangle(top, bottom, right, color);

    // 2. Draw the coordinates text
    // We center the text by moving down half the tile height
    let label = format!("{},{}", gx, gy);

    // Measure text to center it perfectly
    let text_width = d2.measure_text(&label, 10);
    let text_x = (screen_x - (text_width as f32 / 2.0)) as i32;
    let text_y = (screen_y + (tile_height / 2.0) - 5.0) as i32; // -5 to account for font height

    d2.draw_text(&label, text_x, text_y, 10, Color::WHITE);

    // Draw the outline (This makes the grid visible)
    d2.draw_line_v(top, right, Color::WHITE);
    d2.draw_line_v(right, bottom, Color::WHITE);
    d2.draw_line_v(bottom, left, Color::WHITE);
    d2.draw_line_v(left, top, Color::WHITE);
}

struct Cube {
    origin: Vector2,
    top_left: Vector2,
    top_center: Vector2,
    top_right: Vector2,
    bottom_left: Vector2,
    bottom_center: Vector2,
    bottom_right: Vector2,
}

impl Cube {
    fn new(size: f32, origin: Vector2) -> Self {
        // Horizontal offset for the 'wings' of the cube
        let h_dist = size * 0.866; // approx cos(30 degrees)
                                   // Vertical offset for the perspective tilt
        let v_dist = size * 0.5; // approx sin(30 degrees)

        Self {
            origin,
            // Top vertices: These form the "diamond" shape at the top
            top_left: Vector2::new(origin.x - h_dist, origin.y - v_dist),
            top_center: Vector2::new(origin.x, origin.y - size),
            top_right: Vector2::new(origin.x + h_dist, origin.y - v_dist),

            // Bottom vertices: These are just the top vertices shifted down
            bottom_left: Vector2::new(origin.x - h_dist, origin.y + v_dist),
            bottom_center: Vector2::new(origin.x, origin.y + size),
            bottom_right: Vector2::new(origin.x + h_dist, origin.y + v_dist),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        // Top Face
        d.draw_triangle(
            self.origin,
            self.top_left,
            self.top_center,
            Color::ALICEBLUE,
        );
        d.draw_triangle(
            self.origin,
            self.top_right,
            self.top_center,
            Color::ALICEBLUE,
        );

        // Left Face
        d.draw_triangle(
            self.top_left,
            self.bottom_left,
            self.bottom_center,
            Color::AZURE,
        );
        d.draw_triangle(
            self.top_left,
            self.top_center,
            self.bottom_center,
            Color::AZURE,
        );

        // Right Face
        d.draw_triangle(
            self.top_right,
            self.bottom_right,
            self.bottom_center,
            Color::AZURE,
        );
        d.draw_triangle(
            self.top_right,
            self.top_center,
            self.bottom_center,
            Color::AZURE,
        );
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        let delta = Vector2::new(dx, dy);

        self.origin.x += delta.x;
        self.origin.y += delta.y;

        self.top_left.x += delta.x;
        self.top_left.y += delta.y;

        self.top_center.x += delta.x;
        self.top_center.y += delta.y;

        self.top_right.x += delta.x;
        self.top_right.y += delta.y;

        self.bottom_left.x += delta.x;
        self.bottom_left.y += delta.y;

        self.bottom_center.x += delta.x;
        self.bottom_center.y += delta.y;

        self.bottom_right.x += delta.x;
        self.bottom_right.y += delta.y;
    }
}

pub fn run() {
    let fps = 60;
    let screen_width = 1280;
    let screen_height = 720;
    let player_speed = 150.0;
    let mut show_outline = true;
    let mut show_target_lines = true;
    let mut show_tile_coordinates = true;

    let mut player = Rectangle {
        height: 30.0,
        width: 30.0,
        x: -15.0,
        y: -15.0,
    };

    let mut cube = Cube::new(20.0, Vector2::zero());

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Jungo, Hello World!")
        .build();

    let mut camera = raylib::camera::Camera2D {
        offset: Vector2 {
            x: (screen_width / 2) as f32,
            y: (screen_height / 2) as f32,
        },
        rotation: 0.0,
        target: Vector2 { x: 0.0, y: 0.0 },
        zoom: 1.0,
    };

    rl.set_target_fps(fps);

    while !rl.window_should_close() {
        let delta = rl.get_frame_time();
        let delta_player_speed = player_speed * delta;

        // UPDATE
        if rl.is_key_down(KeyboardKey::KEY_W) {
            player.y -= delta_player_speed;
            camera.target.y -= delta_player_speed;
        }

        if rl.is_key_down(KeyboardKey::KEY_A) {
            player.x -= delta_player_speed;
            camera.target.x -= delta_player_speed;
        }

        if rl.is_key_down(KeyboardKey::KEY_S) {
            player.y += delta_player_speed;
            camera.target.y += delta_player_speed;
        }

        if rl.is_key_down(KeyboardKey::KEY_D) {
            player.x += delta_player_speed;
            camera.target.x += delta_player_speed;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_O) {
            show_outline = !show_outline
        }

        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            show_target_lines = !show_target_lines
        }

        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            show_tile_coordinates = !show_tile_coordinates
        }

        // DRAW
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            let tile_width = 128.0;
            let tile_height = 64.0;

            // 2. CALCULATE VISIBLE RANGE
            // We find which tile is at the center of the camera and expand outwards
            let cam_x = camera.target.x;
            let cam_y = camera.target.y;

            // Convert camera pixels back to Isometric Grid coordinates (Inverse Math)
            // This tells us roughly which "index" we are looking at
            let central_grid_x = (cam_y / tile_height + cam_x / tile_width) as i32;
            let central_grid_y = (cam_y / tile_height - cam_x / tile_width) as i32;

            // Render a "culling box" around the camera (e.g., 15 tiles in every direction)
            let range = 15;

            for gx in (central_grid_x - range)..(central_grid_x + range) {
                for gy in (central_grid_y - range)..(central_grid_y + range) {
                    draw_tile(&mut d2, gx, gy, tile_height, tile_width, None);
                }
            }

            d2.draw_rectangle_rec(player, Color::PURPLE);

            // cube.draw(&mut d2);
        }

        if show_target_lines {
            // Horizontal Line
            d.draw_line(
                0,
                screen_height / 2,
                screen_width,
                screen_height / 2,
                Color::BLUE,
            );

            // Vertical Line
            d.draw_line(
                screen_width / 2,
                0,
                screen_width / 2,
                screen_height,
                Color::BLUE,
            );
        }
    }
}

// fn random_color() -> Color {
//     let mut rng = rand::rng();

//     Color {
//         r: rng.random_range(0..255),
//         g: rng.random_range(0..255),
//         b: rng.random_range(0..255),
//         a: 255,
//     }
// }
