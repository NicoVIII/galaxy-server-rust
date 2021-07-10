use super::t;
use log::trace;

/// Helper for debugging purposes.
/// Prints out a DotSpace and the amount of candidates for a single spot.
pub fn dot_space_candidates(space: &t::DotSpace, candidates: &Vec<t::DotPos>) {
    let x_size = space.len();
    let y_size = space[0].len();

    // Print Header
    let mut s = "\n    ".to_owned();
    for i in 0..x_size {
        let line = format!("{:0>3}", i) + " ";
        s.push_str(&line);
    }
    s.push('\n');

    // Every place around the dot is not available anymore
    for i in 0..y_size {
        let line = format!("{:0>3}", i) + " ";
        s.push_str(&line);
        for j in 0..x_size {
            let mut amount = 0;
            for candidate in candidates {
                let x = usize::from(candidate.0);
                let y = usize::from(candidate.1);
                if x == j && y == i {
                    amount += 1;
                }
            }
            let line = format!("{:>3}", amount) + " ";
            s.push_str(&line);
        }
        s.push('\n');
    }
    trace!("{}", &s);
}

/// Helper for debugging purposes.
/// Prints out a DotSpace.
pub fn dot_space(space: &t::DotSpace) {
    let x_size = space.len();
    let y_size = space[0].len();

    // Print Header
    let mut s = "\n   ".to_owned();
    for i in 0..x_size {
        let line = format!("{:0>2}", i) + " ";
        s.push_str(&line);
    }
    s.push('\n');

    // Every place around the dot is not available anymore
    for i in 0..y_size {
        let line = format!("{:0>2}", i) + " ";
        s.push_str(&line);
        for j in 0..x_size {
            s.push_str(&format!(" {} ", space[j][i]));
        }
        s.push('\n');
    }
    trace!("{}", &s);
}
