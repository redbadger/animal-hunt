use anyhow::bail;
use crux_core::{render::Render, App};
use crux_macros::Effect;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::capabilities::tag_reader::{TagReader, TagReaderOutput};

static HOST: &str = "animal-hunt.red-badger.com";
static ANIMALS: [(&str, &str); 10] = [
    ("crocodile", "🐊"),
    ("badger", "🦡"),
    ("sloth", "🦥"),
    ("kangaroo", "🦘"),
    ("pig", "🐖"),
    ("dolphin", "🐬"),
    ("zebra", "🦓"),
    ("giraffe", "🦒"),
    ("mouse", "🐁"),
    ("flamingo", "🦩"),
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    SetMode(Mode),
    Scan,
    ScannedUrl(TagReaderOutput),
    WriteTag(String),
    TagWritten(TagReaderOutput),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Mode {
    Configure,
    Practice,
    // Play,
}

#[derive(Debug)]
pub enum Model {
    Practice {
        animal: Option<Animal>,
        error: Option<String>,
    },
    Configure {
        error: Option<String>,
    },
}

impl Default for Model {
    fn default() -> Self {
        Model::Practice {
            animal: None,
            error: None,
        }
    }
}

#[derive(Debug)]
pub struct Animal {
    idx: usize,
}

impl Animal {
    fn from_url(url_string: &str) -> anyhow::Result<Animal> {
        let url = Url::parse(url_string)?;
        let Some(host) = url.host_str() else {
            bail!("Url missing host string");
        };

        if host != HOST {
            bail!("Unknown host {host}");
        }

        let Some(animal) = url.path().strip_prefix("/animal/") else {
            bail!("Invalid animal URL");
        };

        let Some(item) = ANIMALS.iter().enumerate().find(|(_, (a, _))| *a == animal) else {
            bail!("Animal not found: {animal}");
        };

        Ok(Animal { idx: item.0 })
    }

    fn name(&self) -> &str {
        ANIMALS[self.idx].0
    }

    fn emoji(&self) -> &str {
        ANIMALS[self.idx].1
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ViewModel {
    Practice {
        animal_emoji: String,
        error: Option<String>,
    },
    Configure {
        known_animals: Vec<(String, String)>,
        error: Option<String>,
    },
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
#[effect(app = "AnimalHunt")]
pub struct Capabilities {
    tag_reader: TagReader<Event>,
    render: Render<Event>,
}

#[derive(Default)]
pub struct AnimalHunt;

impl App for AnimalHunt {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match model {
            Model::Practice {
                animal: _,
                error: _,
            } => match event {
                Event::SetMode(Mode::Configure) => {
                    *model = Model::Configure { error: None };
                }
                Event::SetMode(Mode::Practice) => (),
                Event::Scan => {
                    caps.tag_reader.read_url(Event::ScannedUrl);
                }
                Event::ScannedUrl(TagReaderOutput::Url(url_string)) => {
                    *model = match Animal::from_url(&url_string) {
                        Ok(animal) => Model::Practice {
                            animal: Some(animal),
                            error: None,
                        },
                        Err(e) => Model::Practice {
                            animal: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
                Event::ScannedUrl(TagReaderOutput::Error(err)) => {
                    *model = Model::Practice {
                        animal: None,
                        error: Some(format!("Error reading tag: {err}")),
                    };
                }
                Event::WriteTag(_)
                | Event::TagWritten(_)
                | Event::ScannedUrl(TagReaderOutput::Written) => {
                    unreachable!("Invalid event for Practice mode")
                }
            },
            Model::Configure { error: _ } => match event {
                Event::SetMode(Mode::Practice) | Event::TagWritten(TagReaderOutput::Written) => {
                    *model = Model::Practice {
                        animal: None,
                        error: None,
                    };
                }
                Event::SetMode(Mode::Configure) => (),
                Event::WriteTag(animal) => {
                    let url = format!("https://{}/animal/{}", HOST, animal);
                    caps.tag_reader.write_url(&url, Event::TagWritten);
                }
                Event::TagWritten(TagReaderOutput::Error(err)) => {
                    *model = Model::Configure {
                        error: Some(format!("Error writing tag: {err}")),
                    };
                }
                Event::Scan | Event::ScannedUrl(_) | Event::TagWritten(TagReaderOutput::Url(_)) => {
                    panic!("Invalid event for Configure mode");
                }
            },
        }

        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        match model {
            Model::Practice { animal, error } => match animal {
                Some(animal) => ViewModel::Practice {
                    animal_emoji: animal.emoji().to_string(),
                    error: error.clone(),
                },
                None => ViewModel::Practice {
                    animal_emoji: "?".to_string(),
                    error: error.clone(),
                },
            },
            Model::Configure { error } => ViewModel::Configure {
                known_animals: ANIMALS
                    .iter()
                    .map(|(name, emoji)| (name.to_string(), emoji.to_string()))
                    .collect(),
                error: error.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_let_bind::assert_let;

    use crux_core::{assert_effect, testing::AppTester};

    use crate::{
        app::ANIMALS,
        capabilities::tag_reader::{TagReaderOperation, TagReaderOutput},
        AnimalHunt,
    };

    use super::{Effect, Event, Model, ViewModel};

    #[test]
    fn starts_with_no_animal() {
        let model = Model::Practice {
            animal: None,
            error: None,
        };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let expected = ViewModel::Practice {
            animal_emoji: "?".to_string(),
            error: None,
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected)
    }

    #[test]
    fn scans_an_animal() -> anyhow::Result<()> {
        let mut model = Model::Practice {
            animal: None,
            error: None,
        };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let update = app.update(Event::Scan, &mut model);
        let mut requests = update.into_effects().filter_map(Effect::into_tag_reader);

        let mut request = requests.next().unwrap();
        assert_let!(TagReaderOperation::ReadUrl, request.operation.clone());

        let update = app
            .resolve(
                &mut request,
                TagReaderOutput::Url(
                    "https://animal-hunt.red-badger.com/animal/badger".to_string(),
                ),
            )
            .unwrap();
        let update = app.update(update.events[0].clone(), &mut model);

        assert_effect!(update, Effect::Render(_));

        let expected = ViewModel::Practice {
            animal_emoji: "🦡".to_string(),
            error: None,
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn writes_an_animal() -> anyhow::Result<()> {
        let mut model = Model::Configure { error: None };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let expected = ViewModel::Configure {
            known_animals: ANIMALS
                .iter()
                .map(|(n, e)| (n.to_string(), e.to_string()))
                .collect(),
            error: None,
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected);

        let update = app.update(Event::WriteTag("badger".to_string()), &mut model);
        let mut requests = update.into_effects().filter_map(Effect::into_tag_reader);

        let mut request = requests.next().unwrap();
        assert_let!(TagReaderOperation::WriteUrl(url), request.operation.clone());

        assert_eq!(url, "https://animal-hunt.red-badger.com/animal/badger");

        let update = app.resolve(&mut request, TagReaderOutput::Written).unwrap();
        let update = app.update(update.events[0].clone(), &mut model);

        assert_effect!(update, Effect::Render(_));

        let expected = ViewModel::Practice {
            animal_emoji: "?".to_string(),
            error: None,
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn shows_practice_error() {
        let mut model = Model::Practice {
            animal: None,
            error: None,
        };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let update = app.update(Event::Scan, &mut model);
        let mut requests = update.into_effects().filter_map(Effect::into_tag_reader);

        let mut request = requests.next().unwrap();
        assert_let!(TagReaderOperation::ReadUrl, request.operation.clone());

        let update = app
            .resolve(
                &mut request,
                TagReaderOutput::Error("some error".to_string()),
            )
            .unwrap();
        let update = app.update(update.events[0].clone(), &mut model);

        assert_effect!(update, Effect::Render(_));

        let expected = ViewModel::Practice {
            animal_emoji: "?".to_string(),
            error: Some("Error reading tag: some error".to_string()),
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected);
    }

    #[test]
    fn shows_configuration_errors() {
        let mut model = Model::Configure { error: None };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let update = app.update(Event::WriteTag("badger".to_string()), &mut model);
        let mut requests = update.into_effects().filter_map(Effect::into_tag_reader);

        let mut request = requests.next().unwrap();
        assert_let!(TagReaderOperation::WriteUrl(_), request.operation.clone());

        let update = app
            .resolve(
                &mut request,
                TagReaderOutput::Error("some error".to_string()),
            )
            .unwrap();

        let update = app.update(update.events[0].clone(), &mut model);

        assert_effect!(update, Effect::Render(_));

        let expected = ViewModel::Configure {
            known_animals: ANIMALS
                .iter()
                .map(|(n, e)| (n.to_string(), e.to_string()))
                .collect(),
            error: Some("Error writing tag: some error".to_string()),
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected);
    }
}
