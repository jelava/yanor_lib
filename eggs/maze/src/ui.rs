pub mod cursor;
pub mod input;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .add_systems(Startup, spawn_left_sidebar);
    }
}

#[derive(Resource)]
pub struct Theme {
    background_color: Color,
    outer_border_color: Color,
    nested_border_color: Color,
    text_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.7),
            outer_border_color: Color::srgb(0.6, 0.6, 0.6),
            nested_border_color: Color::srgb(0.3, 0.3, 0.3),
            text_color: Color::srgb(0.8, 0.8, 0.8),
        }
    }
}

const BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.7);
const OUTER_BORDER_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
const NESTED_BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

#[derive(Component)]
struct LeftSidebar;

#[derive(Component)]
struct PlayerInfoBox;

#[derive(Component)]
struct PlayerIcon;

#[derive(Component)]
struct PlayerName;

#[derive(Component)]
struct PlayerHp;

#[derive(Component)]
struct PlayerSp;

#[derive(Component)]
struct CursorSelectionList;

fn spawn_left_sidebar(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_icon = asset_server.load("gobbo_icon.png");

    commands
        .spawn((
            LeftSidebar,
            Node {
                // min_width: Val::Px(200.0),
                width: Val::Percent(25.0),
                height: Val::Percent(100.0),
                border: UiRect::right(Val::Px(2.0)),
                // margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor::all(OUTER_BORDER_COLOR),
        ))
        .with_children(|left_sidebar| {
            left_sidebar
                .spawn((
                    PlayerInfoBox,
                    Node {
                        min_width: Val::Px(200.0),
                        width: Val::Percent(100.0),
                        // height: Val::Px(150.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                    BorderColor::all(NESTED_BORDER_COLOR),
                ))
                .with_children(|player_info_box| {
                    player_info_box
                        .spawn((
                            Node {
                                align_items: AlignItems::Center,
                                border: UiRect::bottom(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::all(NESTED_BORDER_COLOR),
                        ))
                        .with_children(|player_info_header| {
                            player_info_header.spawn((PlayerIcon, ImageNode::new(player_icon)));

                            player_info_header.spawn((
                                PlayerName,
                                Text::new("{Player name}"),
                                TextColor(TEXT_COLOR),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                            ));
                        });

                    player_info_box.spawn((
                        PlayerHp,
                        Text::new("HP: ##/## (###%)"),
                        TextColor(TEXT_COLOR),
                    ));

                    player_info_box.spawn((
                        PlayerSp,
                        Text::new("SP: ##/## (###%)"),
                        TextColor(TEXT_COLOR),
                    ));
                });

            left_sidebar.spawn((
                CursorSelectionList,
                Node {
                    min_width: Val::Px(200.0),
                    width: Val::Percent(100.0),
                    // height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                BorderColor::all(NESTED_BORDER_COLOR),
            ));
            // .with_children(|log_box| {
            //     log_box.spawn((
            //         Text::new("{log output will go here}"),
            //         TextColor(TEXT_COLOR),
            //         TextFont {
            //             font_size: 16.0,
            //             ..default()
            //         },
            //     ));
            // });
        });
}

// #[derive(Component)]
// pub struct UiPreviewInfo { pub display_name: String, pub icon: Option<Handle<Image>> }

// enum UiPreviewCategory {
//     Actors,
//     Items,
//     InteractableObjects,
//     Objects,
//     Misc
// }

// TODO: this is not flexible enough
#[derive(Component)]
pub enum UiPreviewKind {
    Block,
    Door,
    Player,
    Potion,
    Stairs,
}

impl UiPreviewKind {
    fn display_text(&self) -> String {
        use UiPreviewKind::*;

        match self {
            Block => "Block".into(),
            Door => "Door".into(),
            Player => "Player".into(),
            Potion => "Potion".into(),
            Stairs => "Stairs".into(),
        }
    }

    fn into_bundle(&self) -> impl Bundle {
        let node = Node {
            width: Val::Percent(100.0),
            // height: Val::Px(150.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(2.0)),
            // margin: UiRect::bottom(Val::Px(4.0)),
            // flex_direction: FlexDirection::Column,
            ..default()
        };

        let text_label = (
            Text::new(self.display_text()),
            // TextColor(TEXT_COLOR),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        );

        (node, children![text_label])
    }
}
