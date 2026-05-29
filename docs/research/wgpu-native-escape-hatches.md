# WGPU Native Escape Hatches

**Status:** RESEARCH COMPLETE
**Date:** 2026-05-27
**Scope:** Bevy + `wgpu` native backend access via `as_hal()` on DX12, Vulkan, and Metal

## Executive Summary

`wgpu` intentionally exposes a safe, portable graphics API. `as_hal()` is the escape hatch that lets you borrow the backend-specific HAL device/queue/etc. when `wgpu` is running on a matching backend. That gives you access to native objects such as:

- DirectX 12: raw `ID3D12Device*`, `ID3D12GraphicsCommandList*`, `ID3D12CommandQueue*`, and related resources
- Vulkan: raw `ash::Device`, `ash::Instance`, queue handles, and Vulkan extension entry points
- Metal: raw `metal::Device`, `metal::CommandQueue`, `metal::CommandBuffer`, and other Metal objects

The important constraint is that this is **interop**, not ownership transfer. You can use native APIs to create additional resources or encode commands, but you must not violate `wgpu`'s synchronization, lifetime, or queue ownership rules.

For Bevy, the render world exposes the underlying `wgpu::Device` and `wgpu::Queue` through `RenderDevice` and `RenderQueue`, so the usual entry point is:

```rust
let wgpu_device = render_device.wgpu_device();
let wgpu_queue = render_queue.wgpu_queue();
```

From there, backend-specific `as_hal::<...>()` access is possible inside render-world systems, assuming the active backend matches the target API.

## 1. How `wgpu::Device::as_hal::<Dx12>()` works

`as_hal()` is a backend-gated borrow of the internal HAL device. In practice:

- it returns `Option<...>` because the active backend may not be the one you requested
- the borrow is tied to the lifetime of the `wgpu::Device`
- the returned handle is backend-specific and only valid while the parent `wgpu` device remains alive
- the API is `unsafe` because you can break `wgpu`'s invariants if you use the raw backend object incorrectly

On DirectX 12, the HAL type gives you access to the native D3D12 device and related low-level methods. That means you can:

- create native D3D12 resources with the raw `ID3D12Device`
- query adapter/device capabilities that `wgpu` does not surface directly
- build native command lists, descriptor heaps, root signatures, PSOs, and DXR objects
- interop with other DX12-native libraries that accept `windows` COM interfaces

What you do **not** get:

- no ownership of the device away from `wgpu`
- no guarantee that `wgpu` will understand or track the resources you create
- no license to submit arbitrary native work on the same queue without respecting `wgpu`'s ordering

### DX12 sketch

The exact HAL type name depends on the `wgpu-hal` version, but the pattern is:

```rust
use wgpu::HalApi;
use wgpu::Device as WgpuDevice;

fn with_dx12_device<R>(device: &WgpuDevice, f: impl FnOnce(&windows::Win32::Graphics::Direct3D12::ID3D12Device) -> R) -> Option<R> {
    unsafe {
        device.as_hal::<wgpu::hal::api::Dx12, _, _>(|hal_device| {
            // HAL device is backend-specific. The exact accessor name is versioned.
            let raw: &windows::Win32::Graphics::Direct3D12::ID3D12Device = hal_device.raw_device();
            f(raw)
        })
    }
}
```

In current `wgpu` releases, the `as_hal` family is the right abstraction for borrowing the native backend object; the concrete helper names on the returned HAL wrapper may vary across versions.

## 2. Can you create custom render passes that use DXR ray tracing?

Yes, but not through `wgpu`'s portable render-pass API.

The practical answer is:

- **Yes** if you use `as_hal()` to get the raw DX12 device/command list and then record native DXR work yourself
- **No** if you mean “a normal `wgpu::RenderPass` configured to use DXR under the hood”

DXR is a native DirectX 12 feature exposed through `ID3D12Device5+` and `ID3D12GraphicsCommandList4+` style interfaces. `wgpu` does not provide a portable DXR render-pass abstraction.

That means a Bevy integration typically looks like:

1. let Bevy run its normal render graph and `wgpu` passes
2. hook a custom render-world system or render graph node
3. borrow the DX12 HAL device/queue
4. create or cache DXR resources natively
5. record ray tracing dispatches into a native command list
6. synchronize with `wgpu` carefully before/after the native submission

### DXR limitations

- DXR acceleration structures are not `wgpu` resources
- shaders/pipelines are native DXIL/DXR constructs, not `wgpu` shader modules
- you must handle barriers, descriptor heaps, root signatures, and scratch buffers yourself
- you need a queue/submission story that does not race `wgpu`'s own work

## 3. Can you use mesh shaders via the native DX12 pipeline state?

Yes, technically, if you drop to native DX12.

`wgpu` itself does not give you a “native DX12 pipeline state object” API. But if you borrow the DX12 HAL device, you can create native PSOs that use the DirectX 12 mesh/task shader pipeline path, provided the adapter and driver support it.

Important distinction:

- **Possible with HAL interop:** native DX12 mesh shaders / pipeline state objects
- **Not exposed by portable `wgpu` API:** no cross-backend mesh-shader pipeline feature in the same sense as standard render/compute pipelines

So the answer is:

- **Yes**, native mesh shaders are possible through the DX12 escape hatch
- **No**, not as a first-class `wgpu` render pipeline abstraction

## 4. How Bevy exposes the `wgpu` device

Bevy's render world exposes the GPU objects through `RenderDevice` and `RenderQueue`.

The normal access pattern is:

```rust
use bevy::render::renderer::{RenderDevice, RenderQueue};

fn system(render_device: Res<RenderDevice>, render_queue: Res<RenderQueue>) {
    let device = render_device.wgpu_device();
    let queue = render_queue.wgpu_queue();

    // Use `device` / `queue` for `wgpu` work, or `as_hal()` for backend-native interop.
}
```

That is the point where you can branch into:

- portable `wgpu` work
- backend-specific HAL interop
- Bevy render graph integration

## 5. Example: get raw DX12 device from Bevy, create a DXR acceleration structure

This is an illustrative skeleton. It shows the flow, but not a complete production DXR implementation.

```rust
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::core::Interface;

#[derive(Resource, Default)]
struct DxrState {
    accel: Option<ID3D12Resource>,
}

fn build_dxr_from_bevy(render_device: Res<RenderDevice>, mut state: ResMut<DxrState>) {
    let wgpu_device = render_device.wgpu_device();

    unsafe {
        let maybe_result = wgpu_device.as_hal::<wgpu::hal::api::Dx12, _, _>(|hal_device| {
            // Version-specific: this is the native D3D12 device behind wgpu.
            let d3d12: &ID3D12Device = hal_device.raw_device();

            // DXR requires a sufficiently new device interface.
            let d3d12_5: ID3D12Device5 = d3d12.cast().expect("DXR requires ID3D12Device5");

            // Minimal example: create an empty buffer to act as a placeholder scratch/output resource.
            // A real BLAS/TLAS build needs committed buffers with the proper flags and sizes.
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: 0,
                Width: 4096,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            };

            let mut accel_buffer: Option<ID3D12Resource> = None;
            d3d12.CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_COMMON,
                None,
                &mut accel_buffer,
            )?;

            // Real DXR work would continue here:
            // - build geometry descriptors
            // - create a bottom-level acceleration structure inputs desc
            // - query prebuild sizes
            // - allocate scratch + result buffers
            // - call BuildRaytracingAccelerationStructure on an ID3D12GraphicsCommandList4
            // - submit and fence

            state.accel = accel_buffer;
            Ok::<(), windows::core::Error>(())
        });

        maybe_result.expect("wgpu HAL access failed");
    }
}
```

### DXR-specific notes

- `CreateCommittedResource` is just a placeholder here so the example stays short.
- A real acceleration-structure build requires `D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_DESC`.
- You will usually need `ID3D12Device5` for prebuild info and `ID3D12GraphicsCommandList4` or newer for the build call.
- The native objects created here are outside `wgpu`'s resource tracking.

## 6. Example: Vulkan, `ash` device extraction, `VK_KHR_ray_tracing_pipeline`

On Vulkan, the pattern is the same:

1. borrow the Vulkan HAL device from `wgpu`
2. extract the raw `ash::Device`
3. enable/use ray tracing pipeline and acceleration structure extensions
4. create native Vulkan ray tracing objects

```rust
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use ash::vk;

fn with_vulkan_device(render_device: Res<RenderDevice>) {
    let wgpu_device = render_device.wgpu_device();

    unsafe {
        let _ = wgpu_device.as_hal::<wgpu::hal::api::Vulkan, _, _>(|hal_device| {
            // Version-specific accessor; the raw ash device is the thing you want.
            let device: &ash::Device = hal_device.raw_device();

            // With `VK_KHR_ray_tracing_pipeline`, you typically use:
            // - vk::AccelerationStructureKHR
            // - vk::RayTracingPipelineCreateInfoKHR
            // - vk::StridedDeviceAddressRegionKHR
            // - vkCmdTraceRaysKHR
            //
            // In ash, extension loaders are usually created from the instance + device.
            let _ = device.handle();
        });
    }
}
```

### Vulkan ray tracing sketch

```rust
use ash::{vk, Device};

fn build_vk_ray_tracing_pipeline(device: &Device) -> vk::Pipeline {
    // Pseudocode: you must wire shader stages, groups, recursion depth,
    // and the ray tracing pipeline extension loader.
    //
    // let rt = ash::khr::ray_tracing_pipeline::Device::new(instance, device);
    // let pipeline = unsafe { rt.create_ray_tracing_pipelines(...) }?;
    //
    // For a real app, also create:
    // - descriptor sets / layouts
    // - SBT buffers
    // - acceleration structures
    // - scratch buffers
    //
    vk::Pipeline::null()
}
```

## 7. Example: Metal, `metal-rs` device extraction, ray tracing support

Metal interop follows the same shape, but the ray tracing story is different from DX12/Vulkan:

- you can extract the raw `metal::Device`
- you can use Metal command queues, buffers, textures, and encoders
- Apple GPU ray tracing exists in the Metal API surface, but it is gated by OS/GPU support and is not the same as DXR or Vulkan ray tracing

```rust
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;

fn with_metal_device(render_device: Res<RenderDevice>) {
    let wgpu_device = render_device.wgpu_device();

    unsafe {
        let _ = wgpu_device.as_hal::<wgpu::hal::api::Metal, _, _>(|hal_device| {
            // Version-specific accessor; this is the native MTLDevice behind wgpu.
            let device: &metal::Device = hal_device.raw_device();

            // Typical Metal objects you can create:
            // - command queues
            // - buffers
            // - textures
            // - acceleration structures on supported platforms
            let queue = device.new_command_queue();
            let _command_buffer = queue.new_command_buffer();
        });
    }
}
```

### Metal ray tracing support

Metal ray tracing is available on supported Apple platforms with the relevant framework/API level. In practice:

- you must check platform and GPU support at runtime
- you must use the Metal acceleration-structure APIs when available
- you should expect feature divergence compared with DXR/Vulkan ray tracing

If the target device does not support the ray tracing APIs, the code must fall back cleanly.

## 8. Safety and lifetime concerns

This is the part that matters most.

### Lifetime

- The raw native device borrowed from `as_hal()` is only valid while the parent `wgpu::Device` is alive.
- Do not store raw native references beyond that lifetime unless you also keep the owning `wgpu` device alive.
- Bevy may recreate or drop GPU state during app shutdown, device loss, or backend reconfiguration.

### Synchronization

- `wgpu` and your native code may be sharing the same underlying queue/device.
- If you write native commands, you must ensure they are ordered correctly relative to `wgpu` work.
- Use fences, queue submission ordering, and explicit resource transitions/barriers where required by the backend.

### Resource ownership

- `wgpu` does not know about native resources you create via the HAL device.
- Native resources must not be fed back into `wgpu` as if they were created by `wgpu` unless the backend/API explicitly supports that interop path.
- Avoid aliasing the same memory in ways that confuse either API.

### `unsafe` boundary

- `as_hal()` is `unsafe` for a reason.
- The compiler cannot verify that your native calls respect the invariants `wgpu` expects.
- You need backend-specific knowledge for descriptor heaps, command allocators, image layouts, and queue ownership.

### Backend specificity

- Any code behind `as_hal::<Dx12>()` must be gated to DX12 runtime paths.
- Vulkan and Metal need their own code paths and their own feature checks.
- Don’t assume a Bevy render target is always running on the backend you want.

## 9. Can this be done per-frame alongside Bevy's normal render passes?

Yes, but only if you treat the native work as a carefully ordered sidecar to Bevy's render graph.

Practical answer:

- **Yes**: per-frame native DX12/Vulkan/Metal work can run alongside Bevy's passes
- **But**: you must schedule it in a render-world stage/node with explicit ordering
- **And**: you must synchronize so that `wgpu` and native commands do not race

Typical pattern:

1. Bevy submits its normal `wgpu` render graph work
2. your custom render graph node runs before or after a chosen Bevy pass
3. the node borrows the native backend device/queue
4. the node records native commands or updates native resources
5. the node signals completion with a fence or equivalent sync primitive

If you need to sample native-rendered output in Bevy passes, you also need a clear texture/resource handoff path.

## Civis Bevy reference client

Civis wires native-only adapter selection in [`clients/bevy-ref/src/native_backend.rs`](../../clients/bevy-ref/src/native_backend.rs):

- `CIV_BEVY_BACKEND` — optional override: `dx12`, `vulkan`, or `metal` (case-insensitive; GLES / WebGPU rejected)
- `native_only_backends()` — Windows default **DX12 \| Vulkan**; macOS **Metal \| Vulkan**; Linux **Vulkan** only
- `native_render_plugin()` — `RenderPlugin` with `WgpuFeatures::POLYGON_MODE_LINE` for chunk wireframe debug

Operator docs: [`clients/bevy-ref/README.md`](../../clients/bevy-ref/README.md#native-gpu-backends-civ_bevy_backend). Traceability: **FR-CIV-BEVY-026** / P-W1 kickoff **item 51**.

## Bottom Line

- `as_hal()` is the correct way to get to backend-native objects from `wgpu`
- Bevy exposes the underlying `wgpu::Device` and `wgpu::Queue` through `RenderDevice` and `RenderQueue`
- DXR, native DX12 mesh shaders, Vulkan ray tracing pipeline, and Metal ray tracing are all possible only as backend-specific interop
- This is feasible per-frame, but only with explicit synchronization and backend-aware scheduling

## References

- `wgpu` `Device::as_hal` docs: https://docs.rs/wgpu/latest/wgpu/struct.Device.html
- Bevy `RenderDevice`: https://docs.rs/bevy/latest/bevy/render/renderer/struct.RenderDevice.html
- Bevy `RenderQueue`: https://docs.rs/bevy/latest/bevy/render/renderer/struct.RenderQueue.html
- Windows `windows` crate / Direct3D 12: https://docs.rs/windows/latest/windows/Win32/Graphics/Direct3D12/index.html
- `ash` Vulkan bindings: https://docs.rs/ash/latest/ash/
- Metal Rust bindings: https://docs.rs/metal/latest/metal/
- Vulkan ray tracing pipeline extension: https://registry.khronos.org/vulkan/specs/latest/man/html/VK_KHR_ray_tracing_pipeline.html
- Apple Metal ray tracing overview: https://developer.apple.com/documentation/metal/acceleration_structures
