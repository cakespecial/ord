use super::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Serialize, PartialOrd)]
pub struct Epoch(pub u32);

impl Epoch {
  pub const STARTING_SATS: [Sat; 33] = [
    Sat(0),
    Sat(15750000000000000),
    Sat(18375000000000000),
    Sat(19687500000000000),
    Sat(20343750000000000),
    Sat(20671875000000000),
    Sat(20835937500000000),
    Sat(20917968750000000),
    Sat(20958984375000000),
    Sat(20979492187500000),
    Sat(20989746092700000),
    Sat(20994873045300000),
    Sat(20997436521600000),
    Sat(20998718258700000),
    Sat(20999359126200000),
    Sat(20999679558900000),
    Sat(20999839774200000),
    Sat(20999919880800000),
    Sat(20999959934100000),
    Sat(20999979959700000),
    Sat(20999989972500000),
    Sat(20999994978900000),
    Sat(20999997482100000),
    Sat(20999998733700000),
    Sat(20999999359500000),
    Sat(20999999672400000),
    Sat(20999999827800000),
    Sat(20999999905500000),
    Sat(20999999943300000),
    Sat(20999999962200000),
    Sat(20999999970600000),
    Sat(20999999974800000),
    Sat(Sat::SUPPLY),
  ];
  pub const FIRST_POST_SUBSIDY: Epoch = Self(32);
  pub const FRACTAL_SUBSIDY_HALVING_INTERVAL: u32 = 2_100_000;
  pub const FRACTAL_DIFFCHANGE_INTERVAL: u32 = 20160;
  pub const FRACTAL_EPOCH_0_OFFSET: u64 = 10_500_000_000_000_000;

  pub fn subsidy(self) -> u64 {
    if self < Self::FIRST_POST_SUBSIDY {
      (25 * COIN_VALUE) >> self.0
    } else {
      0
    }
  }

  pub fn starting_sat(self) -> Sat {
    *Self::STARTING_SATS
      .get(usize::try_from(self.0).unwrap())
      .unwrap_or_else(|| Self::STARTING_SATS.last().unwrap())
  }

  pub fn starting_height(self) -> Height {
    Height(self.0 * Self::FRACTAL_SUBSIDY_HALVING_INTERVAL)
  }
}

impl PartialEq<u32> for Epoch {
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}

impl From<Sat> for Epoch {
  fn from(sat: Sat) -> Self {
    if sat < Self::STARTING_SATS[1] {
      Epoch(0)
    } else if sat < Self::STARTING_SATS[2] {
      Epoch(1)
    } else if sat < Self::STARTING_SATS[3] {
      Epoch(2)
    } else if sat < Self::STARTING_SATS[4] {
      Epoch(3)
    } else if sat < Self::STARTING_SATS[5] {
      Epoch(4)
    } else if sat < Self::STARTING_SATS[6] {
      Epoch(5)
    } else if sat < Self::STARTING_SATS[7] {
      Epoch(6)
    } else if sat < Self::STARTING_SATS[8] {
      Epoch(7)
    } else if sat < Self::STARTING_SATS[9] {
      Epoch(8)
    } else if sat < Self::STARTING_SATS[10] {
      Epoch(9)
    } else if sat < Self::STARTING_SATS[11] {
      Epoch(10)
    } else if sat < Self::STARTING_SATS[12] {
      Epoch(11)
    } else if sat < Self::STARTING_SATS[13] {
      Epoch(12)
    } else if sat < Self::STARTING_SATS[14] {
      Epoch(13)
    } else if sat < Self::STARTING_SATS[15] {
      Epoch(14)
    } else if sat < Self::STARTING_SATS[16] {
      Epoch(15)
    } else if sat < Self::STARTING_SATS[17] {
      Epoch(16)
    } else if sat < Self::STARTING_SATS[18] {
      Epoch(17)
    } else if sat < Self::STARTING_SATS[19] {
      Epoch(18)
    } else if sat < Self::STARTING_SATS[20] {
      Epoch(19)
    } else if sat < Self::STARTING_SATS[21] {
      Epoch(20)
    } else if sat < Self::STARTING_SATS[22] {
      Epoch(21)
    } else if sat < Self::STARTING_SATS[23] {
      Epoch(22)
    } else if sat < Self::STARTING_SATS[24] {
      Epoch(23)
    } else if sat < Self::STARTING_SATS[25] {
      Epoch(24)
    } else if sat < Self::STARTING_SATS[26] {
      Epoch(25)
    } else if sat < Self::STARTING_SATS[27] {
      Epoch(26)
    } else if sat < Self::STARTING_SATS[28] {
      Epoch(27)
    } else if sat < Self::STARTING_SATS[29] {
      Epoch(28)
    } else if sat < Self::STARTING_SATS[30] {
      Epoch(29)
    } else if sat < Self::STARTING_SATS[31] {
      Epoch(30)
    } else if sat < Self::STARTING_SATS[32] {
      Epoch(31)
    } else {
      Epoch(32)
    }
  }
}

impl From<Height> for Epoch {
  fn from(height: Height) -> Self {
    Self(height.0 / Self::FRACTAL_SUBSIDY_HALVING_INTERVAL)
  }
}

#[cfg(test)]
mod tests {
  use super::super::*;

  #[test]
  fn starting_sat() {
    assert_eq!(Epoch(0).starting_sat(), 0);
    assert_eq!(
      Epoch(1).starting_sat(),
      Epoch(0).subsidy() * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + Epoch::FRACTAL_EPOCH_0_OFFSET
    );
    assert_eq!(
      Epoch(2).starting_sat(),
      (Epoch(0).subsidy() + Epoch(1).subsidy()) * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + Epoch::FRACTAL_EPOCH_0_OFFSET
    );
    assert_eq!(Epoch(32).starting_sat(), Sat(Sat::SUPPLY));
    assert_eq!(Epoch(33).starting_sat(), Sat(Sat::SUPPLY));
  }

  #[test]
  fn starting_sats() {
    let mut sat = 0;

    let mut epoch_sats = Vec::new();

    for epoch in 0..33 {
      epoch_sats.push(sat);
      if epoch == 0{
        sat += u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) * Epoch(epoch).subsidy() + Epoch::FRACTAL_EPOCH_0_OFFSET
      } else {
        sat += u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) * Epoch(epoch).subsidy();
      }  
    }

    assert_eq!(Epoch::STARTING_SATS.as_slice(), epoch_sats);
    assert_eq!(Epoch::STARTING_SATS.len(), 33);
  }

  #[test]
  fn subsidy() {
    assert_eq!(Epoch(0).subsidy(), 2500000000);
    assert_eq!(Epoch(1).subsidy(), 1250000000);
    assert_eq!(Epoch(31).subsidy(), 1);
    assert_eq!(Epoch(32).subsidy(), 0);
  }

  #[test]
  fn starting_height() {
    assert_eq!(Epoch(0).starting_height(), 0);
    assert_eq!(Epoch(1).starting_height(), Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL);
    assert_eq!(Epoch(2).starting_height(), Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL * 2);
  }

  #[test]
  fn from_height() {
    assert_eq!(Epoch::from(Height(0)), 0);
    assert_eq!(Epoch::from(Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL)), 1);
    assert_eq!(Epoch::from(Height(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + 1), 1);
  }

  #[test]
  fn from_sat() {
    for (epoch, starting_sat) in Epoch::STARTING_SATS.into_iter().enumerate() {
      if epoch > 0 {
        assert_eq!(
          Epoch::from(Sat(starting_sat.n() - 1)),
          Epoch(u32::try_from(epoch).unwrap() - 1)
        );
      }
      assert_eq!(
        Epoch::from(starting_sat),
        Epoch(u32::try_from(epoch).unwrap())
      );
      assert_eq!(
        Epoch::from(starting_sat + 1),
        Epoch(u32::try_from(epoch).unwrap())
      );
    }
    assert_eq!(Epoch::from(Sat(0)), 0);
    assert_eq!(Epoch::from(Sat(1)), 0);
    assert_eq!(Epoch::from(Epoch(1).starting_sat()), 1);
    assert_eq!(Epoch::from(Epoch(1).starting_sat() + 1), 1);
    assert_eq!(Epoch::from(Sat(u64::MAX)), 32);
  }

  #[test]
  fn eq() {
    assert_eq!(Epoch(0), 0);
    assert_eq!(Epoch(100), 100);
  }

  #[test]
  fn first_post_subsidy() {
    assert_eq!(Epoch::FIRST_POST_SUBSIDY.subsidy(), 0);
    assert!(Epoch(Epoch::FIRST_POST_SUBSIDY.0 - 1).subsidy() > 0);
  }
}
