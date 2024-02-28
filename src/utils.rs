use rand::Rng;

pub fn random
    <T: std::cmp::PartialOrd + rand::distributions::uniform::SampleUniform>
    (l: T, u: T) -> T 
{
    let mut rng = rand::thread_rng(); 

    return rng.gen_range(l..u);
}
