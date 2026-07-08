use crate::Direction;

mod halting;
#[cfg(test)]
mod tests;

impl Direction {
    pub fn reverse(self) -> Self {
        match self {
            Direction::Extend => Direction::Retract,
            Direction::Retract => Direction::Extend,
            Direction::Hold => Direction::Hold,
        }
    }
}
