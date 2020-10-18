use crate::{
    comp::{group, Attacking, Body, CharacterState, Loadout, Ori, Pos, Scale, Stats},
    event::{EventBus, LocalEvent, ServerEvent},
    metrics::SysMetrics,
    span,
    sync::Uid,
    util::Dir,
};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
use vek::*;

pub const BLOCK_EFFICIENCY: f32 = 0.9;
pub const BLOCK_ANGLE: f32 = 180.0;

/// This system is responsible for handling accepted inputs like moving or
/// attacking
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        Read<'a, EventBus<ServerEvent>>,
        Read<'a, EventBus<LocalEvent>>,
        ReadExpect<'a, SysMetrics>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Ori>,
        ReadStorage<'a, Scale>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Loadout>,
        ReadStorage<'a, group::Group>,
        ReadStorage<'a, CharacterState>,
        WriteStorage<'a, Attacking>,
    );

    fn run(
        &mut self,
        (
            entities,
            server_bus,
            local_bus,
            sys_metrics,
            uids,
            positions,
            orientations,
            scales,
            bodies,
            stats,
            loadouts,
            groups,
            character_states,
            mut attacking_storage,
        ): Self::SystemData,
    ) {
        let start_time = std::time::Instant::now();
        span!(_guard, "run", "melee::Sys::run");
        let mut server_emitter = server_bus.emitter();
        let mut _local_emitter = local_bus.emitter();
        // Attacks
        for (entity, uid, pos, ori, scale_maybe, attack) in (
            &entities,
            &uids,
            &positions,
            &orientations,
            scales.maybe(),
            &mut attacking_storage,
        )
            .join()
        {
            if attack.applied {
                continue;
            }
            attack.applied = true;

            // Go through all other entities
            for (b, uid_b, pos_b, ori_b, scale_b_maybe, character_b, stats_b, body_b) in (
                &entities,
                &uids,
                &positions,
                &orientations,
                scales.maybe(),
                character_states.maybe(),
                &stats,
                &bodies,
            )
                .join()
            {
                // 2D versions
                let pos2 = Vec2::from(pos.0);
                let pos_b2 = Vec2::<f32>::from(pos_b.0);
                let ori2 = Vec2::from(*ori.0);

                // Scales
                let scale = scale_maybe.map_or(1.0, |s| s.0);
                let scale_b = scale_b_maybe.map_or(1.0, |s| s.0);
                let rad_b = body_b.radius() * scale_b;

                // Check if it is a hit
                if entity != b
                    && !stats_b.is_dead
                    // Spherical wedge shaped attack field
                    && pos.0.distance_squared(pos_b.0) < (rad_b + scale * attack.range).powi(2)
                    && ori2.angle_between(pos_b2 - pos2) < attack.max_angle + (rad_b / pos2.distance(pos_b2)).atan()
                {
                    // See if entities are in the same group
                    let same_group = groups
                        .get(entity)
                        .map(|group_a| Some(group_a) == groups.get(b))
                        .unwrap_or(false);

                    let damage = if !same_group && attack.damages.enemy.is_some() {
                        attack.damages.enemy.unwrap()
                    } else if same_group && attack.damages.group.is_some() {
                        attack.damages.group.unwrap()
                    } else {
                        continue;
                    };

                    let block = character_b.map(|c_b| c_b.is_block()).unwrap_or(false)
                        && ori_b.0.angle_between(pos.0 - pos_b.0) < BLOCK_ANGLE.to_radians() / 2.0;

                    let change = damage.modify_damage(block, loadouts.get(b), Some(*uid));

                    if change.amount != 0 {
                        server_emitter.emit(ServerEvent::Damage {
                            uid: *uid_b,
                            change,
                        });
                        attack.hit_count += 1;
                    }

                    if attack.knockback != 0.0 && change.amount != 0 {
                        let kb_dir = Dir::new((pos_b.0 - pos.0).try_normalized().unwrap_or(*ori.0));
                        server_emitter.emit(ServerEvent::Knockback {
                            entity: b,
                            impulse: attack.knockback
                                * *Dir::slerp(kb_dir, Dir::new(Vec3::new(0.0, 0.0, 1.0)), 0.5),
                        });
                    }
                }
            }
        }
        sys_metrics.melee_ns.store(
            start_time.elapsed().as_nanos() as i64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}