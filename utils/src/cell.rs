//!
//! Persistent memory cell for non-zeroed SRAM.
//!
//! This module provides [`MemCell`], a low-level abstraction for memory regions
//! that survive resets (for example, Backup SRAM).
//!

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

///
/// A persistent, uninitialized memory cell.
///
/// This type is intended for use with memory regions that are **not cleared**
/// on reset (for example, Backup SRAM).
///
pub struct MemCell<T: Sized> {
    magic: UnsafeCell<MaybeUninit<u32>>,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Sized + Send> Send for MemCell<T> {}
unsafe impl<T: Sized + Sync> Sync for MemCell<T> {}

impl<T> MemCell<T> {
    ///
    /// Magic value indicating that the stored value is valid.
    ///
    /// This value is written **after** the value itself has been fully initialized.
    ///
    const MAGIC: u32 = 0xDEAD_BEEF;

    ///
    /// Creates a new uninitialized memory cell.
    ///
    /// This function does **not** write anything to memory. When placed in a
    /// non-zeroed memory section, existing contents are preserved.
    ///
    #[inline]
    pub const fn new() -> Self {
        Self {
            magic: UnsafeCell::new(MaybeUninit::uninit()),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    ///
    /// Returns a pointer to the magic number.
    ///
    /// The returned pointer may reference uninitialized or stale memory.
    /// No validity checks are performed.
    ///
    #[inline]
    fn magic(&self) -> *mut u32 {
        unsafe { (&mut *self.magic.get()).as_mut_ptr() }
    }

    ///
    /// Returns a pointer to the contained value.
    ///
    /// The memory pointed to by the returned pointer may be uninitialized
    /// or contain data from a previous boot.
    ///
    #[inline]
    fn value(&self) -> *mut T {
        unsafe { (&mut *self.value.get()).as_mut_ptr() }
    }
}

impl<T> MemCell<T> {
    ///
    /// Initialize Memory Cell
    ///
    /// Initializes the cell with a value and marks it as valid.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - No concurrent access to this `MemCell` (including interrupts or DMA)
    /// - No other live pointers exist to the same memory
    /// - `T` does not rely on Rust drop semantics
    /// - The memory backing this cell is valid for the lifetime of the program
    ///
    /// Violating these conditions may result in undefined behavior.
    ///
    pub unsafe fn init(&self, val: T) -> *mut T {
        unsafe {
            self.value().write_volatile(val);
            self.magic().write_volatile(Self::MAGIC);
        }

        self.value()
    }

    ///
    /// Get Value Pointer
    ///
    /// Returns a mutable pointer to the contained value if
    /// the memory cell is initialized, otherwise returns None.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - No concurrent mutation of the cell occurs while the returned pointer
    ///   is in use
    /// - The returned pointer is not used after `invalidate` or `init` is called
    /// - Aliasing rules are manually upheld
    ///
    /// This function does **not** guarantee exclusive access.
    ///
    /// # Note
    ///
    /// This function provides *logical* mutability via a raw pointer.
    /// It does not establish Rust-level exclusive access.
    ///
    pub unsafe fn get(&self) -> Option<*mut T> {
        let magic = unsafe { self.magic().read_volatile() };
        if magic == Self::MAGIC {
            Some(self.value())
        } else {
            None
        }
    }

    ///
    /// Invalidate Memory Cell
    ///
    /// After calling this function, `get()` will return `None` until the cell
    /// is reinitialized.
    ///
    /// The stored value is left untouched.
    ///
    /// # Safety
    ///
    /// The caller must ensure no pointers previously obtained from `get()` or
    /// `init()` are still being used.
    ///
    pub unsafe fn invalidate(&self) {
        unsafe { self.magic().write_volatile(0) }
    }
}
