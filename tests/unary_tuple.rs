use getset::{CopyGetters, Getters, MutGetters, Setters};

#[test]
fn test_unary_tuple() {
    #[derive(Setters, Getters, MutGetters)]
    struct UnaryTuple(#[getset(set, get, get_mut)] i32);

    let mut tup = UnaryTuple(42);
    assert_eq!(tup.get(), &42);
    assert_eq!(tup.get_mut(), &mut 42);
    tup.set(43);
    assert_eq!(tup.get(), &43);

    #[derive(CopyGetters)]
    struct CopyUnaryTuple(#[getset(get_copy)] i32);

    let tup = CopyUnaryTuple(42);
    assert_eq!(tup.get_copy(), 42);
}

#[test]
fn test_unary_tuple_with_attrs() {
    #[derive(Setters, Getters, MutGetters)]
    #[getset(set, get, get_mut)]
    struct UnaryTuple(i32);

    let mut tup = UnaryTuple(42);
    assert_eq!(tup.get(), &42);
    assert_eq!(tup.get_mut(), &mut 42);
    tup.set(43);
    assert_eq!(tup.get(), &43);

    #[derive(CopyGetters)]
    #[getset(get_copy)]
    struct CopyUnaryTuple(i32);

    let tup = CopyUnaryTuple(42);
    assert_eq!(tup.get_copy(), 42);
}
