#[macro_use]
extern crate getset;

#[derive(MutGetters)]
struct ForAsMut {
    #[getset(get_mut = "as_mut")]
    inner: Option<usize>,

    #[getset(get_mut)]
    inner_without_as_mut: Option<usize>,

    #[getset(get_mut = "as_mut")]
    inner_result: Result<usize, String>,

    #[getset(get_mut)]
    inner_result_without_as_mut: Result<usize, String>,
}

impl Default for ForAsMut {
    fn default() -> Self {
        Self {
            inner: None,
            inner_without_as_mut: None,
            inner_result: Ok(0),
            inner_result_without_as_mut: Ok(0),
        }
    }
}

#[derive(MutGetters, Default)]
#[getset(get_mut = "as_mut")]
struct Unnamed(Option<usize>);

#[derive(MutGetters)]
#[getset(get_mut = "as_mut")]
struct UnnamedResult(Result<usize, String>);

impl Default for UnnamedResult {
    fn default() -> Self {
        Self(Ok(0))
    }
}

#[test]
fn test_as_mut() {
    let mut val = ForAsMut::default();
    assert_eq!(val.inner_mut(), None);
    assert_eq!(val.inner_without_as_mut_mut(), &None);

    let mut val = ForAsMut {
        inner: Some(1),
        inner_without_as_mut: Some(2),
        inner_result: Err("error".to_owned()),
        inner_result_without_as_mut: Err("error".to_owned()),
    };
    assert_eq!(val.inner_mut(), Some(&mut 1));
    assert_eq!(val.inner_without_as_mut_mut(), &mut Some(2));
    assert_eq!(val.inner_result_mut(), Err(&mut "error".to_owned()));
    assert_eq!(
        val.inner_result_without_as_mut_mut(),
        &mut Err("error".to_owned())
    );
}

#[test]
fn test_as_mut_unnamed() {
    let mut val = Unnamed::default();
    assert_eq!(val.get_mut(), None);

    let mut val = Unnamed(Some(3));
    assert_eq!(val.get_mut(), Some(&mut 3));

    let mut val = UnnamedResult::default();
    assert_eq!(val.get_mut(), Ok(&mut 0));

    let mut val = UnnamedResult(Err("error".to_owned()));
    assert_eq!(val.get_mut(), Err(&mut "error".to_owned()));
}
