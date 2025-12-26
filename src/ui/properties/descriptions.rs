use crate::model::{ComponentType, Element, ElementWrapper, NumberType};

pub fn get_helper_text(element: &ElementWrapper) -> &'static str {
    if element._cacoco_text.is_some() {
        return "An automated text generator that converts a string into a sequence of graphic patches based on a registered font. How convenient!";
    }

    match &element.data {
        Element::Graphic(_) => "Renders a static image from a WAD patch.",
        Element::Animation(_) => "Renders a sequence of patches with timed delays in gametics.",
        Element::Face(_) => "Renders Doomguy's face looking around for demons and better weapons.",
        Element::FaceBackground(_) => {
            "Renders the background behind the status bar face for multiplayer stuff."
        }
        Element::Canvas(_) => "A container for grouping and offsetting other elements.",
        Element::Carousel(_) => "A rotating list of elements... not quite implemented yet...",
        Element::Number(n) => match n.type_ {
            NumberType::Health => "Displays the player's current health points.",
            NumberType::Armor => "Displays the player's current armor points.",
            NumberType::Frags => "Displays the number of frags in Multiplayer.",
            NumberType::Ammo => "Displays the amount of a specific ammo type.",
            NumberType::AmmoSelected => "Displays the ammo count for the active weapon.",
            NumberType::MaxAmmo => "Displays the maximum capacity for a specific ammo type.",
            NumberType::AmmoWeapon => "Displays the ammo used by a specific weapon.",
            NumberType::MaxAmmoWeapon => "Displays the max capacity for a specific weapon's ammo.",
        },
        Element::Percent(p) => match p.type_ {
            NumberType::Health => "Displays health points followed by a '%'.",
            NumberType::Armor => "Displays armor points followed by a '%'.",
            _ => "Displays a numeric value followed by a '%'.",
        },
        Element::Component(c) => match c.type_ {
            ComponentType::Time => "Displays the time elapsed in the current level.",
            ComponentType::LevelTitle => "Displays the name of the current map.",
            ComponentType::AnnounceLevelTitle => {
                "Briefly displays the map name at level start. Ports handle this differently..."
            }
            ComponentType::StatTotals => {
                "Displays Kills, Items, and Secrets statistics. Ports handle this differently..."
            }
            ComponentType::Coordinates => "Displays player X/Y/Z coordinates.",
            ComponentType::Speedometer => "Displays player velocity... how did you access this?!",
            ComponentType::FpsCounter => "Displays the current rendering framerate. So smooth~",
            ComponentType::Message => "Displays game console log messages.",
            _ => "A special component that displays game information... How did you get here?!",
        },
    }
}
