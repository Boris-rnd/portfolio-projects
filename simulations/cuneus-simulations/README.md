# Project description

Some simulations made using the cuneus rust engine (It allows to make compute shaders easily.)
So you can make your own renderer + physics simulations

The project contains a n-body particle simulator, a fluid simulation, a wave simulation, a wave schrodinger simulation (using the leap-frog euler method).
I'm also working on bringing back my voxel raytracer.


## How to run

`cargo run --bin <program>`
Here are the following simulations available:
- `particle`: n-body particle simulator
- `fluid`: fluid simulation
- `wave`: wave simulation
- `wave_schrodinger`: wave schrodinger simulation (using the leap-frog euler method)
- `raytrace`: voxel raytracer
- `particle-basin`: 2 million particles bouncing... But because of f32 precision in wgsl it doesn't look as good as I wanted... I tried using fp64 code with 2 f32 but it didn't seem to increase precision that much (maybe I'm doing it wrong)

## Wave Schrodinger Simulation
If you select the wave schrodinger simulation, you can change the parameters of the simulation using the following arguments:
An example of prism that looks cool
```bash
cargo run -- --freq 1500.0 --size 0.025 --potential "step(0.1, max(-triangle((uv-vec2(0., 0.2))*8.0), 0.))"
```
Quantum tunneling !
```bash
cargo r -- --freq 1500 --size 0.045 --potential "1.0-step(0.01, abs(uv.y-0.2))*1.0"
```
Or a simple circular trap:
```bash
cargo r -- --freq 700 --size 0.015 --potential "pow(length(uv+vec2(0., 0.2))*10., 3)"
```
Random potential:
```bash
cargo r -- --freq 700 --size 0.015 --potential "random(sin(f32(pos.x))+cos(f32(pos.y)*100000.1313)+time_data.time)*0.5"
```


Freq and size are interpreted as numbers, but potential is code, which has access to the "uv" variable, which is the coordinate of the cell (distance from origin).
And potential code is getting replaced raw in the shader code, so you can play with anything.

You can try release mode if you are CPU-bounded... For this just change the iteration count in the code and run in release mode: `cargo run --release`
I get ~40fps in release mode with 50 iterations whereas I get 25fps in debug
But with 20 iterations, it's ~60fps in debug mode.

TODO: Change wave form from cli

### Results
I got a lot of great results, but I haven't kept track of all of them... But any configuration will lead to mind blowing visuals !
By the way, it's not so complicated to change the shader code to make any other simulation (change brightness, ...) => It has hot reloading !

![positive10_in_center_with_no_momentum_wave](positive10_in_center_with_no_momentum_wave.png)
![wave1500_in_triangle_over8](wave1500_in_triangle_over8.png)

## TODO

### Raytrace

1. The voxel imports are very slow because I insert every voxel individually, I should try to batch them in chunks (maybe make a hashmap of voxels which I can then insert more easily)
2. Make the raytrace faster
3. Add lighting
4. Make the raytracer more realistic
5. Add beam splitter, and other optical effects (also reflect/diffract depending on frequency of wave...)
