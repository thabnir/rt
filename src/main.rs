pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod vec3_ext;

use crate::{
    camera::{gen_scene, Camera, Float},
    hittable::World,
    vec3_ext::Vec3Ext,
};
use camera::Image;
use glam::Vec3;
use indicatif::ParallelProgressIterator;
use pixels::{Pixels, SurfaceTexture};
use rand::{prelude::SliceRandom, thread_rng};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::File,
    ops::Deref,
    process::exit,
    // simd::u8x4,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 1000;

// TODO: figure out how to apply gamma correction to the preview in a performant way
fn window_preview(camera: Camera, world: World) {
    let update_interval = Duration::from_secs_f32(1.0 / 60.0); // 60 FPS

    // TODO: use SIMD? For the render buffer it's kind of a no-brainer. Unstable std feature, though
    // Worth checking if there are significant performance benefits

    // Initialized to 0xff so that the alpha channel is 255, since alpha isn't updated in the render loop
    let render_buffer = Arc::new(Mutex::new([0xffu8; (WIDTH * HEIGHT * 4) as usize]));

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Raytracer Preview")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    // Texture dimensions have to be doubled to match window size for some reason (maybe DPI scaling?)
    let surface_texture = SurfaceTexture::new(WIDTH * 2, HEIGHT * 2, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

    // Raytracing thread
    std::thread::Builder::new()
        .stack_size((WIDTH * HEIGHT * 4 * 3) as usize) // Avoid stack overflow at high res
        .spawn({
            let render_buffer = render_buffer.clone();
            move || {
                render_thread(camera, world, render_buffer);
            }
        })
        .unwrap();

    // Display thread
    let mut last_update = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // Write the image as it is on close request
                // TODO: Maybe halt the render and display threads for this?
                // They're doing useless work and causing the program to freeze before exit.
                let write_thread = std::thread::Builder::new()
                    .stack_size((WIDTH * HEIGHT * 4 * 3) as usize) // Avoid stack overflow
                    .spawn({
                        let render_buffer = render_buffer.clone();
                        move || {
                            let out_file = File::create("preview_out.ppm").unwrap();
                            let mut copy_buf = [0u8; (WIDTH * HEIGHT * 4) as usize];
                            {
                                let buffer = render_buffer.lock().unwrap();
                                copy_buf.clone_from_slice(buffer.deref());
                            }
                            let colors = copy_buf
                                .par_iter()
                                .chunks(4)
                                .enumerate()
                                .map(|(i, chunk)| {
                                    let x = i % WIDTH as usize;
                                    let y = i / WIDTH as usize;
                                    let c = Vec3::new(
                                        *chunk[0] as Float / 255.0,
                                        *chunk[1] as Float / 255.0,
                                        *chunk[2] as Float / 255.0,
                                    );
                                    (x, y, c)
                                })
                                .collect::<_>();
                            let image = Image {
                                colors,
                                width: WIDTH as usize,
                                height: HEIGHT as usize,
                            };
                            Camera::write_image(image, out_file).unwrap();
                        }
                    })
                    .unwrap();
                write_thread.join().unwrap();
                *control_flow = ControlFlow::Exit
            }
            Event::MainEventsCleared => {
                if last_update.elapsed() >= update_interval {
                    window.request_redraw();
                    last_update = Instant::now();
                }
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                // TODO: Find a better way to convert the preview to gamma space. This code is comically slow.
                // frame.par_chunks_mut(4).for_each(|chunk| {
                //     // Fine for alpha as well since sqrt(1.0) = 1.0
                //     for color in chunk.iter_mut() {
                //         let normed = *color as f32 / 255.0;
                //         *color = ((normed.sqrt()) * 255.0) as u8;
                //     }
                // });

                // Update the pixel buffer based on the new rays/pixel colors
                {
                    let buffer = render_buffer.lock().unwrap();
                    frame.clone_from_slice(buffer.deref());
                }

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}

fn render_thread(
    camera: Camera,
    world: World,
    render_buffer: Arc<Mutex<[u8; (WIDTH * HEIGHT * 4) as usize]>>,
) {
    let mut render_pixels: [u32; (WIDTH * HEIGHT) as usize] = core::array::from_fn(|i| i as u32);

    // Pixel render order shuffled so it doesn't render in lines.
    // Looks nicer this way, quicker to make out the general look of the scene.
    let mut rand = thread_rng();
    render_pixels.shuffle(&mut rand);

    // Does a sweep with a single ray per pixel for a fast preview, then accumulates detail
    let num_samples_at_pass: Vec<usize> = vec![
        // If you want more samples than this, that's your problem
        1, 2, 4, 8, 8, 16, 16, 32, 32, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    ];
    let num_samples_total: Vec<usize> = num_samples_at_pass
        .iter()
        .scan(0, |sum, &x| {
            *sum += x;
            Some(*sum)
        })
        .collect();

    // Accumulates samples in multiple passes
    for (i, (num_samples, total_samples)) in num_samples_at_pass
        .iter()
        .zip(num_samples_total)
        .enumerate()
    {
        println!(
            "On sweep {} adding {} sample(s) for a total of {} samples per pixel",
            i + 1,
            num_samples,
            total_samples
        );
        render_pixels.par_iter().progress().for_each(|idx| {
            let x = idx % WIDTH;
            let y = idx / WIDTH;
            let i = (idx * 4) as usize;
            let new_color = camera.pixel_color(&world, x as usize, y as usize, *num_samples);

            let old_color = {
                // This could MAYBE be done without a mutex for better performance
                let buffer = render_buffer.lock().unwrap();
                Vec3::new(
                    buffer[i] as Float / 255.0,
                    buffer[i + 1] as Float / 255.0,
                    buffer[i + 2] as Float / 255.0,
                )
            };

            // Mixes pixel colors proportionally to number of rays used to calculate them
            let new_ratio = *num_samples as Float / total_samples as Float;
            let old_ratio = 1.0 - new_ratio;
            let combined_color = (new_color * new_ratio) + (old_color * old_ratio);

            // Colors must be in a linear color space to accumulate correctly.
            // The math relies on linearity. Gamma is nonlinear.
            // Using a gamma color space with c <- sqrt(c) within the range [0, 1]
            // all colors tends toward white under repeated gamma correction, since sqrt(x) > x for 0 < x < 1
            let (r, g, b) = combined_color.as_rgb_linear();

            if let Ok(mut buffer) = render_buffer.lock() {
                buffer[i] = r;
                buffer[i + 1] = g;
                buffer[i + 2] = b;
                // buffer[i + 3] is the alpha channel. Always 0xff from inception.
            } else {
                println!("failed to acquire buffer lock in render loop");
                exit(1);
            }
        });
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let world: World = gen_scene(8, 8);

    let image_width = WIDTH as usize;
    let image_height = HEIGHT as usize;
    let samples_per_pixel = 32; // not relevant for window_preview
    let max_depth = 100;
    let defocus_angle = 0.0;
    let focus_distance = 10.0;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        focus_distance,
        defocus_angle,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        20.0,
        0.0..Float::MAX,
    );

    window_preview(camera, world);
    println!("Done.");
    Ok(())
}
