use rand::distributions::Distribution;
use rand::Rng;

pub trait Distrib {
    fn gen(&self) -> f32;
}

pub struct Uniform {
    min: f32,
    max: f32,
}

impl Distrib for Uniform {
    fn gen(&self) -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.min..self.max)
    }
}

impl Uniform {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn gen_n<const N: usize>(min: f32, max: f32) -> [f32; N] {
        let u_between = rand::distributions::Uniform::from(min..max);
        let mut rng = rand::thread_rng();

        let mut samples: [f32; N] = [0.0; N];
        for i in 0..N {
            samples[i] = u_between.sample(&mut rng);
        }

        samples
    }

    pub fn gen(min: f32, max: f32) -> f32 {
        <Self as Distrib>::gen(&Self { min, max })
    }
}

pub struct Gaussian {
    mean: f32,
    std: f32,
}

impl Distrib for Gaussian {
    fn gen(&self) -> f32 {
        self.mean + Self::marsaglia_polar_gen() * self.std
    }
}

impl Gaussian {
    pub fn new(mean: f32, std: f32) -> Self {
        Self { mean, std }
    }

    fn marsaglia_polar_gen() -> f32 {
        let (mut v1, mut v2, mut s): (f32, f32, f32);

        loop {
            v1 = 2.0 * Uniform::gen(0.0, 1.0) - 1.0;
            v2 = 2.0 * Uniform::gen(0.0, 1.0) - 1.0;
            s = v1 * v1 + v2 * v2;

            if !(s >= 1.0 || s == 0.0) {
                break;
            }
        }

        s = ((-2.0 * s.ln()) / s).sqrt();

        v1 * s
    }
}

pub struct Exponential {
    mean: f32,
}

impl Distrib for Exponential {
    fn gen(&self) -> f32 {
        let lambda = 1.0 / self.mean;
        (1.0 - Uniform::gen(0.0, 1.0)).ln() / (-lambda)
    }
}

impl Exponential {
    pub fn new(mean: f32) -> Self {
        Self { mean }
    }
}
