use crate::{
    comp::{Attacking, CharacterState, EnergySource, StateUpdate},
    states::utils::*,
    sys::character_behavior::{CharacterBehavior, JoinData},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use vek::Vec3;

/// Separated out to condense update portions of character state
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticData {
    /// How long until the state attacks
    pub buildup_duration: Duration,
    /// How long the state is in the swing duration
    pub swing_duration: Duration,
    /// How long until state ends
    pub recover_duration: Duration,
    /// Base damage
    pub base_damage: u32,
    /// Knockback
    pub knockback: f32,
    /// Range
    pub range: f32,
    /// Energy cost per attack
    pub energy_cost: u32,
    /// Whether spin state is infinite
    pub is_infinite: bool,
    /// Used to maintain classic axe spin physics
    pub is_helicopter: bool,
    /// Whether the state can be interrupted by other abilities
    pub is_interruptible: bool,
    /// Used for forced forward movement
    pub forward_speed: f32,
    /// Number of spins
    pub num_spins: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Struct containing data that does not change over the course of the
    /// character state
    pub static_data: StaticData,
    /// Timer for each stage
    pub timer: Duration,
    /// How many spins it can do before ending
    pub spins_remaining: u32,
    /// What section the character stage is in
    pub stage_section: StageSection,
    /// Whether the state can deal damage
    pub exhausted: bool,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        if self.static_data.is_helicopter {
            update.vel.0 = Vec3::new(data.inputs.move_dir.x, data.inputs.move_dir.y, 0.0) * 5.0;
        }

        // Allows for other states to interrupt this state
        if self.static_data.is_interruptible && !data.inputs.ability3.is_pressed() {
            handle_interrupt(data, &mut update);
            match update.character {
                CharacterState::SpinMelee(_) => {},
                _ => {
                    return update;
                },
            }
        }

        match self.stage_section {
            StageSection::Buildup => {
                if self.timer < self.static_data.buildup_duration {
                    // Build up
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: self.stage_section,
                        exhausted: self.exhausted,
                    });
                } else {
                    // Transitions to swing section of stage
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: Duration::default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: StageSection::Swing,
                        exhausted: self.exhausted,
                    });
                }
            },
            StageSection::Swing => {
                if !self.exhausted {
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: Duration::default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: self.stage_section,
                        exhausted: true,
                    });
                    // Hit attempt
                    data.updater.insert(data.entity, Attacking {
                        base_damage: self.static_data.base_damage,
                        base_heal: 0,
                        range: self.static_data.range,
                        max_angle: 180_f32.to_radians(),
                        applied: false,
                        hit_count: 0,
                        knockback: self.static_data.knockback,
                    });
                } else if self.timer < self.static_data.swing_duration {
                    if !self.static_data.is_helicopter {
                        forward_move(data, &mut update, 0.1, self.static_data.forward_speed);
                        handle_orientation(data, &mut update, 1.0);
                    }

                    // Swings
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: self.stage_section,
                        exhausted: self.exhausted,
                    });
                } else if update.energy.current() >= self.static_data.energy_cost
                    && (self.spins_remaining != 0
                        || (self.static_data.is_infinite && data.inputs.secondary.is_pressed()))
                {
                    let new_spins_remaining = if self.static_data.is_infinite {
                        self.spins_remaining
                    } else {
                        self.spins_remaining - 1
                    };
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: Duration::default(),
                        spins_remaining: new_spins_remaining,
                        stage_section: self.stage_section,
                        exhausted: false,
                    });
                    // Consumes energy if there's enough left and RMB is held down
                    update.energy.change_by(
                        -(self.static_data.energy_cost as i32),
                        EnergySource::Ability,
                    );
                } else {
                    // Transitions to recover section of stage
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: Duration::default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: StageSection::Recover,
                        exhausted: self.exhausted,
                    });
                }
            },
            StageSection::Recover => {
                if self.timer < self.static_data.recover_duration {
                    // Recover
                    update.character = CharacterState::SpinMelee(Data {
                        static_data: self.static_data,
                        timer: self
                            .timer
                            .checked_add(Duration::from_secs_f32(data.dt.0))
                            .unwrap_or_default(),
                        spins_remaining: self.spins_remaining,
                        stage_section: self.stage_section,
                        exhausted: self.exhausted,
                    });
                } else {
                    // Done
                    update.character = CharacterState::Wielding;
                    // Make sure attack component is removed
                    data.updater.remove::<Attacking>(data.entity);
                }
            },
            _ => {
                // If it somehow ends up in an incorrect stage section
                update.character = CharacterState::Wielding;
                // Make sure attack component is removed
                data.updater.remove::<Attacking>(data.entity);
            },
        }

        update
    }
}
