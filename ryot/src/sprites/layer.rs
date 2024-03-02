use crate::position::TilePosition;
#[cfg(feature = "bevy")]
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};
use strum::*;

/// A type that represents the order of an item in an ordered layer. There is a virtual limit of
/// 256 items in the ordered layers.
pub type Order = u8;

const MAX_Z: f32 = 999.;
const MAX_Z_TILE: f32 = i8::MAX as f32;
const MAX_Z_TRANSFORM: f32 = 900. - MAX_Z_TILE;
const COUNT_LAYERS: f32 = Layer::COUNT as f32;
const LAYER_WIDTH: f32 = MAX_Z_TRANSFORM / (2. * u16::MAX as f32);

#[derive(
    Debug,
    Default,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    EnumCount,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "bevy", derive(Component, Reflect))]
pub enum Layer {
    #[default]
    Ground,
    Edge,
    Bottom(BottomLayer),
    Top,
    Hud(Order),
}

impl Display for Layer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ground => write!(f, "Ground"),
            Self::Edge => write!(f, "Edge"),
            Self::Bottom(bottom_layer) => write!(f, "Bottom({})", bottom_layer),
            Self::Top => write!(f, "Top"),
            Self::Hud(order) => write!(f, "Hud({})", order),
        }
    }
}

impl Display for BottomLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.relative_layer, self.order)
    }
}

#[cfg(feature = "debug")]
impl From<&Layer> for Color {
    fn from(value: &Layer) -> Self {
        match value {
            Layer::Ground => Color::ORANGE_RED,
            Layer::Edge => Color::YELLOW,
            Layer::Bottom(layer) => Color::from(layer.relative_layer),
            Layer::Top => Color::PINK,
            Layer::Hud(_) => Color::TURQUOISE,
        }
    }
}

#[cfg(feature = "debug")]
impl From<RelativeLayer> for Color {
    fn from(value: RelativeLayer) -> Self {
        match value {
            RelativeLayer::Object => Color::RED,
            RelativeLayer::Creature => Color::GREEN,
            RelativeLayer::Effect => Color::BLUE,
            RelativeLayer::Missile => Color::ALICE_BLUE,
        }
    }
}

impl Iterator for Layer {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Ground => Some(Self::Edge),
            Self::Edge => Some(Self::Bottom(Default::default())),
            Self::Bottom(mut bottom_layer) => bottom_layer
                .relative_layer
                .next()
                .map(|relative_layer| {
                    Self::Bottom(BottomLayer {
                        order: 0,
                        relative_layer,
                    })
                })
                .or(Some(Self::Top)),
            Self::Top => Some(Self::Hud(0)),
            Self::Hud(order) => {
                if order < Order::MAX {
                    Some(Self::Hud(order + 1))
                } else {
                    None
                }
            }
        }
    }
}

impl DoubleEndedIterator for Layer {
    fn next_back(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Ground => None,
            Self::Edge => Some(Self::Ground),
            Self::Bottom(mut bottom_layer) => bottom_layer
                .relative_layer
                .next_back()
                .map(|relative_layer| {
                    Self::Bottom(BottomLayer {
                        order: BottomLayer::TOP_MOST_LAYER,
                        relative_layer,
                    })
                })
                .or(Some(Self::Edge)),
            Self::Top => Some(Self::Bottom(BottomLayer {
                order: BottomLayer::TOP_MOST_LAYER,
                relative_layer: RelativeLayer::iter().last().unwrap(),
            })),
            Self::Hud(order) => {
                if order == 0 {
                    Some(Self::Top)
                } else {
                    Some(Self::Hud(order - 1))
                }
            }
        }
    }
}

impl Layer {
    pub fn z(&self) -> f32 {
        match *self {
            Self::Ground => 0. * LAYER_WIDTH,
            Self::Edge => 1. * LAYER_WIDTH,
            Self::Bottom(layer) => 2. * LAYER_WIDTH + layer.z(),
            Self::Top => MAX_Z_TRANSFORM + 1.,
            Self::Hud(order) => (MAX_Z_TRANSFORM + 2. + order as f32).min(MAX_Z),
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Serialize, Deserialize,
)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
pub enum RelativeLayer {
    #[default]
    Object,
    Creature,
    Effect,
    Missile,
}

impl Iterator for RelativeLayer {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Object => Some(Self::Creature),
            Self::Creature => Some(Self::Effect),
            Self::Effect => Some(Self::Missile),
            Self::Missile => None,
        }
    }
}

impl DoubleEndedIterator for RelativeLayer {
    fn next_back(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Object => None,
            Self::Creature => Some(Self::Object),
            Self::Effect => Some(Self::Creature),
            Self::Missile => Some(Self::Effect),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
pub struct BottomLayer {
    pub order: Order,
    pub relative_layer: RelativeLayer,
}

impl Default for BottomLayer {
    fn default() -> Self {
        Self {
            order: Order::MAX,
            relative_layer: Default::default(),
        }
    }
}

impl Iterator for BottomLayer {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.order < BottomLayer::TOP_MOST_LAYER {
            Some(Self {
                order: self.order + 1,
                relative_layer: self.relative_layer,
            })
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for BottomLayer {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.order > 0 {
            Some(Self {
                order: self.order - 1,
                relative_layer: self.relative_layer,
            })
        } else {
            None
        }
    }
}

impl BottomLayer {
    const TOP_MOST_LAYER: u8 = 10;
    const COUNT_RELATIVE_LAYERS: f32 = RelativeLayer::COUNT as f32;
    const RELATIVE_WIDTH: f32 = LAYER_WIDTH / COUNT_LAYERS;

    pub fn new(order: Order, relative_layer: RelativeLayer) -> Self {
        Self {
            order,
            relative_layer,
        }
    }

    pub fn stack(relative_layer: RelativeLayer) -> Self {
        Self {
            order: Order::MAX,
            relative_layer,
        }
    }

    pub(crate) fn z(&self) -> f32 {
        // Our layer number relative to others in the stack
        let val = (self.relative_layer as usize) as f32;
        // How much space we can use of the f32 value from 0.0..RELATIVE_WIDTH
        let width = Self::RELATIVE_WIDTH / Self::COUNT_RELATIVE_LAYERS;
        // Where our range starts
        let min = val * width;
        // Our order relative to other elements of the same layer in our stack weighed against our width window
        let weight = self.order as f32 / Order::MAX as f32;
        // Final number between 0.0..TOTAL_WIDTH
        min + weight / Self::COUNT_RELATIVE_LAYERS
    }
}

impl PartialOrd for BottomLayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BottomLayer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.z().partial_cmp(&other.z()).unwrap()
    }
}

pub fn compute_z_transform(pos: &TilePosition, layer: &Layer) -> f32 {
    if matches!(layer, Layer::Hud(_)) {
        return layer.z();
    }

    let weight = u16::MAX as f32;
    let x_weighted = MAX_Z_TRANSFORM * pos.x as f32 / (weight);
    let y_weighted = MAX_Z_TRANSFORM * pos.y as f32 / (weight - 1.);

    pos.z as f32 + x_weighted - y_weighted + layer.z()
}
