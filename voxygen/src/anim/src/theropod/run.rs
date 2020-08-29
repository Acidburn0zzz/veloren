use super::{super::Animation, SkeletonAttr, TheropodSkeleton};
//use std::{f32::consts::PI, ops::Mul};
use super::super::vek::*;
use std::f32::consts::PI;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Dependency = (f32, f64);
    type Skeleton = TheropodSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"theropod_run\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "theropod_run")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, _global_time): Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let wave = (anim_time as f32 * 8.0).sin();
        let wavealt = (anim_time as f32 * 8.0 + PI / 2.0).sin();
        let wave_slow = (anim_time as f32 * 6.5 + PI).sin();

        next.head.position = Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1);
        next.head.orientation = Quaternion::rotation_z(0.0);
        next.head.scale = Vec3::one();

        next.jaw.position = Vec3::new(0.0, skeleton_attr.jaw.0, skeleton_attr.jaw.1);
        next.jaw.orientation = Quaternion::rotation_z(0.0);
        next.jaw.scale = Vec3::one();

        next.neck.position = Vec3::new(0.0, skeleton_attr.neck.0, skeleton_attr.neck.1);
        next.neck.orientation = Quaternion::rotation_z(0.0);
        next.neck.scale = Vec3::one();

        next.chest_front.position = Vec3::new(
            0.0,
            skeleton_attr.chest_front.0,
            skeleton_attr.chest_front.1,
        );
        next.chest_front.orientation = Quaternion::rotation_z(0.0);
        next.chest_front.scale = Vec3::one();

        next.chest_back.position =
            Vec3::new(0.0, skeleton_attr.chest_back.0, skeleton_attr.chest_back.1);
        next.chest_back.orientation = Quaternion::rotation_z(0.0);
        next.chest_back.scale = Vec3::one();

        next.tail_front.position =
            Vec3::new(0.0, skeleton_attr.tail_front.0, skeleton_attr.tail_front.1);
        next.tail_front.orientation = Quaternion::rotation_z(0.0);
        next.tail_front.scale = Vec3::one();

        next.tail_back.position =
            Vec3::new(0.0, skeleton_attr.tail_back.0, skeleton_attr.tail_back.1);
        next.tail_back.orientation = Quaternion::rotation_z(0.0);
        next.tail_back.scale = Vec3::one();

        next.hand_l.position = Vec3::new(
            skeleton_attr.hand_l.0,
            skeleton_attr.hand_l.0,
            skeleton_attr.hand_l.1,
        );
        next.hand_l.orientation = Quaternion::rotation_z(0.0);
        next.hand_l.scale = Vec3::one();

        next.hand_r.position = Vec3::new(
            skeleton_attr.hand_r.0,
            skeleton_attr.hand_r.0,
            skeleton_attr.hand_r.1,
        );
        next.hand_r.orientation = Quaternion::rotation_z(0.0);
        next.hand_l.scale = Vec3::one();

        next.leg_l.position = Vec3::new(
            skeleton_attr.leg_l.0,
            skeleton_attr.leg_l.0,
            skeleton_attr.leg_l.1,
        );
        next.leg_l.orientation = Quaternion::rotation_z(0.0);
        next.leg_l.scale = Vec3::one();

        next.leg_r.position = Vec3::new(
            skeleton_attr.leg_r.0,
            skeleton_attr.leg_r.0,
            skeleton_attr.leg_r.1,
        );
        next.leg_r.orientation = Quaternion::rotation_z(0.0);
        next.leg_r.scale = Vec3::one();

        next.foot_l.position = Vec3::new(
            skeleton_attr.foot_l.0,
            skeleton_attr.foot_l.0,
            skeleton_attr.foot_l.1,
        );
        next.foot_l.orientation = Quaternion::rotation_z(0.0);
        next.foot_l.scale = Vec3::one();

        next.foot_r.position = Vec3::new(
            skeleton_attr.foot_r.0,
            skeleton_attr.foot_r.0,
            skeleton_attr.foot_r.1,
        );
        next.foot_r.orientation = Quaternion::rotation_z(0.0);
        next.foot_r.scale = Vec3::one();

        next
    }
}
