#![feature(const_fn)]

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![allow(intra_doc_link_resolution_failure)]

use std::error::Error;
use std::ffi::*;
use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, };
use std::os::raw::*;
use std::os::unix::io::{AsRawFd, RawFd, };
use std::path::{Path, PathBuf, };
use std::ptr;
use std::ptr::NonNull;
use std::slice::from_raw_parts_mut;
use std::mem::size_of;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! check_err {
  ($expr:expr) => {{
    let v = $expr;
    if v < 0 {
      Err(std::io::Error::from_raw_os_error(-v))
    } else {
      Ok(v)
    }
  }}
}

struct AmdGpuDevice {
  fd: File,
  drm: amdgpu_device_handle,
  major: u32,
  minor: u32,
}
impl AsRawFd for AmdGpuDevice {
  fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
}
impl AmdGpuDevice {
  fn open() -> Result<Self, IoError> {
    let mut opts = OpenOptions::new();
    opts.read(true)
      .write(true);
    let render = opts.open("/dev/dri/renderD128")?;
    let mut handle = std::ptr::null_mut();
    let mut major_ver = 0u32;
    let mut minor_ver = 0u32;
    unsafe {
      let r = amdgpu_device_initialize(render.as_raw_fd(),
                                       &mut major_ver, &mut minor_ver,
                                       &mut handle);
      check_err!(r)?;
    }
    Ok(AmdGpuDevice {
      fd: render,
      drm: handle,
      major: major_ver,
      minor: minor_ver,
    })
  }

  fn mem_info(&self) -> Result<drm_amdgpu_memory_info, IoError> {
    let mut out = Default::default();
    unsafe {
      let size = size_of::<drm_amdgpu_memory_info>() as _;
      let r = amdgpu_query_info(self.drm, AMDGPU_INFO_MEMORY, size,
                                &mut out as *mut _ as *mut _);
      check_err!(r)?;
      Ok(out)
    }
  }

  fn alloc(&self, request: &mut amdgpu_bo_alloc_request)
    -> Result<AmdgpuBo, IoError>
  {
    let mut out = ptr::null_mut();
    unsafe {
      let r = amdgpu_bo_alloc(self.drm, request, &mut out);
      check_err!(r)?;
    }

    Ok(AmdgpuBo {
      dev: self,
      handle: out,
    })
  }
}
impl Drop for AmdGpuDevice {
  fn drop(&mut self) {
    unsafe {
      let _ = amdgpu_device_deinitialize(self.drm);
    }
  }
}
struct AmdgpuBo<'a> {
  dev: &'a AmdGpuDevice,
  handle: amdgpu_bo_handle,
}
impl<'a> AmdgpuBo<'a> {
  pub fn map_cpu(&self) -> Result<AmdgpuBoMap, IoError> {
    unsafe {
      let mut ptr = ptr::null_mut();
      let r = amdgpu_bo_cpu_map(self.handle, &mut ptr);
      check_err!(r)?;
      let ptr = NonNull::new(ptr)
        .expect("got a null ptr for cpu map");
      Ok(AmdgpuBoMap {
        bo: self,
        ptr: ptr.cast(),
      })
    }
  }
}
impl<'a> Drop for AmdgpuBo<'a> {
  fn drop(&mut self) {
    unsafe {
      let r = amdgpu_bo_free(self.handle);
    }
  }
}
struct AmdgpuBoMap<'a> {
  bo: &'a AmdgpuBo<'a>,
  ptr: NonNull<()>,
}
impl<'a> Drop for AmdgpuBoMap<'a> {
  fn drop(&mut self) {
    unsafe {
      let _ = amdgpu_bo_cpu_unmap(self.bo.handle);
    }
  }
}

const BUS_ID: &'static str = "0b:00.0";

pub fn main() {
  let dev = AmdGpuDevice::open()
    .expect("amdgpu_device_initialize");
  println!("major: {}, minor: {}", dev.major, dev.minor);

  let mem_info = dev.mem_info().expect("AMDGPU_INFO_MEMORY");
  println!("mem info: {:#?}", mem_info);

  let mut request = amdgpu_bo_alloc_request::default();
  request.alloc_size = 4096;
  request.flags |= (AMDGPU_GEM_CREATE_CPU_ACCESS_REQUIRED as u64);
  request.flags |= (AMDGPU_GEM_CREATE_VRAM_CLEARED as u64);
  request.preferred_heap |= (AMDGPU_GEM_DOMAIN_VRAM as u32);
  let alloc = dev.alloc(&mut request)
    .expect("amdgpu_bo_alloc");

  println!("got VRAM alloc");

  let ptr = alloc.map_cpu()
    .expect("amdgpu_bo_cpu_map");
  let mut ptr: NonNull<u8> = ptr.ptr.cast();
  unsafe {
    let slice = from_raw_parts_mut(ptr.as_ptr(),
                                   request.alloc_size as usize);
    for v in slice.iter_mut() {
      *v = 1u8;
    }
  }

  println!("GPU VRAM write test passed");
}
