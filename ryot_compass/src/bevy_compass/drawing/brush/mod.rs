use bevy::prelude::*;
use ryot::prelude::{drawing::*, position::*, *};

mod diamond;
pub use diamond::DiamondBrush;

mod geometric;
pub use geometric::GeometricBrush;

mod round;
pub use round::RoundBrush;

mod square;
pub use square::SquareBrush;

mod systems;
pub use systems::update_brush;

pub trait BrushAction: Eq + PartialEq + Clone + Reflect + Send + Sync + 'static {
    fn apply(&self, center: DrawingBundle) -> Vec<DrawingBundle>;
}

#[derive(Component, Eq, Default, PartialEq, Reflect, Copy, Clone, Hash)]
pub enum Brush {
    #[default]
    SingleTile,
    Geometric(GeometricBrush),
}

impl Brush {
    fn increase(&mut self) {
        match self {
            Brush::SingleTile => (),
            Brush::Geometric(brush) => brush.increase(),
        }
    }

    fn decrease(&mut self) {
        match self {
            Brush::SingleTile => (),
            Brush::Geometric(brush) => brush.decrease(),
        }
    }
}

impl BrushAction for Brush {
    fn apply(&self, center: DrawingBundle) -> Vec<DrawingBundle> {
        match self {
            Brush::SingleTile => SingleTileBrush.apply(center),
            Brush::Geometric(brush) => brush.apply(center),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Reflect, Copy, Clone, Hash)]
pub struct SingleTileBrush;
impl BrushAction for SingleTileBrush {
    fn apply(&self, center: DrawingBundle) -> Vec<DrawingBundle> {
        vec![center]
    }
}