use maud::html;

pub fn animal_template(animal: &str, emoji: &str) -> maud::Markup {
    html! {
        h1 { "You found a " (animal) "! Congrats! " }
        p."text-8xl".text-center { (emoji) }
        h2 { "What is this?"}
        p {
            "This is part of a NFC-based game called Animal Hunt.
            The goal is to find as many animals as you can and collect them in the app."
        }
        h2 { "What app?" }
        p {
            "If you'd like the app, you can sign up for the beta. For now, speak to Viktor."
        }
    }
}

pub fn layout(title: &str, content: maud::Markup) -> maud::Markup {
    html! {
        html {
            head {
                title { "Animal Hunt | " (title) }
                script src="https://cdn.tailwindcss.com?plugins=typography" {};
            }
            body {
                div."container mx-auto p-4" {
                    article."prose prose-lg prose-slate" {
                        (content)
                    }
                }
            }
        }
    }
}
