use super::*;
use rand::SeedableRng;

static mut SHARED_ARRAY: [u8; 1 << 16] = [0; 1 << 16];

#[no_mangle]
pub extern "C" fn numberlink_generate(
    height: i32,
    width: i32,
    empty_width: i32,
    corner_clue_low: i32,
    corner_clue_high: i32,
    minimum_chain_length: i32,
    forbid_adjacent_clue: bool,
    seed1: f64,
    seed2: f64,
) -> *const u8 {
    let seed_array: [u8; 16] = unsafe { std::mem::transmute_copy(&(seed1, seed2)) };
    let mut rng = rand::prng::XorShiftRng::from_seed(seed_array);
    let mut generator = numberlink::PlacementGenerator::new(height, width);
    loop {
        let endpoint_constraint = numberlink::generate_endpoint_constraint(
            height,
            width,
            empty_width,
            if corner_clue_low >= 0 {
                Some((corner_clue_low, corner_clue_high))
            } else {
                None
            },
            Symmetry::none(),
            &mut rng,
        );
        let cond = numberlink::GeneratorOption {
            chain_threshold: minimum_chain_length,
            endpoint_constraint: Some(&endpoint_constraint),
            forbid_adjacent_clue,
            symmetry: Symmetry::none(),
            clue_limit: None,
            prioritized_extension: false,
        };
        if let Some(problem) = generator.generate_and_test(&cond, &mut rng) {
            unsafe {
                for y in 0..height {
                    for x in 0..width {
                        let val = problem[(Y(y), X(x))];
                        SHARED_ARRAY[(y * width + x) as usize] = val.0 as u8;
                    }
                }
                return &SHARED_ARRAY[0];
            }
        }
    }
}