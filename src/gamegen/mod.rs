use rand::Rng;
use std::cmp;
use std::convert::TryFrom;
use std::convert::TryInto;

mod print;
mod t;

fn add_border(space: &t::DotSpace, filler: t::Dot) -> t::DotSpace {
    let x_size = space.len();
    let y_size = space[0].len();

    let mut new_space: t::DotSpace = Vec::new();
    // Create border line
    let mut occupied_line: t::DotSpaceColumn = Vec::new();
    for _ in 0..(y_size + 2) {
        occupied_line.push(filler);
    }

    // Create space
    new_space.push(occupied_line.clone());
    for x in 0..x_size {
        let mut line: t::DotSpaceColumn = Vec::new();
        line.push(filler);
        for y in 0..y_size {
            line.push(space[x][y]);
        }
        line.push(filler);
        new_space.push(line);
    }
    new_space.push(occupied_line);

    new_space
}

fn mark_as_occupied(space: &mut t::DotSpace, dot: &t::DotPos, filler: t::Dot) -> () {
    let x_size = space.len();
    let y_size = space[0].len();

    let dot_x = i16::try_from(dot.0).unwrap();
    let dot_y = i16::try_from(dot.1).unwrap();

    // Is a dot on an edge or corner even more places are not available anymore
    let x_diff = if dot_x % 2 == 1 { 2 } else { 1 };
    let y_diff = if dot_y % 2 == 1 { 2 } else { 1 };

    // Every place around the dot is not available anymore
    for i in -x_diff..=x_diff {
        for j in -y_diff..=y_diff {
            let x = dot_x + i;
            let y = dot_y + j;

            let x = usize::try_from(x);
            let y = usize::try_from(y);

            match (x, y) {
                (Ok(x), Ok(y)) => {
                    if x < x_size && y < y_size && space[x][y] == 0 {
                        space[x][y] = filler;
                    }
                }
                _ => (),
            }
        }
    }
}

fn generate_next_dots_by_pattern(
    space: &mut t::DotSpace,
    pattern_list: &Vec<t::Pattern>,
) -> Option<Vec<t::DotPos>> {
    // Prepare patterns and space by adding occupied border
    let tmp_space = add_border(space, 1);

    let mut result = None;

    let x_size = tmp_space.len();
    'outer: for x in 0..x_size {
        let y_size = tmp_space[x].len();
        for y in 0..y_size {
            // Iterate through all patterns
            for pattern in pattern_list {
                let mut new_dot_list = Vec::new();

                // Check if pattern matches
                let px_size = pattern.len();
                let py_size = pattern[0].len();
                for px in 0..px_size {
                    for py in 0..py_size {
                        let x2 = x + px;
                        let y2 = y + py;

                        if x2 < x_size
                            && y2 < y_size
                            && (pattern[px][py] == 0
                                || tmp_space[x2][y2] == 1 && pattern[px][py] == 1
                                || tmp_space[x2][y2] == 0 && pattern[px][py] > 1)
                        {
                            if pattern[px][py] == 3 {
                                let pos = t::DotPos(x2 - 1, y2 - 1);
                                new_dot_list.push(pos);
                            }
                        } else {
                            continue 'outer;
                        }
                    }
                }

                // Found a matching pattern! Occupy space
                {
                    let mut occupied_space = false;
                    for px in 0..px_size {
                        for py in 0..py_size {
                            let x2 = i32::try_from(x + px).unwrap() - 1;
                            let y2 = i32::try_from(y + py).unwrap() - 1;
                            let x2 = usize::try_from(x2);
                            let y2 = usize::try_from(y2);
                            match (x2, y2) {
                                (Ok(x2), Ok(y2)) => {
                                    if x2 % 2 == 0
                                        && y2 % 2 == 0
                                        && x2 < x_size - 2
                                        && y2 < y_size - 2
                                        && pattern[px][py] > 1
                                    {
                                        let pos = t::DotPos(x2, y2);
                                        mark_as_occupied(space, &pos, 1);
                                        occupied_space = true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    if !occupied_space {
                        panic!("Did not occupy space!");
                    }
                }

                result = Some(new_dot_list);
                break 'outer;
            }
        }
    }
    result
}

/// Calculates for a field in DotSpace a weight for the choosing algorithm
fn calculate_neighbor_weight(
    space: &t::DotSpace,
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

fn get_random_dot_candidates(space: &t::DotSpace, min_distance_to_filled: u16) -> Vec<t::DotPos> {
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
                let dot_pos = t::DotPos(x.try_into().unwrap(), y.try_into().unwrap());
                candidates.push(dot_pos);
            }
        }
    }
    candidates
}

/// Generates a random dot in the empty spots. Tries to use a weight function to
/// prefer spaces which allow bigger galaxies.
fn generate_random_dot_in_empty_spot(
    space: &t::DotSpace,
    rng: &mut rand::prelude::ThreadRng,
) -> t::DotPos {
    let mut candidates = Vec::new();
    let search_radius = 3;
    for i in (0..=search_radius).rev() {
        candidates = get_random_dot_candidates(space, i);

        print::dot_space_candidates(space, &candidates);

        if candidates.len() > 0 {
            break;
        }
    }
    if candidates.len() == 0 {
        panic!("Found no candidates!");
    }

    // Choose one candidate randomly
    let new_dot_index = rng.gen_range(0..candidates.len());

    candidates.swap_remove(new_dot_index)
}

/// Takes a DotSpace with 2s where the current galaxy is generated and the
/// center of the galaxy and tries to add another field to this galaxy.
///
/// @return DotSpace after addition of field
fn add_field_to_galaxy(
    space: &mut t::DotSpace,
    dot: &t::DotPos,
    rng: &mut rand::prelude::ThreadRng,
) -> bool {
    // Determine possible candidates
    let mut candidates = Vec::new();

    let x_size = space.len();
    for x in 0..x_size {
        let y_size = space[x].len();
        for y in 0..y_size {
            // Check if field is adjacent
            let left = x > 0 && space[x - 1][y] == 2;
            let right = x + 1 < x_size && space[x + 1][y] == 2;
            let top = y > 0 && space[x][y - 1] == 2;
            let bottom = y + 1 < y_size && space[x][y + 1] == 2;

            if space[x][y] == 0 && (left || right || top || bottom) {
                // Check if corresponding field is free too
                let x2 = i32::try_from(2 * dot.0).unwrap() - i32::try_from(x).unwrap();
                let y2 = i32::try_from(2 * dot.1).unwrap() - i32::try_from(y).unwrap();
                let x2 = usize::try_from(x2);
                let y2 = usize::try_from(y2);
                match (x2, y2) {
                    (Ok(x2), Ok(y2)) => {
                        if x2 < x_size && y2 < y_size && space[x2][y2] == 0 {
                            let x = x.try_into().unwrap();
                            let y = y.try_into().unwrap();
                            let penalty1 =
                                4 * calculate_neighbor_weight(space, x, y, x_size, y_size, 2) + 1;
                            let penalty2 =
                                4 * calculate_neighbor_weight(space, x2, y2, x_size, y_size, 2) + 1;
                            for _ in 0..cmp::max(penalty1, penalty2) {
                                candidates.push(t::DotPos(x, y));
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    print::dot_space_candidates(space, &candidates);

    // Choose one candidate randomly
    if candidates.len() > 0 {
        let new_field_index = rng.gen_range(0..(candidates.len() - 1));

        let winner = candidates.swap_remove(new_field_index);

        mark_as_occupied(space, &winner, 2);

        // Mark corresponding field as well
        let x2 = i16::try_from(2 * dot.0).unwrap() - i16::try_from(winner.0).unwrap();
        let y2 = i16::try_from(2 * dot.1).unwrap() - i16::try_from(winner.1).unwrap();
        let x2 = usize::try_from(x2);
        let y2 = usize::try_from(y2);
        match (x2, y2) {
            (Ok(x2), Ok(y2)) => {
                if x2 < x_size && y2 < space[x2].len() {
                    mark_as_occupied(space, &t::DotPos(x2, y2), 2);
                }
            }
            _ => (),
        }
    }

    candidates.len() > 0
}

/// Takes a space and a dot and places the dot in the space.
/// For this dot a randomised galaxy is created and the space for that gets
/// occupied.
///
/// @return DotSpace after adding dot
fn generate_galaxy_from_dot(
    space: &mut t::DotSpace,
    dot: &t::DotPos,
    rng: &mut rand::prelude::ThreadRng,
) -> () {
    mark_as_occupied(space, dot, 2);

    // Add fields to galaxy/
    let mut field = 0;
    let mid_of_bell = i32::try_from(space.len() * space[0].len()).unwrap() / 40 - 1;
    let steepness = 10;

    loop {
        let result = add_field_to_galaxy(space, dot, rng);
        field += 1;
        // There are no fields to add anymore... :(
        if !result {
            break;
        }

        let denominator: i32 = f64::from(steepness + (field - mid_of_bell).pow(2))
            .sqrt()
            .round() as i32;
        let probability = (-50 * (field - mid_of_bell)) / denominator + 50;
        let dice = rng.gen_range(1..100);

        if dice > probability {
            break;
        }
    }

    // Normalize entries in space
    let x_size = space.len();
    for x in 0..x_size {
        let y_size = space[x].len();
        for y in 0..y_size {
            if space[x][y] == 2 {
                space[x][y] = 1;
            }
        }
    }
}

fn generate_next_dots(
    space: &mut t::DotSpace,
    pattern_list: &Vec<t::Pattern>,
    rng: &mut rand::prelude::ThreadRng,
) -> Vec<t::DotPos> {
    // First try pattern matching
    let next_dots = generate_next_dots_by_pattern(space, pattern_list);
    match next_dots {
        Some(dots) => dots,
        None => {
            // If that doesn't work, generate random dot
            let new_dot = generate_random_dot_in_empty_spot(&space, rng);
            generate_galaxy_from_dot(space, &new_dot, rng);

            vec![new_dot]
        }
    }
}

fn create_patterns() -> Vec<t::Pattern> {
    // Patterns
    // 0: content irrelevant
    // 1: occupied
    // 2: free
    // 3: free / center of a new galaxy
    let mut pattern_list = Vec::new();

    // Most important pattern has to be first in pattern list
    // □ □ □
    // □ ▣ □
    // □ □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    // □ ▣ □
    //   □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 1, 0, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 1, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 0, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    //   □ □
    // □ ▣ □
    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 0, 0, 1, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 1, 0, 0, 0]);
        pattern_list.push(pattern);
    }

    // □ □ □
    //   ▣
    // □ □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 2, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 3, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 2, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □   □
    // □ ▣ □
    // □   □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0, 0, 0, 1, 0]);
        pattern.push(vec![1, 2, 1, 0, 1, 2, 1]);
        pattern.push(vec![1, 2, 1, 1, 1, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 1, 1, 1, 2, 1]);
        pattern.push(vec![1, 2, 1, 0, 1, 2, 1]);
        pattern.push(vec![0, 1, 0, 0, 0, 1, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    // □ □
    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 3, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □ □ □
    // □ □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    //   □ □
    //   ▣
    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 0, 0, 1, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 0, 1, 2, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 3, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 2, 1, 0, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 1, 0, 0, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    //   ▣
    //   □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 0, 0, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 2, 1, 0, 0]);
        pattern.push(vec![0, 0, 1, 3, 1, 0, 0]);
        pattern.push(vec![0, 0, 1, 2, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 0, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □
    // □ ▣ □
    //     □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0, 0, 0, 0, 0]);
        pattern.push(vec![1, 2, 1, 0, 0, 0, 0]);
        pattern.push(vec![1, 2, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 2, 1]);
        pattern.push(vec![0, 0, 0, 0, 1, 2, 1]);
        pattern.push(vec![0, 0, 0, 0, 0, 1, 0]);
        pattern_list.push(pattern);
    }

    //     □
    // □ ▣ □
    // □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 0, 0, 0, 0, 1, 0]);
        pattern.push(vec![0, 0, 0, 0, 1, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 1, 0, 0, 0, 0]);
        pattern.push(vec![0, 1, 0, 0, 0, 0, 0]);
        pattern_list.push(pattern);
    }

    //   □ □
    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 0, 0, 1, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 3, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 1, 0, 0, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    //   □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 0, 0, 0]);
        pattern.push(vec![1, 2, 2, 2, 1, 0, 0]);
        pattern.push(vec![0, 1, 1, 3, 1, 1, 0]);
        pattern.push(vec![0, 0, 1, 2, 2, 2, 1]);
        pattern.push(vec![0, 0, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □
    // □ □
    //   □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0, 0, 0]);
        pattern.push(vec![1, 2, 0, 0, 0]);
        pattern.push(vec![1, 2, 1, 1, 0]);
        pattern.push(vec![1, 2, 3, 2, 1]);
        pattern.push(vec![0, 1, 1, 2, 1]);
        pattern.push(vec![0, 0, 0, 2, 1]);
        pattern.push(vec![0, 0, 0, 1, 0]);
        pattern_list.push(pattern);
    }

    //   □
    // □ □
    // □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 0, 0, 0, 0, 1, 0]);
        pattern.push(vec![0, 0, 0, 0, 1, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 2, 1]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![1, 2, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 1, 0, 0, 0, 0]);
        pattern.push(vec![0, 1, 0, 0, 0, 0, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![1, 2, 3, 2, 1]);
        pattern.push(vec![1, 2, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □ ▣ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 2, 3, 2, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □
    // ▣
    // □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![1, 3, 1]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![0, 1, 0]);
        pattern_list.push(pattern);
    }

    // □ □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern.push(vec![1, 2, 3, 2, 1]);
        pattern.push(vec![0, 1, 1, 1, 0]);
        pattern_list.push(pattern);
    }

    // □
    // □
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![1, 3, 1]);
        pattern.push(vec![1, 2, 1]);
        pattern.push(vec![0, 1, 0]);
        pattern_list.push(pattern);
    }

    // ▣
    {
        let mut pattern = Vec::new();
        pattern.push(vec![0, 1, 0]);
        pattern.push(vec![1, 3, 1]);
        pattern.push(vec![0, 1, 0]);
        pattern_list.push(pattern);
    }

    pattern_list
}

/// Helper function which counts the empty spots in a DotSpace.
///
/// @return number of empty spots
fn count_empty_spots(space: &t::DotSpace) -> u32 {
    let x_size = space.len();
    let mut empty_spaces_total = 0;

    // Determine which y values have empty places for the dots to go to
    // Save y index and amount of empty spaces
    for x in 0..x_size {
        let y_size = space[x].len();
        // Simply counts the empty spaces
        for y in 0..y_size {
            if space[x][y] == 0 {
                empty_spaces_total += 1;
            }
        }
    }

    return empty_spaces_total;
}

/// Entrypoint into dot generation.
/// Takes the size of the field and generates dots in it which are then returned.
///
/// @return List of generated dots
pub fn generate_dots(x_size: usize, y_size: usize) -> Vec<t::DotPos> {
    // Generate empty space
    let mut space: t::DotSpace = Vec::new();
    for _ in 0..x_size * 2 - 1 {
        let mut column: t::DotSpaceColumn = Vec::new();
        for _ in 0..y_size * 2 - 1 {
            column.push(0);
        }
        space.push(column);
    }

    // Generate dots in space
    let pattern_list = create_patterns();
    let mut new_dot_list = Vec::new();
    let mut rng = rand::thread_rng();
    loop {
        let next_dots = generate_next_dots(&mut space, &pattern_list, &mut rng);
        for next_dot in next_dots {
            new_dot_list.push(next_dot);
        }
        let empty_spaces = count_empty_spots(&space);
        // Debugging statements
        print::dot_space(&space);

        if empty_spaces <= 0 {
            break;
        }
    }

    return new_dot_list;
}
