use std::fmt;

#[derive(Eq)]
pub struct DotPos(pub usize, pub usize);

impl PartialEq for DotPos {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl fmt::Debug for DotPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DotPos")
            .field("x", &self.0)
            .field("y", &self.1)
            .finish()
    }
}

pub type Dot = u8;

/// This type is a representation of a column of spots in dot grid coordinates
pub type DotSpaceColumn = Vec<Dot>;

/// This type is a representation of all spots in dot grid coordinates
pub type DotSpace = Vec<DotSpaceColumn>;

pub type Pattern = DotSpace;
