use crate::model::{ComponentType, Element, ElementWrapper, NumberType};

pub fn get_helper_text(element: &ElementWrapper) -> &'static str {
    if element._cacoco_text.is_some() {
        return "An automated text generator that converts a string into a sequence of graphic patches based on a registered font. How convenient!";
    }

    match &element.data {
        Element::Graphic(_) => {
            "Renders a static image from a WAD patch. Supports custom cropping and offset overrides."
        }
        Element::Animation(_) => "Renders a sequence of patches with timed delays in gametics.",
        Element::Face(_) => "Renders Doomguy's face looking around for demons and better weapons.",
        Element::FaceBackground(_) => {
            "Renders the background behind the status bar face for multiplayer stuff."
        }
        Element::Canvas(_) => "A container for grouping and offsetting other elements.",
        Element::List(_) => {
            "A smart container that arranges its children sequentially. Children ignore their internal X/Y in favor of the list's spacing."
        }
        Element::String(s) => match s.type_ {
            0 => "Renders a custom, hardcoded text string using a HUD font.",
            1 => "Displays the current level's title (from UMAPINFO).",
            2 => "Displays the current level's label (e.g., MAP01).",
            3 => "Displays the current level's author.",
            _ => "Renders a dynamic text string.",
        },
        Element::Carousel(_) => {
            "A Unity/KEX style weapon carousel. Renders in fullscreen virtual coordinates."
        }
        Element::Number(n) => match n.type_ {
            NumberType::Health => "Displays the player's current health points.",
            NumberType::Armor => "Displays the player's current armor points.",
            NumberType::Frags => "Displays the number of frags in Multiplayer.",
            NumberType::Ammo => "Displays the amount of a specific ammo type.",
            NumberType::AmmoSelected => "Displays the ammo count for the active weapon.",
            NumberType::MaxAmmo => "Displays the maximum capacity for a specific ammo type.",
            NumberType::AmmoWeapon => "Displays the ammo used by a specific weapon.",
            NumberType::MaxAmmoWeapon => "Displays the max capacity for a specific weapon's ammo.",
            NumberType::Kills => "Displays the player's current monster kill count.",
            NumberType::Items => "Displays the number of items picked up.",
            NumberType::Secrets => "Displays the number of secrets found.",
            NumberType::PowerupDuration => {
                "Displays the remaining duration for a specific powerup."
            }
            _ => "Displays a numeric value from the game state.",
        },
        Element::Percent(p) => match p.type_ {
            NumberType::Health => "Displays health points followed by a '%'.",
            NumberType::Armor => "Displays armor points followed by a '%'.",
            NumberType::KillsPercent => "Displays the percentage of monsters killed.",
            NumberType::ItemsPercent => "Displays the percentage of items collected.",
            NumberType::SecretsPercent => "Displays the percentage of secrets found.",
            _ => "Displays a numeric value followed by a '%'.",
        },
        Element::Component(c) => match c.type_ {
            ComponentType::Time => "Displays the time elapsed in the current level.",
            ComponentType::LevelTitle => "Displays the name of the current map.",
            ComponentType::AnnounceLevelTitle => {
                "Briefly displays the map name at level start. Renders in fullscreen coordinates."
            }
            ComponentType::StatTotals => {
                "Displays Kills, Items, and Secrets statistics. Renders in fullscreen coordinates."
            }
            ComponentType::Coordinates => "Displays player X/Y/Z coordinates.",
            ComponentType::FpsCounter => "Displays the current rendering framerate. So smooth~",
            ComponentType::Message => "Displays game console log messages (Classic Top-Left).",
            _ => {
                "A special component that displays game information. Renders in fullscreen coordinates."
            }
        },
    }
}
