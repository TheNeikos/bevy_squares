use bevy::prelude::*;
use bevy::render::pass::ClearColor;

mod text_sprite;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 600,
            height: 600,
            title: String::from("2048 - Bevy Edition"),
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_system_to_stage(
            bevy::ui::stage::UI,
            text_sprite::update_text_sprites.system(),
        )
        .add_system_to_stage(
            bevy::render::stage::DRAW,
            text_sprite::draw_text_sprites.system(),
        )
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server
        .load("assets/bungee_inline_regular.ttf")
        .unwrap();

    commands
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.5)))
        .spawn(Camera2dComponents::default())
        .spawn((
            text_sprite::TextSprite {
                text: String::from("Hello World"),
                text_style: TextStyle {
                    font_size: 10.0,
                    color: Color::WHITE,
                },
                font: font_handle.clone(),
                ..Default::default()
            },
            Draw::default(),
            Transform::default(),
            Translation::new(0., 0., 0.),
            text_sprite::TextSpriteSize::default(),
        ))
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
            translation: Translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(120.0, 30.0),
            },
            ..Default::default()
        });
}
