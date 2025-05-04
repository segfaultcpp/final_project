use std::marker::PhantomData;

pub(super) trait ScopedBind: Sized {
    fn scoped_bind<'a>(&self, gl: &'a glow::Context) -> UnbindOnDrop<'a, Self>;
    fn unbind(gl: &glow::Context);
}

pub(super) struct UnbindOnDrop<'a, T: ScopedBind> {
    gl: &'a glow::Context,
    _a: PhantomData<T>,
}

impl<'a, T: ScopedBind> UnbindOnDrop<'a, T> {
    pub(super) fn new(gl: &'a glow::Context) -> Self {
        Self {
            gl,
            _a: PhantomData,
        }
    }
}

#[macro_export]
macro_rules! unbind_on_drop {
    ($gl:expr) => {
        UnbindOnDrop::<Self>::new($gl)
    };
}

impl<'a, T: ScopedBind> Drop for UnbindOnDrop<'a, T> {
    fn drop(&mut self) {
        <T as ScopedBind>::unbind(self.gl);
    }
}
