# rust-ray-tracer
Ray Tracer written in Rust so I can practice Rust

So far this is the implementation of books 1, 2, & 3 of [*Ray Tracing: In One Weekend*](https://raytracing.github.io/) in Rust on the CPU.

I also have added multi-threading & saving of PNGs through the libraries [**scoped_threadpool**](https://crates.io/crates/scoped_threadpool) and [**softbuffer**](https://github.com/rust-windowing/softbuffer).

Plans for this repo are:

- Copy over NURBS Library from private personal repo
- Implement ray tracing (similar to that of books 1-3 of the [*Ray Tracing: In One Weekend*](https://raytracing.github.io/) series) on the GPU (Vulkan) with versions using the compute pipeline and the hardware accelerated ray tracing pipeline
- Implement PBR
- Add support for ray tracing NURBS on CPU & GPU
- Add more geometry types

Note:
The name of the repository will likely change over time due to the scope of the project growing (i.e. rust-cpu-ray-tracer to rust-ray-tracer)
