use bevy::render::pass::ClearColor;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use rand::{prelude::SliceRandom, thread_rng};
use std::collections::{HashMap, HashSet};

mod animation;

const UI_OFFSET: u32 = 100;
const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = WINDOW_WIDTH + UI_OFFSET;
const PADDING: u32 = 25;

fn main() {
    env_logger::init();
    App::build()
        .add_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            title: String::from("Squares - Bevy Edition"),
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(Msaa { samples: 1 })
        .add_resource(ClearColor(Color::rgb(
            255. / 255.,
            211. / 255.,
            182. / 255.,
        )))
        .add_resource(GameState {
            state: RunningGameState::Running,
            event_reader: Default::default(),
        })
        .add_event::<ScoreChange>()
        .add_event::<RunningGameState>()
        .init_resource::<ScoreState>()
        .add_default_plugins()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(animation::AnimationPlugin)
        .add_startup_system(setup.system())
        .add_system_to_stage(
            bevy::app::stage::PRE_UPDATE,
            handle_game_state_updates.system(),
        )
        .add_system(move_squares.system())
        .add_system(update_score_text.system())
        .add_system(text_update_system.system())
        .add_system_to_stage(bevy::app::stage::POST_UPDATE, update_colors.system())
        .add_system_to_stage(bevy::app::stage::POST_UPDATE, kill_after_update.system())
        .add_system_to_stage(
            bevy::app::stage::POST_UPDATE,
            sync_square_grid_position.system(),
        )
        .run();
}

#[derive(PartialEq)]
enum RunningGameState {
    Running,
    GameOver,
}

struct GameState {
    state: RunningGameState,
    event_reader: EventReader<RunningGameState>,
}

fn handle_game_state_updates(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    game_events: Res<Events<RunningGameState>>,
) {
    for event in game_state.event_reader.iter(&game_events) {
        match event {
            RunningGameState::GameOver => {
                let font_handle = asset_server
                    .load("assets/bungee_inline_regular.ttf")
                    .unwrap();
                commands
                    .spawn(NodeComponents {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: Rect::all(Val::Percent(25.)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        material: materials.add(Color::NONE.into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextComponents {
                            style: Style {
                                margin: Rect::all(Val::Px(5.0)),
                                ..Default::default()
                            },
                            text: Text {
                                value: "GAME OVER".to_string(),
                                font: font_handle.clone(),
                                style: TextStyle {
                                    font_size: 40.0,
                                    color: Color::rgb(0., 0., 0.),
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    });
                game_state.state = RunningGameState::GameOver;
            }
            _ => continue,
        }
    }
}

struct FPS;

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<With<FPS, &mut Text>>) {
    for mut text in &mut query.iter() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.2}", average);
            }
        }
    }
}

const GRID_SIZE: u32 = 4;
const SQUARE_WIDTH: u32 = 125;
const SQUARE_MARGIN: u32 =
    ((WINDOW_WIDTH - 2 * PADDING) - GRID_SIZE * SQUARE_WIDTH) / (GRID_SIZE - 1);

fn calculate_grid_position(x: u32, y: u32) -> (f32, f32) {
    assert!(x < GRID_SIZE && y < GRID_SIZE);
    (
        (PADDING + (SQUARE_MARGIN + SQUARE_WIDTH) * x) as f32,
        (UI_OFFSET + PADDING + (SQUARE_MARGIN + SQUARE_WIDTH) * y) as f32,
    )
}

struct SquareColors(HashMap<u64, Handle<ColorMaterial>>);

impl SquareColors {
    fn new(materials: &mut Assets<ColorMaterial>) -> SquareColors {
        let mut map = HashMap::new();

        map.insert(0, materials.add(Color::rgb(1., 1., 1.).into()));
        for (i, score) in [1, 2, 4, 8, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8172]
            .iter()
            .enumerate()
        {
            map.insert(*score, {
                let rgb = bracket_color::prelude::HSV::from_f32(i as f32 / 13., 0.8, 0.95).to_rgb();

                materials.add(Color::rgb(rgb.r, rgb.g, rgb.b).into())
            });
        }

        SquareColors(map)
    }

    fn get(&self, score: u64) -> Handle<ColorMaterial> {
        self.0
            .get(&score)
            .unwrap_or_else(|| self.0.get(&0).unwrap())
            .clone()
    }
}

enum ScoreChange {
    Add(u64),
}

#[derive(Default)]
struct ScoreState {
    score: u64,
    event_reader: EventReader<ScoreChange>,
}

struct ScoreText;

fn update_score_text(
    mut score: ResMut<ScoreState>,
    score_events: Res<Events<ScoreChange>>,
    mut score_query: Query<With<ScoreText, (&mut Text,)>>,
) {
    for score_change in score.event_reader.iter(&score_events) {
        match score_change {
            ScoreChange::Add(s) => score.score += s,
        }
    }
    for (mut text,) in &mut score_query.iter() {
        text.value = score.score.to_string();
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
struct GridPosition(u32, u32);
#[derive(Debug)]
struct Grid(Vec<Option<Square>>);
#[derive(Clone, PartialEq, Debug)]
struct Square {
    entity: Entity,
    score: u64,
}

impl Grid {
    fn new() -> Grid {
        Grid(vec![None; (GRID_SIZE * GRID_SIZE) as usize])
    }

    fn add_at(&mut self, (x, y): (u32, u32), square: Square) {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            panic!("Tried to add to a position outside of grid: {} {}", x, y);
        }
        let pos = (x + GRID_SIZE * y) as usize;
        self.0[pos] = Some(square);
    }

    fn get_at(&self, (x, y): (u32, u32)) -> Option<&Square> {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            return None;
        }
        let pos = (x + GRID_SIZE * y) as usize;
        self.0.get(pos).map(Option::as_ref).flatten()
    }

    fn take_at(&mut self, (x, y): (u32, u32)) -> Option<Square> {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            return None;
        }
        let pos = (x + GRID_SIZE * y) as usize;
        let mut inserted = None;
        std::mem::swap(&mut inserted, self.0.get_mut(pos).unwrap());
        inserted
    }

    fn is_filled(&self, (x, y): (u32, u32)) -> bool {
        self.get_at((x, y)).is_some()
    }

    fn move_to(&mut self, square: Square, new_pos: (u32, u32)) {
        for ent in self.0.iter_mut() {
            if ent.as_ref().map(|x| &x.entity) == Some(&square.entity) {
                *ent = None;
            }
        }
        self.add_at(new_pos, square);
    }

    fn get_neighbors(&self, pos: (u32, u32)) -> [Option<&Square>; 4] {
        [
            self.get_at((pos.0 + 1, pos.1)),
            self.get_at((pos.0 - 1, pos.1)),
            self.get_at((pos.0, pos.1 + 1)),
            self.get_at((pos.0, pos.1 - 1)),
        ]
    }
}

struct KillAfter {
    timer: Timer,
}

impl KillAfter {
    fn new(duration: f32) -> KillAfter {
        KillAfter {
            timer: Timer::from_seconds(duration, false),
        }
    }
}

fn kill_after_update(
    mut commands: Commands,
    time: Res<Time>,
    mut kill_query: Query<With<GridPosition, (Entity, &mut KillAfter)>>,
) {
    for (entity, mut kill_after) in &mut kill_query.iter() {
        kill_after.timer.tick(time.delta_seconds);

        if kill_after.timer.finished {
            commands.despawn_recursive(entity);
        }
    }
}

struct BackgroundSquare;
struct GameSquare;

fn spawn_square(
    commands: &mut Commands,
    grid: &mut Grid,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    colors: &SquareColors,
    pos: (u32, u32),
    score: u64,
) {
    let (x, y) = calculate_grid_position(pos.0, pos.1);
    let color = colors.get(score);
    let entity = commands
        .spawn((
            GameSquare,
            GridPosition(pos.0, pos.1),
            Translation::new(x, y, 1.0),
            Transform::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn(primitive(
                    color,
                    &mut meshes,
                    ShapeType::RoundedRectangle {
                        width: SQUARE_WIDTH as f32,
                        height: SQUARE_WIDTH as f32,
                        border_radius: SQUARE_WIDTH as f32 * 0.25,
                    },
                    TessellationMode::Fill(&FillOptions::default()),
                    Vec3::new(
                        -(WINDOW_WIDTH as f32) / 2.0,
                        -(WINDOW_HEIGHT as f32) / 2.0 - UI_OFFSET as f32,
                        0.,
                    )
                    .into(),
                ))
                .with(Scale::default());
        })
        .current_entity()
        .unwrap();
    grid.add_at(pos, Square { score, entity });
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server
        .load("assets/bungee_inline_regular.ttf")
        .unwrap();

    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    let mut grid = Grid::new();

    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let (x_pos, y_pos) = calculate_grid_position(x, y);
            commands
                .spawn((
                    GridPosition(x, y),
                    BackgroundSquare,
                    Transform::default(),
                    Translation::new(x_pos, y_pos, 0.),
                ))
                .with_children(|parent| {
                    parent.spawn(SpriteComponents {
                        material: materials.add(Color::rgba(0., 0., 0., 0.2).into()),
                        draw: Draw {
                            is_transparent: true,
                            ..Default::default()
                        },
                        sprite: Sprite {
                            size: Vec2::new(SQUARE_WIDTH as f32 + 2.0, SQUARE_WIDTH as f32 + 2.0),
                        },
                        translation: Translation::new(
                            (SQUARE_WIDTH as f32 - WINDOW_WIDTH as f32) / 2.0,
                            (SQUARE_WIDTH as f32 - WINDOW_HEIGHT as f32) / 2.0 - UI_OFFSET as f32,
                            0.,
                        ),
                        scale: Scale(1.05),
                        ..Default::default()
                    });
                });
        }
    }

    let square_colors = SquareColors::new(&mut materials);

    spawn_square(
        &mut commands,
        &mut grid,
        &mut meshes,
        &square_colors,
        (1, 2),
        1,
    );
    spawn_square(
        &mut commands,
        &mut grid,
        &mut meshes,
        &square_colors,
        (2, 1),
        2,
    );
    commands.insert_resource(grid);
    commands.insert_resource(square_colors);

    commands
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Px(UI_OFFSET as f32)),
                margin: Rect {
                    bottom: Val::Auto,
                    ..Default::default()
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextComponents {
                    style: Style {
                        margin: Rect::all(Val::Px(5.0)),
                        ..Default::default()
                    },
                    text: Text {
                        value: "Score".to_string(),
                        font: font_handle.clone(),
                        style: TextStyle {
                            font_size: 20.0,
                            color: Color::rgb(204. / 255., 112. / 255., 119. / 255.),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .spawn(TextComponents {
                    style: Style {
                        margin: Rect::all(Val::Px(5.0)),
                        ..Default::default()
                    },
                    text: Text {
                        value: "0".to_string(),
                        font: font_handle.clone(),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(76. / 255., 42. / 255., 44. / 255.),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(ScoreText);
        })
        .spawn(TextComponents {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: font_handle,
                style: TextStyle {
                    font_size: 12.0,
                    color: Color::BLACK,
                },
            },
            ..Default::default()
        })
        .with(FPS);
}

fn update_colors(
    colors: Res<SquareColors>,
    grid: Res<Grid>,
    mut query: Query<With<GameSquare, (&GridPosition, &Children)>>,
    material_query: Query<(&mut Handle<ColorMaterial>, &Sprite)>,
) {
    for (position, children) in &mut query.iter() {
        let square = grid.get_at((position.0, position.1)).unwrap();
        let color = colors.get(square.score);

        for &child in children.as_slice() {
            let mut handle = material_query
                .get_mut::<Handle<ColorMaterial>>(child)
                .unwrap();
            *handle = color.clone();
        }
    }
}

fn sync_square_grid_position(
    mut commands: Commands,
    mut query: Query<With<GameSquare, (Entity, Changed<GridPosition>, &Translation)>>,
) {
    for (entity, pos, translation) in &mut query.iter() {
        let (x, y) = calculate_grid_position(pos.0, pos.1);
        let move_to = animation::MoveTo {
            start_position: *translation,
            end_position: Translation::new(x, y, 1.0),
            duration: 0.15,
            ease: animation::Easing::EaseOutBack,
            ..Default::default()
        };

        commands.insert_one(entity, move_to);
    }
}

enum MovementDirection {
    Up,
    Down,
    Right,
    Left,
}

fn move_squares(
    mut commands: Commands,
    game_state: Res<GameState>,
    colors: Res<SquareColors>,
    mut grid: ResMut<Grid>,
    mut score_events: ResMut<Events<ScoreChange>>,
    mut game_events: ResMut<Events<RunningGameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<With<GameSquare, (&mut GridPosition, &Children)>>,
    mut background_query: Query<With<BackgroundSquare, (Entity, &GridPosition, &Translation)>>,
) {
    if game_state.state == RunningGameState::GameOver {
        return;
    }

    let direction = if keyboard_input.just_pressed(KeyCode::Up) {
        MovementDirection::Up
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        MovementDirection::Down
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        MovementDirection::Right
    } else if keyboard_input.just_pressed(KeyCode::Left) {
        MovementDirection::Left
    } else {
        return;
    };

    let x_iter: Box<dyn Iterator<Item = u32>> = match direction {
        MovementDirection::Up | MovementDirection::Left => Box::new(0..GRID_SIZE),
        MovementDirection::Down | MovementDirection::Right => Box::new((0..GRID_SIZE).rev()),
    };

    let mut scores = HashSet::new();
    let mut moved_squares = HashSet::new();

    for x in x_iter {
        let y_iter: Box<dyn Iterator<Item = u32>> = match direction {
            MovementDirection::Up | MovementDirection::Left => Box::new((0..GRID_SIZE).rev()),
            MovementDirection::Down | MovementDirection::Right => Box::new(0..GRID_SIZE),
        };
        for y in y_iter {
            let square = if let Some(square) = grid.get_at((x, y)) {
                square
            } else {
                continue;
            };

            let mut pos = query.get_mut::<GridPosition>(square.entity).unwrap();

            scores.insert(square.score);

            let mut new_pos: GridPosition = *pos;

            match direction {
                MovementDirection::Up => {
                    new_pos.1 = (pos.1 + 1).min(GRID_SIZE - 1);
                }
                MovementDirection::Down => {
                    new_pos.1 = pos.1.saturating_sub(1);
                }
                MovementDirection::Right => {
                    new_pos.0 = (pos.0 + 1).min(GRID_SIZE - 1);
                }
                MovementDirection::Left => {
                    new_pos.0 = pos.0.saturating_sub(1);
                }
            }

            if *pos == new_pos {
                continue;
            }

            let mut square_score = square.score;

            if let Some(other) = grid.get_at((new_pos.0, new_pos.1)) {
                let other_square = grid.get_at((new_pos.0, new_pos.1)).unwrap();
                if other_square.score != square.score {
                    continue;
                }

                commands.insert_one(other.entity, KillAfter::new(0.15));
                for &child in query.get::<Children>(other.entity).unwrap().as_slice() {
                    commands.insert_one(
                        child,
                        animation::ScaleTo {
                            start_scale: Scale(1.),
                            end_scale: Scale(0.),
                            duration: 0.075,
                            ..Default::default()
                        },
                    );
                }
                commands.remove_one::<GridPosition>(other.entity);
                square_score += other_square.score;
                score_events.send(ScoreChange::Add(square_score));
            }

            let mut square = grid.take_at((pos.0, pos.1)).unwrap();
            square.score = square_score;

            *pos = new_pos;
            grid.move_to(square, (pos.0, pos.1));
            moved_squares.insert(*pos);
        }
    }

    for (entity, grid_pos, translation) in &mut background_query.iter() {
        if moved_squares.contains(grid_pos) {
            let mag = 5.;
            let bump = match direction {
                MovementDirection::Up => Vec3::new(0., 1., 0.),
                MovementDirection::Down => Vec3::new(0., -1., 0.),
                MovementDirection::Right => Vec3::new(1., 0., 0.),
                MovementDirection::Left => Vec3::new(-1., 0., 0.),
            };
            let end_position = translation.0 + bump * mag;

            let (x, y) = calculate_grid_position(grid_pos.0, grid_pos.1);
            commands.insert_one(
                entity,
                animation::MoveTo {
                    start_position: Translation::new(x, y, 0.),
                    end_position: Translation(end_position),
                    duration: 0.1,
                    bounce: true,
                    loop_count: 1,
                    ..Default::default()
                },
            );
        }
    }

    let mut rng = thread_rng();

    let mut possible_coords: Vec<u32> = (0..GRID_SIZE).collect();
    let coords = loop {
        if possible_coords.is_empty() {
            break None;
        }

        possible_coords.shuffle(&mut rng);

        let random_pos = possible_coords.pop().unwrap();

        let coords = match direction {
            MovementDirection::Up => (random_pos, 0),
            MovementDirection::Down => (random_pos, GRID_SIZE - 1),
            MovementDirection::Right => (0, random_pos),
            MovementDirection::Left => (GRID_SIZE - 1, random_pos),
        };

        if grid.is_filled(coords) {
            continue;
        } else {
            break Some(coords);
        }
    };

    if let Some(coords) = coords {
        let mut scores: Vec<u64> = scores.iter().copied().collect();

        scores.sort();

        scores.truncate((scores.len() / 3).max(1));

        spawn_square(
            &mut commands,
            &mut grid,
            &mut meshes,
            &colors,
            coords,
            *scores.choose(&mut rng).unwrap(),
        );
    } else {
        let mut found = false;
        'outer: for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if let Some(square) = grid.get_at((x, y)) {
                    let neighbors = grid.get_neighbors((x, y));

                    for neighbor in &neighbors {
                        if let Some(neighbor) = neighbor {
                            if neighbor.score == square.score {
                                found = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if !found {
            game_events.send(RunningGameState::GameOver);
        }
    }
}
