use macroquad::{audio::{load_sound, play_sound_once, Sound}, prelude::*};

// Defines the data for a game object.
struct Player { pos: Vec2, size: Vec2 }
struct Bullet { pos: Vec2 }
struct Enemy { pos: Vec2 }

// Returns a smaller, centered collision box for an object.
fn get_hitbox(pos: Vec2, size: Vec2, inset: f32) -> Rect {
    Rect::new(
        pos.x + inset,
        pos.y + inset,
        size.x - inset * 2.0,
        size.y - inset * 2.0,
    )
}

// Checks for bullet-enemy collisions, removes hit objects, and returns the number of enemies killed.
fn handle_collisions(
    bullets: &mut Vec<Bullet>,
    enemies: &mut Vec<Enemy>,
    bullet_size: Vec2,
    enemy_size: Vec2,
    explosion_sound: &Sound,
) -> u32 {
    let mut enemies_killed = 0;
    bullets.retain(|bullet| {
        let bullet_rect = Rect::new(bullet.pos.x, bullet.pos.y, bullet_size.x, bullet_size.y);
        let mut hit_an_enemy = false;
        for enemy in enemies.iter_mut() {
            let enemy_hitbox = get_hitbox(enemy.pos, enemy_size, 8.0);
            if bullet_rect.overlaps(&enemy_hitbox) {
                play_sound_once(explosion_sound);
                enemy.pos.y = screen_height() + 100.0; // Mark enemy for deletion.
                hit_an_enemy = true;
                enemies_killed += 1;
                break;
            }
        }
        !hit_an_enemy // Remove bullet if it hit something.
    });
    enemies_killed
}

// Runs a single round of the game and returns the final score.
async fn play_game(
    player_texture: &Texture2D,
    enemy_texture: &Texture2D,
    shoot_sound: &Sound,
    explosion_sound: &Sound,
    game_over_sound: &Sound,
    background_texture: &Texture2D,
) -> u32 {
    let mut score = 0;

    // Game object state setup.
    let entity_size = vec2(64.0, 64.0);
    let mut player = Player {
        pos: vec2(screen_width() * 0.5 - entity_size.x / 2.0, screen_height() - entity_size.y - 10.0),
        size: entity_size,
    };
    let player_speed = 700.0;
    let mut bullets: Vec<Bullet> = vec![];
    let bullet_speed = 800.0;
    let bullet_size = vec2(10.0, 20.0);
    let mut enemies: Vec<Enemy> = vec![];
    let enemy_speed = 400.0;
    let enemy_size = entity_size;
    let mut spawn_timer = 0.5;
    let shoot_cooldown = 0.4;
    let mut shoot_timer = 0.0;

    // Main gameplay loop for one round.
    loop {
        let dt = get_frame_time();

        if shoot_timer > 0.0 {
            shoot_timer -= dt;
        }

        // Handle player input and movement.
        if is_key_down(KeyCode::Left) { player.pos.x -= player_speed * dt; }
        if is_key_down(KeyCode::Right) { player.pos.x += player_speed * dt; }
        if player.pos.x < 0.0 { player.pos.x = 0.0; }
        if player.pos.x > screen_width() - player.size.x { player.pos.x = screen_width() - player.size.x; }
        if is_key_down(KeyCode::Space) && shoot_timer <= 0.0{
            shoot_timer = shoot_cooldown;
            play_sound_once(shoot_sound);
            bullets.push(Bullet { pos: vec2(player.pos.x + player.size.x / 2.0 - bullet_size.x / 2.0, player.pos.y) });
        }

        // Update all object positions.
        for bullet in bullets.iter_mut() { bullet.pos.y -= bullet_speed * dt; }
        for enemy in enemies.iter_mut() { enemy.pos.y += enemy_speed * dt; }

        // Spawn new enemies on a timer.
        spawn_timer -= dt;
        if spawn_timer <= 0.0 {
            spawn_timer = 1.5;
            enemies.push(Enemy { pos: vec2(rand::gen_range(0.0, screen_width() - enemy_size.x), -enemy_size.y) });
        }

        // Process collisions and update score.
        let hits = handle_collisions(&mut bullets, &mut enemies, bullet_size, enemy_size, explosion_sound);
        score += hits;

        // Check for game over condition.
        let player_hitbox = get_hitbox(player.pos, player.size, 10.0);
        for enemy in &enemies {
            let enemy_hitbox = get_hitbox(enemy.pos, enemy_size, 8.0);
            if player_hitbox.overlaps(&enemy_hitbox) {
                play_sound_once(game_over_sound);
                return score; // End the game and return the score.
            }
        }

        // Remove off-screen enemies.
        enemies.retain(|enemy| enemy.pos.y < screen_height());

        // Draw everything to the screen.
        draw_texture_ex(background_texture, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        });
        draw_texture_ex(player_texture, player.pos.x, player.pos.y, WHITE, DrawTextureParams { dest_size: Some(player.size), ..Default::default() });
        for bullet in &bullets { draw_rectangle(bullet.pos.x, bullet.pos.y, bullet_size.x, bullet_size.y, RED); }
        for enemy in &enemies { draw_texture_ex(enemy_texture, enemy.pos.x, enemy.pos.y, WHITE, DrawTextureParams { dest_size: Some(enemy_size), ..Default::default() }); }

        // Draw the current score.
        draw_text(&format!("Score: {}", score), 20.0, 30.0, 30.0, WHITE);

        next_frame().await
    }
}

// Manages the overall application state (playing -> game over -> playing).
#[macroquad::main("Shooter Game")]
async fn main() {
    // Load all assets once at the start.
    let player_texture = load_texture("assets/player.png").await.unwrap();
    let enemy_texture = load_texture("assets/enemy.png").await.unwrap();
    let shoot_sound = load_sound("assets/shoot.wav").await.unwrap();
    let explosion_sound = load_sound("assets/short_explode.wav").await.unwrap();
    let game_over_sound = load_sound("assets/game_over.wav").await.unwrap();
    let background_texture = load_texture("assets/background_2.png").await.unwrap();

    // The main application loop.
    loop {
        // Start a game round and wait for it to end, capturing the final score.
        let final_score = play_game(&player_texture, &enemy_texture, &shoot_sound, &explosion_sound, &game_over_sound, &background_texture).await;

        // Display the "Game Over" screen until the user restarts.
        loop {
            // Draw the background and overlay.
            draw_texture_ex(&background_texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() });
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));

            // Draw text elements.
            let text = "GAME OVER";
            let text2 = "Press ENTER to play again";
            let final_score_text = format!("Final Score: {}", final_score);

            let text_dims = measure_text(text, None, 80, 1.0);
            draw_text(text, screen_width() / 2.0 - text_dims.width / 2.0, screen_height() / 2.0 - 40.0, 80.0, WHITE);

            let text_dims2 = measure_text(&final_score_text, None, 40, 1.0);
            draw_text(&final_score_text, screen_width() / 2.0 - text_dims2.width / 2.0, screen_height() / 2.0 + 40.0, 40.0, WHITE);

            let text_dims3 = measure_text(text2, None, 20, 1.0);
            draw_text(text2, screen_width() / 2.0 - text_dims3.width / 2.0, screen_height() / 2.0 + 80.0, 20.0, WHITE);

            // Check for restart input.
            if is_key_pressed(KeyCode::Enter) {
                break;
            }

            next_frame().await
        }
    }
}
