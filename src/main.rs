use clap::Parser;
use image::{ImageBuffer, Luma};
use rand::Rng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    threads: u16,

    #[arg(short, long, default_value = "opensimplex")]
    noise: String,

    #[arg(short, long, default_value = "2.0")]
    scale: f64
}   

#[derive(Clone)]
pub struct OpenSimplexNoise {
    perm: Vec<usize>,
}

impl OpenSimplexNoise {
    pub fn new(seed: usize) -> Self {
        let mut perm = vec![0; 256];
        for i in 0..256 {
            perm[i] = i;
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
        perm.shuffle(&mut rng);
        Self { perm }
    }

    pub fn noise(&self, xin: f64, yin: f64) -> f64 {
        // Skewing and unskewing factors for 2D
        let sqrt_3 = 3f64.sqrt();
        let f2 = 0.5 * (sqrt_3 - 1.0);
        let g2 = (3.0 - sqrt_3) / 6.0;

        // Noise contributions from the three corners
        let mut n0;
        let mut n1;
        let mut n2;

        // Skew the input space to determine which simplex cell we're in
        let s = (xin + yin) * f2;
        let i = (xin + s).floor() as isize;
        let j = (yin + s).floor() as isize;

        // Unskew the cell origin back to (x, y) space
        let t = (i + j) as f64 * g2;
        let x0 = xin - (i as f64 - t);
        let y0 = yin - (j as f64 - t);

        // Determine which simplex we are in
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        // Offsets for corners
        let x1 = x0 - i1 as f64 + g2;
        let y1 = y0 - j1 as f64 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;

        // Calculate hashed gradient indices of the three simplex corners
        let ii = i & 255;
        let jj = j & 255;
        let gi0 = self.perm[(ii as usize + self.perm[jj as usize % 256]) % 256];
        let gi1 = self.perm[((ii + i1) as usize + self.perm[(jj + j1) as usize % 256]) % 256];
        let gi2 = self.perm[((ii + 1) as usize + self.perm[(jj + 1) as usize % 256]) % 256];

        // Calculate the contribution from the three corners
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 < 0.0 {
            n0 = 0.0;
        } else {
            n0 = t0 * t0 * t0 * t0 * grad2(gi0, x0, y0);
        }
        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 < 0.0 {
            n1 = 0.0;
        } else {
            n1 = t1 * t1 * t1 * t1 * grad2(gi1, x1, y1);
        }
        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 < 0.0 {
            n2 = 0.0;
        } else {
            n2 = t2 * t2 * t2 * t2 * grad2(gi2, x2, y2);
        }

        // Add contributions from each corner to get the final noise value
        // The result is scaled to return values in the interval [-1, 1]
        70.0 * (n0 + n1 + n2)
    }
}

fn grad2(hash: usize, x: f64, y: f64) -> f64 {
    let h = hash & 7;
    let u = if h < 4 { x } else { y };
    let v = if h < 4 { y } else { x };
    let mut res = u;
    if h & 1 != 0 {
        res = -u;
    }
    if h & 2 != 0 {
        res -= 2.0 * v;
    } else {
        res += 2.0 * v;
    }
    res
}

async fn simplex(imgsize: u32, scale: f64, seed: usize) {
    let imgsize = imgsize as usize;
    let open_simplex = OpenSimplexNoise::new(seed);

    let mut noise_values = vec![vec![0u8; imgsize]; imgsize];
    for y in 0..imgsize {
        for x in 0..imgsize {
            let xf = scale * (x as f64 / imgsize as f64) - 1.0;
            let yf = scale * (y as f64 / imgsize as f64) - 1.0;
            let noise = open_simplex.noise(xf, yf);
            noise_values[y][x] = ((noise + 1.0) * 0.5 * 255.0) as u8;
        }
    }

    let img = ImageBuffer::from_fn(imgsize as u32, imgsize as u32, |x, y| {
        Luma([noise_values[y as usize][x as usize]])
    });

    img.save("noise.png").unwrap();
}

fn main() {
    let args = Args::parse();
    println!("Generating with {} threads and {:?} noise", args.threads, args.noise);

    let imgsize = 1024; // replace with your desired image size

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut rng = rand::thread_rng();
    let seed: u64 = rng.gen();
    let scale = args.scale;
    runtime.block_on(simplex(imgsize, scale, seed.try_into().unwrap()));
}