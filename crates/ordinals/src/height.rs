use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Display, FromStr)]
pub struct Height(pub u32);

impl Height {
  pub fn n(self) -> u32 {
    self.0
  }

  pub fn subsidy(self) -> u64 {
    if self.0 > 1 {
      Epoch::from(self).subsidy()
    }else{
      if self.0 == 0 {
        50 * 100_000_000  
      }else {
        105_000_000 * 100_000_000
      }
    }

  }

  pub fn starting_sat(self) -> Sat {
    let epoch = Epoch::from(self);
    let epoch_starting_sat = epoch.starting_sat();
    let epoch_starting_height = epoch.starting_height();
    if epoch.0 != 0{
      epoch_starting_sat + u64::from(self.n() - epoch_starting_height.n()) * epoch.subsidy()
    }else{
      if self.n() > 1 {
        epoch_starting_sat + u64::from(self.n() - epoch_starting_height.n()) * epoch.subsidy() + Epoch::FRACTAL_EPOCH_0_OFFSET          
      }else{
        if self.n() == 0 {
          Sat(0)
        } else {
          Sat(50 * 100_000_000)
        }
      }
    }
  }

  pub fn period_offset(self) -> u32 {
    self.0 % Epoch::FRACTAL_DIFFCHANGE_INTERVAL
  }
}

impl Add<u32> for Height {
  type Output = Self;

  fn add(self, other: u32) -> Height {
    Self(self.0 + other)
  }
}

impl Sub<u32> for Height {
  type Output = Self;

  fn sub(self, other: u32) -> Height {
    Self(self.0 - other)
  }
}

impl PartialEq<u32> for Height {
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Height(0).n(), 0);
    assert_eq!(Height(1).n(), 1);
  }

  #[test]
  fn add() {
    assert_eq!(Height(0) + 1, 1);
    assert_eq!(Height(1) + 100, 101);
  }

  #[test]
  fn sub() {
    assert_eq!(Height(1) - 1, 0);
    assert_eq!(Height(100) - 50, 50);
  }

  #[test]
  fn eq() {
    assert_eq!(Height(0), 0);
    assert_eq!(Height(100), 100);
  }

  #[test]
  fn from_str() {
    assert_eq!("0".parse::<Height>().unwrap(), 0);
    assert!("foo".parse::<Height>().is_err());
  }

  #[test]
  fn subsidy() {
    assert_eq!(Height(0).subsidy(), 50 * 100_000_000);
    assert_eq!(Height(1).subsidy(), 105_000_000 * 100_000_000);
    assert_eq!(Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL - 1).subsidy(), 2500000000);
    assert_eq!(Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL).subsidy(), 1250000000);
    assert_eq!(Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL + 1).subsidy(), 1250000000);
  }

  #[test]
  fn starting_sat() {
    assert_eq!(Height(0).starting_sat(), 0);
    assert_eq!(Height(1).starting_sat(), 5000000000);
    assert_eq!(
      Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL - 1).starting_sat(),
      (u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) - 1) * 2500000000 + Epoch::FRACTAL_EPOCH_0_OFFSET
    );
    assert_eq!(
      Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL).starting_sat(),
      u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) * 2500000000 + Epoch::FRACTAL_EPOCH_0_OFFSET
    );
    assert_eq!(
      Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL + 1).starting_sat(),
      u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) * 2500000000 + Epoch::FRACTAL_EPOCH_0_OFFSET + 1250000000
    );
    assert_eq!(
      Height(u32::MAX).starting_sat(),
      *Epoch::STARTING_SATS.last().unwrap()
    );
  }

  #[test]
  fn period_offset() {
    assert_eq!(Height(0).period_offset(), 0);
    assert_eq!(Height(1).period_offset(), 1);
    assert_eq!(Height(Epoch::FRACTAL_DIFFCHANGE_INTERVAL - 1).period_offset(), 20159);
    assert_eq!(Height(Epoch::FRACTAL_DIFFCHANGE_INTERVAL).period_offset(), 0);
    assert_eq!(Height(Epoch::FRACTAL_DIFFCHANGE_INTERVAL + 1).period_offset(), 1);
  }
}
