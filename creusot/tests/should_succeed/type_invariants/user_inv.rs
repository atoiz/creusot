extern crate creusot_contracts;
use creusot_contracts::{invariant::Invariant, *};

pub struct Foo<T>(T);

impl Invariant for Foo<i32> {
    #[open]
    #[predicate]
    fn invariant(self) -> bool {
        pearlite! { self.0@ > 0 }
    }
}

/*
trait Bar {}
impl<T: Bar> Invariant for Foo<T> {
    #[open]
    #[predicate]
    fn invariant(self) -> bool {
        true
    }
}
*/

pub fn take_foo(x: Foo<i32>) {
    proof_assert! { x.0@ > 0 }
}

pub struct Bar(i32);

impl Invariant for Bar {
    #[open]
    #[predicate]
    fn invariant(self) -> bool {
        pearlite! { self.0@ > 0 }
    }
}

pub fn take_foo2(_x: Foo<Bar>) {}
