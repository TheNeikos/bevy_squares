use bevy::prelude::*;

#[derive(Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_move_to.system());
        app.add_system(update_scale_to.system());
    }
}

#[derive(Default)]
pub struct ScaleTo {
    pub elapsed_time: f32,
    pub duration: f32,
    pub looping: bool,
    pub start_scale: Scale,
    pub end_scale: Scale,
    pub ease: Easing,
}

pub fn update_scale_to(
    mut commands: Commands,
    time: Res<Time>,
    mut move_to_query: Query<(Entity, &mut ScaleTo, &mut Scale)>,
) {
    for (entity, mut move_to, mut scale) in &mut move_to_query.iter() {
        move_to.elapsed_time += time.delta_seconds;

        let done = (move_to.elapsed_time / move_to.duration).min(1.0);

        let ease: &dyn Fn(f32) -> f32 = move_to.ease.get_ease_fn();

        scale.0 =
            move_to.start_scale.0 + (move_to.end_scale.0 - move_to.start_scale.0) * ease(done);

        if move_to.elapsed_time > move_to.duration {
            if move_to.looping {
                move_to.elapsed_time %= move_to.duration;
            } else {
                *scale = move_to.end_scale;
                commands.remove_one::<ScaleTo>(entity);
            }
        }
    }
}

#[derive(Default)]
pub struct MoveTo {
    pub elapsed_time: f32,
    pub duration: f32,
    pub loop_count: u32,
    pub start_position: Translation,
    pub end_position: Translation,
    pub ease: Easing,
    pub bounce: bool,
}

pub fn update_move_to(
    mut commands: Commands,
    time: Res<Time>,
    mut move_to_query: Query<(Entity, &mut MoveTo, &mut Translation)>,
) {
    for (entity, mut move_to, mut translation) in &mut move_to_query.iter() {
        move_to.elapsed_time += time.delta_seconds;

        let done = (move_to.elapsed_time / move_to.duration).min(1.0);

        let ease: &dyn Fn(f32) -> f32 = move_to.ease.get_ease_fn();

        translation.0 = move_to.start_position.0
            + (move_to.end_position.0 - move_to.start_position.0) * ease(done);

        if move_to.elapsed_time > move_to.duration {
            if move_to.loop_count > 0 {
                move_to.loop_count -= 1;
                move_to.elapsed_time %= move_to.duration;
            } else {
                *translation = move_to.end_position;
                commands.remove_one::<MoveTo>(entity);
            }

            if move_to.bounce {
                let end = move_to.end_position;
                move_to.end_position = move_to.start_position;
                move_to.start_position = end;
            }
        }
    }
}

#[allow(unused)]
pub enum Easing {
    EaseInCirc,
    EaseInOutCirc,
    EaseOutBack,
    EaseOutBounce,
}

impl Easing {
    fn get_ease_fn(&self) -> &dyn Fn(f32) -> f32 {
        match self {
            Easing::EaseInCirc => &ease_in_circ,
            Easing::EaseInOutCirc => &ease_in_out_circ,
            Easing::EaseOutBack => &ease_out_back,
            Easing::EaseOutBounce => &ease_out_bounce,
        }
    }
}

impl Default for Easing {
    fn default() -> Easing {
        Easing::EaseOutBack
    }
}

fn ease_out_bounce(mut x: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if x < 1. / D1 {
        N1 * x * x
    } else if x < 2. / D1 {
        x -= 1.5 / D1;
        N1 * x * x + 0.75
    } else if x < 2.5 / D1 {
        x -= 2.25 / D1;
        N1 * x * x + 0.9375
    } else {
        x -= 2.625 / D1;
        N1 * x * x + 0.984375
    }
}

fn ease_in_circ(x: f32) -> f32 {
    1. - (1. - x.powf(2.)).sqrt()
}

fn ease_in_out_circ(x: f32) -> f32 {
    if x < 0.5 {
        (1. - (1. - (2. * x).powf(2.)).sqrt()) / 2.0
    } else {
        ((1. - (-2. * x + 2.).powf(2.).sqrt()) + 1.) / 2.0
    }
}

fn ease_out_back(x: f32) -> f32 {
    const C1: f32 = 1.70518;
    const C3: f32 = C1 + 1.0;

    1. + C3 * (x - 1.0).powf(3.0) + C1 * (x - 1.).powf(2.)
}
