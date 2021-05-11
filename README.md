# Iris

CPU path tracer written in Rust.

Licensed under the GPLv3.

Features (WIP):
* Spectral rendering (including wavelength-dependent path generation) with [Hero Wavelength Spectral Sampling](https://cgg.mff.cuni.cz/~wilkie/Website/EGSR_14_files/WNDWH14HWSS.pdf)
* Spectral upsampling ([Jakob et al.](http://rgl.epfl.ch/publications/Jakob2019Spectral))
* Parallel and progressive refinement
* Multiple importance sampling
* Russian roulette
* Next event estimation
* HDR environment maps

TODO:
* Add README image
* Clean up tile
* SIMD more things (matmul, vec3, Spectrum eval, upsampling)
* Analytic light integration test (Le = 0.5, f = 0.5, radiance should be 1)
* More shapes
* Serialize scene from RON
* BVH / other spatial accel
* MTL file handling
* Reconstruction filtering
* Adaptive sampling (?)
* Direct image output
* Tonemapping options (ACES)
* Camera lens sim + vigenetting + DoF
* Volume rendering
* Motion blur / animation
* Real time rasterizing preview 
* Own PNG / HDR code
* PGO
* Clean up normal offseting
* MIS compensation
* Triangles
* Coherent ray bundles
* SDF shapes
* Mipmapping / texture filtering
* Catmull-Clark
* Denoising
* License
