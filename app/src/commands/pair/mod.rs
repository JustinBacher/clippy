use anyhow::{Ok, Result};
use rand::seq::IteratorRandom;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub mod pair;
pub use pair::*;

fn read_lines<P>(filename: P) -> Result<Vec<Result<String>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file).lines();
    let seed = reader.next().and_then(|line| line.and_then(|to_int| to_int.parse::<i64>()))?;
    let mut rng = StdRng::seed_from_u64(seed);
    let choices = reader.choose_multiple(&mut rng, 5);
    Ok(choices)
}
