//! Linear region-based memory manager (ownership-based, zero-GC).

use crate::error::MemError;
use std::fmt;

/// Opaque region identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionId(pub u32);

/// Permission flags for a memory region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions(pub u8);

impl Permissions {
    pub const READ: u8 = 0x01;
    pub const WRITE: u8 = 0x02;
    pub const EXECUTE: u8 = 0x04;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn read_write() -> Self {
        Self(Self::READ | Self::WRITE)
    }

    pub const fn read_execute() -> Self {
        Self(Self::READ | Self::EXECUTE)
    }

    pub const fn all() -> Self {
        Self(Self::READ | Self::WRITE | Self::EXECUTE)
    }

    #[must_use]
    pub const fn contains(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.contains(Self::READ) {
            write!(f, "R")?;
        }
        if self.contains(Self::WRITE) {
            write!(f, "W")?;
        }
        if self.contains(Self::EXECUTE) {
            write!(f, "X")?;
        }
        if self.0 == 0 {
            write!(f, "-")?;
        }
        Ok(())
    }
}

/// A contiguous region of virtual memory.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub id: RegionId,
    pub base: u64,
    pub size: u64,
    pub data: Vec<u8>,
    pub permissions: Permissions,
    pub owner: Option<String>,
    /// Whether this region has been freed.
    freed: bool,
}

// ────────────────────────────────────────────────────────────────

/// Default base address for the code (bytecode) region.
pub const CODE_BASE: u64 = 0x1_0000;
/// Default base address for the heap.
pub const HEAP_BASE: u64 = 0x100_0000;
/// Default base address for the stack (top of stack region).
pub const STACK_BASE: u64 = 0x1000_0000;

/// The FLUX memory manager.
///
/// Manages a set of non-overlapping memory regions.  Allocation is
/// sequential (bump allocator).
#[derive(Debug, Clone)]
pub struct MemoryManager {
    regions: Vec<MemoryRegion>,
    next_id: u32,
    heap_base: u64,
    _stack_base: u64,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryManager {
    /// Create a new memory manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            next_id: 0,
            heap_base: HEAP_BASE,
            _stack_base: STACK_BASE,
        }
    }

    /// Allocate a new memory region.
    pub fn allocate(
        &mut self,
        size: u64,
        perms: Permissions,
        owner: Option<&str>,
    ) -> Result<RegionId, MemError> {
        let id = RegionId(self.next_id);
        self.next_id += 1;

        let base = self.heap_base;
        self.heap_base += size;
        // Keep 8-byte alignment.
        self.heap_base = (self.heap_base + 7) & !7;

        let data = vec![0u8; size as usize];

        self.regions.push(MemoryRegion {
            id,
            base,
            size,
            data,
            permissions: perms,
            owner: owner.map(String::from),
            freed: false,
        });

        Ok(id)
    }

    /// Free a previously allocated region.
    pub fn free(&mut self, id: RegionId) -> Result<(), MemError> {
        for region in &mut self.regions {
            if region.id == id {
                if region.freed {
                    return Err(MemError::RegionFreed(id.0));
                }
                region.freed = true;
                return Ok(());
            }
        }
        Err(MemError::RegionNotFound(id.0))
    }

    /// Look up which region contains `[addr, addr+len)` and return
    /// the region index and the offset within that region.
    fn find_region(&self, addr: u64, len: usize) -> Result<(usize, usize), MemError> {
        let end = addr.checked_add(len as u64);
        match end {
            Some(end) if end <= addr => {
                return Err(MemError::ReadOutOfBounds { addr, size: len });
            }
            _ => {}
        }
        for (i, region) in self.regions.iter().enumerate() {
            if region.freed {
                continue;
            }
            if addr >= region.base && end.unwrap_or(addr) <= region.base + region.size {
                let offset = (addr - region.base) as usize;
                return Ok((i, offset));
            }
        }
        Err(MemError::AddressOutOfBounds { addr })
    }

    // ── Reads ───────────────────────────────────────────────────

    pub fn read_u8(&self, addr: u64) -> Result<u8, MemError> {
        let (idx, off) = self.find_region(addr, 1)?;
        let r = &self.regions[idx];
        if !r.permissions.contains(Permissions::READ) {
            return Err(MemError::PermissionDenied { addr });
        }
        Ok(r.data[off])
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16, MemError> {
        let (idx, off) = self.find_region(addr, 2)?;
        let r = &self.regions[idx];
        if !r.permissions.contains(Permissions::READ) {
            return Err(MemError::PermissionDenied { addr });
        }
        Ok(u16::from_le_bytes([r.data[off], r.data[off + 1]]))
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32, MemError> {
        let (idx, off) = self.find_region(addr, 4)?;
        let r = &self.regions[idx];
        if !r.permissions.contains(Permissions::READ) {
            return Err(MemError::PermissionDenied { addr });
        }
        Ok(u32::from_le_bytes([
            r.data[off],
            r.data[off + 1],
            r.data[off + 2],
            r.data[off + 3],
        ]))
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64, MemError> {
        let (idx, off) = self.find_region(addr, 8)?;
        let r = &self.regions[idx];
        if !r.permissions.contains(Permissions::READ) {
            return Err(MemError::PermissionDenied { addr });
        }
        Ok(u64::from_le_bytes([
            r.data[off],
            r.data[off + 1],
            r.data[off + 2],
            r.data[off + 3],
            r.data[off + 4],
            r.data[off + 5],
            r.data[off + 6],
            r.data[off + 7],
        ]))
    }

    pub fn read_f64(&self, addr: u64) -> Result<f64, MemError> {
        let bits = self.read_u64(addr)?;
        Ok(f64::from_le_bytes(bits.to_le_bytes()))
    }

    /// Return a borrowed slice of `len` bytes starting at `addr`.
    pub fn read_bytes(&self, addr: u64, len: usize) -> Result<&[u8], MemError> {
        let (idx, off) = self.find_region(addr, len)?;
        let r = &self.regions[idx];
        if !r.permissions.contains(Permissions::READ) {
            return Err(MemError::PermissionDenied { addr });
        }
        Ok(&r.data[off..off + len])
    }

    // ── Writes ──────────────────────────────────────────────────

    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<(), MemError> {
        let (idx, off) = self.find_region(addr, 1)?;
        let r = &mut self.regions[idx];
        if !r.permissions.contains(Permissions::WRITE) {
            return Err(MemError::PermissionDenied { addr });
        }
        r.data[off] = val;
        Ok(())
    }

    pub fn write_u16(&mut self, addr: u64, val: u16) -> Result<(), MemError> {
        let (idx, off) = self.find_region(addr, 2)?;
        let r = &mut self.regions[idx];
        if !r.permissions.contains(Permissions::WRITE) {
            return Err(MemError::PermissionDenied { addr });
        }
        let bytes = val.to_le_bytes();
        r.data[off..off + 2].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn write_u32(&mut self, addr: u64, val: u32) -> Result<(), MemError> {
        let (idx, off) = self.find_region(addr, 4)?;
        let r = &mut self.regions[idx];
        if !r.permissions.contains(Permissions::WRITE) {
            return Err(MemError::PermissionDenied { addr });
        }
        let bytes = val.to_le_bytes();
        r.data[off..off + 4].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<(), MemError> {
        let (idx, off) = self.find_region(addr, 8)?;
        let r = &mut self.regions[idx];
        if !r.permissions.contains(Permissions::WRITE) {
            return Err(MemError::PermissionDenied { addr });
        }
        let bytes = val.to_le_bytes();
        r.data[off..off + 8].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn write_f64(&mut self, addr: u64, val: f64) -> Result<(), MemError> {
        self.write_u64(addr, val.to_bits())
    }

    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), MemError> {
        if data.is_empty() {
            return Ok(());
        }
        let (idx, off) = self.find_region(addr, data.len())?;
        let r = &mut self.regions[idx];
        if !r.permissions.contains(Permissions::WRITE) {
            return Err(MemError::PermissionDenied { addr });
        }
        r.data[off..off + data.len()].copy_from_slice(data);
        Ok(())
    }

    // ── Queries ─────────────────────────────────────────────────

    /// Return a reference to a region by ID, or `None` if not found / freed.
    pub fn region(&self, id: RegionId) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.id == id && !r.freed)
    }

    /// Load bytecode into a new region starting at [`CODE_BASE`].
    ///
    /// Returns the region ID on success.
    pub fn load_bytecode(&mut self, bytecode: &[u8]) -> Result<RegionId, MemError> {
        if bytecode.is_empty() {
            // Still create a minimal region so PC has somewhere valid to point.
            let size = 1u64;
            let id = RegionId(self.next_id);
            self.next_id += 1;
            self.regions.push(MemoryRegion {
                id,
                base: CODE_BASE,
                size,
                data: vec![0x01], // HALT
                permissions: Permissions::read_execute(),
                owner: Some("bytecode".to_string()),
                freed: false,
            });
            return Ok(id);
        }

        let size = bytecode.len() as u64;
        let id = RegionId(self.next_id);
        self.next_id += 1;

        let mut data = vec![0u8; size as usize];
        data.copy_from_slice(bytecode);

        self.regions.push(MemoryRegion {
            id,
            base: CODE_BASE,
            size,
            data,
            permissions: Permissions::read_execute(),
            owner: Some("bytecode".to_string()),
            freed: false,
        });

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_free() {
        let mut mm = MemoryManager::new();
        let id = mm.allocate(128, Permissions::read_write(), Some("test")).unwrap();
        assert!(mm.region(id).is_some());
        assert!(mm.free(id).is_ok());
        assert!(mm.region(id).is_none());
    }

    #[test]
    fn read_write_u64() {
        let mut mm = MemoryManager::new();
        let id = mm.allocate(64, Permissions::read_write(), None).unwrap();
        let r = mm.region(id).unwrap();
        let addr = r.base + 8;
        mm.write_u64(addr, 0xDEAD_BEEF_CAFE_BABE).unwrap();
        assert_eq!(mm.read_u64(addr).unwrap(), 0xDEAD_BEEF_CAFE_BABE);
    }

    #[test]
    fn read_write_bytes() {
        let mut mm = MemoryManager::new();
        let id = mm.allocate(32, Permissions::read_write(), None).unwrap();
        let r = mm.region(id).unwrap();
        let addr = r.base;
        mm.write_bytes(addr, b"hello").unwrap();
        assert_eq!(mm.read_bytes(addr, 5).unwrap(), b"hello");
    }

    #[test]
    fn out_of_bounds() {
        let mm = MemoryManager::new();
        assert!(mm.read_u8(0).is_err());
    }

    #[test]
    fn permission_denied() {
        let mut mm = MemoryManager::new();
        let id = mm.allocate(16, Permissions::read_execute(), None).unwrap();
        let base = mm.region(id).unwrap().base;
        assert!(mm.write_u8(base, 42).is_err());
        assert!(mm.read_u8(base).is_ok());
    }

    #[test]
    fn load_bytecode() {
        let mut mm = MemoryManager::new();
        let bc = vec![0x01, 0x02, 0x03];
        let id = mm.load_bytecode(&bc).unwrap();
        let r = mm.region(id).unwrap();
        assert_eq!(r.base, CODE_BASE);
        assert_eq!(r.data, bc);
    }
}
