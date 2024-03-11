use rand::Rng;

pub fn random
    <T: std::cmp::PartialOrd + rand::distributions::uniform::SampleUniform>
    (l: T, u: T) -> T 
{
    let mut rng = rand::thread_rng(); 

    return rng.gen_range(l..u);
}

pub fn nth(n: u32) -> u32 {
    let x = if n <= 10 { 10.0 } else { n as f64 };
    
    let limit: usize = (x * (x * (x).ln()).ln()).ceil() as usize;
    let mut sieve = vec![true; limit];
    let mut count = 0;

    sieve[0] = false;
    sieve[1] = false;

    for prime in 2..limit {

        if !sieve[prime] {
            continue;
        }
        if count == n {
            return prime as u32;
        }
        count += 1;

        let mut sieve = sieve.clone();
        for multiple in ((2 * prime)..limit).step_by(prime) {
            sieve[multiple] = false;
        }
    }
    return <u32>::max_value();
}

pub fn vec3_to_idx(x: usize, y: usize, z: usize, size: usize,) -> usize {
   z * size * size + y * size + x 
}

use glam::{Vec3A, vec3a};
pub fn idx_to_vec3(index: usize, size: usize) -> Vec3A {
    let z = index / (size * size);
    let remaining = index % (size * size);
    let y = remaining / size;
    let x = remaining % size;

    vec3a(x as f32, y as f32, z as f32)
}
