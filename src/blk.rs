//! Block Device

use bitfield_struct::bitfield;
use core::alloc::Layout;
use core::ptr::{self, NonNull, addr_of_mut};
use endian_num::{le16, le32};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use volatile::access::ReadOnly;
use volatile_macro::VolatileFieldAccess;

pub use super::features::blk::F;
use crate::le64;

///Block Device Configuration Layout
///
/// Use [`ConfigVolatileFieldAccess`] to work with this struct.
#[doc(alias = "virtio_blk_config")]
#[cfg_attr(
    feature = "zerocopy",
    derive(
        zerocopy_derive::KnownLayout,
        zerocopy_derive::Immutable,
        zerocopy_derive::FromBytes,
    )
)]
#[derive(VolatileFieldAccess)]
#[repr(C)]
pub struct Config {
    #[access(ReadOnly)]
    capacity: le64,
    #[access(ReadOnly)]
    size_max: le32,
    #[access(ReadOnly)]
    seg_max: le32,
    #[access(ReadOnly)]
    geometry: VirtioBlkGeometry,
    #[access(ReadOnly)]
    blk_size: le32,
    #[access(ReadOnly)]
    topology: VirtioBlkTopology,
    #[access(ReadOnly)]
    writeback: u8,
    #[access(ReadOnly)]
    unused0: u8,
    #[access(ReadOnly)]
    num_queues: u8,
    #[access(ReadOnly)]
    max_discard_sectors: le32,
    #[access(ReadOnly)]
    max_discard_seg: le32,
    #[access(ReadOnly)]
    discard_sector_alignment: le32,
    #[access(ReadOnly)]
    max_write_zeroes_sectors: le32,
    #[access(ReadOnly)]
    max_write_zeroes_seg: le32,
    #[access(ReadOnly)]
    write_zeroes_may_unmap: u8,
    #[access(ReadOnly)]
    unused1: [u8; 3],
    #[access(ReadOnly)]
    max_secure_erase_sectors: le32,
    #[access(ReadOnly)]
    max_secure_erase_seg: le32,
    #[access(ReadOnly)]
    secure_erase_sector_alignment: le32,
}

#[doc(alias = "virtio_blk_geometry")]
#[cfg_attr(
    feature = "zerocopy",
    derive(
        zerocopy_derive::KnownLayout,
        zerocopy_derive::Immutable,
        zerocopy_derive::FromBytes,
    )
)]
#[derive(VolatileFieldAccess)]
#[repr(C)]
pub struct VirtioBlkGeometry {
    #[access(ReadOnly)]
    cylinders: le16,
    #[access(ReadOnly)]
    heads: u8,
    #[access(ReadOnly)]
    sectors: u8,
}

#[doc(alias = "virtio_blk_topology")]
#[cfg_attr(
    feature = "zerocopy",
    derive(
        zerocopy_derive::KnownLayout,
        zerocopy_derive::Immutable,
        zerocopy_derive::FromBytes,
    )
)]
#[derive(VolatileFieldAccess)]
#[repr(C)]
pub struct VirtioBlkTopology {
    // # of logical blocks per physical block (log2)
    #[access(ReadOnly)]
    physical_block_exp: u8,
    // offset of first aligned logical block
    #[access(ReadOnly)]
    alignment_offset: u8,
    // suggested minimum I/O size in blocks
    #[access(ReadOnly)]
    min_io_size: le16,
    // optimal (suggested maximum) I/O size in blocks
    #[access(ReadOnly)]
    opt_io_size: le32,
}

/// A request
#[doc(alias = "virtio_blk_req")]
#[derive(Debug)]
#[repr(C)]
pub struct Req {
    pub ty: le32,
    pub reserved: le32,
    pub sector: le64,
    data_and_status: [u8],
}

impl Req {
    pub fn layout(data_len: usize) -> Layout {
        let start = Layout::array::<le64>(2).unwrap();
        let (new_layout, _offset) = start
            .extend(Layout::array::<u8>(data_len + 1).unwrap())
            .unwrap();
        new_layout.pad_to_align()
    }

    pub fn from_ptr(ptr: NonNull<[u8]>) -> Option<NonNull<Self>> {
        let len = ptr.as_ptr().len();
        // FIXME: use ptr::as_mut_ptr once stable
        // https://github.com/rust-lang/rust/issues/74265
        let ptr = ptr.as_ptr() as *mut u8;

        if !ptr.cast::<le64>().is_aligned() {
            return None;
        }

        let len = len - 16;
        let ptr = ptr::slice_from_raw_parts(ptr, len) as *mut Self;
        Some(NonNull::new(ptr).unwrap())
    }

    pub fn data_ptr(this: NonNull<Self>) -> NonNull<[u8]> {
        let ptr = unsafe { addr_of_mut!((*this.as_ptr()).data_and_status) };
        let len = ptr.len().saturating_sub(1);
        let ptr = NonNull::new(ptr).unwrap().cast::<u8>();
        NonNull::slice_from_raw_parts(ptr, len)
    }

    pub fn data(&self) -> &[u8] {
        let ptr = Self::data_ptr(NonNull::from(self));
        unsafe { ptr.as_ref() }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        let mut ptr = Self::data_ptr(NonNull::from(self));
        unsafe { ptr.as_mut() }
    }

    pub fn status_ptr(this: NonNull<Self>) -> Option<NonNull<u8>> {
        let ptr = unsafe { addr_of_mut!((*this.as_ptr()).data_and_status) };
        let len = ptr.len();

        if len == 0 {
            return None;
        }

        let ptr = NonNull::new(ptr).unwrap().cast::<u8>();
        let ptr = unsafe { ptr.add(len - 1) };
        Some(ptr)
    }

    pub fn status(&self) -> Option<&u8> {
        Self::status_ptr(NonNull::from(self)).map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn status_mut(&mut self) -> Option<&mut u8> {
        Self::status_ptr(NonNull::from(self)).map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

#[doc(alias = "VIRTIO_BLK_T")]
#[derive(IntoPrimitive, TryFromPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum T {
    #[doc(alias = "VIRTIO_BLK_T_IN")]
    In = 0,

    #[doc(alias = "VIRTIO_BLK_T_OUT")]
    Out = 1,

    #[doc(alias = "VIRTIO_BLK_T_FLUSH")]
    Flush = 4,

    #[doc(alias = "VIRTIO_BLK_T_GET_ID")]
    GetId = 8,

    #[doc(alias = "VIRTIO_BLK_T_GET_LIFETIME")]
    GetLifetime = 10,

    #[doc(alias = "VIRTIO_BLK_T_DISCARD")]
    Discard = 11,

    #[doc(alias = "VIRTIO_BLK_T_WRITE_ZEROES")]
    WriteZeroes = 13,

    #[doc(alias = "VIRTIO_BLK_T_SECURE_ERASE")]
    SecureErase = 14,
}

#[doc(alias = "virtio_blk_discard_write_zeroes")]
#[derive(VolatileFieldAccess)]
#[repr(C)]
pub struct VirtioBlkDiscardWriteZeroes {
    pub sector: le64,
    pub num_sectors: le32,
    pub flags: le32, // TODO bit fields, type??
}

#[bitfield(u32, repr = le32, from = le32::from_ne, into = le32::to_ne)]
struct Flags {
    #[bits(1)]
    pub unmap: u8,

    #[bits(31)]
    pub reserved: u32,
}

#[doc(alias = "virtio_blk_lifetime")]
#[derive(VolatileFieldAccess)]
#[repr(C)]
pub struct VirtioBlkLifetime {
    pub pre_eol_info: le16,
    pub device_lifetime_est_typ_a: le16,
    pub device_lifetime_est_typ_b: le16,
}

#[doc(alias = "VIRTIO_BLK_PRE_EOL_INFO")]
#[derive(IntoPrimitive, TryFromPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum PreEolInfo {
    #[doc(alias = "VIRTIO_BLK_PRE_EOL_INFO_UNDEFINED")]
    Undefined = 0,

    #[doc(alias = "VIRTIO_BLK_PRE_EOL_INFO_NORMAL")]
    Normal = 1,

    #[doc(alias = "VIRTIO_BLK_PRE_EOL_INFO_WARNING")]
    Warning = 2,

    #[doc(alias = "VIRTIO_BLK_PRE_EOL_INFO_URGENT")]
    Urgent = 3,
}

#[doc(alias = "VIRTIO_BLK_S")]
#[derive(IntoPrimitive, TryFromPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum S {
    #[doc(alias = "VIRTIO_BLK_S_OK")]
    OK = 0,

    #[doc(alias = "VIRTIO_BLK_S_IOERR")]
    IOERR = 1,

    #[doc(alias = "VIRTIO_BLK_S_UNSUPP")]
    UNSUPP = 2,
}
