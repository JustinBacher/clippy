use std::{
    pin::Pin,
    task::{Context, Poll},
};

use genawaiter::{Generator, GeneratorState};
use tokio_stream::Stream;

pub struct GeneratorStream<T> {
    generator: Pin<Box<dyn Generator<Yield = T, Return = ()>>>,
}

impl<T> GeneratorStream<T> {
    pub fn new(generator: Box<dyn Generator<Yield = T, Return = ()>>) -> Self {
        Self {
            generator: generator.into(),
        }
    }
}

impl<T> Stream for GeneratorStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Use the generator's resume method to drive it
        let this = self.get_mut();
        match this.generator.as_mut().resume() {
            GeneratorState::Yielded(clip) => Poll::Ready(Some(clip)),
            GeneratorState::Complete(_) => Poll::Ready(None),
        }
    }
}
