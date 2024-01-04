use crux_core::{render::Render, App};
use crux_macros::Effect;
use serde::{Deserialize, Serialize};

static ANIMALS: [(&str, &str); 2] = [("crocodile", "🐊"), ("badger", "🦡")];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Scan,
    Scanned(String),
}

#[derive(Default)]
pub struct Model {
    animal_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ViewModel {
    pub animal_emoji: String,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
#[effect(app = "AnimalHunt")]
pub struct Capabilities {
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
        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            animal_emoji: "?".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crux_core::testing::AppTester;

    use crate::AnimalHunt;

    use super::{Effect, Model, ViewModel};

    #[test]
    fn starts_with_no_animal() {
        let model = Model { animal_id: None };
        let app: AppTester<AnimalHunt, Effect> = Default::default();

        let expected = ViewModel {
            animal_emoji: "?".to_string(),
        };
        let actual = app.view(&model);

        assert_eq!(actual, expected)
    }
}
