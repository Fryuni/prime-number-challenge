#![feature(is_sorted)]
#[macro_use]
extern crate lazy_static;

use std::sync::{RwLock, Mutex, TryLockError};

lazy_static! {
  static ref KNOWN_PRIMES: RwLock<Vec<usize>> = RwLock::new(vec![2]);
  static ref CALCULATION_LOCK: Mutex<()> = Mutex::new(());
}

fn nth_prime(n: usize) -> usize {
    let read_vec = KNOWN_PRIMES.read().unwrap();
    if n < read_vec.len() {
        return read_vec[n];
    }
    drop(read_vec);


    let guard = loop {
        match CALCULATION_LOCK.try_lock() {
            Ok(guard) => break guard,
            Err(TryLockError::WouldBlock) => {
                // Another thread went through our target during its calculation but after
                // we contended for the guard. We can use their result and stop contending.

                // XXX: This could be changed into a signal to awake once the required value is calculated
                //  making this effectively
                let vec = KNOWN_PRIMES.read().unwrap();
                if vec.len() > n {
                    return vec[n];
                }
                drop(vec);
                std::thread::yield_now();
            }
            Err(TryLockError::Poisoned(_)) => unreachable!(),
        }
    };
    println!("Acquired guard calculating {}", n);

    let mut candidate = {
        let mut vec = KNOWN_PRIMES.write().unwrap();
        vec.reserve(n);

        *vec.last().unwrap() + 1
    };

    while KNOWN_PRIMES.read().unwrap().len() <= n {
        if is_prime(candidate) {
            KNOWN_PRIMES.write().unwrap().push(candidate);
        }
        candidate += 1;
    }

    drop(guard);

    KNOWN_PRIMES.read().unwrap()[n]
}

fn is_prime(n: usize) -> bool {
    let prime_list = KNOWN_PRIMES.read().unwrap();

    for &prime in &*prime_list {
        let (d, m) = (n / prime, n % prime);

        if m == 0 {
            return false;
        }

        if d < prime {
            return true;
        }
    }

    true
}

macro_rules! test_nth_prime {
    ($name:ident, $left:literal $right:literal) => {
        #[test]
        fn $name() {
            assert_eq!(nth_prime($left), $right);
        }
    };
}

test_nth_prime!(test_99999, 99999 1299709);
test_nth_prime!(test_9999, 9999 104729);
test_nth_prime!(test_999, 999 7919);
test_nth_prime!(test_50, 50 233);
test_nth_prime!(test_2, 2 5);
test_nth_prime!(test_1, 1 3);
test_nth_prime!(test_0, 0 2);

fn main() {
    assert_eq!(nth_prime(4), 11);
    assert_eq!(nth_prime(99999), 1299709);

    let vec = KNOWN_PRIMES.read().unwrap();

    println!("Last 1000 primes: {:?}", &vec[..100])
}
