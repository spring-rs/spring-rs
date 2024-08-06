use std::{any::Any, sync::Arc};

#[derive(Clone)]
pub struct ComponentRef(Arc<dyn Any + Send + Sync>);

impl ComponentRef {
    pub fn new<T>(component: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self(Arc::new(component))
    }

    pub fn downcast<T>(self) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        self.0.downcast::<T>().ok()
    }
}
