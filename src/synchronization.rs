use core::cell::UnsafeCell;

use crate::utils;

pub mod interface {
    use core::ops::{Deref, DerefMut};

    pub trait Mutex {
        type Data;

        fn lock(&self) -> Result<MutexGuard<Self>, ()>;
        fn unlock(&self) -> Result<(), ()>;
        unsafe fn get_data(&self) -> &Self::Data;
        unsafe fn get_data_mut(&self) -> &mut Self::Data;
    }

    pub struct MutexGuard<'a, T: ?Sized + Mutex> {
        inner: &'a T,
    }
    
    impl<'a, T: ?Sized + Mutex> MutexGuard<'a, T> {
        pub fn new(inner: &'a T) -> Self {
            Self {
                inner
            }
        }
    }

    impl<T: ?Sized + Mutex> Drop for MutexGuard<'_, T> {
        fn drop(&mut self) {
            self.inner.unlock().unwrap();
        }
    }

    impl<T: ?Sized + Mutex> Deref for MutexGuard<'_, T> {
        type Target = T::Data;

        fn deref(&self) -> &Self::Target {
            unsafe {
                self.inner.get_data()
            }
        }
    }

    impl<T: ?Sized + Mutex> DerefMut for MutexGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {
                self.inner.get_data_mut()
            }
        }
    }

}

pub struct SpinLock<T> where T: ?Sized {
    guard: u64,
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for SpinLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for SpinLock<T> where T: ?Sized + Send {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            guard: 0,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::Mutex for SpinLock<T> {
    type Data = T;

    fn lock(&self) -> Result<interface::MutexGuard<Self>, ()> {
        unsafe {
            utils::u64_lock_acquire_asm(&self.guard as *const u64);
        }
        Ok(interface::MutexGuard::new(self))
    }

    fn unlock(&self) -> Result<(), ()> {
        unsafe {
            utils::u64_lock_release_asm(&self.guard as *const u64);
        }
        Ok(())
    }

    unsafe fn get_data(&self) -> &Self::Data {
        & *self.data.get()
    }

    unsafe fn get_data_mut(&self) -> &mut Self::Data {
        &mut *self.data.get()
    }

}


pub struct FakeLock<T> where T: ?Sized {
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for FakeLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for FakeLock<T> where T: ?Sized + Send {}

impl<T> FakeLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::Mutex for FakeLock<T> {
    type Data = T;


    fn lock(&self) -> Result<interface::MutexGuard<Self>, ()> {
        Ok(interface::MutexGuard::new(self))
    }

    fn unlock(&self) -> Result<(), ()> {
        Ok(())
    }

    unsafe fn get_data(&self) -> &Self::Data {
        & *self.data.get()
    }

    unsafe fn get_data_mut(&self) -> &mut Self::Data {
        &mut *self.data.get()
    }

}
