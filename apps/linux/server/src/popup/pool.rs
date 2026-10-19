//! A `memfd` the shell hands to `wl_shm`, divided into equally sized slots the render thread draws into one at a time.

use std::fs::File;
use std::io;

use memmap2::MmapMut;
use rustix::fs::{MemfdFlags, ftruncate, memfd_create};

/// Slots per pool: one on screen, one the compositor may still read, one being drawn.
pub const SLOTS: usize = 3;

/// Slot sizes are rounded up to this, so a window that grows a little does not need a new pool every time.
const SLOT_GRANULE: usize = 64 * 1024;

pub struct Pool {
    id: u64,
    file: File,
    map: MmapMut,
    slot_size: usize,
    busy: [bool; SLOTS],
}

impl Pool {
    /// A pool whose slots hold at least `bytes` each.
    pub fn new(id: u64, bytes: usize) -> io::Result<Self> {
        let slot_size = bytes.max(1).div_ceil(SLOT_GRANULE) * SLOT_GRANULE;
        let fd = memfd_create("bornopata-popup", MemfdFlags::CLOEXEC)?;
        ftruncate(&fd, (slot_size * SLOTS) as u64)?;
        let file = File::from(fd);
        // The mapping is private to this process while the shell only passes the descriptor on; nobody truncates
        // the file, so the mapping stays valid.
        let map = unsafe { MmapMut::map_mut(&file)? };
        Ok(Self {
            id,
            file,
            map,
            slot_size,
            busy: [false; SLOTS],
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    /// Bytes in the whole pool.
    pub fn size(&self) -> usize {
        self.slot_size * SLOTS
    }

    pub fn slot_size(&self) -> usize {
        self.slot_size
    }

    /// Takes a free slot, or `None` when the compositor still holds all of them.
    pub fn acquire(&mut self) -> Option<usize> {
        let slot = self.busy.iter().position(|busy| !busy)?;
        self.busy[slot] = true;
        Some(slot)
    }

    pub fn release(&mut self, slot: usize) {
        if let Some(busy) = self.busy.get_mut(slot) {
            *busy = false;
        }
    }

    /// No slot is in use, so nothing refers to the pool any more.
    pub fn is_idle(&self) -> bool {
        self.busy.iter().all(|busy| !busy)
    }

    pub fn offset(&self, slot: usize) -> usize {
        slot * self.slot_size
    }

    pub fn slot_mut(&mut self, slot: usize) -> &mut [u8] {
        let start = slot * self.slot_size;
        &mut self.map[start..start + self.slot_size]
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom};

    use super::*;

    #[test]
    fn slots_are_handed_out_until_none_is_left_and_come_back_when_released() {
        let mut pool = Pool::new(1, 1000).unwrap();
        let slots: Vec<_> = (0..SLOTS).map(|_| pool.acquire().unwrap()).collect();
        assert_eq!(slots, [0, 1, 2]);
        assert!(pool.acquire().is_none());
        assert!(!pool.is_idle());
        pool.release(1);
        assert_eq!(pool.acquire(), Some(1));
        for slot in 0..SLOTS {
            pool.release(slot);
        }
        assert!(pool.is_idle());
        pool.release(99);
    }

    #[test]
    fn a_slot_has_room_for_the_request_rounded_up_and_the_file_has_every_slot() {
        let pool = Pool::new(1, SLOT_GRANULE + 1).unwrap();
        assert_eq!(pool.slot_size(), 2 * SLOT_GRANULE);
        assert_eq!(pool.size(), SLOTS * 2 * SLOT_GRANULE);
        assert_eq!(pool.file().metadata().unwrap().len(), pool.size() as u64);
        assert_eq!(pool.offset(2), 4 * SLOT_GRANULE);
    }

    #[test]
    fn what_is_written_in_a_slot_is_in_the_file_the_shell_gets() {
        let mut pool = Pool::new(1, 16).unwrap();
        let slot = pool.acquire().unwrap();
        pool.slot_mut(slot)[..4].copy_from_slice(&[1, 2, 3, 4]);
        let mut file = pool.file().try_clone().unwrap();
        file.seek(SeekFrom::Start(pool.offset(slot) as u64))
            .unwrap();
        let mut bytes = [0u8; 4];
        file.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [1, 2, 3, 4]);
    }
}
