use std::fs;
use bevy::prelude::*;
use crate::dungeon::modular_kit::{CavernObstacle, WaterfallMarker};

/// Zero-Vision Telemetry dumper: exports camera viewpoint and scene graph in text/JSON
/// consuming <100 tokens instead of thousands of tokens per vision prompt.
pub fn dump_camera_frustum_telemetry(
    camera_q: Query<(&Transform, &Camera), With<Camera3d>>,
    obstacles_q: Query<(&Name, &Transform), With<CavernObstacle>>,
    waterfall_q: Query<&Transform, With<WaterfallMarker>>,
) {
    let Ok((cam_tf, _camera)) = camera_q.get_single() else {
        return;
    };

    let cam_pos = cam_tf.translation;
    let cam_forward = cam_tf.forward();

    let mut visible_items = Vec::new();
    let mut diagnostics = Vec::new();

    // Check waterfall alignment
    for wf_tf in waterfall_q.iter() {
        let to_obj = wf_tf.translation - cam_pos;
        let dist = to_obj.length();
        let dot = cam_forward.dot(to_obj.normalize_or_zero());
        let in_frustum = dot > 0.45;
        visible_items.push(format!(
            r#"{{"name":"Waterfall","world_pos":[{:.2},{:.2},{:.2}],"dist_m":{:.2},"in_frustum":{}}}"#,
            wf_tf.translation.x, wf_tf.translation.y, wf_tf.translation.z, dist, in_frustum
        ));
    }

    // Check all cavern obstacles (mushrooms, bench, rock terraces)
    for (name, ob_tf) in obstacles_q.iter() {
        let to_obj = ob_tf.translation - cam_pos;
        let dist = to_obj.length();
        let dot = cam_forward.dot(to_obj.normalize_or_zero());
        let in_frustum = dot > 0.35;

        let name_str = name.as_str();
        if name_str.contains("Stalactite") && ob_tf.translation.x.abs() < 2.8 {
            diagnostics.push(format!(
                "WARNING: {} is obstructing central waterfall corridor at X={:.2}",
                name_str, ob_tf.translation.x
            ));
        }

        visible_items.push(format!(
            r#"{{"name":"{}","world_pos":[{:.2},{:.2},{:.2}],"dist_m":{:.2},"in_frustum":{}}}"#,
            name_str, ob_tf.translation.x, ob_tf.translation.y, ob_tf.translation.z, dist, in_frustum
        ));
    }

    let diag_json = diagnostics
        .iter()
        .map(|d| format!("\"{}\"", d.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(",");

    let json_content = format!(
        r#"{{
  "camera": {{
    "position": [{:.2},{:.2},{:.2}],
    "forward": [{:.2},{:.2},{:.2}]
  }},
  "visible_entities_count": {},
  "entities": [{}],
  "diagnostics": [{}]
}}"#,
        cam_pos.x, cam_pos.y, cam_pos.z,
        cam_forward.x, cam_forward.y, cam_forward.z,
        visible_items.len(),
        visible_items.join(","),
        diag_json
    );

    fs::create_dir_all("debug_output").ok();
    fs::write("debug_output/camera_frustum_dump.json", json_content).ok();
    info!(">>> [ZERO-VISION TELEMETRY] Dumped camera frustum telemetry to debug_output/camera_frustum_dump.json");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_json_output() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Ensure function compiles and runs in minimal ECS environment
        app.add_systems(Update, dump_camera_frustum_telemetry);
        app.update();
    }
}
