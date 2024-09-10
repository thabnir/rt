# Things to maybe do for `rt`

Read more and learn more

## Resources

[RTIOW Further readings](https://github.com/RayTracing/raytracing.github.io/wiki/Further-Readings)
[RTIOW Next Steps](https://github.com/RayTracing/raytracing.github.io/wiki/Aggregation-of-Possible-Next-Steps)
[PBRT](https://pbr-book.org/4ed/Monte_Carlo_Integration)

### Related repos

[Toy Ray Tracer in Rust](https://github.com/Twinklebear/tray_rust/blob/master/src/film/filter/mitchell_netravali.rs)
[Ray Tracing in Vulkan](https://github.com/GPSnoopy/RayTracingInVulkan)
[Ray Tracing in WebGPU](https://github.com/Nelarius/weekend-raytracer-wgpu?tab=readme-ov-file)

- [Blogpost 1](https://nelari.us/post/weekend_raytracing_with_wgpu_1/)
- [Blogpost 2](https://nelari.us/post/weekend_raytracing_with_wgpu_2/)

[WebGPU Interactive Ray Tracer](https://github.com/nelarius/rayfinder)

- [Blogpost 1](https://nelari.us/post/pathtracer_devlog/)

If I ever implement GPU support:

- [wgpu](https://github.com/gfx-rs/wgpu) WebGPU for Rust
- [vulkano](https://github.com/vulkano-rs/vulkano) Vulkan for Rust (would be faster since Vulkan exposes ray tracing functionality, but probably won't use it, since I want this to work on all of my computers without much hassle)

## Important:

- Refactor color. The progressive renderer should never touch a 0-256 RGB value. 8-bit RGB colors should NEVER be converted to color `Vec3`s.
- Figure out how color values greater than 1.0 work and if they should ever be possible

## Bugs

- [ ] Sphere textures have rendered upside-down ever since I changed the UV mapping code to make it not upside-down for meshes
- [ ] I don't think my Halton sampling actually works right. TODO: [read this](https://extremelearning.com.au/unreasonable-effectiveness-of-quasirandom-sequences/)
- [x] Weird floating point shadow acne type errors near the top of very large spheres when using `f32` instead of `f64` (damn you, floating point numbers)
  - The noise is visually ripple-patterned when using Halton sampling instead of `thread_rng()` as the source of randomness for ray sampling
  - This is currently "solved" by using `f64` and no large spheres
- Can't make the camera look straight down or up or the entire thing totally crashes and explodes
- [x] Meshes are sometimes sort of transparent in a weird way when they absolutely should NOT be. Happens with metals for sure, likely other materials too.
  - Fixed! The normals of triangles weren't actually being normalized on calculation, which caused many issues.

## Rendering Features

- [ ] Shapes

  - [x] Spheres
  - [x] Triangles
    - [x] Meshes / model importing
      - [x] basic .obj support (no materials, only triangles)
        - Likely that .obj support will be dropped in favor of gltf, since my gltf support is already much better, and it's way easier to find gltf scenes and models than for obj, and gltf is generally a better, more modern format.
      - [x] .gltf support (basic materials and texturing support, triangles)
  - [ ] Quads
  - [ ] Arbitrary polygon primitives (for .obj if a mesh can't be triangulated)
  - [ ] Cubes

- [ ] .gltf support

  - [x] Meshes
  - [x] Base color textures
  - [x] Roughness/metallic fuzz textures
  - [ ] Emission maps (or light emission of any kind, actually)
  - [ ] Transparency maps
  - [ ] Normal maps
  - [ ] Cameras
  - [ ] Performance improvements
    - [ ] Not taking up 40GB RAM and then crashing while loading Sponza (or taking 30GB of RAM but running for the suit of armor)
    - Maybe has to do with how images are stored? Have to do a memory profile to see what's getting me, then fix it.

- [ ] Materials & Textures

  - [x] Diffuse (Lambertians)
  - [x] Metal
  - [x] Glass (Dielectrics)
  - [ ] Emissives (goes with light sources)
  - [x] Texture maps
    - [x] Rotate spheres and their textures
    - [x] Texture maps for meshes
  - [ ] Normal maps
  - [ ] Bump maps
  - [x] [Better sky model](https://nelari.us/post/weekend_raytracing_with_wgpu_2/)
  - [x] Mesh texturing system

- [ ] Improved light rendering

  - [ ] Lights/emissives

    - [ ] Point lights
    - [ ] Surface lights
    - [ ] Sun model (make it play nicely with sky model for a whole time-of-day system)

  - [ ] Specular highlights
  - [ ] Bloom

- [ ] Volumes (volumetric substances like smoke/fog)

- [x] Improved camera simulation

  - [x] Depth of field
  - [ ] <s>Motion blur</s> support dropped when I added the BVH.
    - [ ] Re-add this feature? I believe this requires a time value for every ray-object intersection, which I can't do when using the `bvh` crate since they use their own custom ray type. Would have to find a workaround or alternate method.

- [ ] Image denoising (I know literally nothing about this)

- [x] Russian Roulette for unbiased early termination on low-impact rays

## User Features

- [x] Progressive rendering preview with multiple sweeps

  - [ ] Live gamma correction (figure out exactly what gamma correction is, too)
    - [ ] Figure out how exactly tonemapping plays into the rendering pipeline.
    - [ ] Figure out color values and emissive stuff.
    - Could probably do tonemapping and gamma correction on the GPU via a simple fragment shader to get it to run in real time.

- [ ] Live interactivity & re-rendering

  - [ ] Camera position controls w/ mouse and keyboard
  - [ ] GUI
    - [ ] Camera setting sliders (depth of field, field of view, etc.)
    - [ ] Scene selector
    - [ ] Scene editing (IDEK if I even want this feature)
    - [x] Click on a pixel to fire a ray over there and get debug information about where it hits

- [ ] Web support.

  - [ ] Progressive rendering preview
  - [ ] Single-threaded version
  - [ ] Multi-threaded version
  - [WGPU Tutorial with web support](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/#the-code)
    - Multithreading for WASM requires much more work than other targets; browser has no locks
      - [parallel ray tracer with wasm-bindgen for Rayon](https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html)

- [ ] CLI or file-based rendering support
  - [ ] <s>Scene descriptions and settings files [JSON?](https://blog.singleton.io/posts/2022-01-02-raytracing-with-rust/#read-scene-data-from-json-file)
  - Probably not to do until much later in the project. Don't need to calcify a scene description format when most of the requisite features aren't in place.</s>
  - Just using .gltf instead since that's what actual 3d scenes use so I can yoink and render from online
    - [ ] Add a CLI to specify which .gltf scene to render

## Optimizations

- [x] Multithreaded concurrency with Rayon

  - [ ] Actually benchmark and tweak settings. Maybe use tiling?

- [x] Bounding Volume Hierarchy

- [ ] SIMD for rays and pixels.

- [ ] Improve sampling efficiency

  - [x] Quasi-Monte-Carlo sampling technique (Halton numbers)
    - [ ] Figure out if this even works (MSE from long baseline render + visual noise inspection of various techniques vs true Monte-Carlo)
      - [ ] Compare against uniform and stratified sampling methods
    - [ ] Read PBRT's stuff about Monte Carlo and sampling techniques. [book](https://pbr-book.org/4ed/Monte_Carlo_Integration/Improving_Efficiency)
      - [ ] [Multiple-Importance Sampling](https://pbr-book.org/4ed/Monte_Carlo_Integration/Improving_Efficiency#MultipleImportanceSampling) to find the best sampling technique for a given region on the fly
  - [ ] Adaptive sampling to target more rays at noisy areas
  - [ ] [ReSTIR](https://www.youtube.com/watch?v=gsZiJeaMO48) [paper 1](https://d1qx31qr3h6wln.cloudfront.net/publications/ReSTIR%20GI.pdf)

- [ ] GPU support. **Big** project. Total rewrite. Maybe better suited to a sequel project

## Development features and chores

- [ ] Refactor the render preview code to be more separate and generally less shit

- [ ] Debug view support

  - [ ] Surface normal visualization
  - [ ] Ray bounce count visualization

- [ ] Decouple the render display from the ray tracer (currently, the accumulator uses the previous pixel values rather than vec3s for rendering information)

- [ ] Performance benchmarks with Criterion

- [ ] Noise/image quality per sample (or per second) benchmarks

  - [ ] Fixed-seed scene generation for better comparisons

- [ ] Tests

  - [ ] Unit tests
  - [ ] Other tests (how does one test a renderer?)

- [ ] Update `winit` and `pixels` to use their newest versions (ugh)

- [ ] Automatic ppm to png conversion (or other, similar lossless format, e.g. jxl, webp, etc.)
