//! Microsoft RSS standard hash types

/// Has hash properties
pub const M_HASHTYPE_HASHPROP: u32 = 0x80;

pub const fn hashtype_hash(t: u32) -> u32 {
    M_HASHTYPE_HASHPROP | t
}

/// No hashing
pub const M_HASHTYPE_NONE: u32 = 0;

/// IPv4 2-tuple
pub const M_HASHTYPE_RSS_IPV4: u32 = hashtype_hash(1);

/// TCPv4 4-tuple
pub const M_HASHTYPE_RSS_TCP_IPV4: u32 = hashtype_hash(2);

/// IPv6 2-tuple
pub const M_HASHTYPE_RSS_IPV6: u32 = hashtype_hash(3);

/// TCPv6 4-tuple
pub const M_HASHTYPE_RSS_TCP_IPV6: u32 = hashtype_hash(4);

/// IPv6 2-tuple + ext hdrs
pub const M_HASHTYPE_RSS_IPV6_EX: u32 = hashtype_hash(5);

/// TCPv6 4-tuple + ext hdrs
pub const M_HASHTYPE_RSS_TCP_IPV6_EX: u32 = hashtype_hash(6);

/// IPv4 UDP 4-tuple
pub const M_HASHTYPE_RSS_UDP_IPV4: u32 = hashtype_hash(7);

/// IPv6 UDP 4-tuple
pub const M_HASHTYPE_RSS_UDP_IPV6: u32 = hashtype_hash(9);

/// IPv6 UDP 4-tuple + ext hdrs
pub const M_HASHTYPE_RSS_UDP_IPV6_EX: u32 = hashtype_hash(10);

/// ordering, not affinity
pub const M_HASHTYPE_OPAQUE: u32 = 63;

/// ordering+hash, not affinity
pub const M_HASHTYPE_OPAQUE_HASH: u32 = hashtype_hash(M_HASHTYPE_OPAQUE);
