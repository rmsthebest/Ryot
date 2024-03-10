#[cfg(feature = "bevy")]
use bevy::{prelude::*, sprite::Anchor};

use std::hash::Hash;
use std::ops::{Add, AddAssign, DerefMut, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::{
    fmt::{self, Formatter},
    ops::Deref,
};

#[cfg(feature = "bevy")]
use crate::bevy_ryot::{drawing::Elevation, sprites::LoadedSprite};
use crate::SpriteLayout;
#[cfg(feature = "debug")]
use bevy_stroked_text::StrokedText;
#[cfg(feature = "bevy")]
use std::time::Duration;

use crate::layer::{compute_z_transform, Layer};
#[cfg(not(test))]
use crate::TILE_SIZE;
use derive_more::{Add, Sub};
use glam::{IVec2, IVec3, UVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// A 2d position in the tile grid. This is is not the position of the tile on
/// the screen, because it doesn't take into account the tile size. Z is used to
/// calculate the rendering order of the tile.
#[derive(Eq, PartialEq, Deserialize, Serialize, Default, Clone, Copy, Debug, Hash, Add, Sub)]
#[cfg_attr(feature = "bevy", derive(Component, Reflect))]
pub struct TilePosition(pub IVec3);

#[cfg(feature = "debug")]
#[derive(Component)]
pub struct PositionDebugText;

#[cfg(feature = "bevy")]
#[derive(Component, Debug, Clone)]
pub struct SpriteMovement {
    pub origin: (TilePosition, Elevation),
    pub destination: TilePosition,
    pub timer: Timer,
    pub despawn_on_end: bool,
}

impl TilePosition {
    /// The minimum possible tile position. This has to be something that when multiplied by the tile size does not overflow f32.
    pub const MIN: TilePosition = TilePosition(IVec3::new(i16::MIN as i32, i16::MIN as i32, 0));
    /// The maximum possible tile position. This has to be something that when multiplied by the tile size does not overflow f32.
    pub const MAX: TilePosition = TilePosition(IVec3::new(i16::MAX as i32, i16::MAX as i32, 0));

    pub const ZERO: TilePosition = TilePosition(IVec3::ZERO);

    const BOTTOM_RIGHT_OFFSET: Vec2 = Vec2::new(0., -1.);

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn with_z(self, z: i32) -> Self {
        Self(self.0.truncate().extend(z))
    }

    pub fn is_valid(self) -> bool {
        self.clamp(Self::MIN.0, Self::MAX.0).truncate() == self.truncate()
    }

    pub fn distance(self, other: Self) -> f32 {
        self.truncate()
            .as_vec2()
            .distance(other.truncate().as_vec2())
    }

    pub fn to_vec3(self, layer: &Layer) -> Vec3 {
        Vec2::from(self).extend(compute_z_transform(&self, layer))
    }

    pub fn from_elevated_translation(
        translation: Vec3,
        layout: SpriteLayout,
        anchor: Anchor,
    ) -> Self {
        let position =
            translation.truncate() + (layout.get_size(&tile_size()).as_vec2() * anchor.as_vec());
        TilePosition::from(position)
    }

    fn to_elevated_translation(self, layout: SpriteLayout, layer: Layer, anchor: Anchor) -> Vec3 {
        self.to_vec3(&layer)
            - (layout.get_size(&tile_size()).as_vec2() * anchor.as_vec()).extend(0.)
    }
}

impl Deref for TilePosition {
    type Target = IVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TilePosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for TilePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Vec2> for TilePosition {
    fn from(screen_pos: Vec2) -> Self {
        Self(
            ((screen_pos - TilePosition::BOTTOM_RIGHT_OFFSET) / tile_size().as_vec2())
                .ceil()
                .as_ivec2()
                .extend(0),
        )
    }
}

#[cfg(feature = "bevy")]
impl From<Transform> for TilePosition {
    fn from(transform: Transform) -> Self {
        transform.translation.truncate().into()
    }
}

#[cfg(feature = "bevy")]
impl From<&Transform> for TilePosition {
    fn from(transform: &Transform) -> Self {
        TilePosition::from(*transform)
    }
}

impl From<TilePosition> for Vec2 {
    fn from(tile_pos: TilePosition) -> Self {
        (tile_pos.as_vec3().truncate() + TilePosition::BOTTOM_RIGHT_OFFSET) * tile_size().as_vec2()
    }
}

impl From<&TilePosition> for Vec2 {
    fn from(tile_pos: &TilePosition) -> Self {
        Vec2::from(*tile_pos)
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for TilePosition {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self::new(
            i16::arbitrary(g) as i32,
            i16::arbitrary(g) as i32,
            i8::arbitrary(g) as i32,
        )
    }
}

#[cfg(feature = "bevy")]
impl SpriteMovement {
    pub fn new(
        origin: (TilePosition, Elevation),
        destination: TilePosition,
        duration: Duration,
    ) -> Self {
        Self {
            origin,
            destination,
            timer: Timer::new(duration, TimerMode::Once),
            despawn_on_end: false,
        }
    }

    pub fn despawn_on_end(self, despawn_on_end: bool) -> Self {
        Self {
            despawn_on_end,
            ..self
        }
    }
}

#[cfg(test)]
pub fn tile_size() -> UVec2 {
    UVec2::new(32, 32)
}

#[cfg(not(test))]
pub fn tile_size() -> UVec2 {
    *TILE_SIZE.get().expect("TILE_SIZE not initialized")
}

pub fn tile_offset() -> Vec2 {
    Vec2::new(-1., 1.) * tile_size().as_vec2()
}

#[cfg(feature = "debug")]
pub fn debug_y_offset(layer: &Layer) -> f32 {
    (tile_size().as_vec2().y / 8.)
        * match layer {
            Layer::Ground => 0.,
            Layer::Edge => 1.,
            Layer::Bottom(layer) => match layer.relative_layer {
                crate::layer::RelativeLayer::Object => 2.,
                crate::layer::RelativeLayer::Creature => 3.,
                crate::layer::RelativeLayer::Effect => 4.,
                crate::layer::RelativeLayer::Missile => 5.,
            },
            Layer::Top => 6.,
            Layer::Hud(_) => 7.,
        }
}

#[derive(Hash, Eq, PartialEq, Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct Sector {
    pub min: TilePosition,
    pub max: TilePosition,
}

impl Sector {
    pub const ZERO: Sector = Sector {
        min: TilePosition::ZERO,
        max: TilePosition::ZERO,
    };
}

impl fmt::Display for Sector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Edges({}, {})", self.min, self.max)
    }
}

impl Sector {
    pub const BASE_CANVAS_SECTOR: Sector = Sector {
        min: TilePosition(IVec3 {
            x: -30,
            y: -30,
            z: 0,
        }),
        max: TilePosition(IVec3 { x: 30, y: 30, z: 0 }),
    };

    pub fn new(min: TilePosition, max: TilePosition) -> Self {
        Self { min, max }
    }

    #[cfg(feature = "bevy")]
    pub fn from_transform_and_projection(
        transform: &Transform,
        projection: &OrthographicProjection,
    ) -> Self {
        let visible_width = projection.area.max.x - projection.area.min.x;
        let visible_height = projection.area.max.y - projection.area.min.y;

        // Adjust by the camera scale if necessary
        let visible_width = visible_width * transform.scale.x;
        let visible_height = visible_height * transform.scale.y;

        // Calculate boundaries based on the camera's position
        let camera_position = transform.translation;
        let left_bound = camera_position.x - visible_width / 2.0;
        let right_bound = camera_position.x + visible_width / 2.0;
        let bottom_bound = camera_position.y - visible_height / 2.0;
        let top_bound = camera_position.y + visible_height / 2.0;

        Self {
            min: TilePosition::from(Vec2::new(left_bound, bottom_bound)),
            max: TilePosition::from(Vec2::new(right_bound, top_bound)),
        }
    }

    pub fn size(&self) -> IVec2 {
        IVec2::new(self.max.x - self.min.x, self.max.y - self.min.y)
    }

    pub fn area(&self) -> u32 {
        (self.size().x * self.size().y).unsigned_abs()
    }
}

impl Sub<Sector> for Sector {
    type Output = Vec<Sector>;

    fn sub(self, rhs: Sector) -> Self::Output {
        if self == rhs {
            return Vec::new();
        }

        if rhs == Sector::ZERO {
            return vec![self];
        }

        if self == Sector::ZERO {
            return vec![rhs];
        }

        let mut result = Vec::new();

        // Left area (corrected to ensure no overlap and accurate representation)
        if rhs.min.x < self.min.x {
            result.push(Self {
                min: TilePosition::new(rhs.min.x, rhs.min.y, 0),
                max: TilePosition::new(self.min.x, rhs.max.y, 0),
            });
        }

        // Bottom area
        if rhs.min.y < self.min.y {
            result.push(Self {
                min: TilePosition::new(self.min.x, rhs.min.y, 0),
                max: TilePosition::new(self.max.x, self.min.y, 0),
            });
        }

        // Right area (corrected for the same reason as the left area)
        if rhs.max.x > self.max.x {
            result.push(Self {
                min: TilePosition::new(self.max.x, rhs.min.y, 0),
                max: TilePosition::new(rhs.max.x, rhs.max.y, 0),
            });
        }

        // Top area
        if rhs.max.y > self.max.y {
            result.push(Self {
                min: TilePosition::new(self.min.x, self.max.y, 0),
                max: TilePosition::new(self.max.x, rhs.max.y, 0),
            });
        }

        result
    }
}

impl Mul<f32> for Sector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let delta = (self.size().as_vec2() * (rhs - 1.0)) / 2.0;
        let delta = delta.as_ivec2();

        Sector::new(self.min - delta, self.max + delta)
    }
}

impl MulAssign<f32> for Sector {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Sector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl DivAssign<f32> for Sector {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

/// This system syncs the sprite position with the TilePosition.
/// Every spawned sprite has a Transform component, which is used to position the sprite on
/// the screen. However, in this library our world components are treated in terms of TilePosition.
/// So, we need to sync the sprite position with the TilePosition.
///
/// This system listen to all new and changed TilePosition components and update the Transform
/// component of the sprite accordingly, if it exist. Ideally this should run in the end of
/// the Update stage, so it can be sure that all TilePosition components have been updated.
///
/// ```rust
/// use bevy::prelude::*;
/// use ryot::sprites::position::update_sprite_position;
///
/// App::new()
///     .init_resource::<Time>()
///     .add_systems(PostUpdate, update_sprite_position)
///     .run();
/// ```
#[cfg(feature = "bevy")]
pub fn update_sprite_position(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &TilePosition,
            &Elevation,
            &Layer,
            &LoadedSprite,
            &mut Transform,
            Option<&mut SpriteMovement>,
            Option<&Children>,
        ),
        MovingSpriteFilter,
    >,
    #[cfg(feature = "debug")] mut children_query: Query<&mut StrokedText, With<PositionDebugText>>,
    time: Res<Time>,
) {
    for (entity, tile_pos, elevation, layer, loaded_sprite, mut transform, movement, _children) in
        query.iter_mut()
    {
        if let Some(mut movement) = movement {
            movement.timer.tick(time.delta());
            // We need the moving entity to be on top of other entities
            // This is to ensure that the layering logic is consistent no matter what direction
            // the entity is moving in
            let z = compute_z_transform(&movement.origin.0, layer)
                .max(compute_z_transform(&movement.destination, layer));
            let origin_elevation = movement.origin.1;
            let destination_elevation = *elevation;
            let origin_translation = movement.origin.0.to_elevated_translation(
                loaded_sprite.sprite_sheet.layout,
                *layer,
                Anchor::from(origin_elevation),
            );
            let destination_translation = movement.destination.to_elevated_translation(
                loaded_sprite.sprite_sheet.layout,
                *layer,
                Anchor::from(destination_elevation),
            );
            transform.translation = origin_translation
                .lerp(destination_translation, movement.timer.fraction())
                .truncate()
                .extend(z);
            if movement.timer.just_finished() {
                if movement.despawn_on_end {
                    commands.entity(entity).despawn_recursive();
                } else {
                    commands.entity(entity).remove::<SpriteMovement>();
                }
            }
        } else {
            transform.translation = tile_pos.to_elevated_translation(
                loaded_sprite.sprite_sheet.layout,
                *layer,
                Anchor::from(*elevation),
            );
        }
    }

    #[cfg(feature = "debug")]
    for (_entity, _tile_pos, elevation, _layer, _loaded_sprite, transform, _movement, children) in
        query.iter_mut()
    {
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut text) = children_query.get_mut(*child) {
                    text.text = format!("{:.02} [{}]", 1000. * transform.translation.z, elevation);
                }
            }
        }
    }
}

#[cfg(feature = "bevy")]
type MovingSpriteFilter = Or<(
    Changed<TilePosition>,
    Added<TilePosition>,
    Added<Transform>,
    With<SpriteMovement>,
    Changed<Elevation>,
)>;

impl Add<IVec2> for TilePosition {
    type Output = Self;
    #[inline]
    fn add(self, rhs: IVec2) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z)
    }
}

impl AddAssign<IVec2> for TilePosition {
    #[inline]
    fn add_assign(&mut self, rhs: IVec2) {
        *self = *self + rhs;
    }
}

impl Sub<IVec2> for TilePosition {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: IVec2) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z)
    }
}

impl SubAssign<IVec2> for TilePosition {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec2) {
        *self = *self - rhs;
    }
}
