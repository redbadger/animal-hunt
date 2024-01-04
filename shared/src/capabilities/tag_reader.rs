use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TagReaderOperation {
    WriteUrl(String),
    ReadUrl,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TagReaderOutput {
    Url(String),
    Written,
}

impl Operation for TagReaderOperation {
    type Output = TagReaderOutput;
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
        F: Fn(TagReaderOutput) -> Ev + Clone + Send + 'static,
    {
        self.context.spawn({
            let context = self.context.clone();

            async move {
                let result = context
                    .request_from_shell(TagReaderOperation::ReadUrl)
                    .await;

                context.update_app(make_event(result));
            }
        })
    }

    pub fn write_url<F>(&self, url_string: &str, make_event: F)
    where
        F: Fn(TagReaderOutput) -> Ev + Clone + Send + 'static,
    {
        self.context.spawn({
            let context = self.context.clone();
            let url = url_string.to_string();

            async move {
                context
                    .request_from_shell(TagReaderOperation::WriteUrl(url))
                    .await;

                context.update_app(make_event(TagReaderOutput::Written));
            }
        })
    }
}
