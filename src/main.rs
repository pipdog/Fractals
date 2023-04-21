use rayon::iter::{ParallelBridge, ParallelIterator};
use std::time::Instant;

const MAX_ITERATIONS: u32 = 20_000;

fn main() {
    let now = Instant::now();

    // Image size
    let imgx = 1920;
    let imgy = 1080;

    //-1.74995768370609350360221450607069970727110579726252077930242837820286008082972804887,
    //0.00000000000000000278793706563379402178294753790944364927085054500163081379043930650,
    let point = (
        -1.74995768370609350360221450607069970727110579726252077930242837820286008082972804887,
        0.00000000000000000278793706563379402178294753790944364927085054500163081379043930650,
    );
    let mut zoom = 1.0;

    let aspect_ratio = imgx as f64 / imgy as f64;

    // Create color gradient
    let grad = colorgrad::cubehelix_default();

    for n in 1..100 {
        let padding = 2.0 * zoom;
        // Real and imaginary space where calculations will be performed
        let re_dimensions = (
            point.0 - padding * aspect_ratio,
            point.0 + padding * aspect_ratio,
        );
        let im_dimensions = (point.1 - padding, point.1 + padding);

        // Values needed for coordinate scaling
        let xscale = ((imgx as f64 / (re_dimensions.0 - re_dimensions.1)) as f64).abs();
        let yscale = ((imgy as f64 / (im_dimensions.0 - im_dimensions.1)) as f64).abs();

        // Create a new ImgBuf with specified height and width
        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        imgbuf
            .enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, pixel)| {
                // Scales pixel to coordinate
                let cx = (x as f64 / xscale) + re_dimensions.0;
                let cy = (y as f64 / yscale) + (im_dimensions.1 * -1.0); // Inverted imaginary axis
                let c = num::Complex::new(cx, cy);

                let mut iteration = 0;

                // Start z at 0, 0
                let mut z: num::Complex<f64> = num::Complex::new(0.0, 0.0);

                // Main itteration, while |z| <= 2.
                // `Re(z)^2 + Im(z)^2 <= 4.0` <=> `z.norm() <= 2.0` but former is much faster
                while z.re.powi(2) + z.im.powi(2) <= 4.0 && iteration < MAX_ITERATIONS {
                    // |Re(z)| & |Im(z)|
                    // z.re = z.re.abs(); // Comment out for mandelbrot set
                    // z.im = z.im.abs(); // Comment out for mandelbrot set
                    z = z.powi(2) + c;
                    iteration += 1;
                }

                // Assigns pixel color based on number of iterations
                *pixel = image::Rgba(grad.at(iteration as f64 / MAX_ITERATIONS as f64).to_rgba8());
            });

        // Save image and count elapsed time.
        println!("Saving image...");
        imgbuf.save(format!("./frames/frame{n}.png")).unwrap();
        let elapsed = now.elapsed();
        println!("Image {} saved. Time elapsed: {:.2?}", n, elapsed);
        println!("Zoom: {zoom}");

        zoom *= 0.95;
    }
}
