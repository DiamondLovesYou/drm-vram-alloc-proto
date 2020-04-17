A small prototype I wrote to test writing to GPU memory from the CPU, without going through a burdensome API like
Vulkan (it's portable, sure, but pretty heavy for the sole purpose of allocating host visible GPU local memory).

Uses libdrm_amdgpu directly to get a CPU mapping. Using HSA/ROCm here isn't possible for this for whatever reason (on
dGPUs at least).

I hope to incorporate this sort of thing into Geobacter so that GPU local memory allocations can be used in Rust
Boxes, Vecs etc safely (if not *fastly*, since, you know, reads have to traverse the PCIE bus) with the usual modern
conveniences.
