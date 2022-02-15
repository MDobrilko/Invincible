use bevy::prelude::*;

use crate::camera::WorldCamera;

pub fn resolve_cursor_position(wnds: Res<Windows>, q_camera: &Query<&Transform, With<WorldCamera>>) -> Option<Vec2> {
    let wnd = wnds.get_primary().unwrap();

    if let Some(pos) = wnd.cursor_position() {
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let p = pos - size / 2.0;
        let camera_transform = q_camera.single();
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        
		Some(pos_wld.truncate().truncate())
    } else {
		None
	}
}