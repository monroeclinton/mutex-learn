use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Sync> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    fn lock(&self) -> MutexGuard<T> {
        while self.lock.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        MutexGuard { mutex: self }
    }
}

struct MutexGuard<'a, T: 'a> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
    }
}

fn main() {
    let mutex = Arc::new(Mutex::new(1));

    let mut threads = Vec::new();

    for _ in 0..70 {
        let mutex = mutex.clone();

        threads.push(std::thread::spawn(move || {
            let mut data = mutex.lock();
            *data += 1;
        }));
    }

    while let Some(thread) = threads.pop() {
        thread.join().unwrap();
    }

    let data = mutex.lock();

    println!("{}", *data);
}
