use rand::Rng;

pub struct Roulette {
    pub numbers: Vec<i32>,
}

impl Roulette {
    pub fn new() -> Self {
        Roulette {
            numbers: (0..37).collect(),
        }
    }

    pub fn spin(&self) -> i32 {
        let mut rng = rand::thread_rng();
        let result = rng.gen_range(0..37);
        self.numbers[result]
    }

    pub fn bet(&self, bet_number: i32, bet_amount: i32) -> (bool, i32) {
        let result = self.spin();
        if result == bet_number {
            (true, bet_amount * 36)
        } else {
            (false, 0)
        }
    }
}
