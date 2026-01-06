use crate::state::PreviewState;

/// All possible items and weapons that can trigger a message.
#[derive(Debug, Clone, Copy)]
pub enum MessageItem {
    // Keys
    BlueCard,
    YellowCard,
    RedCard,
    BlueSkull,
    YellowSkull,
    RedSkull,
    // Powerups
    Invisibility,
    Invulnerability,
    Berserk,
    Map,
    Radsuit,
    Liteamp,
    // Weapons
    Chainsaw,
    Pistol,
    Shotgun,
    SuperShotgun,
    Chaingun,
    RocketLauncher,
    PlasmaGun,
    BFG,
    // Ammo
    Clip,
    Shells,
    Rocket,
    Cell,
    Backpack,
    // Vitals
    HealthBonus,
    ArmorBonus,
    GreenArmor,
    Megaarmor,
    DoomguyDeath,
}

/// High-level editor events that deserve a status message.
#[derive(Debug, Clone)]
pub enum EditorEvent {
    ProjectNew,
    ProjectLoaded(String),
    ProjectSaved(String),
    ProjectExported(String),
    TemplateApplied(String),
    Undo,
    Redo,
    Duplicate,
    Delete,
    ClipboardCopy(usize),
    ClipboardPaste(usize),
    ImportImages(usize),
    ImportFolder(usize),
    AssetsDeleted,
    Pickup(MessageItem),
    Cheat(String),
}

/// The single point of entry for pushing messages to the log.
pub fn log_event(state: &mut PreviewState, event: EditorEvent) {
    let msg = match event {
        EditorEvent::ProjectNew => "Created new empty project.".to_string(),
        EditorEvent::ProjectLoaded(path) => format!("Project Loaded: {}", path),
        EditorEvent::ProjectSaved(path) => format!("Saved: {}", path),
        EditorEvent::ProjectExported(path) => format!("Exported: {}", path),
        EditorEvent::TemplateApplied(name) => format!("Template: {}", name),
        EditorEvent::Undo => "Undo performed.".to_string(),
        EditorEvent::Redo => "Redo performed.".to_string(),
        EditorEvent::Duplicate => "Duplicate performed.".to_string(),
        EditorEvent::Delete => "Delete performed.".to_string(),
        EditorEvent::ClipboardCopy(count) => format!("Clipboard: Copied {} elements.", count),
        EditorEvent::ClipboardPaste(count) => format!("Clipboard: Pasted {} elements.", count),
        EditorEvent::ImportImages(count) => format!("Imported {} images.", count),
        EditorEvent::ImportFolder(count) => format!("Imported {} images from folder.", count),
        EditorEvent::AssetsDeleted => "Deleted assets from project.".to_string(),
        EditorEvent::Cheat(m) => m,
        EditorEvent::Pickup(item) => match item {
            MessageItem::BlueCard => "Picked up a blue keycard.",
            MessageItem::YellowCard => "Picked up a yellow keycard.",
            MessageItem::RedCard => "Picked up a red keycard.",
            MessageItem::BlueSkull => "Picked up a blue skull key.",
            MessageItem::YellowSkull => "Picked up a yellow skull key.",
            MessageItem::RedSkull => "Picked up a red skull key.",
            MessageItem::Invisibility => "Invisibility!",
            MessageItem::Invulnerability => "Invulnerability!",
            MessageItem::Berserk => "Berserk!",
            MessageItem::Map => "Computer Area Map!",
            MessageItem::Radsuit => "Radiation Shielding Suit",
            MessageItem::Liteamp => "Light Amplification Goggles",
            MessageItem::Chainsaw => "A chainsaw!  Find some meat!",
            MessageItem::Pistol => "You got the pistol!",
            MessageItem::Shotgun => "You got the shotgun!",
            MessageItem::SuperShotgun => "You got the super shotgun!",
            MessageItem::Chaingun => "You got the chaingun!",
            MessageItem::RocketLauncher => "You got the rocket launcher!",
            MessageItem::PlasmaGun => "You got the plasma rifle!",
            MessageItem::BFG => "You got the BFG9000! Oh, yes.",
            MessageItem::Clip => "Picked up a clip.",
            MessageItem::Shells => "Picked up 4 shotgun shells.",
            MessageItem::Rocket => "Picked up a rocket.",
            MessageItem::Cell => "Picked up an energy cell.",
            MessageItem::Backpack => "Picked up a backpack full of ammo!",
            MessageItem::HealthBonus => "Picked up a health bonus.",
            MessageItem::ArmorBonus => "Picked up an armor bonus.",
            MessageItem::GreenArmor => "Picked up the armor.",
            MessageItem::Megaarmor => "Picked up the Megaarmor!",
            MessageItem::DoomguyDeath => "Doomguy was killed by a cruel SBARDEF editor.",
        }
        .to_string(),
    };

    state.push_message(msg);
}
