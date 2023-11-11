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

pub struct Hammer;

pub struct GameState;

impl GameState {
    pub fn new(_: &mut EngineContext) -> Self {
        GameState {}
    }
}

pub struct GameContext<'a, 'b: 'a> {
    pub engine: &'a mut EngineContext<'b>,
}

pub fn make_context<'a, 'b: 'a>(
    _: &'a mut GameState,
    engine: &'a mut EngineContext<'b>,
) -> GameContext<'a, 'b> {
    GameContext { engine }
}

pub fn setup(c: &mut GameContext) {
    c.engine
        .load_texture_from_bytes("tilemap", include_bytes!("../assets/tilemap.png"));
    c.engine
        .load_texture_from_bytes("hammer", include_bytes!("../assets/hammer1.png"));

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

    commands().spawn((
        Sprite::new("hammer", splat(2.0), 1, WHITE).with_rect(0, 0, 16, 16),
        Transform::position(vec2(8.0, 5.5)),
        Hammer,
    ));

    main_camera_mut().center = vec2(8.0, 5.5);
    main_camera_mut().zoom = 17.0;
}

fn update(_: &mut GameContext) {
    for (_, (_, sprite, transform)) in world()
        .query::<(&Hammer, &mut Sprite, &mut Transform)>()
        .iter()
    {
        let mouse_pos = mouse_world() + vec2(0.7, 0.3);
        transform.position = mouse_pos;

        sprite.source_rect = if is_mouse_button_down(MouseButton::Left) {
            Some(IRect::new(ivec2(16, 0), isplat(16)))
        } else {
            Some(IRect::new(isplat(0), isplat(16)))
        };
    }
}
