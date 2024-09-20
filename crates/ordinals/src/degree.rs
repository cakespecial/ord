use super::*;

#[derive(PartialEq, Debug)]
pub struct Degree {
  pub hour: u32,
  pub minute: u32,
  pub second: u32,
  pub third: u64,
}

impl Display for Degree {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}°{}′{}″{}‴",
      self.hour, self.minute, self.second, self.third
    )
  }
}

impl From<Sat> for Degree {
  fn from(sat: Sat) -> Self {
    let height = sat.height().n();
    Degree {
      hour: height / (CYCLE_EPOCHS * Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL),
      minute: height % Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL,
      second: height % Epoch::FRACTAL_DIFFCHANGE_INTERVAL,
      third: sat.third(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn case(sat: u64, hour: u32, minute: u32, second: u32, third: u64) {
    assert_eq!(
      Degree::from(Sat(sat)),
      Degree {
        hour,
        minute,
        second,
        third,
      }
    );
  }

  #[test]
  fn from() {
    case(0, 0, 0, 0, 0);
    case(1, 0, 0, 0, 1);
    case(5_000_000_000, 0, 1, 1, 0);
    case(
      2_500_000_000 * u64::from(Epoch::FRACTAL_DIFFCHANGE_INTERVAL) + u64::from(Epoch::FRACTAL_EPOCH_0_OFFSET),
      0,
      Epoch::FRACTAL_DIFFCHANGE_INTERVAL,
      0,
      0,
    );
    case(
      2_500_000_000 * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + u64::from(Epoch::FRACTAL_EPOCH_0_OFFSET),
      0,
      0,
      3360,
      0,
    );
    case(
      (2_500_000_000 + 1_250_000_000 + 625_000_000 + 312_500_000 + 156_250_000 + 78_125_000)
        * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + u64::from(Epoch::FRACTAL_EPOCH_0_OFFSET) ,
      1,
      0,
      0,
      0,
    );
  }
}
