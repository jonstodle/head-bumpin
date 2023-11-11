use comfy::winit::dpi::PhysicalSize;
use comfy::*;

comfy_game!(
    "Head Bumpin'",
    GameContext,
    GameState,
    make_context,
    setup,
    update
);

pub struct Mallet;
pub struct Enemy;

pub struct GameState {
    spawn_interval: f32,
    spawn_timer: f32,
    player_score: i32,
}

impl GameState {
    pub fn new(_: &mut EngineContext) -> Self {
        GameState {
            spawn_interval: 5.0,
            spawn_timer: 0.0,
            player_score: 0,
        }
    }
}

pub struct GameContext<'a, 'b: 'a> {
    delta: f32,
    spawn_interval: &'a mut f32,
    spawn_timer: &'a mut f32,
    player_score: &'a mut i32,
    pub engine: &'a mut EngineContext<'b>,
}

pub fn make_context<'a, 'b: 'a>(
    state: &'a mut GameState,
    engine: &'a mut EngineContext<'b>,
) -> GameContext<'a, 'b> {
    GameContext {
        delta: engine.delta,
        spawn_interval: &mut state.spawn_interval,
        spawn_timer: &mut state.spawn_timer,
        player_score: &mut state.player_score,
        engine,
    }
}

pub fn setup(c: &mut GameContext) {
    c.engine
        .load_texture_from_bytes("tilemap", include_bytes!("../assets/tilemap.png"));
    c.engine
        .load_texture_from_bytes("mallet", include_bytes!("../assets/mallet.png"));

    c.engine.renderer.window.set_resizable(false);
    c.engine
        .renderer
        .window
        .set_inner_size(PhysicalSize::new(1088, 768));
    c.engine.renderer.window.set_cursor_visible(false);

    // 15x10 tiles plus border
    for x in 0..17 {
        for y in 0..12 {
            let (tile_x, tile_y) = match (x, y) {
                (0, 0) => (1, 5),
                (16, 0) => (0, 5),
                (0, 11) => (2, 5),
                (16, 11) => (3, 5),
                (_, 0) => (1, 1),
                (_, 11) => (1, 3),
                (0, _) => (2, 2),
                (16, _) => (0, 2),
                _ => {
                    let variant = random_i32(0, 3);
                    (variant, 0)
                }
            };

            commands().spawn((
                Sprite::new("tilemap", splat(1.0), 0, WHITE).with_rect(
                    tile_x * 16,
                    tile_y * 16,
                    16,
                    16,
                ),
                Transform::position(vec2(x as f32, y as f32)),
            ))
        }
    }

    main_camera_mut().center = vec2(8.0, 5.5);
    main_camera_mut().zoom = 17.0;

    //17x5-9
    for x in 1..16 {
        for y in 1..11 {
            if random_i32(1, 10) != 1 {
                continue;
            }

            let variant = random_i32(5, 9);
            commands().spawn((
                Sprite::new("tilemap", splat(1.0), 10, WHITE).with_rect(
                    16 * 17,
                    16 * variant,
                    16,
                    16,
                ),
                Transform::position(vec2(x as f32, y as f32)),
                Enemy,
            ))
        }
    }

    commands().spawn((
        AnimatedSpriteBuilder::new()
            .flip_x(true)
            .z_index(100)
            .size(splat(2.0))
            .add_animation(
                "idle",
                0.1,
                true,
                AnimationSource::Atlas {
                    name: "mallet".into(),
                    offset: ivec2(56, 0),
                    step: ivec2(56, 0),
                    size: isplat(56),
                    frames: 1,
                },
            )
            .add_animation(
                "slam",
                0.06,
                true,
                AnimationSource::Atlas {
                    name: "mallet".into(),
                    offset: isplat(0),
                    step: ivec2(56, 0),
                    size: isplat(56),
                    frames: 4,
                },
            )
            .build(),
        Transform::position(vec2(8.0, 5.5)),
        Mallet,
    ));

    c.engine.load_sound_from_bytes(
        "slam",
        include_bytes!("../assets/bump.wav"),
        StaticSoundSettings::new().volume(0.3),
    )
}

fn update(c: &mut GameContext) {
    let mut slam = false;

    for (_, (_, sprite, transform)) in world()
        .query::<(&Mallet, &mut AnimatedSprite, &mut Transform)>()
        .iter()
    {
        let mouse_pos = mouse_world() + vec2(0.55, 0.75);
        transform.position = mouse_pos;

        if is_mouse_button_pressed(MouseButton::Left) {
            sprite.play("slam");
        };
        if sprite.state.animation_name == "slam" && sprite.state.progress() > 0.9 {
            sprite.play("idle");
            slam = true;
            play_sound("slam");
        }
    }

    let mut existing_coordinates = Vec::<Vec2>::with_capacity(100);
    for (entity, (_, transform)) in world().query::<(&Enemy, &Transform)>().iter() {
        existing_coordinates.push(transform.position);
        if slam && (transform.position - mouse_world()).length() < 0.7 {
            commands().despawn(entity);
            *c.player_score += 1;
        }
    }

    *c.spawn_timer += c.delta;
    if *c.spawn_timer > *c.spawn_interval {
        *c.spawn_timer = 0.0;

        if *c.player_score % 5 == 0 {
            *c.spawn_interval = (*c.spawn_interval * 0.8).max(0.2);
        }

        let (mut x, mut y, variant) = (random_i32(1, 16), random_i32(1, 11), random_i32(5, 9));
        while existing_coordinates.contains(&vec2(x as f32, y as f32)) {
            x = random_i32(1, 16);
            y = random_i32(1, 11);
        }

        commands().spawn((
            Sprite::new("tilemap", splat(1.0), 10, WHITE).with_rect(16 * 17, 16 * variant, 16, 16),
            Transform::position(vec2(x as f32, y as f32)),
            Enemy,
        ));
    }

    for (i, digit) in c
        .player_score
        .to_string()
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .enumerate()
    {
        draw_sprite_ex(
            texture_id("tilemap"),
            vec2((i as f32 / 2.0) - 0.5, 11.5),
            WHITE,
            11,
            DrawTextureParams {
                source_rect: Some(IRect::new(ivec2(16 * digit as i32, 16 * 10), isplat(16))),
                dest_size: Some(Size::world(1.0, 1.0)),
                ..Default::default()
            },
        )
    }

    if world().query::<&Enemy>().iter().count() > 100 {
        draw_text("Game Over!", main_camera().center, WHITE, TextAlign::Center);
    }
}
