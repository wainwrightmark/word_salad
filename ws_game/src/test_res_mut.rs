use bevy::{ecs::component::Tick, prelude::*};
use std::ops::DerefMut;

pub trait AnyResMut<T: Resource>:
    std::ops::Deref<Target = T> + AsMut<T> + DerefMut<Target = T>
{
}

impl<'a, T: Resource> AnyResMut<T> for ResMut<'a, T> {}

pub struct TestResMut<'a, T> {
    pub value: &'a mut T,
    pub added: bool,
    pub last_changed: Option<Tick>,
}

impl<'a, T: Resource> AnyResMut<T> for TestResMut<'a, T> {}

impl<'a, T> AsRef<T> for TestResMut<'a, T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<'a, T> DerefMut for TestResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set_changed();
        self.value
    }
}

impl<'a, T> AsMut<T> for TestResMut<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'a, T> std::ops::Deref for TestResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DetectChangesMut for TestResMut<'a, T> {
    type Inner = T;

    fn set_changed(&mut self) {
        self.last_changed = Some(Tick::new(1))
    }

    fn set_last_changed(&mut self, last_changed: Tick) {
        self.last_changed = Some(last_changed)
    }

    fn bypass_change_detection(&mut self) -> &mut Self::Inner {
        self.value
    }
}

impl<'a, T> DetectChanges for TestResMut<'a, T> {
    fn is_added(&self) -> bool {
        self.added
    }

    fn is_changed(&self) -> bool {
        self.last_changed.is_some()
    }

    fn last_changed(&self) -> bevy::ecs::component::Tick {
        Tick::new(0)
    }
}

impl<'w, 'a, T: Resource> IntoIterator for &'a TestResMut<'w, T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<'w, 'a, T: Resource> IntoIterator for &'a mut TestResMut<'w, T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.set_changed();
        self.value.into_iter()
    }
}
