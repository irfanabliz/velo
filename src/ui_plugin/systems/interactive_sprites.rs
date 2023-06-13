use bevy::{prelude::*, window::PrimaryWindow};

use crate::{components::MainCamera, utils::get_timestamp};

use std::time::Duration;

use super::{ui_helpers::InteractiveNode, NodeInteractionEvent, NodeInteractionType};

#[derive(Default)]
pub struct HoldingState {
    duration: Duration,
    entity: Option<Entity>,
    is_holding: bool,
}

pub fn interactive_sprite(
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    res_images: Res<Assets<Image>>,
    mut sprite_query: Query<
        (&Sprite, &Handle<Image>, &GlobalTransform, Entity),
        With<InteractiveNode>,
    >,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut node_interaction_events: EventWriter<NodeInteractionEvent>,
    mut double_click: Local<(Duration, Option<Entity>)>,
    mut holding_state: Local<HoldingState>,
) {
    let (camera, camera_transform) = camera_q.single();
    let window = windows.single();
    let scale_factor = window.scale_factor() as f32;
    let mut active_entity = None;
    for _ in cursor_moved_events.iter() {
        for (sprite, handle, node_transform, entity) in &mut sprite_query.iter_mut() {
            let size = match sprite.custom_size {
                Some(size) => (size.x, size.y),
                None => {
                    if let Some(sprite_image) = res_images.get(handle) {
                        (
                            sprite_image.size().x / scale_factor,
                            sprite_image.size().y / scale_factor,
                        )
                    } else {
                        (1., 1.)
                    }
                }
            };

            let x_min = node_transform.affine().translation.x - size.0 / 2.;
            let y_min = node_transform.affine().translation.y - size.1 / 2.;
            let x_max = node_transform.affine().translation.x + size.0 / 2.;
            let y_max = node_transform.affine().translation.y + size.1 / 2.;
            let z_current = node_transform.affine().translation.z;

            window.cursor_position().and_then(|pos| {
                Some({
                    if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                        if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                            if let Some((_, z)) = active_entity {
                                if z < z_current {
                                    active_entity = Some((entity, z_current));
                                }
                            } else {
                                active_entity =
                                    Some((entity, node_transform.affine().translation.z));
                            }
                        }
                    }
                })
            });
        }
    }
    if let Some((active, _)) = active_entity {
        let now_ms = get_timestamp();

        if buttons.just_pressed(MouseButton::Left) {
            if double_click.1 == Some(active)
                && Duration::from_millis(now_ms as u64) - double_click.0
                    < Duration::from_millis(500)
            {
                node_interaction_events.send(NodeInteractionEvent {
                    entity: active,
                    node_interaction_type: NodeInteractionType::LeftDoubleClick,
                });
            } else {
                node_interaction_events.send(NodeInteractionEvent {
                    entity: active,
                    node_interaction_type: NodeInteractionType::LeftClick,
                });
                *double_click = (Duration::from_millis(now_ms as u64), Some(active));
                *holding_state = HoldingState {
                    duration: Duration::from_millis(now_ms as u64),
                    entity: Some(active),
                    is_holding: false,
                };
            }

            // RETURN
            return;
        }
        if buttons.just_pressed(MouseButton::Right) {
            node_interaction_events.send(NodeInteractionEvent {
                entity: active,
                node_interaction_type: NodeInteractionType::RightClick,
            });
            // RETURN
            return;
        }

        if buttons.just_released(MouseButton::Left) {
            node_interaction_events.send(NodeInteractionEvent {
                entity: active,
                node_interaction_type: NodeInteractionType::LeftMouseRelease,
            });
            *holding_state = HoldingState {
                duration: Duration::from_millis(0),
                entity: None,
                is_holding: false,
            };

            // RETURN
            return;
        }

        if !holding_state.is_holding
            && Duration::from_millis(now_ms as u64) - holding_state.duration
                > Duration::from_millis(50)
            && holding_state.entity.is_some()
        {
            holding_state.is_holding = true;
            node_interaction_events.send(NodeInteractionEvent {
                entity: active,
                node_interaction_type: NodeInteractionType::LeftMouseHoldAndDrag,
            });

            // RETURN
            return;
        }

        node_interaction_events.send(NodeInteractionEvent {
            entity: active,
            node_interaction_type: NodeInteractionType::Hover,
        });
    }
}
