//! Vulkan compute for the pinhole projector. Unsafe is limited to the
//! ash calls and the host/device copies; the shader is SPIR-V, not WGSL.

use crate::{Backend, Error, ProjectedSplat, ProjectionBatch, Result};
use ash::{vk, Device, Entry, Instance};
use std::ffi::{CStr, CString};

pub struct VulkanDevice {
    pub name: String,
    pub kind: String,
}

pub struct VulkanBackend {
    _entry: Entry,
    instance: Instance,
    device: Device,
    phys: vk::PhysicalDevice,
    queue: vk::Queue,
    _queue_family: u32,
    pool: vk::CommandPool,
    layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    pipeline: vk::Pipeline,
    shader: vk::ShaderModule,
    name: String,
}

impl VulkanBackend {
    pub fn open() -> Result<Self> {
        Self::open_named(None)
    }

    pub fn open_named(prefer: Option<&str>) -> Result<Self> {
        let entry = unsafe { Entry::load() }
            .map_err(|e| Error::new(format!("Vulkan loader: {e}")))?;
        let app = CString::new("spirula").unwrap();
        let engine = CString::new("spirula-rs").unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(app.as_c_str())
            .engine_name(engine.as_c_str())
            .api_version(vk::API_VERSION_1_1);
        let (flags, ext_names) = portability_instance(&entry)?;
        let ext_ptrs: Vec<*const i8> = ext_names.iter().map(|n| n.as_ptr()).collect();
        let create = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .flags(flags)
            .enabled_extension_names(&ext_ptrs);
        let instance = unsafe { entry.create_instance(&create, None) }
            .map_err(|e| Error::new(format!("vkCreateInstance: {e}")))?;
        let mut devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(|e| Error::new(format!("enumerate devices: {e}")))?;
        devices.sort_by_key(|d| device_rank(&instance, *d, prefer));
        let mut last = Error::new("no Vulkan device could be created");
        for phys in devices {
            match build(&entry, instance.clone(), phys) {
                Ok(backend) => return Ok(backend),
                Err(e) => last = e,
            }
        }
        unsafe { instance.destroy_instance(None) };
        Err(last)
    }
}

pub fn list_devices() -> Result<Vec<VulkanDevice>> {
    let entry = unsafe { Entry::load() }.map_err(|e| Error::new(format!("Vulkan loader: {e}")))?;
    let app = CString::new("spirula").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(app.as_c_str())
        .api_version(vk::API_VERSION_1_1);
    let (flags, ext_names) = portability_instance(&entry)?;
    let ext_ptrs: Vec<*const i8> = ext_names.iter().map(|n| n.as_ptr()).collect();
    let create = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .flags(flags)
        .enabled_extension_names(&ext_ptrs);
    let instance = unsafe { entry.create_instance(&create, None) }
        .map_err(|e| Error::new(format!("vkCreateInstance: {e}")))?;
    let devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|e| Error::new(format!("enumerate devices: {e}")))?;
    let mut out = Vec::new();
    for d in devices {
        let props = unsafe { instance.get_physical_device_properties(d) };
        out.push(VulkanDevice {
            name: device_name(&props),
            kind: device_kind(props.device_type).into(),
        });
    }
    unsafe { instance.destroy_instance(None) };
    Ok(out)
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().ok();
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_shader_module(self.shader, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Backend for VulkanBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn project(&mut self, batch: &ProjectionBatch<'_>) -> Result<Vec<ProjectedSplat>> {
        let n = batch.means.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        if batch.means.len() != batch.quats.len()
            || batch.means.len() != batch.log_scales.len()
            || batch.means.len() != batch.opacity_logits.len()
        {
            return Err(Error::new("projection batch arrays have different lengths"));
        }
        dispatch(self, batch)
    }
}

fn build(entry: &Entry, instance: Instance, phys: vk::PhysicalDevice) -> Result<VulkanBackend> {
    let props = unsafe { instance.get_physical_device_properties(phys) };
    let queues = unsafe { instance.get_physical_device_queue_family_properties(phys) };
    let queue_family = queues
        .iter()
        .position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE))
        .ok_or_else(|| Error::new("device has no compute queue"))? as u32;
    let priorities = [1.0_f32];
    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family)
        .queue_priorities(&priorities);
    let dev_exts = portability_device(&instance, phys)?;
    let dev_ptrs: Vec<*const i8> = dev_exts.iter().map(|n| n.as_ptr()).collect();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&dev_ptrs);
    let device = unsafe { instance.create_device(phys, &device_info, None) }
        .map_err(|e| Error::new(format!("vkCreateDevice: {e}")))?;
    let queue = unsafe { device.get_device_queue(queue_family, 0) };
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let pool = unsafe { device.create_command_pool(&pool_info, None) }
        .map_err(|e| Error::new(format!("command pool: {e}")))?;
    let words = spirv_words();
    let shader_info = vk::ShaderModuleCreateInfo::default().code(&words);
    let shader = unsafe { device.create_shader_module(&shader_info, None) }
        .map_err(|e| Error::new(format!("shader module: {e}")))?;
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..6)
        .map(|i| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(i)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        })
        .collect();
    let set_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let set_layout = unsafe { device.create_descriptor_set_layout(&set_info, None) }
        .map_err(|e| Error::new(format!("descriptor layout: {e}")))?;
    let layouts = [set_layout];
    let pipe_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let layout = unsafe { device.create_pipeline_layout(&pipe_layout_info, None) }
        .map_err(|e| Error::new(format!("pipeline layout: {e}")))?;
    let entry_name = CStr::from_bytes_with_nul(b"main\0").unwrap();
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(shader)
        .name(entry_name);
    let compute = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);
    let pipeline = unsafe {
        device.create_compute_pipelines(vk::PipelineCache::null(), &[compute], None)
    }
    .map_err(|(_, e)| Error::new(format!("compute pipeline: {e}")))?[0];
    let name = format!("{} ({})", device_name(&props), device_kind(props.device_type));
    Ok(VulkanBackend {
        _entry: entry.clone(),
        instance,
        device,
        phys,
        queue,
        _queue_family: queue_family,
        pool,
        layout,
        set_layout,
        pipeline,
        shader,
        name,
    })
}

fn dispatch(backend: &VulkanBackend, batch: &ProjectionBatch<'_>) -> Result<Vec<ProjectedSplat>> {
    let n = batch.means.len();
    let device = &backend.device;
    let mut cam = [0u8; 80];
    write_f32(&mut cam[0..], batch.fx);
    write_f32(&mut cam[4..], batch.fy);
    write_f32(&mut cam[8..], batch.cx);
    write_f32(&mut cam[12..], batch.cy);
    write_u32(&mut cam[16..], batch.width);
    write_u32(&mut cam[20..], batch.height);
    write_u32(&mut cam[24..], n as u32);
    write_u32(&mut cam[28..], if batch.antialiased { 1 } else { 0 });
    for i in 0..12 {
        write_f32(&mut cam[32 + i * 4..], batch.view[i]);
    }
    let means = pack_vec4(batch.means, |m| [m[0], m[1], m[2], 0.0]);
    let quats = pack_vec4(batch.quats, |q| [q[0], q[1], q[2], q[3]]);
    let scales = pack_vec4(batch.log_scales, |s| [s[0], s[1], s[2], 0.0]);
    let mut opacs = Vec::with_capacity(n * 4);
    for v in batch.opacity_logits {
        opacs.extend_from_slice(&v.to_le_bytes());
    }
    let out_bytes = (n * 48) as u64;

    let b_cam = GpuBuf::upload(backend, &cam)?;
    let b_means = GpuBuf::upload(backend, &means)?;
    let b_quats = GpuBuf::upload(backend, &quats)?;
    let b_scales = GpuBuf::upload(backend, &scales)?;
    let b_opac = GpuBuf::upload(backend, &opacs)?;
    let b_out = GpuBuf::empty(backend, out_bytes)?;
    let buffers = [&b_cam, &b_means, &b_quats, &b_scales, &b_opac, &b_out];

    let pool_size = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 6,
    }];
    let dpool_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&pool_size);
    let dpool = unsafe { device.create_descriptor_pool(&dpool_info, None) }
        .map_err(|e| Error::new(format!("descriptor pool: {e}")))?;
    let layouts = [backend.set_layout];
    let alloc = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(dpool)
        .set_layouts(&layouts);
    let set = unsafe { device.allocate_descriptor_sets(&alloc) }
        .map_err(|e| Error::new(format!("descriptor set: {e}")))?[0];
    let infos: Vec<vk::DescriptorBufferInfo> = buffers
        .iter()
        .map(|b| vk::DescriptorBufferInfo {
            buffer: b.buffer,
            offset: 0,
            range: b.size,
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = (0..6)
        .map(|i| {
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(i as u32)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&infos[i..i + 1])
        })
        .collect();
    unsafe { device.update_descriptor_sets(&writes, &[]) };

    let cmd_alloc = vk::CommandBufferAllocateInfo::default()
        .command_pool(backend.pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = unsafe { device.allocate_command_buffers(&cmd_alloc) }
        .map_err(|e| Error::new(format!("command buffer: {e}")))?[0];
    let begin = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(cmd, &begin) }
        .map_err(|e| Error::new(format!("begin: {e}")))?;
    let to_shader = vk::MemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::HOST_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ);
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::HOST,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[to_shader],
            &[],
            &[],
        );
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, backend.pipeline);
        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::COMPUTE,
            backend.layout,
            0,
            &[set],
            &[],
        );
        device.cmd_dispatch(cmd, (n as u32 + 63) / 64, 1, 1);
    }
    let to_host = vk::MemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::SHADER_WRITE)
        .dst_access_mask(vk::AccessFlags::HOST_READ);
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            &[to_host],
            &[],
            &[],
        );
        device.end_command_buffer(cmd).map_err(|e| Error::new(format!("end: {e}")))?;
    }
    let fence_info = vk::FenceCreateInfo::default();
    let fence = unsafe { device.create_fence(&fence_info, None) }
        .map_err(|e| Error::new(format!("fence: {e}")))?;
    let cmds = [cmd];
    let submit = vk::SubmitInfo::default().command_buffers(&cmds);
    unsafe { device.queue_submit(backend.queue, &[submit], fence) }
        .map_err(|e| Error::new(format!("submit: {e}")))?;
    unsafe { device.wait_for_fences(&[fence], true, 30_000_000_000) }
        .map_err(|e| Error::new(format!("fence wait: {e}")))?;
    let raw = b_out.read(device)?;
    unsafe {
        device.destroy_fence(fence, None);
        device.free_command_buffers(backend.pool, &[cmd]);
        device.destroy_descriptor_pool(dpool, None);
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let o = i * 48;
        let mx = f32_at(&raw, o);
        let my = f32_at(&raw, o + 4);
        let depth = f32_at(&raw, o + 8);
        let opacity = f32_at(&raw, o + 12);
        let c00 = f32_at(&raw, o + 16);
        let c01 = f32_at(&raw, o + 20);
        let c11 = f32_at(&raw, o + 24);
        let valid = f32_at(&raw, o + 28);
        let sorting_depth = f32_at(&raw, o + 32);
        let sigmoid_opacity = f32_at(&raw, o + 36);
        if valid > 0.5 {
            out.push(ProjectedSplat {
                mean2d: [mx, my],
                depth,
                sorting_depth,
                conic: [c00, c01, c11],
                opacity,
                sigmoid_opacity,
                valid: true,
            });
        } else {
            out.push(ProjectedSplat::invalid());
        }
    }
    Ok(out)
}

struct GpuBuf {
    device: Device,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

impl GpuBuf {
    fn upload(backend: &VulkanBackend, bytes: &[u8]) -> Result<Self> {
        let buf = Self::empty(backend, bytes.len() as u64)?;
        buf.write(&backend.device, bytes)?;
        Ok(buf)
    }

    fn empty(backend: &VulkanBackend, size: vk::DeviceSize) -> Result<Self> {
        let size = size.max(4);
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { backend.device.create_buffer(&info, None) }
            .map_err(|e| Error::new(format!("buffer: {e}")))?;
        let req = unsafe { backend.device.get_buffer_memory_requirements(buffer) };
        let mem_props = unsafe {
            backend.instance.get_physical_device_memory_properties(backend.phys)
        };
        let flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let type_index = (0..mem_props.memory_type_count).find(|&i| {
            (req.memory_type_bits & (1 << i)) != 0
                && mem_props.memory_types[i as usize].property_flags.contains(flags)
        });
        let type_index = type_index.ok_or_else(|| {
            Error::new("no host-visible coherent memory type for a storage buffer")
        })?;
        let alloc = vk::MemoryAllocateInfo::default()
            .allocation_size(req.size)
            .memory_type_index(type_index);
        let memory = unsafe { backend.device.allocate_memory(&alloc, None) }
            .map_err(|e| Error::new(format!("allocate: {e}")))?;
        unsafe { backend.device.bind_buffer_memory(buffer, memory, 0) }
            .map_err(|e| Error::new(format!("bind: {e}")))?;
        Ok(Self {
            device: backend.device.clone(),
            buffer,
            memory,
            size,
        })
    }

    fn write(&self, device: &Device, bytes: &[u8]) -> Result<()> {
        let ptr = unsafe { device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty()) }
            .map_err(|e| Error::new(format!("map: {e}")))?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
            device.unmap_memory(self.memory);
        }
        Ok(())
    }

    fn read(&self, device: &Device) -> Result<Vec<u8>> {
        let ptr = unsafe { device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty()) }
            .map_err(|e| Error::new(format!("map: {e}")))?;
        let mut out = vec![0u8; self.size as usize];
        unsafe {
            std::ptr::copy_nonoverlapping(ptr as *const u8, out.as_mut_ptr(), out.len());
            device.unmap_memory(self.memory);
        }
        Ok(out)
    }
}

impl Drop for GpuBuf {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

fn portability_instance(entry: &Entry) -> Result<(vk::InstanceCreateFlags, Vec<CString>)> {
    let listed = unsafe { entry.enumerate_instance_extension_properties(None) }
        .map_err(|e| Error::new(format!("instance extensions: {e}")))?;
    let mut flags = vk::InstanceCreateFlags::empty();
    let mut names = Vec::new();
    let port = CStr::from_bytes_with_nul(b"VK_KHR_portability_enumeration\0").unwrap();
    if listed.iter().any(|e| extension_name(e) == port) {
        flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        names.push(CString::new("VK_KHR_portability_enumeration").unwrap());
    }
    Ok((flags, names))
}

fn portability_device(instance: &Instance, phys: vk::PhysicalDevice) -> Result<Vec<CString>> {
    let listed = unsafe { instance.enumerate_device_extension_properties(phys) }
        .map_err(|e| Error::new(format!("device extensions: {e}")))?;
    let subset = CStr::from_bytes_with_nul(b"VK_KHR_portability_subset\0").unwrap();
    let mut names = Vec::new();
    if listed.iter().any(|e| extension_name(e) == subset) {
        names.push(CString::new("VK_KHR_portability_subset").unwrap());
    }
    Ok(names)
}

fn extension_name(ext: &vk::ExtensionProperties) -> &CStr {
    unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }
}

fn spirv_words() -> Vec<u32> {
    let bytes = include_bytes!("../shaders/project_pinhole.spv");
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn write_f32(dst: &mut [u8], v: f32) {
    dst[..4].copy_from_slice(&v.to_le_bytes());
}

fn write_u32(dst: &mut [u8], v: u32) {
    dst[..4].copy_from_slice(&v.to_le_bytes());
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn pack_vec4<T>(items: &[T], f: impl Fn(&T) -> [f32; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * 16);
    for item in items {
        for v in f(item) {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

fn device_name(props: &vk::PhysicalDeviceProperties) -> String {
    unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn device_kind(kind: vk::PhysicalDeviceType) -> &'static str {
    match kind {
        vk::PhysicalDeviceType::DISCRETE_GPU => "discrete",
        vk::PhysicalDeviceType::INTEGRATED_GPU => "integrated",
        vk::PhysicalDeviceType::VIRTUAL_GPU => "virtual",
        vk::PhysicalDeviceType::CPU => "cpu",
        _ => "other",
    }
}

fn device_rank(instance: &Instance, device: vk::PhysicalDevice, prefer: Option<&str>) -> u8 {
    let props = unsafe { instance.get_physical_device_properties(device) };
    if let Some(needle) = prefer {
        if device_name(&props).to_lowercase().contains(&needle.to_lowercase()) {
            return 0;
        }
    }
    match props.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => 1,
        vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
        vk::PhysicalDeviceType::VIRTUAL_GPU => 3,
        vk::PhysicalDeviceType::CPU => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Backend, CpuBackend};
    use spirula_core::project::ProjectionBatch;

    #[test]
    fn vulkan_pinhole_matches_the_cpu_projector() {
        let mut gpu = match VulkanBackend::open() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("skipping Vulkan parity: {e}");
                return;
            }
        };
        let view = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.2];
        let means = [[0.1, -0.2, 2.0], [0.0, 0.0, -1.0], [1.5, 0.3, 3.0]];
        let quats = [
            [0.9, 0.1, -0.2, 0.3],
            [1.0, 0.0, 0.0, 0.0],
            [0.5, 0.5, 0.5, 0.5],
        ];
        let scales = [[-2.0, -2.2, -1.8], [0.0, 0.0, 0.0], [-1.0, -3.0, -1.5]];
        let opac = [0.2, 2.0, -0.4];
        let batch = ProjectionBatch {
            means: &means,
            quats: &quats,
            log_scales: &scales,
            opacity_logits: &opac,
            view: &view,
            fx: 80.0,
            fy: 70.0,
            cx: 32.0,
            cy: 24.0,
            width: 64,
            height: 48,
            antialiased: false,
        };
        let cpu = CpuBackend.project(&batch).unwrap();
        let gpu_out = gpu.project(&batch).unwrap();
        assert_eq!(cpu.len(), gpu_out.len());
        for (c, g) in cpu.iter().zip(gpu_out.iter()) {
            assert_eq!(c.valid, g.valid, "valid flag on {}", gpu.name());
            if c.valid {
                assert!((c.mean2d[0] - g.mean2d[0]).abs() < 2e-3, "{c:?} vs {g:?}");
                assert!((c.mean2d[1] - g.mean2d[1]).abs() < 2e-3);
                assert!((c.depth - g.depth).abs() < 2e-3);
                assert!((c.sorting_depth - g.sorting_depth).abs() < 2e-3);
                assert!((c.sigmoid_opacity - g.sigmoid_opacity).abs() < 2e-4);
                assert!((c.opacity - g.opacity).abs() < 2e-4);
                for k in 0..3 {
                    assert!((c.conic[k] - g.conic[k]).abs() < 2e-3, "conic {c:?} vs {g:?}");
                }
            }
        }
    }
}
