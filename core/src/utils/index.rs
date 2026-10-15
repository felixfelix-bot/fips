//! Session indices and their allocator.
//!
//! A session index is the 32-bit number a receiver chooses for each Noise
//! session and its peer puts in every encrypted frame, so the receiver can
//! find the session without a search.
//!
//! ## Design
//!
//! - Indices are random (cryptographically secure) to prevent guessing
//! - Unique across the node, not per transport: one allocator serves every
//!   transport
//! - 32-bit space supports ~65K concurrent sessions before birthday collision
//! - Indices are rotated on rekey for anti-correlation
//!
//! ## Wire Format
//!
//! Encrypted frames include receiver_idx for session lookup:
//!
//! ```text
//! [0x00][receiver_idx:4 LE][counter:8 LE][ciphertext+tag]
//! ```

use alloc::collections::BTreeSet;
use core::marker::PhantomData;

use thiserror::Error;

use crate::entropy::EntropySource;

/// A 32-bit session index.
///
/// Wrapper type for type safety and clarity in APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionIndex(u32);

impl SessionIndex {
    /// Create from raw u32.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Get the raw u32 value.
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Convert to little-endian bytes.
    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Create from little-endian bytes.
    pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }
}

impl core::fmt::Display for SessionIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

/// Errors related to index allocation.
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("no available indices (too many active sessions)")]
    Exhausted,

    #[error("index {0} not found")]
    NotFound(u32),

    #[error("index {0} already in use")]
    AlreadyInUse(u32),
}

/// Allocator for session indices, node-global across every transport.
///
/// Manages a pool of random 32-bit indices, tracking which are in use
/// to prevent collision. Thread-safe for single-threaded async use.
///
/// There is exactly one of these per node and its record of allocated indices
/// is the `in_use` set below, so an index value is unique across the node, not
/// per transport.
///
/// `E` is the host's random source, from which every candidate index is drawn.
#[derive(Debug)]
pub struct IndexAllocator<E> {
    /// Set of currently allocated indices.
    in_use: BTreeSet<u32>,
    /// Maximum allocation attempts before giving up.
    max_attempts: usize,
    entropy: PhantomData<fn() -> E>,
}

impl<E: EntropySource> IndexAllocator<E> {
    /// Create a new index allocator.
    pub fn new() -> Self {
        Self {
            in_use: BTreeSet::new(),
            max_attempts: 100,
            entropy: PhantomData,
        }
    }

    /// Create with a specific max attempts limit.
    pub fn with_max_attempts(max_attempts: usize) -> Self {
        Self {
            in_use: BTreeSet::new(),
            max_attempts,
            entropy: PhantomData,
        }
    }

    /// Allocate a new random index.
    ///
    /// Returns a cryptographically random 32-bit index that is not
    /// currently in use. Returns error if allocation fails after
    /// max_attempts tries (indicates too many active sessions).
    pub fn allocate(&mut self) -> Result<SessionIndex, IndexError> {
        for _ in 0..self.max_attempts {
            let mut b = [0u8; 4];
            E::fill(&mut b);
            let candidate = u32::from_le_bytes(b);
            if !self.in_use.contains(&candidate) {
                self.in_use.insert(candidate);
                return Ok(SessionIndex::new(candidate));
            }
        }

        Err(IndexError::Exhausted)
    }

    /// Free an index, returning it to the available pool.
    ///
    /// Returns error if the index was not allocated.
    pub fn free(&mut self, index: SessionIndex) -> Result<(), IndexError> {
        if self.in_use.remove(&index.as_u32()) {
            Ok(())
        } else {
            Err(IndexError::NotFound(index.as_u32()))
        }
    }

    /// Check if an index is currently allocated.
    pub fn is_allocated(&self, index: SessionIndex) -> bool {
        self.in_use.contains(&index.as_u32())
    }

    /// Number of currently allocated indices.
    pub fn count(&self) -> usize {
        self.in_use.len()
    }

    /// Check if the allocator is empty (no indices allocated).
    pub fn is_empty(&self) -> bool {
        self.in_use.is_empty()
    }

    /// Reserve a specific index (for testing or migration).
    ///
    /// Returns error if the index is already in use.
    pub fn reserve(&mut self, index: SessionIndex) -> Result<(), IndexError> {
        if self.in_use.contains(&index.as_u32()) {
            Err(IndexError::AlreadyInUse(index.as_u32()))
        } else {
            self.in_use.insert(index.as_u32());
            Ok(())
        }
    }

    /// Clear all allocations (use with caution).
    pub fn clear(&mut self) {
        self.in_use.clear();
    }
}

impl<E: EntropySource> Default for IndexAllocator<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests/index.rs"]
mod tests;
