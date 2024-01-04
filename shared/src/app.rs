use anyhow::bail;
use crux_core::{render::Render, App};
use crux_macros::Effect;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::capabilities::tag_reader::TagReader;

static ANIMALS: [(&str, &str); 2] = [("crocodile", "🐊"), ("badger", "🦡")];
static HOST: &str = "animal-hunt.red-badger.com";
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Scan,
    ScannedUrl(String),
}

#[derive(Default)]
pub struct Model {
    animal: Option<Animal>,
}

struct Animal {
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
pub struct ViewModel {
    pub animal_emoji: String,
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
        match event {
            Event::Scan => caps.tag_reader.read_url(Event::ScannedUrl),
            Event::ScannedUrl(url_string) => match Animal::from_url(&url_string) {
                Ok(animal) => model.animal = Some(animal),
                Err(e) => panic!("Invalid url {e}"), // TODO error handling
            },
        }

        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            animal_emoji: model.animal.as_ref().map_or("?", |a| a.emoji()).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_let_bind::assert_let;

    use crux_core::{assert_effect, testing::AppTester};

    use crate::{capabilities::tag_reader::TagReaderOperation, AnimalHunt};

    use super::{Effect, Event, Model, ViewModel};

    #[test]
    fn starts_with_no_animal() {
        let model = Model { animal: None };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let expected = ViewModel {
            animal_emoji: "?".to_string(),
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected)
    }

    #[test]
    fn scans_an_animal() {
        let mut model = Model { animal: None };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let update = app.update(Event::Scan, &mut model);
        let mut requests = update.into_effects().filter_map(Effect::into_tag_reader);

        let mut request = requests.next().unwrap();
        assert_let!(TagReaderOperation::ReadUrl, request.operation.clone());

        let update = app
            .resolve(
                &mut request,
                "https://animal-hunt.red-badger.com/animal/badger".to_string(),
            )
            .unwrap();
        let update = app.update(update.events[0].clone(), &mut model);

        println!("Update {:?}", update);
        assert_effect!(update, Effect::Render(_));

        let expected = ViewModel {
            animal_emoji: "🦡".to_string(),
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected)
    }
}
