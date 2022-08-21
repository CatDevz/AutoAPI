pub trait BaseClient<T: BaseClient = Self> {
    fn default_base_url() -> &'static str;
    fn new(options: ClientOptions<T>) -> Self;
}

pub struct ClientOptions<'a, T: BaseClient> {
    pub base_url: &'a str,
    pub _marker: std::marker::PhantomData<&'a T>,
}

impl<T: BaseClient> Default for ClientOptions<'_, T> {
    fn default() -> Self {
        Self {
            base_url: T::default_base_url(),
            _marker: std::marker::PhantomData,
        }
    }
}
