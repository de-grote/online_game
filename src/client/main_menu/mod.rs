use bevy::color::palettes::css as color;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::net::ToSocketAddrs;

#[cfg(feature = "server")]
use crate::server;

use super::{
    despawn_screen,
    networking::{ConnectionAddress, MakeConnectionEvent},
    GameState, FONT,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TextSelectionState>()
            .init_resource::<ConnectionText>()
            .add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(
                Update,
                (
                    text_update_system.run_if(in_state(GameState::MainMenu)),
                    text_color_system.run_if(in_state(GameState::MainMenu)),
                    keyboard_input_system.run_if(in_state(GameState::MainMenu)),
                    select_ui.run_if(in_state(GameState::MainMenu)),
                    change_background.run_if(in_state(GameState::MainMenu)),
                    connection_text_input.run_if(in_state(TextSelectionState::Connection)),
                ),
            )
            .add_systems(OnExit(GameState::MainMenu), despawn_screen::<Menu>);
    }
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ColorText;

#[derive(Component)]
struct TextSelectionInput;

#[derive(States, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Component)]
enum TextSelectionState {
    #[default]
    None,
    Connection,
}

#[derive(Resource, Deref, DerefMut)]
struct ConnectionText(pub String);

impl Default for ConnectionText {
    fn default() -> Self {
        Self("127.0.0.1:1812".into())
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(
                    Srgba {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    }
                    .into(),
                ),
                ..default()
            },
            ..default()
        },
        Menu,
    ));

    commands.spawn((
        TextBundle::from_section(
            "epic chess game!\nenter for server + client\nspace for client only",
            TextStyle {
                font: asset_server.load(FONT),
                font_size: 100.0,
                color: color::WHITE.into(),
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,

            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        ColorText,
        Menu,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load(FONT),
                    font_size: 60.0,
                    color: color::WHITE.into(),
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load(FONT),
                font_size: 60.0,
                color: color::GOLD.into(),
            }),
        ]),
        FpsText,
        Menu,
    ));

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    min_width: Val::Px(100.0),
                    min_height: Val::Px(60.0),
                    ..default()
                },
                background_color: color::GRAY.into(),
                border_color: color::ALICE_BLUE.into(),
                ..default()
            },
            TextSelectionInput,
            TextSelectionState::Connection,
            Menu,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    ConnectionText::default().0,
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: 60.0,
                        color: color::WHITE.into(),
                    },
                ),
                TextSelectionInput,
            ));
        });
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in query.iter_mut() {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Srgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        }
        .into();
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut start_game: EventWriter<MakeConnectionEvent>,
    #[cfg(feature = "server")] server_port: Res<ConnectionAddress>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        #[cfg(feature = "server")]
        {
            let port = server_port.0;
            std::thread::spawn(move || server::start_server(port));
        }
        start_game.send(MakeConnectionEvent);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        start_game.send(MakeConnectionEvent);
    }
}

fn connection_text_input(
    mut evr_char: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut input: Query<&mut Text, With<TextSelectionInput>>,
    mut string: ResMut<ConnectionText>,
    mut address: ResMut<ConnectionAddress>,
) {
    let mut changed = false;
    for ev in evr_char.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        let control_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

        match (control_pressed, &ev.logical_key, &ev.key_code) {
            (true, Key::Backspace, _) | (true, _, KeyCode::Backspace) => {
                string.clear();
            }
            (false, Key::Backspace, _) | (false, _, KeyCode::Backspace) => {
                string.pop();
            }
            (true, _, KeyCode::KeyV) => {
                if let Ok(mut ctx) = ClipboardContext::new() {
                    if let Ok(clipboard) = ctx.get_contents() {
                        string.push_str(&clipboard);
                    }
                }
            }
            (true, _, KeyCode::KeyC) => {
                if let Ok(mut ctx) = ClipboardContext::new() {
                    let _ = ctx.set_contents(string.0.clone());
                }
            }
            (true, _, KeyCode::KeyX) => {
                if let Ok(mut ctx) = ClipboardContext::new() {
                    if let Ok(()) = ctx.set_contents(string.0.clone()) {
                        string.clear();
                    }
                }
            }
            (false, Key::Character(str), _) => {
                string.push_str(str);
            }
            (false, Key::Space, _) => {
                string.push(' ');
            }
            _ => (),
        }
        changed = true;
    }
    if changed {
        let input = input.single_mut().into_inner();
        input.sections[0].value.clone_from(&string);
        match string.to_socket_addrs().map(|mut p| p.next()) {
            Ok(Some(v)) => {
                *address = ConnectionAddress(v);
                input.sections[0].style.color = color::WHITE.into();
            }
            Err(_) | Ok(None) => input.sections[0].style.color = color::ORANGE_RED.into(),
        };
    }
}

fn change_background(
    mut input: Query<(&mut BackgroundColor, &TextSelectionState), With<TextSelectionInput>>,
    state: Res<State<TextSelectionState>>,
) {
    if state.is_changed() {
        for (mut b, tss) in input.iter_mut() {
            if *state == *tss {
                b.0 = color::GRAY.into();
            } else {
                b.0 = color::DARK_GRAY.into();
            }
        }
    }
}

fn select_ui(
    text_selection: Query<(&Interaction, &TextSelectionState)>,
    mut selection_state: ResMut<NextState<TextSelectionState>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        for (selection, state) in text_selection.iter() {
            if *selection == Interaction::Pressed {
                selection_state.set(*state);
            }
        }
        if !selection_state.is_changed() {
            selection_state.set(TextSelectionState::None);
        }
    }
}
