use crate::alloc::{Alloc, GlobalAlloc, alloc_new, alloc_array, drop_and_free};
use crate::mem::{NonNull, ManuallyDrop};


pub struct Box<T: ?Sized, A: Alloc = GlobalAlloc> {
    value: NonNull<T>,
    alloc: A,
}

impl<T> Box<T, GlobalAlloc> {
    #[inline]
    pub fn new(value: T) -> Self {
        Box::new_in(GlobalAlloc, value)
    }

    #[inline]
    pub fn into_raw_parts(self) -> NonNull<T> {
        let this = ManuallyDrop::new(self);
        return this.value;
    }

    /// #safety:
    /// - `value` must be a live allocation of a `T` in `GlobalAlloc`.
    /// - in particular, `Layout::for_value(value.as_ref())`
    ///   must be the active layout.
    /// - `value` must be valid at `T`.
    #[inline]
    pub unsafe fn from_raw_parts(value: NonNull<T>) -> Self {
        Self { value, alloc: GlobalAlloc }
    }

}

impl<T> Box<[T], GlobalAlloc> {
    #[inline]
    pub fn from_slice(values: &[T]) -> Self  where T: Clone {
        Box::from_slice_in(GlobalAlloc, values)
    }
}

impl<T, A: Alloc> Box<T, A> {
    #[track_caller]
    #[inline]
    pub fn new_in(alloc: A, value: T) -> Self {
        let value = alloc_new(&alloc, value).expect("oom");
        Self { value, alloc }
    }
}

impl<T, A: Alloc> Box<[T], A> {
    #[track_caller]
    #[inline]
    pub fn from_slice_in(alloc: A, values: &[T]) -> Self  where T: Clone {
        let ptr = alloc_array::<T, _>(&alloc, values.len()).expect("oom").as_ptr();
        for i in 0..values.len() {
            unsafe { ptr.add(i).write(values[i].clone()) };
        }
        let value = unsafe { NonNull::from(core::slice::from_raw_parts_mut(ptr, values.len())) };
        Self { value, alloc }
    }
}

impl<T: ?Sized, A: Alloc> Box<T, A> {
    #[inline]
    pub fn inner(&self) -> NonNull<T> { self.value }

    #[inline]
    pub fn into_raw_parts_in(self) -> (NonNull<T>, A) {
        let this = ManuallyDrop::new(self);
        let alloc = unsafe { core::ptr::read(&this.alloc) };
        return (this.value, alloc);
    }

    /// #safety:
    /// - `value` must be a live allocation of a `T` in `alloc`.
    /// - in particular, `Layout::for_value(value.as_ref())`
    ///   must be the active layout.
    /// - `value` must be valid at `T`.
    #[inline]
    pub unsafe fn from_raw_parts_in(value: NonNull<T>, alloc: A) -> Self {
        Self { value, alloc }
    }

    /// - this does not drop the allocator.
    #[inline]
    pub fn leak<'a>(self) -> &'a mut T  where A: 'a {
        let mut this = core::mem::ManuallyDrop::new(self);
        return unsafe { this.value.as_mut() };
    }
}

unsafe impl<T: ?Sized + Sync, A: Alloc + Sync> Sync for Box<T, A> {}
unsafe impl<T: ?Sized + Send, A: Alloc + Send> Send for Box<T, A> {}


impl<T: ?Sized, A: Alloc> core::ops::Deref for Box<T, A> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value.as_ptr() }
    }
}

impl<T: ?Sized, A: Alloc> core::ops::DerefMut for Box<T, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value.as_ptr() }
    }
}

impl<T: ?Sized, A: Alloc> Drop for Box<T, A> {
    #[inline]
    fn drop(&mut self) {
        unsafe { drop_and_free(&self.alloc, self.value) }
    }
}

