use ::std::cell::Cell;
use ::std::ptr::NonNull;
use ::std::raw::TraitObject;
use trace::Trace;

// Private: keeps track of the roots and marked state
pub struct GcBox<T: Trace + ? Sized> {
  // 8 bytes
  pub roots: Cell<usize>,
  // 1 byte
  pub marked: Cell<bool>,
  // 16 bytes
  pub next: Option<NonNull<TraitObject>>,
  pub value: T,
}

impl<T: Trace + ? Sized> GcBox<T> {
  pub fn mark_box(&self) {
    if !self.marked.get() {
      self.marked.set(true);
      self.value.trace()
    }
  }

  pub fn inc_roots(&self) {
    self.roots.set(self.roots.get().checked_add(1).unwrap())
  }

  pub fn dec_roots(&self) {
    self.roots.set(self.roots.get().checked_sub(1).unwrap())
  }
}
