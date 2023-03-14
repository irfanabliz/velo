use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Rectangle {
    pub id: u32,
}

#[derive(Component)]
pub struct CreateRectButton;

#[derive(Component)]
pub struct EditableText {
    pub id: u32,
}

#[derive(Component)]
pub struct Top {
    pub id: u32,
}
pub struct AddRect;

#[derive(Resource, Default)]
pub struct AppState {
    pub entity_to_edit: Option<u32>,
    pub focused_entity: Option<u32>,
    pub entity_counter: u32,
    pub entity_to_resize: Option<(u32, ResizeMarker)>,
}

#[derive(Component)]
pub struct ArrowConnectMarker;

#[derive(Component, Copy, Clone, Debug)]
pub enum ResizeMarker {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
