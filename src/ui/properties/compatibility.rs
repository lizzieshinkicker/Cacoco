use crate::models::sbardef::{Element, ElementWrapper, ExportTarget};

/// Evaluates if a given HUD element is compatible with the specified export target.
pub fn is_compatible(element: &ElementWrapper, target: ExportTarget) -> bool {
    if target == ExportTarget::Extended {
        return true;
    }

    if matches!(
        element.data,
        Element::List(_) | Element::String(_) | Element::Component(_) | Element::Carousel(_)
    ) {
        return false;
    }

    let common = element.get_common();

    if common.translucency {
        return false;
    }

    match &element.data {
        Element::Graphic(g) if g.crop.is_some() => return false,
        Element::Face(f) if f.crop.is_some() => return false,
        Element::FaceBackground(f) if f.crop.is_some() => return false,
        _ => {}
    }
    match &element.data {
        Element::Number(n) | Element::Percent(n) => {
            if (n.type_ as u8) > 7 {
                return false;
            }
        }
        _ => {}
    }

    for cond in &common.conditions {
        if (cond.condition as u8) > 18 {
            return false;
        }
    }

    true
}
