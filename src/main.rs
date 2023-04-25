use std::time::Instant;

const IMG_X: u32 = 1920;
const IMG_Y: u32 = 1080;

const MAX_ITERATIONS: u32 = 10_000;

const BOX_SIZES: [u32; 5] = [120, 60, 30, 15, 5];

fn main() {
    let now = Instant::now();

    //-1.74995768370609350360221450607069970727110579726252077930242837820286008082972804887,
    //0.00000000000000000278793706563379402178294753790944364927085054500163081379043930650,
    let point = (
        -1.74995768370609350360221450607069970727110579726252077930242837820286008082972804887,
        0.00000000000000000278793706563379402178294753790944364927085054500163081379043930650,
    );

    let zoom = 0.01;

    let aspect_ratio = IMG_X as f64 / IMG_Y as f64;

    // Create color gradient
    let grad = colorgrad::cubehelix_default();

    let padding = 2.0 * zoom;
    // Real and imaginary space where calculations will be performed
    let re_dimensions = (
        point.0 - padding * aspect_ratio,
        point.0 + padding * aspect_ratio,
    );
    let im_dimensions = (point.1 - padding, point.1 + padding);

    // Values needed for coordinate scaling
    let xscale = ((IMG_X as f64 / (re_dimensions.0 - re_dimensions.1)) as f64).abs();
    let yscale = ((IMG_Y as f64 / (im_dimensions.0 - im_dimensions.1)) as f64).abs();

    // Create a new ImgBuf with specified height and width
    let mut imgbuf = image::ImageBuffer::new(IMG_X, IMG_Y);

    // traverse top left of each box
    render_box(
        &mut imgbuf,
        0,
        0,
        IMG_X,
        IMG_Y,
        &grad,
        xscale,
        yscale,
        re_dimensions,
        im_dimensions,
        1,
    );

    // Save image and count elapsed time.
    println!("Saving image...");
    imgbuf.save("output.png").unwrap();
    let elapsed = now.elapsed();
    println!("Image saved. Time elapsed: {:.2?}", elapsed);
    println!("Zoom: {zoom}");
    println!("Size: {IMG_X}x{IMG_Y}px");
}

fn render_box(
    imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    startx: u32,
    starty: u32,
    stopx: u32,
    stopy: u32,
    gradient: &colorgrad::Gradient,
    xscale: f64,
    yscale: f64,
    re_dimensions: (f64, f64),
    im_dimensions: (f64, f64),
    depth: u32,
) {
    let box_size = BOX_SIZES[depth as usize];
    for y_box in (starty..stopy).step_by(box_size as usize) {
        for x_box in (startx..stopx).step_by(box_size as usize) {
            let mut least_iterations = MAX_ITERATIONS + 1;
            for y in y_box..y_box + box_size {
                for x in x_box..x_box + box_size {
                    // only if the pixels are the border
                    if !(x == x_box
                        || x == x_box + box_size - 1
                        || y == y_box
                        || y == y_box + box_size - 1)
                    {
                        continue;
                    }
                    // Scales pixel to coordinate
                    let cx = (x as f64 / xscale) + re_dimensions.0;
                    let cy = (y as f64 / yscale) + (im_dimensions.1 * -1.0); // Inverted imaginary axis
                    let c = num::Complex::new(cx, cy);

                    let mut iteration = 0;

                    // Start z at 0, 0
                    let mut z: num::Complex<f64> = num::Complex::new(0.0, 0.0);
                    // If alpha value of pixel at coordinate is not zero
                    // the pixel is already calculated.
                    let current_pixel  = imgbuf.get_pixel(x, y);
                    if current_pixel.0[3] != 0 {
                        // If color is already calculated and not completely white
                        // set least iterations to 0 to avoid filling in the box
                        // TODO: maybe slow to create new rgba struct just to compare?
                        if *current_pixel != image::Rgba([255_u8, 255_u8, 255_u8, 1_u8]) {
                            least_iterations = 0;
                        }
                    } else {
                        // Main itteration, while |z| <= 2.
                        while z.re.powi(2) + z.im.powi(2) <= 4.0 && iteration < MAX_ITERATIONS {
                            // |Re(z)| & |Im(z)|
                            // z.re = z.re.abs(); // Comment out for mandelbrot set
                            // z.im = z.im.abs(); // Comment out for mandelbrot set
                            z = z.powi(2) + c;
                            iteration += 1;
                        }
                    }

                    if iteration < least_iterations {
                        least_iterations = iteration;
                    }

                    // Assigns pixel color based on number of iterations
                    imgbuf.put_pixel(
                        x,
                        y,
                        image::Rgba(
                            gradient
                                .at(iteration as f64 / MAX_ITERATIONS as f64)
                                .to_rgba8(),
                        ),
                    );
                }
            }
            // If all borders are iterated to the max the inside is also max
            if least_iterations == MAX_ITERATIONS {
                //colour the inside square
                draw_square(
                    imgbuf,
                    x_box,
                    y_box,
                    box_size,
                    box_size,
                    image::Rgba(gradient.at(1.0).to_rgba8()),
                );
            } else {
                if depth < 4 {
                    render_box(
                        imgbuf,
                        x_box,
                        y_box,
                        x_box + box_size,
                        y_box + box_size,
                        gradient,
                        xscale,
                        yscale,
                        re_dimensions,
                        im_dimensions,
                        depth + 1,
                    );
                } else {
                    for y in y_box..y_box + box_size {
                        for x in x_box..x_box + box_size {
                            // Scales pixel to coordinate
                            let cx = (x as f64 / xscale) + re_dimensions.0;
                            let cy = (y as f64 / yscale) + (im_dimensions.1 * -1.0); // Inverted imaginary axis
                            let c = num::Complex::new(cx, cy);
                            render_mandelbrot_at_point(imgbuf, x, y, c, &gradient);
                        }
                    }
                }
            }
        }
    }
}

fn render_mandelbrot_at_point(
    imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    c: num::Complex<f64>,
    gradient: &colorgrad::Gradient,
) {
    // Assigns pixel color based on number of iterations
    imgbuf.put_pixel(
        x,
        y,
        image::Rgba(
            gradient
                .at(iterate(c, MAX_ITERATIONS) as f64 / MAX_ITERATIONS as f64)
                .to_rgba8(),
        ),
    );
}

fn iterate(c: num::Complex<f64>, max_iterations: u32) -> u32 {
    let mut iteration = 0;

    // Start z at 0, 0
    let mut z: num::Complex<f64> = num::Complex::new(0.0, 0.0);

    // Main itteration, while |z| <= 2.
    while z.re.powi(2) + z.im.powi(2) <= 4.0 && iteration < max_iterations {
        // z.re = z.re.abs(); // Uncomment for burning ship
        // z.im = z.im.abs(); // Uncomment for burnish ship
        z = z.powi(2) + c; //Main mandelbrot iteration
        iteration += 1;
    }

    return iteration;
}

fn draw_square(
    imgbuf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    colour: image::Rgba<u8>,
) {
    for x in x..x + width {
        for y in y..y + height {
            imgbuf.put_pixel(x, y, colour);
        }
    }
}
