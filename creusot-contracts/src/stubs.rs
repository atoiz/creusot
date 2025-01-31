#[creusot::no_translate]
#[rustc_diagnostic_item = "fin"]
pub fn fin<T: ?Sized>(_: &mut T) -> Box<T> {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "equal"]
pub fn equal<T: ?Sized>(_: T, _: T) -> bool {
    panic!();
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "neq"]
pub fn neq<T: ?Sized>(_: T, _: T) -> bool {
    panic!();
}

// FIXME : T should be ?Sized
#[creusot::no_translate]
#[rustc_diagnostic_item = "exists"]
pub fn exists<T, F: Fn(T) -> bool>(_: F) -> bool {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "forall"]
pub fn forall<T, F: Fn(T) -> bool>(_: F) -> bool {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "implication"]
pub fn implication(_: bool, _: bool) -> bool {
    panic!();
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "old"]
pub fn old<T: ?Sized>(_: T) -> Box<T> {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "absurd"]
pub fn abs<T: ?Sized>() -> Box<T> {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "variant_check"]
pub fn variant_check<R: crate::well_founded::WellFounded + ?Sized>(_: R) -> Box<R> {
    panic!()
}

#[creusot::no_translate]
#[rustc_diagnostic_item = "closure_result_constraint"]
pub fn closure_result<R: ?Sized>(_: R, _: R) {}

#[creusot::no_translate]
#[rustc_diagnostic_item = "snapshot_from_fn"]
pub fn snapshot_from_fn<T: ?Sized, F: Fn() -> crate::Snapshot<T>>(_: F) -> crate::Snapshot<T> {
    panic!()
}

#[creusot::no_translate]
#[creusot::builtins = "prelude.prelude.Mapping.from_fn"]
pub fn mapping_from_fn<A, B, F: FnOnce(A) -> B>(_: F) -> crate::logic::Mapping<A, B> {
    panic!()
}
