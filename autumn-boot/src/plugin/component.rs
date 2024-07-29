use std::{any::Any, ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct ComponentRef(Arc<dyn Any + Send + Sync>);

impl ComponentRef {
    pub fn new<T>(component: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self(Arc::new(component))
    }

    pub fn downcast<T>(self) -> Result<Arc<T>, Arc<dyn Any + Send + Sync>>
    where
        T: Any + Send + Sync,
    {
        self.0.downcast::<T>()
    }
}

impl Deref for ComponentRef {
    type Target = dyn Any;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
