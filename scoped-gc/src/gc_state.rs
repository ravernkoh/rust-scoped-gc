use ::std::cell::Cell;
use ::std::mem;
use ::std::ptr::NonNull;
use ::std::raw::TraitObject;
use gc_alloc_err::GcAllocErr;
use gc_box::GcBox;
use trace::Trace;

unsafe fn from_trait_object<'a>(raw_object: &'a TraitObject) -> &'a GcBox<Trace> {
  mem::transmute(*raw_object)
}

unsafe fn to_trait_object<T: Trace>(object: &GcBox<T>) -> TraitObject {
  mem::transmute(*object)
}

pub struct GcState {
  pub allocated_bytes: usize,
  //  threshold: usize,
  // Linked-list of boxes
  pub boxes: Option<NonNull<TraitObject>>,
}

impl GcState {
  pub fn new() -> GcState {
    GcState {
      allocated_bytes: 0,
      boxes: None,
    }
  }

  // Allocates GC-managed memory for T
  pub fn alloc<T: Trace>(&mut self, value: T) -> Result<NonNull<GcBox<T>>, GcAllocErr> {
    // into_raw -> mem::forget, so we need to make sure we deallocate it ourselve
    let gc_box_ptr: *mut GcBox<T> = Box::into_raw(Box::new(GcBox {
      roots: Cell::new(1),
      marked: Cell::new(false),
      next: self.boxes,
      value: value,
    }));
    // We know that `gc_box` is not null so we can use `new_unchecked`
    self.allocated_bytes += mem::size_of::<GcBox<T>>();
    let box_ptr: NonNull<GcBox<T>> = unsafe { NonNull::new_unchecked(gc_box_ptr) };
    self.boxes = Some(unsafe { to_trait_object(box_ptr.as_ptr()) });
    Ok(unsafe { NonNull::new_unchecked(gc_box_ptr) })
  }

  pub fn collect_garbage(&mut self) {
    {
      // Mark
      let mut next_gc_box_ptr = self.boxes;
      while let Some(gc_box_ptr) = next_gc_box_ptr {
        let gc_box: &GcBox<Trace> = unsafe { from_trait_object(&gc_box_ptr).as_ref().unwrap() };
        if gc_box.roots.get() > 0 {
          gc_box.mark_box();
        }
        next_gc_box_ptr = gc_box.next;
      }
    }

    let mut unmarked: Vec<*mut GcBox<Trace>> = Vec::new();
    unsafe {
      // Collect
      let mut next_gc_box_ptr_ref = &mut self.boxes;
      while let Some(gc_box_ptr) = *next_gc_box_ptr_ref {
        let gc_box_ptr = to_trait_object(gc_box_ptr.as_ptr());
        if (*gc_box_ptr).marked.get() {
          (*gc_box_ptr).marked.set(false);
          next_gc_box_ptr_ref = &mut (*gc_box_ptr).next;
        } else {
          *next_gc_box_ptr_ref = (*gc_box_ptr).next;
          unmarked.push(gc_box_ptr);
        }
      }
    }

    for gc_box_ptr in unmarked.iter() {
      let gc_box = unsafe { Box::from_raw(*gc_box_ptr) };
      self.allocated_bytes = self.allocated_bytes.checked_sub(mem::size_of_val::<GcBox<_>>(gc_box.as_ref())).unwrap()
      // Implicitly drops `gc_box` and frees the associated memory
    }
  }
}

impl Drop for GcState {
  fn drop(&mut self) {
    let mut cur_box = self.boxes;
    while let Some(gc_box_ptr) = cur_box {
      let gc_box = unsafe { Box::from_raw(gc_box_ptr.as_ptr()) };
      cur_box = (*gc_box).next;
      // Implicitly drops `gc_box` and frees the associated memory
    }
  }
}
