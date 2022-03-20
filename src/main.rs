use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
fn main() {
    let now = Instant::now();

    // Image size
    let imgx = 8000;
    let imgy = 6000;

    // Real and imaginary space where calculations will be performed
    let re_dimensions = (-2.2, 1.8);
    let im_dimensions = (-1.0, 2.0);

    // Values needed for coordinate scaling
    let xscale = ((imgx as f64 / (re_dimensions.0 - re_dimensions.1)) as f64).abs();
    let yscale = ((imgy as f64 / (im_dimensions.0 - im_dimensions.1)) as f64).abs();

    // Create color gradient
    let grad = colorgrad::cubehelix_default();

    // Create a new ImgBuf with specified height and width
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Mutex with total number of iterations for keeping track of progress
    let count = Arc::new(Mutex::new(0));

    imgbuf
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            // Scales pixel to coordinate
            let cx = (x as f64 / xscale) + re_dimensions.0;
            // let cy = ((y as f64 / yscale) + (im_dimensions.1 * -1.0)) * -1.0; // Normal space
            let cy = (y as f64 / yscale) + (im_dimensions.1 * -1.0); // Inverted imaginary axis

            let c = num::Complex::new(cx, cy);

            let mut iteration = 0;
            let max_iterations = 100;

            // Start z at 0, 0
            let mut z: num::Complex<f64> = num::Complex::new(0.0, 0.0);

            // Main itteration, while |z| <= 2.
            // `Re(z)^2 + Im(z)^2 <= 4.0` <=> `z.norm() <= 2.0` but former is much faster
            while z.re.powi(2) + z.im.powi(2) <= 4.0 && iteration < max_iterations {
                // |Re(z)| & |Im(z)|
                z.re = z.re.abs(); // Comment out for mandelbrot set
                z.im = z.im.abs(); // Comment out for mandelbrot set
                z = z.powi(2) + c;
                iteration += 1;
            }

            // Assigns pixel color based on number of iterations
            let (r, g, b, a) = grad.at(iteration as f64 / max_iterations as f64).rgba_u8();
            *pixel = image::Rgba([r, g, b, a]);

            // Progress update
            let mut count = count.lock().unwrap();
            *count += 1;

            if *count % 1_000_000 == 0 {
                println!(
                    "Rendering {:.2?}% complete",
                    (*count as f64 / (imgx as f64 * imgy as f64)) * 100.0
                );
            }
        });

    // Save image and count elapsed time.
    println!("Saving image...");
    imgbuf.save("output.png").unwrap();
    let elapsed = now.elapsed();
    println!("Image saved. Time elapsed: {:.2?}", elapsed);
}
