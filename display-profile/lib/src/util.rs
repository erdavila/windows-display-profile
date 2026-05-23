// An implementation of [try_find](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.try_find) to be used
// while the official one is still experimental.
pub(crate) trait TryFind: Iterator {
    fn try_find<E>(
        &mut self,
        f: impl FnMut(&Self::Item) -> Result<bool, E>,
    ) -> Result<Option<Self::Item>, E>;
}

impl<I> TryFind for I
where
    I: Iterator,
{
    fn try_find<E>(
        &mut self,
        mut f: impl FnMut(&Self::Item) -> Result<bool, E>,
    ) -> Result<Option<Self::Item>, E> {
        for item in self {
            if f(&item)? {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }
}
