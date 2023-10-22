use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use image::{ImageBuffer, Luma, imageops::blur};

fn grad(hash: u8, x: f64, y: f64) -> f64 {
    let h = hash & 7; // Convert low 3 bits of hash code
    let u = if h < 4 { x } else { y }; // into 8 simple gradient directions,
    let v = if h < 4 { y } else { x };
    (if h & 1 != 0 { -u } else { u }) + (if h & 2 != 0 { -2.0 * v } else { 2.0 * v })
}

fn noise(x: f64, y: f64, perm: &[u8; 256]) -> f64 {
    // Find the unit grid cell containing the point
    let xi = x.floor() as i64;
    let yi = y.floor() as i64;

    // Compute the relative coordinates of the point within the cell
    let xf = x - xi as f64;
    let yf = y - yi as f64;

    // Compute the fade curves for x and y
    let u = fade(xf);
    let v = fade(yf);

    // Hash coordinates of the 4 square corners
    let aa = perm[(xi & 255) as usize + perm[(yi & 255) as usize] as usize];
    let ab = perm[(xi & 255) as usize + perm[((yi + 1) & 255) as usize] as usize];
    let ba = perm[((xi + 1) & 255) as usize + perm[(yi & 255) as usize] as usize];
    let bb = perm[((xi + 1) & 255) as usize + perm[((yi + 1) & 255) as usize] as usize];

    // Add blended results from 4 corners of the square
    lerp(v, lerp(u, grad(aa, xf, yf), grad(ba, xf - 1.0, yf)), lerp(u, grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0)))
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

pub async fn perlin(imgsize: u32, scale: f64, seed: usize) {
    let mut perm: [u8; 256] = [0; 256];
    for i in 0..256 {
        perm[i] = i as u8;
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
    perm.shuffle(&mut rng);

    let mut noise_values = vec![vec![0.0; imgsize.try_into().unwrap()]; imgsize.try_into().unwrap()];
    for y in 0..imgsize {
        for x in 0..imgsize {
            let xf = scale * (x as f64 / imgsize as f64);
            let yf = scale * (y as f64 / imgsize as f64);
            noise_values[y as usize][x as usize] = noise(xf, yf, &perm);
        }
    }

    let mut img = ImageBuffer::from_fn(imgsize as u32, imgsize as u32, |x, y| {
        let noise = noise_values[y as usize][x as usize];
        // Convert the noise value from f64 to u8
        let noise = (noise * 255.0) as u8;
        Luma([noise])
    });

img.save("noise.png").unwrap();

}