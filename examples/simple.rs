use getset::{CopyGetters, Getters, MutGetters, Setters, Withers};

#[derive(Getters, Setters, Withers, MutGetters, CopyGetters, Default)]
pub struct Foo<T>
where
    T: Copy + Clone + Default,
{
    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get, set, with, get_mut)]
    private: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[getset(get_copy = "pub", set = "pub", get_mut = "pub")]
    public: T,
}

fn main() {
    let mut foo = Foo::default();
    foo.set_private(1);
    foo = foo.with_private(2);
    (*foo.private_mut()) += 1;
    assert_eq!(*foo.private(), 3);
}