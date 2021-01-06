use std::time::{SystemTime, UNIX_EPOCH};

const MS_PER_TIMER_TICK: u128 = 1000 / 60;

#[derive(Debug)]
pub struct Timer {
    end_time: u128,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            end_time: 0,
        }
    }

    pub fn get_timeout(&self) -> u8 {
        let now = get_time_millis();
        if now >= self.end_time {
            return 0;
        }

        ((self.end_time - now) / MS_PER_TIMER_TICK) as u8
    }

    pub fn set_timeout(&mut self, ticks: u8) {
        self.end_time = get_time_millis() + ((ticks as u128) * MS_PER_TIMER_TICK);
    }
}

fn get_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Invalid time")
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick() {
        let mut timer = Timer::new();
        assert_eq!(0, timer.get_timeout());

        let max_timeout = u8::MAX & 0x0F;
        timer.set_timeout(max_timeout);
        assert!(timer.get_timeout() > max_timeout - 2);
        assert!(timer.get_timeout() <= max_timeout);

        let some_timeout = 10 as u8;
        timer.set_timeout(some_timeout);
        assert!(timer.get_timeout() > some_timeout - 2);
        assert!(timer.get_timeout() <= some_timeout);

        timer.set_timeout(0);
        assert_eq!(0, timer.get_timeout());
    }
}
