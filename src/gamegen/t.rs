pub struct DotPos(pub usize, pub usize);

pub type Dot = u8;

/// This type is a representation of a column of spots in dot grid coordinates
pub type DotSpaceColumn = Vec<Dot>;

/// This type is a representation of all spots in dot grid coordinates
pub type DotSpace = Vec<DotSpaceColumn>;

pub type Pattern = DotSpace;
