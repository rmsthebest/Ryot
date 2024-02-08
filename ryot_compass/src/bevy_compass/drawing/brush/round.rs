use crate::BrushAction;
use bevy::prelude::*;
use ryot::bevy_ryot::drawing::{DrawingBundle, Tile};
use ryot::position::TilePosition;

#[derive(Debug, Eq, PartialEq, Deref, Reflect, DerefMut, Copy, Clone, Hash)]
pub struct RoundBrush(pub i32);

impl BrushAction for RoundBrush {
    fn apply(&self, center: DrawingBundle) -> Vec<DrawingBundle> {
        let mut positions = Vec::new();
        let DrawingBundle {
            layer,
            tile_pos,
            appearance,
            visibility,
            ..
        } = center;

        let Self(radius) = *self;

        for x in tile_pos.x.saturating_sub(radius)..=tile_pos.x.saturating_add(radius) {
            for y in tile_pos.y.saturating_sub(radius)..=tile_pos.y.saturating_add(radius) {
                let distance = tile_pos.distance(TilePosition::new(x, y, tile_pos.z));
                if distance <= self.0 as f32 {
                    positions.push(DrawingBundle {
                        layer,
                        tile_pos: TilePosition::new(x, y, tile_pos.z),
                        appearance,
                        visibility,
                        tile: Tile,
                    });
                }
            }
        }

        positions
    }
}