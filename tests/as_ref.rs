#[macro_use]
extern crate getset;

#[derive(Getters)]
struct ForAsRef {
    #[getset(get = "as_ref")]
    inner: Option<usize>,

    #[getset(get)]
    inner_without_as_ref: Option<usize>,

    #[getset(get = "as_ref")]
    inner_result: Result<usize, String>,

    #[getset(get)]
    inner_result_without_as_ref: Result<usize, String>,
}

impl Default for ForAsRef {
    fn default() -> Self {
        Self {
            inner: None,
            inner_without_as_ref: None,
            inner_result: Ok(0),
            inner_result_without_as_ref: Ok(0),
        }
    }
}

#[derive(Getters, Default)]
#[getset(get = "as_ref")]
struct Unnamed(Option<usize>);

#[derive(Getters)]
#[getset(get = "as_ref")]
struct UnnamedResult(Result<usize, String>);

impl Default for UnnamedResult {
    fn default() -> Self {
        Self(Ok(0))
    }
}

#[test]
fn test_as_ref() {
    let val = ForAsRef::default();
    assert_eq!(val.inner(), None);
    assert_eq!(val.inner_without_as_ref(), &None);
    assert_eq!(val.inner_result(), Ok(&0));
    assert_eq!(val.inner_result_without_as_ref(), &Ok(0));

    let val = ForAsRef {
        inner: Some(1),
        inner_without_as_ref: Some(2),
        inner_result: Err("error".to_owned()),
        inner_result_without_as_ref: Err("error".to_owned()),
    };
    assert_eq!(val.inner(), Some(&1));
    assert_eq!(val.inner_without_as_ref(), &Some(2));
    assert_eq!(val.inner_result(), Err(&"error".to_owned()));
    assert_eq!(val.inner_result_without_as_ref(), &Err("error".to_owned()));
}

#[test]
fn test_as_ref_unnamed() {
    let val = Unnamed::default();
    assert_eq!(val.get(), None);

    let val = Unnamed(Some(3));
    assert_eq!(val.get(), Some(&3));

    let val = UnnamedResult::default();
    assert_eq!(val.get(), Ok(&0));

    let val = UnnamedResult(Err("error".to_owned()));
    assert_eq!(val.get(), Err(&"error".to_owned()));
}
