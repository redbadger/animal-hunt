use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

// TODO add topics

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TagReaderOperation {
    WriteUrl(String),
    ReadUrl,
}

impl Operation for TagReaderOperation {
    type Output = String;
}

#[derive(Capability)]
pub struct TagReader<Event> {
    context: CapabilityContext<TagReaderOperation, Event>,
}

impl<Ev> TagReader<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<TagReaderOperation, Ev>) -> Self {
        Self { context }
    }

    pub fn read_url<F>(&self, make_event: F)
    where
        F: Fn(String) -> Ev + Clone + Send + 'static,
    {
        self.context.spawn({
            let context = self.context.clone();

            async move {
                let url_string = context
                    .request_from_shell(TagReaderOperation::ReadUrl)
                    .await;

                context.update_app(make_event(url_string));
            }
        })
    }

    pub fn write_url(&self, url_string: &str, event: Ev)
    where
        Ev: Send,
    {
        self.context.spawn({
            let context = self.context.clone();
            let url = url_string.to_string();

            async move {
                context
                    .notify_shell(TagReaderOperation::WriteUrl(url))
                    .await;

                context.update_app(event);
            }
        })
    }
}
