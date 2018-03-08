use ::std::cell::{Cell, RefCell};
use ::std::marker::PhantomData;
use gc_alloc_err::GcAllocErr;
use gc::Gc;
use gc_state::GcState;
use trace::Trace;

pub struct GcScope {
  state: RefCell<GcState>,
}

impl GcScope {
  pub fn new() -> GcScope {
    GcScope { state: RefCell::new(GcState::new()) }
  }

  /// Allocates `value` in this garbage-collected scope and returns a `Gc` smart pointer to it.
  pub fn alloc<'gc, T: Trace + 'gc>(&'gc self, value: T) -> Result<Gc<'gc, T>, GcAllocErr> {
    value.unroot();
    self.state.borrow_mut()
      .alloc(value)
      .map(|ptr| Gc { ptr, phantom: PhantomData, rooted: Cell::new(true) })
  }

  pub fn collect_garbage(self) {
    self.state.borrow_mut().collect_garbage()
  }
}
