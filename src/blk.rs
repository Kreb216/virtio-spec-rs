//! Block Device

use bitfield_struct::bitfield;
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
    cylinders: le16,
    heads: u8,
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
    physical_block_exp: u8,
    // offset of first aligned logical block
    alignment_offset: u8,
    // suggested minimum I/O size in blocks
    min_io_size: le16,
    // optimal (suggested maximum) I/O size in blocks
    opt_io_size: le32,
}

#[doc(alias = "virtio_blk_req")]
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
pub struct VirtioBlkReq {
    pub type_: le32,
    pub reserved: le32,
    pub sector: le64,
    pub data: [u8; 1], //TODO: u8 data[]; Will be modelled
    pub status: u8,
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
