use clap::Parser;
use rand::Rng;
mod opensimplex;
mod perlin;
use perlin::perlin;
use opensimplex::simplex;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    threads: u16,

    #[arg(short, long, default_value = "perlin")]
    noise: String,

    #[arg(short, long, default_value = "2.0")]
    scale: f64
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
    if args.noise == "perlin" {
        runtime.block_on(perlin(imgsize, scale, seed.try_into().unwrap()));
    } else if args.noise == "simplex" {
        runtime.block_on(simplex(imgsize, scale, seed.try_into().unwrap()));
    } else {
        panic!("Invalid noise type");
    }
}