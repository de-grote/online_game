use bevy::{prelude::*, window::WindowResized};

use crate::api::{
    chessmove::{ChessColor, ChessMove, ChessPieceType, ChessboardLocation},
    chessstate::ChessState,
};

use super::{despawn_screen, GameState};

mod chess_pieces;
mod gameplay;
mod ui;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileSize>()
            .init_resource::<ChessState>()
            .init_resource::<ChessColor>()
            .init_resource::<SelectedPiece>()
            .add_event::<MoveEvent>()
            .add_event::<OpponentMoveEvent>()
            .add_event::<RedrawBoardEvent>()
            .add_event::<ResignEvent>()
            .add_event::<RequestDrawEvent>()
            .add_event::<DrawRequestedEvent>()
            .add_event::<PromotionEvent>()
            .add_event::<PromotionMoveEvent>()
            .add_event::<OpponentPromotionEvent>()
            .add_systems(
                OnEnter(GameState::Gaming),
                (setup, chess_pieces::spawn_chess_pieces, ui::setup),
            )
            .add_systems(
                Update,
                (
                    resize_notifier,
                    gameplay::select_piece.run_if(in_state(GameState::Gaming)),
                    gameplay::highlight_piece.run_if(in_state(GameState::Gaming)),
                    gameplay::resign.run_if(in_state(GameState::Gaming)),
                    gameplay::request_draw.run_if(in_state(GameState::Gaming)),
                    chess_pieces::move_chess_piece.run_if(in_state(GameState::Gaming)),
                    chess_pieces::respawn_chess_pieces.run_if(in_state(GameState::Gaming)),
                    resize_chessboard.run_if(in_state(GameState::Gaming)),
                    ui::turn_notifier.run_if(in_state(GameState::Gaming)),
                    ui::end_game.run_if(in_state(GameState::Gaming)),
                    ui::spawn_draw_message.run_if(in_state(GameState::Gaming)),
                    ui::despawn_messages.run_if(in_state(GameState::Gaming)),
                    ui::spawn_promotion_menu.run_if(in_state(GameState::Gaming)),
                    gameplay::clicked_promotion_menu.run_if(in_state(GameState::Gaming)),
                ),
            )
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

#[derive(Event)]
pub struct MoveEvent(pub ChessMove);

#[derive(Event)]
pub struct OpponentMoveEvent(pub ChessMove);

#[derive(Resource, Default, DerefMut, Deref, Debug)]
pub struct SelectedPiece(pub Option<ChessboardLocation>);

#[derive(Component)]
pub struct GameWindow;

#[derive(Component)]
pub struct Highlight;

#[derive(Component)]
pub struct ChessBoardComponent;

#[derive(Resource, Default)]
pub struct TileSize(pub f32);

#[derive(Event)]
pub struct RedrawBoardEvent;

#[derive(Event)]
pub struct ResignEvent;

#[derive(Event)]
pub struct RequestDrawEvent;

#[derive(Event)]
pub struct DrawRequestedEvent;

#[derive(Event)]
pub struct PromotionEvent;

#[derive(Event)]
pub struct PromotionMoveEvent(pub ChessPieceType);

#[derive(Event)]
pub struct OpponentPromotionEvent;

fn setup(mut commands: Commands) {
    commands.insert_resource(ChessState::default());

    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgba(0.3, 1.0, 1.0, 0.0)),
                ..default()
            },
            ..default()
        },
        GameWindow,
    ));

    // spawn chessboard
    for x in 0..8 {
        for y in 0..8 {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if (x + y) % 2 == 0 {
                            Color::srgb(0.0, 0.0, 0.0)
                        } else {
                            Color::srgb(1.0, 1.0, 1.0)
                        },
                        ..default()
                    },
                    ..default()
                },
                ChessboardLocation::new(x, y),
                ChessBoardComponent,
                GameWindow,
            ));
        }
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 0.0, 0.3),
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        ChessboardLocation::new(0, 0),
        Highlight,
        GameWindow,
    ));
}

fn resize_notifier(mut resize_event: EventReader<WindowResized>, mut tile_size: ResMut<TileSize>) {
    const BOARD_SIZE: f32 = 0.10;
    for e in resize_event.read() {
        tile_size.0 = e.width.min(e.height) * BOARD_SIZE;
    }
}

fn resize_chessboard(
    mut chessboard: Query<(&mut Transform, &mut ChessboardLocation)>,
    color: Res<ChessColor>,
    tile_size: Res<TileSize>,
) {
    for (mut sprite, location) in chessboard.iter_mut() {
        if !(location.is_changed() || tile_size.is_changed()) {
            continue;
        }
        sprite.scale = Vec3::splat(tile_size.0);
        match *color {
            ChessColor::White => {
                sprite.translation.x = (-3.5 + location.file as u8 as f32) * tile_size.0;
                sprite.translation.y = (-3.5 + location.rank as u8 as f32) * tile_size.0;
            }
            ChessColor::Black => {
                sprite.translation.x = (3.5 - location.file as u8 as f32) * tile_size.0;
                sprite.translation.y = (3.5 - location.rank as u8 as f32) * tile_size.0;
            }
        }
    }
}
