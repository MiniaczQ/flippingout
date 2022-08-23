use bevy::prelude::Quat;

pub fn rot_z(q: Quat) -> f32 {
    (2.0 * (q.x * q.y + q.w * q.z)).atan2(q.w * q.w + q.x * q.x - q.y * q.y - q.z * q.z)
}
