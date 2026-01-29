//! Network addressing for daemon routing
//!
//! ## Address Prefix Format
//!
//! Single byte prefix for routing:
//! - `0x00` = local daemon (Unix socket /run/st.sock)
//! - `0x01-0x7F` = cached host index (up to 127 known hosts)
//! - `0x80-0xFE` = inline address follows (len = byte - 0x80)
//! - `0xFF` = broadcast/discover

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

/// Network address for daemon communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    /// Local daemon via Unix socket
    Local,
    /// Cached host by index (1-127)
    Cached(u8),
    /// Inline address string (hostname:port or IP:port)
    Inline(AddressString),
    /// Broadcast/discover all daemons
    Broadcast,
}

/// Inline address string (max 126 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressString {
    data: [u8; 126],
    len: usize,
}

impl AddressString {
    /// Create from string
    pub fn new(s: &str) -> Option<Self> {
        if s.len() > 126 {
            return None;
        }
        let mut data = [0u8; 126];
        data[..s.len()].copy_from_slice(s.as_bytes());
        Some(AddressString {
            data,
            len: s.len(),
        })
    }

    /// Get as string slice
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Address {
    /// Encode address as prefix byte(s)
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn encode(&self) -> alloc::vec::Vec<u8> {
        match self {
            Address::Local => alloc::vec![0x00],
            Address::Cached(idx) => alloc::vec![*idx],
            Address::Inline(addr) => {
                let len = addr.len();
                let mut out = alloc::vec::Vec::with_capacity(len + 1);
                out.push((len as u8) + 0x80);
                out.extend_from_slice(addr.as_bytes());
                out
            }
            Address::Broadcast => alloc::vec![0xFF],
        }
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn encode_to(&self, buf: &mut [u8]) -> usize {
        match self {
            Address::Local => {
                buf[0] = 0x00;
                1
            }
            Address::Cached(idx) => {
                buf[0] = *idx;
                1
            }
            Address::Inline(addr) => {
                let len = addr.len();
                buf[0] = (len as u8) + 0x80;
                buf[1..1 + len].copy_from_slice(addr.as_bytes());
                1 + len
            }
            Address::Broadcast => {
                buf[0] = 0xFF;
                1
            }
        }
    }

    /// Decode address from prefix byte(s)
    pub fn decode(data: &[u8]) -> Option<(Self, usize)> {
        if data.is_empty() {
            return None;
        }

        let first = data[0];

        match first {
            0x00 => Some((Address::Local, 1)),
            0x01..=0x7F => Some((Address::Cached(first), 1)),
            0x80..=0xFE => {
                let len = (first - 0x80) as usize;
                if data.len() < 1 + len {
                    return None;
                }
                let addr = AddressString::new(core::str::from_utf8(&data[1..1 + len]).ok()?)?;
                Some((Address::Inline(addr), 1 + len))
            }
            0xFF => Some((Address::Broadcast, 1)),
        }
    }

    /// Check if this is a local address
    pub fn is_local(&self) -> bool {
        matches!(self, Address::Local)
    }

    /// Check if this is a remote address
    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }
}

/// Host cache for remembered remote daemons
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct HostCache {
    /// Index -> (hostname:port, display_name)
    hosts: HashMap<u8, (String, String)>,
    /// Hostname -> index (reverse lookup)
    by_name: HashMap<String, u8>,
    /// Next available index
    next_index: u8,
}

#[cfg(feature = "std")]
impl Default for HostCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl HostCache {
    /// Create empty cache
    pub fn new() -> Self {
        HostCache {
            hosts: HashMap::new(),
            by_name: HashMap::new(),
            next_index: 1, // 0 is reserved for local
        }
    }

    /// Add or update a host
    pub fn add(&mut self, host: &str, name: &str) -> Option<u8> {
        // Check if already exists
        if let Some(&idx) = self.by_name.get(host) {
            return Some(idx);
        }

        // Check capacity (1-127)
        if self.next_index > 127 {
            return None;
        }

        let idx = self.next_index;
        self.next_index += 1;

        self.hosts.insert(idx, (host.to_string(), name.to_string()));
        self.by_name.insert(host.to_string(), idx);

        Some(idx)
    }

    /// Lookup by index
    pub fn get(&self, idx: u8) -> Option<&(String, String)> {
        self.hosts.get(&idx)
    }

    /// Lookup by hostname
    pub fn get_by_name(&self, host: &str) -> Option<u8> {
        self.by_name.get(host).copied()
    }

    /// Remove a host
    pub fn remove(&mut self, idx: u8) {
        if let Some((host, _)) = self.hosts.remove(&idx) {
            self.by_name.remove(&host);
        }
    }

    /// List all hosts
    pub fn list(&self) -> impl Iterator<Item = (u8, &str, &str)> {
        self.hosts.iter().map(|(&idx, (host, name))| (idx, host.as_str(), name.as_str()))
    }

    /// Number of cached hosts
    pub fn len(&self) -> usize {
        self.hosts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty()
    }

    /// Resolve address - returns connection string
    pub fn resolve(&self, addr: &Address) -> Option<String> {
        match addr {
            Address::Local => Some("local".to_string()),
            Address::Cached(idx) => self.get(*idx).map(|(host, _)| host.clone()),
            Address::Inline(s) => Some(s.as_str().to_string()),
            Address::Broadcast => None, // Cannot resolve broadcast
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_address() {
        let addr = Address::Local;
        let encoded = addr.encode();
        assert_eq!(encoded, vec![0x00]);

        let (decoded, len) = Address::decode(&encoded).unwrap();
        assert_eq!(decoded, Address::Local);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_cached_address() {
        let addr = Address::Cached(5);
        let encoded = addr.encode();
        assert_eq!(encoded, vec![0x05]);

        let (decoded, len) = Address::decode(&encoded).unwrap();
        assert_eq!(decoded, Address::Cached(5));
        assert_eq!(len, 1);
    }

    #[test]
    fn test_inline_address() {
        let addr = Address::Inline(AddressString::new("192.168.1.5:8420").unwrap());
        let encoded = addr.encode();

        // First byte: 0x80 + 16 = 0x90
        assert_eq!(encoded[0], 0x90);

        let (decoded, len) = Address::decode(&encoded).unwrap();
        if let Address::Inline(s) = decoded {
            assert_eq!(s.as_str(), "192.168.1.5:8420");
        } else {
            panic!("expected inline address");
        }
        assert_eq!(len, 17);
    }

    #[test]
    fn test_broadcast() {
        let addr = Address::Broadcast;
        let encoded = addr.encode();
        assert_eq!(encoded, vec![0xFF]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_host_cache() {
        let mut cache = HostCache::new();

        let idx1 = cache.add("server1.local:8420", "Server 1").unwrap();
        let idx2 = cache.add("server2.local:8420", "Server 2").unwrap();

        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);

        // Duplicate returns same index
        let idx1_again = cache.add("server1.local:8420", "Server 1").unwrap();
        assert_eq!(idx1_again, idx1);

        // Lookup
        assert_eq!(cache.get_by_name("server1.local:8420"), Some(1));
        assert_eq!(cache.get(1).unwrap().0, "server1.local:8420");
    }
}
