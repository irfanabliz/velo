use std::collections::HashMap;

use bevy::prelude::*;
use bevy_pkv::PkvStore;

use super::ui_helpers::{DocListItemButton, ModalCancel, ModalConfirm, ModalEntity, ModalTop};
use crate::chart_plugin::UpdateListHighlight;
use crate::components::Doc;
use crate::resources::{AppState, LoadRequest};
use crate::utils::ReflectableUuid;
pub fn cancel_modal(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &ModalCancel),
        (Changed<Interaction>, With<ModalCancel>),
    >,
    mut state: ResMut<AppState>,
    query: Query<(Entity, &ModalTop), With<ModalTop>>,
) {
    for (interaction, path_modal_cancel) in interaction_query.iter_mut() {
        if let Interaction::Clicked = interaction {
            for (entity, path_modal_top) in query.iter() {
                if path_modal_cancel.id == path_modal_top.id {
                    commands.entity(entity).despawn_recursive();
                    state.modal_id = None;
                }
            }
        }
    }
}

pub fn confirm_modal(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &ModalConfirm),
        (Changed<Interaction>, With<ModalConfirm>),
    >,
    mut state: ResMut<AppState>,
    query_top: Query<(Entity, &ModalTop), With<ModalTop>>,
    mut query_button: Query<(Entity, &DocListItemButton), With<DocListItemButton>>,
    mut events: EventWriter<UpdateListHighlight>,
    mut pkv: ResMut<PkvStore>,
) {
    for (interaction, path_modal_confirm) in interaction_query.iter_mut() {
        if let Interaction::Clicked = interaction {
            for (entity, path_modal_top) in query_top.iter() {
                if path_modal_confirm.id == path_modal_top.id {
                    let current_document = state.current_document.unwrap();
                    if path_modal_confirm.delete == ModalEntity::Tab
                        && state.docs.get_mut(&current_document).unwrap().tabs.len() > 1
                    {
                        let index = state
                            .docs
                            .get_mut(&current_document)
                            .unwrap()
                            .tabs
                            .iter()
                            .position(|x| x.is_active)
                            .unwrap();
                        state
                            .docs
                            .get_mut(&current_document)
                            .unwrap()
                            .tabs
                            .remove(index);
                        let mut last_tab = state
                            .docs
                            .get_mut(&current_document)
                            .unwrap()
                            .tabs
                            .last_mut()
                            .unwrap();
                        last_tab.is_active = true;
                        commands.insert_resource(LoadRequest {
                            doc_id: None,
                            drop_last_checkpoint: false,
                        });
                    }
                    if path_modal_confirm.delete == ModalEntity::Document && state.docs.len() > 1 {
                        let id_to_remove = current_document;
                        for (entity, button) in query_button.iter_mut() {
                            if button.id == id_to_remove {
                                commands.entity(entity).despawn_recursive();
                            }
                        }
                        state.docs.remove(&current_document);
                        for (_, button) in query_button.iter_mut() {
                            if button.id != id_to_remove {
                                state.current_document = Some(button.id);
                                break;
                            }
                        }
                        commands.insert_resource(LoadRequest {
                            doc_id: None,
                            drop_last_checkpoint: false,
                        });
                        events.send(UpdateListHighlight);
                        remove_from_pkv(&mut pkv, id_to_remove, state.current_document.unwrap());
                    }
                    commands.entity(entity).despawn_recursive();
                    state.modal_id = None;
                }
            }
        }
    }
}

pub fn modal_keyboard_input_system(
    mut state: ResMut<AppState>,
    input: Res<Input<KeyCode>>,
    query_top: Query<(Entity, &ModalTop), With<ModalTop>>,
    mut commands: Commands,
    mut query_button: Query<(Entity, &DocListItemButton), With<DocListItemButton>>,
    mut events: EventWriter<UpdateListHighlight>,
    mut pkv: ResMut<PkvStore>,
) {
    if input.just_pressed(KeyCode::Return) {
        for (entity, path_modal_top) in query_top.iter() {
            if Some(path_modal_top.id) == state.modal_id {
                let current_document = state.current_document.unwrap();
                if path_modal_top.delete == ModalEntity::Tab
                    && state.docs.get_mut(&current_document).unwrap().tabs.len() > 1
                {
                    let index = state
                        .docs
                        .get_mut(&current_document)
                        .unwrap()
                        .tabs
                        .iter()
                        .position(|x| x.is_active)
                        .unwrap();
                    state
                        .docs
                        .get_mut(&current_document)
                        .unwrap()
                        .tabs
                        .remove(index);
                    let mut last_tab = state
                        .docs
                        .get_mut(&current_document)
                        .unwrap()
                        .tabs
                        .last_mut()
                        .unwrap();
                    last_tab.is_active = true;
                    commands.insert_resource(LoadRequest {
                        doc_id: None,
                        drop_last_checkpoint: false,
                    });
                }
                if path_modal_top.delete == ModalEntity::Document && state.docs.len() > 1 {
                    let id_to_remove = current_document;
                    for (entity, button) in query_button.iter_mut() {
                        if button.id == id_to_remove {
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                    state.docs.remove(&current_document);
                    for (_, button) in query_button.iter_mut() {
                        if button.id != id_to_remove {
                            state.current_document = Some(button.id);
                            break;
                        }
                    }
                    commands.insert_resource(LoadRequest {
                        doc_id: None,
                        drop_last_checkpoint: false,
                    });
                    events.send(UpdateListHighlight);
                    remove_from_pkv(&mut pkv, id_to_remove, state.current_document.unwrap());
                }
                commands.entity(entity).despawn_recursive();
                state.modal_id = None;
            }
        }
    }
}

fn remove_from_pkv(
    pkv: &mut ResMut<PkvStore>,
    id_to_remove: ReflectableUuid,
    new_id: ReflectableUuid,
) {
    if let Ok(mut docs) = pkv.get::<HashMap<ReflectableUuid, Doc>>("docs") {
        if docs.remove(&id_to_remove).is_some() {
            pkv.set("docs", &docs).unwrap();
        }
    }
    if let Ok(mut tags) = pkv.get::<HashMap<ReflectableUuid, Vec<String>>>("tags") {
        if tags.remove(&id_to_remove).is_some() {
            pkv.set("tags", &tags).unwrap();
        }
    }
    if let Ok(mut tags) = pkv.get::<HashMap<ReflectableUuid, String>>("names") {
        if tags.remove(&id_to_remove).is_some() {
            pkv.set("names", &tags).unwrap();
        }
    }
    if let Ok(last_saved) = pkv.get::<ReflectableUuid>("last_saved") {
        if last_saved == id_to_remove {
            pkv.set("last_saved", &new_id).unwrap();
        }
    }
}
