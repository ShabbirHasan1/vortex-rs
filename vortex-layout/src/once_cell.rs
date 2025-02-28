use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Weak};

use async_once_cell::OnceCell;
use futures::FutureExt;

/// Lock that can only be initialized once by an by an async function.
/// It returns an [`Arc`], and once all handles to the data are dropped it'll free the value and never be initialized again.
pub struct VortexOnceCell<T: ?Sized> {
    inner: OnceCell<Weak<T>>,
    is_populated: AtomicBool,
}

impl<T: ?Sized> VortexOnceCell<T> {
    pub fn new() -> Self {
        Self {
            inner: OnceCell::new(),
            is_populated: AtomicBool::new(false),
        }
    }

    // pub fn get(&self) -> Option<Arc<T>> {
    //     self.inner.get().and_then(|w| w.upgrade())
    // }

    pub async fn get_or_try_init<E>(
        &self,
        init: impl Future<Output = Result<Arc<T>, E>>,
    ) -> Result<Arc<T>, E> {
        let init = init.map(|v| {
            let arc = v?;
            let weak_ref = Arc::downgrade(&arc);

            // When we initialize, we increment the refcount by 1 so it won't get dropped immediately.
            unsafe {
                let raw_ptr = Arc::into_raw(arc);
                Arc::increment_strong_count(raw_ptr);
                Arc::from_raw(raw_ptr)
            };

            Ok(weak_ref)
        });
        let inner = self.inner.get_or_try_init(init).await?;

        use std::sync::atomic::Ordering::SeqCst;

        if let Some(upgraded) = inner.upgrade() {
            // If the current value is false, we switch and replace but also have to decrement the refcount back.

            if self
                .is_populated
                .compare_exchange(false, true, SeqCst, SeqCst)
                .is_ok_and(|v| !v)
            {
                let arc = upgraded.clone();
                unsafe {
                    let raw_ptr = Arc::into_raw(arc);
                    Arc::decrement_strong_count(raw_ptr);
                    Arc::from_raw(raw_ptr);
                }
            }
            println!("Got some data!");
            Ok(upgraded)
        } else {
            panic!("Value is already gone")
        }
    }
}

#[cfg(test)]
mod tests {
    use vortex_error::VortexResult;

    use super::*;

    #[tokio::test]
    async fn basic_counter() {
        let cell = VortexOnceCell::new();
        let v = cell
            .get_or_try_init(async move { VortexResult::Ok(Arc::new(5)) })
            .await
            .unwrap();

        assert_eq!(Arc::strong_count(&v), 1);
    }

    #[tokio::test]
    async fn get_or_init_while_alive() {
        let cell = VortexOnceCell::new();
        let x = cell
            .get_or_try_init(async move { VortexResult::Ok(Arc::new(5)) })
            .await
            .unwrap();
        // Even though we're initializing a different value, we should still get the original
        let y = cell
            .get_or_try_init(async move { VortexResult::Ok(Arc::new(6)) })
            .await
            .unwrap();

        assert_eq!(Arc::strong_count(&x), 2);
        assert_eq!(Arc::strong_count(&y), 2);
        assert_eq!(x, y);
    }

    #[tokio::test]
    #[should_panic(expected = "Value is already gone")]
    async fn panic_if_empty_and_reinitialized() {
        let cell = VortexOnceCell::new();
        let v = cell
            .get_or_try_init(async move { VortexResult::Ok(Arc::new(5)) })
            .await
            .unwrap();
        drop(v);
        cell.get_or_try_init(async move { VortexResult::Ok(Arc::new(5)) })
            .await
            .unwrap();
    }
}
