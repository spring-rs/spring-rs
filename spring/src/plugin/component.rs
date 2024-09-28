use std::{any::Any, ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct DynComponentRef(Arc<dyn Any + Send + Sync>);

impl DynComponentRef {
    pub fn new<T>(component: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self(Arc::new(component))
    }

    pub fn downcast<T>(self) -> Option<ComponentRef<T>>
    where
        T: Any + Send + Sync,
    {
        self.0.downcast::<T>().ok().map(ComponentRef::new)
    }
}

#[derive(Debug, Clone)]
pub struct ComponentRef<T>(Arc<T>);

impl<T> ComponentRef<T> {
    fn new(target_ref: Arc<T>) -> Self {
        Self(target_ref)
    }

    pub fn into_raw(self) -> *const T {
        Arc::into_raw(self.0)
    }
}

impl<T> Deref for ComponentRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
