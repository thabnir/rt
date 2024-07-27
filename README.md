# rt

Raytracer made in Rust following Peter Shirley's [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).
Parallelized using [Rayon](https://github.com/rayon-rs/rayon).

## Live Rendering

Supports a live render preview so you can monitor your render's progress. Uses [winit](https://github.com/rust-windowing/winit) and [pixels](https://github.com/parasyte/pixels) for the hard work of drawing the render to the screen.

![Live render preview](https://github.com/user-attachments/assets/73a87dbe-7503-44db-82e9-313ffc7b4dbb)


## Example Render

Renders a 1920x1080 image with 10 bounces per ray and 1000 rays per pixel in 1hr 17mins on a 16-inch 2021 M1 Macbook Pro.

![final_render](./images/final_render.png)
