/// Simple struct to manually cache the results of one
/// function call (or many)
///
/// Requires a new function instance at each `get` call
/// to allow maximum laziness on the part of the caller--
/// as the long as the result of the function is the same,
/// the function implementation may be different
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LazyCached<V> {
    v: Option<V>,
}

impl<V> LazyCached<V> {
    #[allow(unused)]
    pub fn new_cached(v: V) -> Self {
        Self { v: Some(v) }
    }

    pub fn new_uncached() -> Self {
        Self { v: None }
    }

    pub fn get<F: FnOnce() -> V>(&mut self, f: F) -> &V {
        if self.v.is_none() {
            self.v.replace((f)());
        }
        self.v.as_ref().unwrap()
    }

    #[allow(unused)]
    pub fn get_mut<F: FnOnce() -> V>(&mut self, f: F) -> &mut V {
        if self.v.is_none() {
            self.v.replace((f)());
        }
        self.v.as_mut().unwrap()
    }

    pub fn invalidate(&mut self) -> &mut Self {
        self.v.take();
        self
    }
}

#[cfg(test)]
mod test_cached {
    #[cfg(test)]
    mod new_cached {
        use std::{cell::RefCell, rc::Rc};

        use crate::LazyCached;

        #[test]
        fn should_accept_initial_value() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                String::from("new value")
            };

            let mut cached = LazyCached::new_cached(String::from("example"));

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // data stored correctly
            assert_eq!(cached.get(f), "example");
        }

        #[test]
        fn should_not_rerun_function_while_cache_is_valid() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                String::from("new value")
            };

            let mut cached = LazyCached::new_cached(String::from("example"));

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // data stored correctly
            assert_eq!(cached.get(f), "example");

            // calling a second time should not cause function to get rerun
            assert_eq!(cached.get(f), "example");

            // still not called
            assert_eq!(*num_times_called.borrow(), 0);
        }

        #[test]
        fn should_rerun_fn_after_cached_invalidated() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                String::from("new value")
            };

            let mut cached = LazyCached::new_cached(String::from("example"));

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // data stored correctly
            assert_eq!(cached.get(f), "example");

            // calling a second time should not cause function to get rerun
            assert_eq!(cached.get(f), "example");

            // still not called
            assert_eq!(*num_times_called.borrow(), 0);

            cached.invalidate();

            // now new value calculated
            assert_eq!(cached.get(f), "new value");

            assert_eq!(*num_times_called.borrow(), 1);
        }
    }

    #[cfg(test)]
    mod new_uncached {
        use std::{cell::RefCell, rc::Rc};

        use crate::LazyCached;

        #[test]
        fn should_run_callback_on_first_get() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                String::from("example")
            };

            let mut cached = LazyCached::new_uncached();

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // first `get`
            // data is correct
            assert_eq!(cached.get(f), "example");
            // function run once
            assert_eq!(*num_times_called.borrow(), 1);
        }

        #[test]
        fn should_not_rerun_function_while_cache_is_valid() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                String::from("example")
            };

            let mut cached = LazyCached::new_uncached();

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // first `get`
            // data is correct
            assert_eq!(cached.get(f), "example");
            // function run once
            assert_eq!(*num_times_called.borrow(), 1);

            // calling a second time should not cause function to get rerun
            assert_eq!(cached.get(f), "example");

            // still not called
            assert_eq!(*num_times_called.borrow(), 1);
        }

        #[test]
        fn should_rerun_fn_after_cached_invalidated() {
            let num_times_called = Rc::new(RefCell::new(0));
            let f = || {
                *num_times_called.borrow_mut() += 1;
                if *num_times_called.borrow() > 1 {
                    return String::from("new example");
                }

                String::from("example")
            };

            let mut cached = LazyCached::new_uncached();

            // not called at first
            assert_eq!(*num_times_called.borrow(), 0);

            // first `get`
            // data is correct
            assert_eq!(cached.get(f), "example");
            // function run once
            assert_eq!(*num_times_called.borrow(), 1);

            // calling a second time should not cause function to get rerun
            assert_eq!(cached.get(f), "example");

            // still only called once
            assert_eq!(*num_times_called.borrow(), 1);

            // cache is now invalid
            cached.invalidate();

            // now new value calculated
            assert_eq!(cached.get(f), "new example");

            assert_eq!(*num_times_called.borrow(), 2);
        }
    }
}
