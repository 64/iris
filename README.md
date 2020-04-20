# Iris

CPU ray tracer written in Rust.

Features (WIP):
* Spectral rendering (including wavelength-dependent path generation) with [Hero Wavelength Spectral Sampling](https://cgg.mff.cuni.cz/~wilkie/Website/EGSR_14_files/WNDWH14HWSS.pdf)
* Parallel and progressive refinement
* Next event estimation
* Multiple importance sampling
* Spectral upsampling (Jakob et al.)
* HDR env maps

TODO:
* SIMD more things (matmul)
* More shapes
* PdfSet? Vec4?
* Serialize from RON
* Write own sobol code
* BVH / other spatial accel
* MTL file handling
* Reconstruction filtering
* Adaptive sampling (?)
* Image / HDR output
* Tonemapping options (ACES)
* Camera lens sim + vigenetting + DoF
* Volume rendering
* Motion blur / animation
* Real time preview
* Own PNG / HDR code
* PGO
* Clean up normal offseting
