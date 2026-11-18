# Rust workspace

This is a Rust port of Spirula Studio, built beside the C++ trainer. The C++
tree is the implementation that trains, meshes and reconstructs. The Rust
workspace is the start of a replacement: it compiles, it reads the dataset
formats the C++ parsers read, and it runs a real pinhole training step. It
does not claim to match a C++ run.

License remains GPL-3.0-or-later. The port is derived from Spirula Studio by
harry7557558 (https://github.com/harry7557558/spirula-studio). There is no
PyTorch, nerfstudio or gsplat dependency.

## Build

```bash
cargo test --workspace
cargo build --release
./target/release/spirula --help
./target/release/spirula train --synthetic --num-iterations 40
./target/release/spirula train /path/to/dataset --num-iterations 30 --max-edge 64
./target/release/spirula devices
```

The workspace `rust-version` is 1.88, which is what the locked `image` crate requires. CI builds with that toolchain and `cargo test --workspace --locked`.

`cargo build` does not need the Vulkan SDK. `spirula train --backend vulkan` and `spirula devices` need a Vulkan loader (`libvulkan.so.1`) and a device.
A software device such as lavapipe is enough for the projection test.

The C++ build is unchanged: `bash build_develop.bash -DSS_BACKEND=vulkan`.

## Layout

| crate | role |
|---|---|
| `spirula-core` | camera models, lens distortion, SH, spherical-beta lobes, pinhole 3DGS projection, the training flags this step reads |
| `spirula-data` | COLMAP text/binary, Nerfstudio `transforms.json`, splat PLY, point-cloud PLY, PNG/JPEG |
| `spirula-backend` | `Backend` trait, CPU projector, Vulkan compute using `shaders/project_pinhole.comp` |
| `spirula-engine` | composite, L1/L2, Adam, optional split/prune, SSIM as a metric |
| `spirula` | `spirula` binary |

## What works

- Camera names and the host lens math from `src/data/CameraMath.cpp`: pinhole,
  fisheye, equisolid, equirectangular and orthographic rays, OpenCV and
  thin-prism distortion, and the 0.75 pinhole-coverage split test.
- COLMAP `cameras` / `images` / `points3D` in `.txt` and `.bin`, for the
  models that map onto a distortion tier exactly (pinhole, opencv, the
  fisheye family, thin-prism fisheye, equirectangular).
- Nerfstudio `transforms.json` in that same exact-tier subset. Poses stay in
  the file frame. `applied_transform` is reported and not inverted.
- 3DGS PLY read and write, including `f_rest` and `sb_params`, plus an xyz/rgb
  point cloud.
- Pinhole 3DGS projection (means, quaternion, log-scale, logit opacity) on
  the CPU and, when a device exists, as a Vulkan compute dispatch. The two
  are tested against each other. The dispatch is a GLSL compute shader of
  the no-distortion formulas in `src/shaders/primitive_3dgs.slang`, compiled
  with glslang and committed as SPIR-V. It is not yet the C++ backend's
  Slang pipeline.
- A training step on pinhole frames with no lens distortion: front-to-back
  composite sorted by camera-space z, L1 and L2, Adam on means, opacity and
  DC colour. 8-bit images stay code values, matching the C++ default
  (`image_color_is_linear` false). The run writes `config.json` with
  `splat_color_is_linear` set the same way, which is what the viewer reads.
  SH above DC and spherical-beta lobes are evaluated and not updated.
  `spirula train --synthetic` is a closed scene whose loss falls.
- l1, PSNR and an 11x11 SSIM metric (the no-mask path of `src/app/EvalMetrics.cpp`).
  SSIM is not in the gradient.
- `--densify split-prune` prunes faint splats and splits large ones, and it
  stops at `cap_max`. `--densify mcmc` exits before any step is run.

## What is partial

- Only pinhole frames with zero distortion are trained. Fisheye, equirect and
  distorted frames are skipped and counted in the log. Their camera math is
  tested on the host; it is not the rasterizer.
- The backward holds the 2D covariance fixed, so scales and quaternions do
  not move except when split/prune edits them.
- `config.json` loads the flags this step reads. Every other key is listed
  in `rust-port.json` and ignored. Defaults for the known flags match
  `src/config/TrainConfig.h`.
- Bilateral grid, PPISP, mask, depth, normal and pose-normalization flags are
  not applied. If they are on, the log says so. `--require-ported` refuses
  to start when one of those result-changing flags is enabled.
- The Vulkan backend projects. Compositing and the optimizer stay on the CPU.
- `--viewer` binds a status URL that returns JSON. It is not `viewer.html`.
- Images are resized so the long edge is `--max-edge` (default 64). The seed
  cloud is truncated to `--max-splats` (default 1024).

## Still C++

SfM, masking, meshing, geometry models, partitioning, the GUI, i18n, the
fused training kernels, MCMC densification, checkpoints (`state.tar`), and
the CUDA backend. `spirula sfm`, `sam`, `mesh`, `geometry` and `partition`
exit 2 and point here.

Metashape projects, and COLMAP models that the C++ reader fits (FOV, division,
EUCM, a rational `FULL_OPENCV`), are errors rather than a silent pinhole.

## Next port steps

1. Distorted and wide-angle projection on the device, reusing the Slang
   kernels under `src/backend/vulkan/shaders/` instead of new WGSL.
2. Covariance backward, then SH bands and spherical-beta lobes.
3. The densification strategy the C++ trainer actually runs.
4. `state.tar` resume and the rest of `TrainConfig`.
5. Bilateral grid and PPISP, after the raster matches on a pinhole scene.
