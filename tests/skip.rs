#[macro_use]
extern crate getset;

#[derive(CopyGetters)]
#[get_copy]
pub struct Plain {
    /// A doc comment.
    #[getset(skip)]
    non_copyable: String,

    copyable: usize,
}

impl Plain {
    fn custom_non_copyable(&self) -> &str {
        &self.non_copyable
    }
}

impl Default for Plain {
    fn default() -> Self {
        Plain {
            non_copyable: "foo".to_string(),
            copyable: 3,
        }
    }
}

#[test]
fn test_plain() {
    let val = Plain::default();
    val.copyable();
    val.custom_non_copyable();
}
