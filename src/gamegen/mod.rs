use std::convert::TryFrom;
use std::convert::TryInto;

pub struct DotPos(u16, u16);

/// This type is a representation of a column of spots in dot grid coordinates
type DotSpaceColumn = Vec<u8>;

/// This type is a representation of all spots in dot grid coordinates
pub type DotSpace = Vec<DotSpaceColumn>;

type Pattern = DotSpace;

/// Calculates for a field in DotSpace a weight for the choosing algorithm
fn calculate_neighbor_weight(
    space: &DotSpace,
    x: usize,
    y: usize,
    x_size: usize,
    y_size: usize,
    neighbor_span: i16,
) -> u8 {
    let mut result = 0;

    let x = i16::try_from(x).expect("x is no valid i16");
    let y = i16::try_from(y).expect("y is no valid i16");

    'outer: for i in 1..neighbor_span {
        for j in -i..i {
            let x2 = x + j;
            let y2_1 = y - (i - j.abs());
            let y2_2 = y + (i - j.abs());

            let x2 = usize::try_from(x2);
            let y2_1 = usize::try_from(y2_1);
            let y2_2 = usize::try_from(y2_2);

            match (x2, y2_1, y2_2) {
                (Ok(x2), Ok(y2_1), Ok(y2_2)) => {
                    if x2 >= x_size {
                        result = 1;
                        break 'outer;
                    }

                    // First y
                    if y2_1 >= y_size || space[x2][y2_1] == 1 {
                        result = 1;
                        break 'outer;
                    };

                    // Second y, if existing
                    if y2_1 != y2_2 && (y2_2 >= y_size || space[x2][y2_2] == 1) {
                        result = 1;
                        break 'outer;
                    }
                }
                _ => {
                    result = 1;
                    break 'outer;
                }
            }
        }
    }

    result
}

fn get_random_dot_candidates(space: &DotSpace, min_distance_to_filled: u16) -> Vec<DotPos> {
    // Determine possible candidates
    let mut candidates = Vec::new();
    let x_size = space.len();
    for x in 0..x_size {
        let y_size = space[x].len();
        for y in 0..y_size {
            // Calculate neighbor score
            let min_distance_to_filled = i16::try_from(min_distance_to_filled)
                .expect("min_distance_to_filled is no valid i16");
            let penalty =
                calculate_neighbor_weight(space, x, y, x_size, y_size, min_distance_to_filled);
            let neighbor_score = 1 - penalty;

            if space[x][y] == 0 && neighbor_score > 0 {
                let dot_pos = DotPos(x.try_into().unwrap(), y.try_into().unwrap());
                candidates.push(dot_pos);
            }
        }
    }
    candidates
}
