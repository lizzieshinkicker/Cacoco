//! Logic for generating UMAPINFO lumps to override map-specific metadata.

/// Generates a basic UMAPINFO text lump that assigns the first bespoke
/// sky texture found in the project to MAP01.
///
/// This acts as a bridge for ID24 SKYDEFS, allowing custom-named textures
/// to be recognized as map skies without overwriting the default RSKY1.
pub fn generate_simple_umapinfo(lumps: &[crate::models::ProjectData]) -> String {
    // Search the project for a SKYDEFS lump
    let sky_lump = lumps.iter().find_map(|l| l.as_sky());

    if let Some(sky_file) = sky_lump {
        if let Some(first_sky_entry) = sky_file.data.skies.first() {
            let mut output = String::new();
            output.push_str("map MAP01\n{\n");
            output.push_str(&format!(
                "   skytexture = \"{}\"\n",
                first_sky_entry.name.to_uppercase()
            ));
            output.push_str("}\n");
            return output;
        }
    }

    String::new()
}
