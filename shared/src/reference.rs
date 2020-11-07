cfg_if! {
    if #[cfg(feature = "multithread")] {
        use std::{
            ops::Deref,
            sync::{Arc, Mutex, MutexGuard},
        };

        pub struct Guard<'a, T> {
            inner: MutexGuard<'a, T>,
        }

        impl<'a, T> Deref for Guard<'a, T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                Deref::deref(&self.inner)
            }
        }

        /// A reference abstraction that can handle single-threaded and multi-threaded environments
        #[derive(Debug)]
        pub struct Ref<T> {
            inner: Arc<Mutex<T>>,
        }

        impl<T> Ref<T> {
            fn new(value: T) -> Self {
                Ref {
                    inner: Arc::new(Mutex::new(value)),
                }
            }

            fn borrow(&self) -> Guard<T> {
                Guard {
                    inner: self.inner.lock().unwrap(),
                }
            }
        }
    } else {
        use std::{
            cell::{Ref as StdRef, RefCell},
            ops::Deref,
            rc::Rc,
        };

        pub struct Guard<'a, T> {
            inner: StdRef<'a, T>,
        }

        impl<'a, T> Deref for Guard<'a, T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                Deref::deref(&self.inner)
            }
        }

        /// A reference abstraction that can handle single-threaded and multi-threaded
        /// environments
        #[derive(Debug)]
        pub struct Ref<T> {
            inner: Rc<RefCell<T>>,
        }

        impl<T> Ref<T> {
            fn new(value: T) -> Self {
                Ref {
                    inner: Rc::new(RefCell::new(value)),
                }
            }

            fn borrow(&self) -> Guard<T> {
                Guard {
                    inner: self.inner.borrow(),
                }
            }
        }
    }
}
