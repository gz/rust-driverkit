//! Flags indicating checksum, segmentation and other offload work to be
//! done, or already done, by hardware or lower layers.  It is split into
//! separate inbound and outbound flags.
//!
//! Outbound flags that are set by upper protocol layers requesting lower
//! layers, or ideally the hardware, to perform these offloading tasks.
//! For outbound packets this field and its flags can be directly tested
//! against ifnet if_hwassist.  Note that the outbound and the inbound flags do
//! not collide right now but they could be allowed to (as long as the flags are
//! scrubbed appropriately when the direction of an mbuf changes).  CSUM_BITS
//! would also have to split into CSUM_BITS_TX and CSUM_BITS_RX.
//!
//! CSUM_INNER_<x> is the same as CSUM_<x> but it applies to the inner frame.
//! The CSUM_ENCAP_<x> bits identify the outer encapsulation.

/// IP header checksum offload
pub const CSUM_IP: u32 = 0x00000001;

/// UDP checksum offload
pub const CSUM_IP_UDP: u32 = 0x00000002;

/// TCP checksum offload
pub const CSUM_IP_TCP: u32 = 0x00000004;

/// SCTP checksum offload
pub const CSUM_IP_SCTP: u32 = 0x00000008;

/// TCP segmentation offload
pub const CSUM_IP_TSO: u32 = 0x00000010;

/// iSCSI checksum offload
pub const CSUM_IP_ISCSI: u32 = 0x00000020;

pub const CSUM_INNER_IP6_UDP: u32 = 0x00000040;
pub const CSUM_INNER_IP6_TCP: u32 = 0x00000080;
pub const CSUM_INNER_IP6_TSO: u32 = 0x00000100;

/// UDP checksum offload
pub const CSUM_IP6_UDP: u32 = 0x00000200;

/// TCP checksum offload
pub const CSUM_IP6_TCP: u32 = 0x00000400;

/// SCTP checksum offload
pub const CSUM_IP6_SCTP: u32 = 0x00000800;

/// TCP segmentation offload
pub const CSUM_IP6_TSO: u32 = 0x00001000;

/// iSCSI checksum offload
pub const CSUM_IP6_ISCSI: u32 = 0x00002000;

pub const CSUM_INNER_IP: u32 = 0x00004000;
pub const CSUM_INNER_IP_UDP: u32 = 0x00008000;
pub const CSUM_INNER_IP_TCP: u32 = 0x00010000;
pub const CSUM_INNER_IP_TSO: u32 = 0x00020000;

/// VXLAN outer encapsulation
pub const CSUM_ENCAP_VXLAN: u32 = 0x00040000;
pub const CSUM_ENCAP_RSVD1: u32 = 0x00080000;

// Inbound checksum support where the checksum was verified by hardware

pub const CSUM_INNER_L3_CALC: u32 = 0x00100000;
pub const CSUM_INNER_L3_VALID: u32 = 0x00200000;
pub const CSUM_INNER_L4_CALC: u32 = 0x00400000;
pub const CSUM_INNER_L4_VALID: u32 = 0x00800000;

/// calculated layer 3 csum
pub const CSUM_L3_CALC: u32 = 0x01000000;

/// checksum is correct
pub const CSUM_L3_VALID: u32 = 0x02000000;

/// calculated layer 4 csum
pub const CSUM_L4_CALC: u32 = 0x04000000;

/// checksum is correct
pub const CSUM_L4_VALID: u32 = 0x08000000;

/// calculated layer 5 csum
pub const CSUM_L5_CALC: u32 = 0x10000000;

/// checksum is correct
pub const CSUM_L5_VALID: u32 = 0x20000000;

/// contains merged segments
pub const CSUM_COALESCED: u32 = 0x40000000;

/// Packet header has send tag
pub const CSUM_SND_TAG: u32 = 0x80000000;

// CSUM flags compatibility mappings:

pub const CSUM_IP_CHECKED: u32 = CSUM_L3_CALC;
pub const CSUM_IP_VALID: u32 = CSUM_L3_VALID;
pub const CSUM_DATA_VALID: u32 = CSUM_L4_VALID;
pub const CSUM_PSEUDO_HDR: u32 = CSUM_L4_CALC;
pub const CSUM_SCTP_VALID: u32 = CSUM_L4_VALID;
pub const CSUM_DELAY_DATA: u32 = CSUM_TCP | CSUM_UDP;
/// Only v4, no v6 IP hdr csum
pub const CSUM_DELAY_IP: u32 = CSUM_IP;
pub const CSUM_DELAY_DATA_IPV6: u32 = CSUM_TCP_IPV6 | CSUM_UDP_IPV6;
pub const CSUM_DATA_VALID_IPV6: u32 = CSUM_DATA_VALID;
pub const CSUM_TCP: u32 = CSUM_IP_TCP;
pub const CSUM_UDP: u32 = CSUM_IP_UDP;
pub const CSUM_SCTP: u32 = CSUM_IP_SCTP;
pub const CSUM_TSO: u32 = CSUM_IP_TSO | CSUM_IP6_TSO;
pub const CSUM_INNER_TSO: u32 = CSUM_INNER_IP_TSO | CSUM_INNER_IP6_TSO;
pub const CSUM_UDP_IPV6: u32 = CSUM_IP6_UDP;
pub const CSUM_TCP_IPV6: u32 = CSUM_IP6_TCP;
pub const CSUM_SCTP_IPV6: u32 = CSUM_IP6_SCTP;

pub const CSUM_FLAGS_TX: u32 = (CSUM_IP
    | CSUM_IP_UDP
    | CSUM_IP_TCP
    | CSUM_IP_SCTP
    | CSUM_IP_TSO
    | CSUM_IP_ISCSI
    | CSUM_INNER_IP6_UDP
    | CSUM_INNER_IP6_TCP
    | CSUM_INNER_IP6_TSO
    | CSUM_IP6_UDP
    | CSUM_IP6_TCP
    | CSUM_IP6_SCTP
    | CSUM_IP6_TSO
    | CSUM_IP6_ISCSI
    | CSUM_INNER_IP
    | CSUM_INNER_IP_UDP
    | CSUM_INNER_IP_TCP
    | CSUM_INNER_IP_TSO
    | CSUM_ENCAP_VXLAN
    | CSUM_ENCAP_RSVD1
    | CSUM_SND_TAG);

pub const CSUM_FLAGS_RX: u32 = (CSUM_INNER_L3_CALC
    | CSUM_INNER_L3_VALID
    | CSUM_INNER_L4_CALC
    | CSUM_INNER_L4_VALID
    | CSUM_L3_CALC
    | CSUM_L3_VALID
    | CSUM_L4_CALC
    | CSUM_L4_VALID
    | CSUM_L5_CALC
    | CSUM_L5_VALID
    | CSUM_COALESCED);
