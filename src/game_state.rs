use cgmath::InnerSpace;
use ggez::audio::SoundSource;
use ggez::graphics::Color;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{Assets, Explosion, Landscape, Missile, Tank, Vector2, G};

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    TanksThrowing,
    Aiming,
    FlyingOfMissile(Missile),
    Exploding(Explosion),
    Subsidence,
}

pub struct Round {
    pub rng: ThreadRng,
    pub width: f32,
    pub height: f32,
    pub landscape: Landscape,
    pub wind_power: f32,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    pub state: GameState,
}

impl Round {
    pub fn new(width: f32, height: f32) -> Result<Round, String> {
        let mut rng = rand::thread_rng();
        let mut landscape = Landscape::new(width as u16, height as u16)?;
        landscape.set_seed(rng.gen());
        landscape.dx = rng.gen_range(0, width as i32 / 2);
        landscape.generate();

        let tanks = vec![
            Tank::new([100., 50.], Color::from_rgb(245, 71, 32)),
            Tank::new([width - 100., 50.], Color::from_rgb(42, 219, 39)),
        ];

        let mut round = Round {
            rng,
            width,
            height,
            landscape,
            wind_power: 0.0,
            tanks,
            current_tank: 0,
            state: GameState::TanksThrowing,
        };
        round.change_wind();
        Ok(round)
    }

    #[inline]
    pub fn update_landscape_seed(&mut self) {
        self.landscape.set_seed(self.rng.gen());
    }

    #[inline]
    pub fn regenerate_landscape(&mut self) {
        self.landscape.generate();
        for tank in self.tanks.iter_mut() {
            tank.throw_down(Some(50.));
        }
        self.change_wind();
    }

    fn change_wind(&mut self) {
        self.wind_power = (self.rng.gen_range(-10.0_f32, 10.0_f32) * 10.0).round() / 10.0;
    }

    pub fn update(&mut self, assets: &mut Assets) {
        self.update_tanks();
        self.update_missile(assets);
        self.update_explosion();
        self.update_landscape();
    }

    fn update_tanks(&mut self) {
        if let GameState::TanksThrowing = self.state {
            let mut all_placed = true;
            for tank in self.tanks.iter_mut() {
                all_placed &= tank.update(&mut self.landscape).is_placed();
            }

            if all_placed {
                self.state = GameState::Aiming;
            }
        }
    }

    fn update_missile(&mut self, assets: &mut Assets) {
        if let GameState::FlyingOfMissile(ref mut missile) = self.state {
            if let Some(pos) = missile.update(&self.landscape) {
                assets.explosion_sound.play().unwrap();
                self.state = GameState::Exploding(Explosion::new(pos, 50.0));
            }
        }
    }

    fn update_explosion(&mut self) {
        if let GameState::Exploding(ref mut explosion) = self.state {
            if explosion.update(&mut self.landscape) {
                self.landscape.subsidence();
                self.state = GameState::Subsidence;
            }
        }
    }

    fn update_landscape(&mut self) {
        if let GameState::Subsidence = self.state {
            if self.landscape.update() {
                self.current_tank += 1;
                if self.current_tank >= self.tanks.len() {
                    self.current_tank = 0;
                }
                for tank in self.tanks.iter_mut() {
                    tank.throw_down(None);
                }
                self.change_wind();
                self.state = GameState::TanksThrowing;
            }
        }
    }

    /// Increment angle of gun of current tank
    pub fn inc_gun_angle(&mut self, delta: f32) {
        if let Some(tank) = self.tanks.get_mut(self.current_tank) {
            let mut angle = tank.angle + delta;
            if angle > 90.0 {
                angle = 90.0;
            } else if angle < -90.0 {
                angle = -90.0;
            }
            tank.angle = angle;
        }
    }

    #[inline]
    pub fn gun_angle(&self) -> f32 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 90.0, |tank| tank.angle)
    }

    #[inline]
    pub fn gun_power(&self) -> f32 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 0.0, |tank| tank.power)
    }

    #[inline]
    pub fn missile_speed(&self) -> f32 {
        if let GameState::FlyingOfMissile(ref missile) = self.state {
            let velocity = missile.cur_velocity();
            return velocity.magnitude();
        }
        0.0
    }

    /// Increment power of gun of current tank
    pub fn inc_gun_power(&mut self, delta: f32) {
        if let Some(tank) = self.tanks.get_mut(self.current_tank) {
            let mut power = tank.power + delta;
            if power > 100.0 {
                power = 100.0;
            } else if power < 0.0 {
                power = 0.0;
            }
            tank.power = power;
        }
    }

    pub fn shoot(&mut self, assets: &mut Assets) {
        if let GameState::Aiming = self.state {
            if let Some(tank) = self.tanks.get(self.current_tank) {
                assets.tank_fire_sound.play().unwrap();
                let acceleration = Vector2::new(self.wind_power, G);
                self.state = GameState::FlyingOfMissile(tank.shoot(acceleration));
            }
        }
    }
}
