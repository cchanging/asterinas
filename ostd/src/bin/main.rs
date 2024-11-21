#![no_std]
#![no_main]

extern crate ostd;
extern crate alloc;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    extern "Rust" {
        pub fn __ostd_panic_handler(info: &core::panic::PanicInfo) -> !;
    }
    unsafe { __ostd_panic_handler(info); }
}

#[no_mangle]
extern "Rust" fn __ostd_main() -> ! {
    let k = 0;
    loop {}
}

pub struct Foo {
        first: u8,
        second: u32,
}

#[derive(Copy, Clone)]
struct EvilSend<T>(pub T);

unsafe impl<T> Send for EvilSend<T> {}
unsafe impl<T> Sync for EvilSend<T> {}

#[cfg(miri)]
#[no_mangle]
fn miri_start(argc: isize, argv: *const *const u8) -> isize {
    use core::cell::RefCell;
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering;

    use alloc::sync::Arc;
    use local::init_on_bsp;
    use ostd::cpu::*;
    use ostd::offset_of;
    use ostd::prelude::*;
    use alloc::vec::Vec;
    use ostd::sync::Mutex;
    use ostd::sync::PreemptDisabled;
    use ostd::sync::SpinLock;

    fn test_full_cpu_set_iter_is_all() {
        let set = CpuSet::new_full();
        let num_cpus = num_cpus();
        let all_cpus = all_cpus().collect::<Vec<_>>();
        let set_cpus = set.iter().collect::<Vec<_>>();

        assert!(set_cpus.len() == num_cpus);
        assert_eq!(set_cpus, all_cpus);
    }

    unsafe { 
        ostd::task::reset_preempt_info();
        ostd::mm::heap_allocator::init();
        ostd::boot::init_for_miri();
        ostd::mm::page::allocator::init();
        ostd::cpu::init_num_cpus();
        ostd::cpu::set_this_cpu_id(0);
    }
    test_full_cpu_set_iter_is_all();
    unsafe { init_on_bsp(); }
    ostd::cpu_local! {
        static FOO: RefCell<usize> = RefCell::new(1);
    }
    let irq_guard = ostd::trap::disable_local();
    let foo_guard = FOO.get_with(&irq_guard);
    assert_eq!(*foo_guard.borrow(), 1);
    *foo_guard.borrow_mut() = 2;
    assert_eq!(*foo_guard.borrow(), 2);
    drop(foo_guard);

    let test_task = || {
        let mut a = AtomicUsize::new(0);
        let b = &mut a as *mut AtomicUsize;
        let c = EvilSend(b);

        let spin_locked_v: Arc<SpinLock<i32, PreemptDisabled>> = Arc::new(SpinLock::new(12));
        let cloned_spin_locked_v = spin_locked_v.clone();

        let mutex_locked_v = Arc::new(Mutex::new(12));
        let cloned_mutex_locked_v = mutex_locked_v.clone();
        
        let task1 = move || {
            assert_eq!(1, 1);
            let c = c.clone(); // avoid field capturing
            //unsafe{*(c.0 as *mut usize) = 32;} // race error;
            unsafe {(&*c.0).store(32, Ordering::SeqCst);}

            *spin_locked_v.lock() = 15;
            *mutex_locked_v.lock() = 15;
        };
        let task2 = move || {
            assert_eq!(1, 1);
            let c = c.clone(); // avoid field capturing
            unsafe {(&*c.0).load(Ordering::SeqCst);} //~ ERROR: Data race detected between (1) non-atomic write on thread `unnamed-1` and (2) atomic load on thread `unnamed-2`
            
            *cloned_spin_locked_v.lock() = 16;
            *cloned_mutex_locked_v.lock() = 16;
        };

        let task1 = alloc::sync::Arc::new(
            ostd::task::TaskOptions::new(task1)
                .data(())
                .build()
                .unwrap(),
        );
        let task2 = alloc::sync::Arc::new(
            ostd::task::TaskOptions::new(task2)
                .data(())
                .build()
                .unwrap(),
        );
        task1.run();
        task2.run();
        ostd::task::Task::yield_now();
        //panic!("hhh");
        // loop {
        //     ostd::task::Task::yield_now();
        // }
    };

    let task = alloc::sync::Arc::new(
        ostd::task::TaskOptions::new(test_task)
            .data(())
            .build()
            .unwrap(),
    );

    task.run();
    0
    // Call the actual start function that your project implements, based on your target's conventions.
}