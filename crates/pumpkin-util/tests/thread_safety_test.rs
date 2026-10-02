use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use pumpkin_util::math::atomic_f32::AtomicF32;

#[test]
fn test_atomic_f32_thread_safety() {
    let atomic_val = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    for _ in 0..10 {
        let val_clone = Arc::clone(&atomic_val);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut current = val_clone.load(Ordering::SeqCst);
                while let Err(actual) = val_clone.compare_exchange(
                    current,
                    current + 1.0,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    current = actual;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert_eq!(atomic_val.load(Ordering::SeqCst), 1000.0);
}

#[test]
fn test_atomic_f32_store_and_load() {
    let val = AtomicF32::new(3.14159);
    assert!((val.load(Ordering::SeqCst) - 3.14159).abs() < f32::EPSILON);

    val.store(42.0, Ordering::SeqCst);
    assert!((val.load(Ordering::SeqCst) - 42.0).abs() < f32::EPSILON);
}
