use anyhow::Result;
use futures::StreamExt;
use genawaiter::{Generator, GeneratorState};
use std::pin::Pin;
use std::task::{Context, Poll};

struct GeneratorStream {
    generator: Pin<Box<dyn Generator<Yield = ClipEntry, Return = ()>>>,
}

impl GeneratorStream {
    fn new(generator: Box<dyn Generator<Yield = ClipEntry, Return = ()>>) -> Self {
        Self {
            generator: generator.into(),
        }
    }
}

impl Stream for GeneratorStream {
    type Item = ClipEntry;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Use the generator's resume method to drive it
        let this = self.get_mut();
        match this.generator.as_mut().resume() {
            GeneratorState::Yielded(clip) => Poll::Ready(Some(clip)),
            GeneratorState::Complete(_) => Poll::Ready(None),
        }
    }
}