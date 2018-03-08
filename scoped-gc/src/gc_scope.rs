use ::std::cell::{Cell, RefCell};
use ::std::marker::PhantomData;
use gc::Gc;
use gc_alloc_err::GcAllocErr;
use gc_state::GcState;
use trace::Trace;

#[derive(Debug)]
pub struct GcScope<'gc> {
  state: RefCell<GcState<'gc>>,
}

impl<'gc> GcScope<'gc> {
  pub fn with<R, F>(f: F) -> R where F: FnOnce(&GcScope) -> R {
    let gc_scope = GcScope { state: RefCell::new(GcState::new()) };
    f(&gc_scope)
  }

  /// Allocates `value` in this garbage-collected scope and returns a `Gc` smart pointer to it.
  pub fn alloc<T: Trace + 'gc>(&self, value: T) -> Result<Gc<'gc, T>, GcAllocErr> {
    value.unroot();
    self.state.borrow_mut()
      .alloc(value)
      .map(|ptr| Gc { ptr, phantom: PhantomData, rooted: Cell::new(true) })
  }

  pub fn collect_garbage(self) {
    self.state.borrow_mut().collect_garbage()
  }
}
