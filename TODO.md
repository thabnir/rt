# Things to maybe do for `rt`

Read more and learn more

Resources:
[RTIOW Further readings](https://github.com/RayTracing/raytracing.github.io/wiki/Further-Readings)
[RTIOW Next Steps](https://github.com/RayTracing/raytracing.github.io/wiki/Aggregation-of-Possible-Next-Steps)
[PBRT](https://pbr-book.org/4ed/Monte_Carlo_Integration)

## Rendering Features

- [ ] Shapes

  - [x] Spheres
  - [ ] Quads
  - [ ] Triangles
    - [ ] Meshes / model importing
  - [ ] Cubes

- [ ] Materials & Textures

  - [x] Diffuse (Lambertians)
  - [x] Metal
  - [x] Glass (Dielectrics)
  - [ ] Emissive (goes with light sources)
  - [ ] Texture map
  - [ ] Normal map
  - [ ] Bump map

- [ ] Improved light rendering

  - [ ] Lights

    - [ ] Point lights
    - [ ] Surface lights
  - [ ] Shadows
  - [ ] Specular highlights
  - [ ] Bloom

- [ ] Volumes

- [x] Improved camera simulation
  - [x] Depth of field
  - [x] Motion blur

- [ ] Image denoising (I know literally nothing about this)

## User Features

- [x] Progressive rendering preview with multiple sweeps

- [ ] Live interactivity & re-rendering

  - [ ] Camera position controls w/ mouse and keyboard
  - [ ] GUI
    - [ ] Camera setting sliders (depth of field, field of view, etc.)
    - [ ] Scene selector
    - [ ] Scene editing (IDEK if I even want this feature)

- [ ] Web support.

  - [ ] Progressive rendering preview
  - [ ] Single-threaded version
  - [ ] Multi-threaded version
  - [WGPU Tutorial with web support](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/#the-code)
    - Multithreading for WASM requires much more work than other targets; browser has no locks
      - [parallel ray tracer with wasm-bindgen for Rayon](https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html)

- [ ] CLI or file-based rendering support
  - [ ] Scene descriptions and settings files (JSON?)
  - [ ] Default camera settings
  - Probably not to do until much later in the project. Don't need to calcify a scene description format when most of the requisite features aren't in place.

## Optimizations

- [x] Multithreaded concurrency with Rayon
  - [ ] Actually benchmark and tweak settings. Maybe use tiling?

- [ ] Bounding Volume Hierarchy

- [ ] SIMD for rays and pixels.

  - Shouldn't be too hard to implement.

- [ ] Improve sampling efficiency

  - [x] Quasi-Monte-Carlo sampling technique (Halton numbers)
    - [ ] Figure out if this even works (MSE from long baseline render + visual noise inspection of various techniques vs true Monte-Carlo)
        - [ ] Compare against uniform and stratified sampling methods
    - [ ] Read PBRT's stuff about Monte Carlo and sampling techniques. [book](https://pbr-book.org/4ed/Monte_Carlo_Integration/Improving_Efficiency)
      - [ ] [Multiple-Importance Sampling](https://pbr-book.org/4ed/Monte_Carlo_Integration/Improving_Efficiency#MultipleImportanceSampling) to find the best sampling technique for a given region on the fly
  - [ ] Adaptive sampling to target more rays at noisy areas

- [ ] GPU support. **Big** project. Total rewrite. Maybe better suited to a sequel project

## Development features and chores

- [ ] Debug view support
  - [ ] Surface normal visualization
  - [ ] Ray bounding-box collision check visualization (like Sebastian Lague's)
  - [ ] Ray bounce count visualization

- [ ] Performance benchmarks with Criterion

- [ ] Noise/image quality per sample (or per second) benchmarks
    - [ ] Fixed-seed scene generation for better comparisons

- [ ] Tests

  - [ ] Unit tests
  - [ ] Other tests (how does one test a renderer?)

- [ ] Update `winit` and `pixels` to use their newest versions (ugh)

- [ ] Automatic ppm to png conversion (or other, similar lossless format, e.g. jxl, webp, etc.)