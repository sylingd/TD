use futures::{Future, Poll};

use super::error::Error;

#[must_use = "futures do nothing unless polled"]
pub struct TdFuture<T> {
    inner: Box<Future<Item=T, Error=Error>>
}

pub trait NewTdFuture<T> {
    fn new(inner: Box<Future<Item=T, Error=Error>>) -> Self;
}

impl<T> NewTdFuture<T> for TdFuture<T> {
    fn new(inner: Box<Future<Item=T, Error=Error>>) -> Self {
        Self {
            inner: inner
        }
    }
}

impl<T> Future for TdFuture<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}