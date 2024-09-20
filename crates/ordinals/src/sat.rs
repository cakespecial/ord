use {super::*, std::num::ParseFloatError, rust_decimal::Decimal};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Sat(pub u64);

impl Sat {
  pub const LAST: Self = Self(Self::SUPPLY - 1);
  pub const SUPPLY: u64 = 20999999976900000;

  pub fn n(self) -> u64 {
    self.0
  }

  pub fn degree(self) -> Degree {
    self.into()
  }

  pub fn height(self) -> Height {
    if self.epoch() > epoch::Epoch(0) {
      self.epoch().starting_height()
        + u32::try_from(self.epoch_position() / self.epoch().subsidy()).unwrap()        
    } else {
      let position = self.epoch_position();
      if position < 50 * 100_000_000 {
        Height(0)
      } else if position < (105_000_000 + 50) * 100_000_000   {
        Height(1)
      } else {
        Height(u32::try_from((position - Epoch::FRACTAL_EPOCH_0_OFFSET) / self.epoch().subsidy()).unwrap())
      }
    }
  }

  pub fn cycle(self) -> u32 {
    Epoch::from(self).0 / CYCLE_EPOCHS
  }

  pub fn nineball(self) -> bool {
    self.n() >= 25 * COIN_VALUE * 9 + Epoch::FRACTAL_EPOCH_0_OFFSET && self.n() < 25 * COIN_VALUE * 10 + Epoch::FRACTAL_EPOCH_0_OFFSET
  }

  pub fn percentile(self) -> String {
    format!("{}%", ((Decimal::new(self.0 as i64, 0) / Decimal::new(Self::LAST.0 as i64, 0)) * Decimal::new(100, 0)).to_string())
  }

  pub fn epoch(self) -> Epoch {
    self.into()
  }

  pub fn period(self) -> u32 {
    self.height().n() / Epoch::FRACTAL_DIFFCHANGE_INTERVAL
  }

  pub fn third(self) -> u64 {
    if self.epoch() > epoch::Epoch(0) {
      self.epoch_position() % self.epoch().subsidy()
    } else {
      let position = self.epoch_position();
      if position < 50 * 100_000_000 {
        position
      } else if position < (105_000_000 + 50) * 100_000_000   {
        position - 50 * 100_000_000
      } else {
        (position - Epoch::FRACTAL_EPOCH_0_OFFSET) % self.epoch().subsidy()
      }
    }
  }

  pub fn epoch_position(self) -> u64 {
    self.0 - self.epoch().starting_sat().0
  }

  pub fn decimal(self) -> DecimalSat {
    self.into()
  }

  pub fn rarity(self) -> Rarity {
    self.into()
  }

  /// `Sat::rarity` is expensive and is called frequently when indexing.
  /// Sat::is_common only checks if self is `Rarity::Common` but is
  /// much faster.
  pub fn common(self) -> bool {
    let epoch = self.epoch();
    if self.0 > 0 && self.0 < 50 * 100_000_000 {
      true
    }else if self.0 > 50 * 100_000_000 && self.0 < (105_000_000 + 50) * 100_000_000 {
      true
    } else{
      (self.0 - epoch.starting_sat().0) % epoch.subsidy() != 0
    }
  }

  pub fn coin(self) -> bool {
    self.n() % COIN_VALUE == 0
  }

  pub fn name(self) -> String {
    let mut x = Self::SUPPLY - self.0;
    let mut name = String::new();
    while x > 0 {
      name.push(
        "abcdefghijklmnopqrstuvwxyz"
          .chars()
          .nth(((x - 1) % 26) as usize)
          .unwrap(),
      );
      x = (x - 1) / 26;
    }
    name.chars().rev().collect()
  }

  pub fn charms(self) -> u16 {
    let mut charms = 0;

    if self.nineball() {
      Charm::Nineball.set(&mut charms);
    }

    if self.coin() {
      Charm::Coin.set(&mut charms);
    }

    match self.rarity() {
      Rarity::Common => {}
      Rarity::Epic => Charm::Epic.set(&mut charms),
      Rarity::Legendary => Charm::Legendary.set(&mut charms),
      Rarity::Mythic => Charm::Mythic.set(&mut charms),
      Rarity::Rare => Charm::Rare.set(&mut charms),
      Rarity::Uncommon => Charm::Uncommon.set(&mut charms),
    }

    charms
  }

  fn from_name(s: &str) -> Result<Self, Error> {
    let mut x = 0;
    for c in s.chars() {
      match c {
        'a'..='z' => {
          x = x * 26 + c as u64 - 'a' as u64 + 1;
          if x > Self::SUPPLY {
            return Err(ErrorKind::NameRange.error(s));
          }
        }
        _ => return Err(ErrorKind::NameCharacter.error(s)),
      }
    }
    Ok(Sat(Self::SUPPLY - x))
  }

  fn from_degree(degree: &str) -> Result<Self, Error> {
    let (cycle_number, rest) = degree
      .split_once('°')
      .ok_or_else(|| ErrorKind::MissingDegree.error(degree))?;

    let cycle_number = cycle_number
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    let (epoch_offset, rest) = rest
      .split_once('′')
      .ok_or_else(|| ErrorKind::MissingMinute.error(degree))?;

    let epoch_offset = epoch_offset
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    if epoch_offset >= Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL {
      return Err(ErrorKind::EpochOffset.error(degree));
    }

    let (period_offset, rest) = rest
      .split_once('″')
      .ok_or_else(|| ErrorKind::MissingSecond.error(degree))?;

    let period_offset = period_offset
      .parse::<u32>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?;

    if period_offset >= Epoch::FRACTAL_DIFFCHANGE_INTERVAL {
      return Err(ErrorKind::PeriodOffset.error(degree));
    }

    let cycle_start_epoch = cycle_number * CYCLE_EPOCHS;

    const HALVING_INCREMENT: u32 = Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL % Epoch::FRACTAL_DIFFCHANGE_INTERVAL;

    // For valid degrees the relationship between epoch_offset and period_offset
    // will increment by 336 every halving.
    let relationship = period_offset + Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL * CYCLE_EPOCHS - epoch_offset;

    if relationship % HALVING_INCREMENT != 0 {
      return Err(ErrorKind::EpochPeriodMismatch.error(degree));
    }

    let epochs_since_cycle_start = relationship % Epoch::FRACTAL_DIFFCHANGE_INTERVAL / HALVING_INCREMENT;

    let epoch = cycle_start_epoch + epochs_since_cycle_start;

    let height = Height(epoch * Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL + epoch_offset);

    let (block_offset, rest) = match rest.split_once('‴') {
      Some((block_offset, rest)) => (
        block_offset
          .parse::<u64>()
          .map_err(|source| ErrorKind::ParseInt { source }.error(degree))?,
        rest,
      ),
      None => (0, rest),
    };

    if !rest.is_empty() {
      return Err(ErrorKind::TrailingCharacters.error(degree));
    }

    if block_offset >= height.subsidy() {
      return Err(ErrorKind::BlockOffset.error(degree));
    }

    Ok(height.starting_sat() + block_offset)
  }

  fn from_decimal(decimal: &str) -> Result<Self, Error> {
    let (height, offset) = decimal
      .split_once('.')
      .ok_or_else(|| ErrorKind::MissingPeriod.error(decimal))?;

    let height = Height(
      height
        .parse()
        .map_err(|source| ErrorKind::ParseInt { source }.error(decimal))?,
    );

    let offset = offset
      .parse::<u64>()
      .map_err(|source| ErrorKind::ParseInt { source }.error(decimal))?;

    if offset >= height.subsidy() {
      return Err(ErrorKind::BlockOffset.error(decimal));
    }

    Ok(height.starting_sat() + offset)
  }

  fn from_percentile(percentile: &str) -> Result<Self, Error> {
    if !percentile.ends_with('%') {
      return Err(ErrorKind::Percentile.error(percentile));
    }

    let percentile_string = percentile;

    let d_percentile = percentile[..percentile.len() - 1]
      .parse::<Decimal>()
      .map_err(|_source| ErrorKind::ParseDecimal.error(percentile))?;

    if d_percentile < Decimal::new(0, 0) {
      return Err(ErrorKind::Percentile.error(percentile_string));
    }

    let last = Decimal::new(Sat::LAST.n() as i64,0);

    let n = (d_percentile /  Decimal::new(100, 0) * last).round();

    if n > last {
      return Err(ErrorKind::Percentile.error(percentile_string));
    }
    let u64n = n.to_string().parse::<u64>().map_err(|source| ErrorKind::ParseInt { source }.error(percentile))?;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Ok(Sat(u64n))
  }
}

#[derive(Debug, Error)]
pub struct Error {
  input: String,
  kind: ErrorKind,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "failed to parse sat `{}`: {}", self.input, self.kind)
  }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
  IntegerRange,
  NameRange,
  NameCharacter,
  Percentile,
  BlockOffset,
  MissingPeriod,
  TrailingCharacters,
  MissingDegree,
  MissingMinute,
  MissingSecond,
  PeriodOffset,
  EpochOffset,
  EpochPeriodMismatch,
  ParseInt { source: ParseIntError },
  ParseFloat { source: ParseFloatError },
  ParseDecimal,
}

impl ErrorKind {
  fn error(self, input: &str) -> Error {
    Error {
      input: input.to_string(),
      kind: self,
    }
  }
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::IntegerRange => write!(f, "invalid integer range"),
      Self::NameRange => write!(f, "invalid name range"),
      Self::NameCharacter => write!(f, "invalid character in name"),
      Self::Percentile => write!(f, "invalid percentile"),
      Self::BlockOffset => write!(f, "invalid block offset"),
      Self::MissingPeriod => write!(f, "missing period"),
      Self::TrailingCharacters => write!(f, "trailing character"),
      Self::MissingDegree => write!(f, "missing degree symbol"),
      Self::MissingMinute => write!(f, "missing minute symbol"),
      Self::MissingSecond => write!(f, "missing second symbol"),
      Self::PeriodOffset => write!(f, "invalid period offset"),
      Self::EpochOffset => write!(f, "invalid epoch offset"),
      Self::EpochPeriodMismatch => write!(
        f,
        "relationship between epoch offset and period offset must be multiple of 3360"
      ),
      Self::ParseInt { source } => write!(f, "invalid integer: {source}"),
      Self::ParseFloat { source } => write!(f, "invalid float: {source}"),
      Self::ParseDecimal => write!(f, "invalid float to decimal"),
    }
  }
}

impl PartialEq<u64> for Sat {
  fn eq(&self, other: &u64) -> bool {
    self.0 == *other
  }
}

impl PartialOrd<u64> for Sat {
  fn partial_cmp(&self, other: &u64) -> Option<cmp::Ordering> {
    self.0.partial_cmp(other)
  }
}

impl Add<u64> for Sat {
  type Output = Self;

  fn add(self, other: u64) -> Sat {
    Sat(self.0 + other)
  }
}

impl AddAssign<u64> for Sat {
  fn add_assign(&mut self, other: u64) {
    *self = Sat(self.0 + other);
  }
}

impl FromStr for Sat {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.chars().any(|c| c.is_ascii_lowercase()) {
      Self::from_name(s)
    } else if s.contains('°') {
      Self::from_degree(s)
    } else if s.contains('%') {
      Self::from_percentile(s)
    } else if s.contains('.') {
      Self::from_decimal(s)
    } else {
      let sat = Self(
        s.parse()
          .map_err(|source| ErrorKind::ParseInt { source }.error(s))?,
      );
      if sat > Self::LAST {
        Err(ErrorKind::IntegerRange.error(s))
      } else {
        Ok(sat)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Sat(1).n(), 1);
    assert_eq!(Sat(100).n(), 100);
  }

  #[test]
  fn height() {
    assert_eq!(Sat(0).height(), 0);
    assert_eq!(Sat(1).height(), 0);
    assert_eq!(Sat(50 * 100_000_000).height(), 1);
    assert_eq!(Sat(Epoch(0).subsidy() * 2 + Epoch::FRACTAL_EPOCH_0_OFFSET).height(), 2);
    assert_eq!(
      Epoch(2).starting_sat().height(),
      Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL * 2
    );
    assert_eq!(Sat(50 * COIN_VALUE).height(), 1);
    assert_eq!(Sat(20999999976899999).height(), 67199999);
    assert_eq!(Sat(20999999976899998).height(), 67199998);
  }

  #[test]
  fn name() {
    assert_eq!(Sat(0).name(), "erssqpdkejvd");
    assert_eq!(Sat(1).name(), "erssqpdkejvc");
    assert_eq!(Sat(26).name(), "erssqpdkejud");
    assert_eq!(Sat(27).name(), "erssqpdkejuc");
    assert_eq!(parse("a").unwrap(), Sat(20999999976899999));
    assert_eq!(Sat(20999999976899999).name(), "a");
    assert_eq!(Sat(20999999976899999 - 1).name(), "b");
    assert_eq!(Sat(20999999976899999 - 25).name(), "z");
    assert_eq!(Sat(20999999976899999 - 26).name(), "aa");
  }

  #[test]
  fn number() {
    assert_eq!(Sat(2099999997689999).n(), 2099999997689999);
  }

  #[test]
  fn degree() {
    assert_eq!(Sat(0).degree().to_string(), "0°0′0″0‴");
    assert_eq!(Sat(1).degree().to_string(), "0°0′0″1‴");
    assert_eq!(
      Sat(50 * COIN_VALUE - 1).degree().to_string(),
      "0°0′0″4999999999‴"
    );
    assert_eq!(Sat(50 * COIN_VALUE).degree().to_string(), "0°1′1″0‴");
    assert_eq!(Sat(50 * COIN_VALUE + 1).degree().to_string(), "0°1′1″1‴");
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_DIFFCHANGE_INTERVAL) - 1 + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°20159′20159″2499999999‴"
    );
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_DIFFCHANGE_INTERVAL) + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°20160′0″0‴"
    );
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_DIFFCHANGE_INTERVAL) + 1 + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°20160′0″1‴"
    );
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) - 1 + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°2099999′3359″2499999999‴"
    );
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°0′3360″0‴"
    );
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + 1 + Epoch::FRACTAL_EPOCH_0_OFFSET)
        .degree()
        .to_string(),
      "0°0′3360″1‴"
    );
    assert_eq!(
      Sat(20835937500000000 - 1).degree().to_string(),
      "0°2099999′20159″78124999‴"
    );
    assert_eq!(Sat(20835937500000000).degree().to_string(), "1°0′0″0‴");
    assert_eq!(Sat(20835937500000000 + 1).degree().to_string(), "1°0′0″1‴");
  }

  #[test]
  fn invalid_degree_bugfix() {
    // Break glass in case of emergency:
    // for height in 0..(2 * CYCLE_EPOCHS * Epoch::BLOCKS) {
    //   // 1054200000000000
    //   let expected = Height(height).starting_sat();
    //   // 0°1680′0″0‴
    //   let degree = expected.degree();
    //   // 2034637500000000
    //   let actual = degree.to_string().parse::<Sat>().unwrap();
    //   assert_eq!(
    //     actual, expected,
    //     "Sat at height {height} did not round-trip from degree {degree} successfully"
    //   );
    // }
    assert_eq!(Sat(1054200000000000 + Epoch::FRACTAL_EPOCH_0_OFFSET).degree().to_string(), "0°421680′18480″0‴");
    assert_eq!(parse("0°421680′18480″0‴").unwrap(), 1054200000000000 + Epoch::FRACTAL_EPOCH_0_OFFSET);
    assert_eq!(
      Sat(1914226250000000 + Epoch::FRACTAL_EPOCH_0_OFFSET).degree().to_string(),
      "0°765690′19770″1250000000‴"
    );
    assert_eq!(parse("0°765690′19770″1250000000‴").unwrap(), 1914226250000000 + Epoch::FRACTAL_EPOCH_0_OFFSET);
  }

  #[test]
  fn period() {
    assert_eq!(Sat(0).period(), 0);
    assert_eq!(Height(Epoch::FRACTAL_DIFFCHANGE_INTERVAL).starting_sat(),Sat(10550400000000000));
    assert_eq!(Sat(10550400000000000).period(), 1);
    assert_eq!(Sat(20999999976899999).period(), 3333);
    assert_eq!(Sat(10540400000000000).period(), 0);
    assert_eq!(Sat(10550400000000000 - 1).period(), 0);
    assert_eq!(Sat(10550400000000000).period(), 1);
    assert_eq!(Sat(10550400000000000 + 1).period(), 1);
    assert_eq!(Sat(10555400000000000).period(), 1);
    assert_eq!(Sat(20999999976899999).period(), 3333);
  }

  #[test]
  fn epoch() {
    assert_eq!(Sat(0).epoch(), 0);
    assert_eq!(Sat(1).epoch(), 0);
    assert_eq!(
      Sat(25 * COIN_VALUE * u64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) + Epoch::FRACTAL_EPOCH_0_OFFSET).epoch(),
      1
    );
    assert_eq!(Sat(20999999976899999).epoch(), 31);
  }

  #[test]
  fn epoch_position() {
    assert_eq!(Epoch(0).starting_sat().epoch_position(), 0);
    assert_eq!((Epoch(0).starting_sat() + 100).epoch_position(), 100);
    assert_eq!(Epoch(1).starting_sat().epoch_position(), 0);
    assert_eq!(Epoch(2).starting_sat().epoch_position(), 0);
  }

  #[test]
  fn subsidy_position() {
    assert_eq!(Sat(0).third(), 0);
    assert_eq!(Sat(1).third(), 1);
    assert_eq!(
      Sat(Height(0).subsidy() - 1).third(),
      Height(0).subsidy() - 1
    );
    assert_eq!(Sat(Height(2).subsidy() + Epoch::FRACTAL_EPOCH_0_OFFSET + 50 * 100_000_000).third(), 0);
    assert_eq!(Sat(Height(2).subsidy() + Epoch::FRACTAL_EPOCH_0_OFFSET + 50 * 100_000_000 + 1).third(), 1);
    assert_eq!(
      Sat(Epoch(1).starting_sat().n() + Epoch(1).subsidy()).third(),
      0
    );
    assert_eq!(Sat::LAST.third(), 0);
  }

  #[test]
  fn supply() {
    let mut mined = 0;

    for height in 0.. {
      let subsidy = Height(height).subsidy();

      if subsidy == 0 {
        break;
      }

      mined += subsidy;
    }

    assert_eq!(Sat::SUPPLY, mined);
  }

  #[test]
  fn last() {
    assert_eq!(Sat::LAST, Sat::SUPPLY - 1);
  }

  #[test]
  fn eq() {
    assert_eq!(Sat(0), 0);
    assert_eq!(Sat(1), 1);
  }

  #[test]
  fn partial_ord() {
    assert!(Sat(1) > 0);
    assert!(Sat(0) < 1);
  }

  #[test]
  fn add() {
    assert_eq!(Sat(0) + 1, 1);
    assert_eq!(Sat(1) + 100, 101);
  }

  #[test]
  fn add_assign() {
    let mut sat = Sat(0);
    sat += 1;
    assert_eq!(sat, 1);
    sat += 100;
    assert_eq!(sat, 101);
  }

  fn parse(s: &str) -> Result<Sat, String> {
    s.parse::<Sat>().map_err(|e| e.to_string())
  }

  #[test]
  fn from_str_decimal() {
    assert_eq!(parse("0.0").unwrap(), 0);
    assert_eq!(parse("0.1").unwrap(), 1);
    assert_eq!(parse("1.0").unwrap(), 50 * COIN_VALUE);
    assert_eq!(parse("67199999.0").unwrap(), 20999999976899999);
    assert!(parse("0.4999999999").is_ok());
    assert!(parse("0.5000000000").is_err());
    assert!(parse("1.10500000000000000").is_err());
    assert!(parse("1.10499999999999999").is_ok());
    assert!(parse("2.2499999999").is_ok());
    assert!(parse("2.4999999999").is_err());
    assert!(parse("67200000.0").is_err());
  }

  #[test]
  fn from_str_degree() {
    assert_eq!(parse("0°0′0″0‴").unwrap(), 0);
    assert_eq!(parse("0°0′0″").unwrap(), 0);
    assert_eq!(parse("0°0′0″1‴").unwrap(), 1);
    assert_eq!(parse("0°20159′20159″0‴").unwrap(), 10550397500000000);
    assert_eq!(parse("0°20160′0″0‴").unwrap(), 10550400000000000);
    assert_eq!(parse("0°20161′1″0‴").unwrap(), 10550402500000000);
    assert_eq!(parse("0°20160′0″1‴").unwrap(), 10550400000000001);
    assert_eq!(parse("0°20161′1″1‴").unwrap(), 10550402500000001);
    assert_eq!(parse("0°2099999′3359″0‴").unwrap(), 15749997500000000);
    assert_eq!(parse("0°0′3360″0‴").unwrap(), 15750000000000000);
    assert_eq!(parse("0°0′6720″0‴").unwrap(), 18375000000000000);
    assert_eq!(parse("0°2099999′10079″0‴").unwrap(), 19687499375000000);
    assert_eq!(parse("0°0′10080″0‴").unwrap(), 19687500000000000);
    assert_eq!(parse("1°0′0″0‴").unwrap(), 20835937500000000);
    assert_eq!(parse("2°0′0″0‴").unwrap(), 20997436521600000);
    assert_eq!(parse("3°0′0″0‴").unwrap(), 20999959934100000);
    assert_eq!(parse("4°0′0″0‴").unwrap(), 20999999359500000);
    assert_eq!(parse("5°0′0″0‴").unwrap(), 20999999970600000);
    assert_eq!(parse("5°0′3360″0‴").unwrap(), 20999999974800000);
    assert_eq!(parse("5°2099999′6719″0‴").unwrap(), 20999999976899999);
    assert_eq!(Sat(20999999976899999).degree().to_string(), "5°2099999′6719″0‴" );
  }

  #[test]
  fn from_str_number() {
    assert_eq!(parse("0").unwrap(), 0);
    assert_eq!(parse("20999999976899999").unwrap(), 20999999976899999);
    assert!(parse("20999999976900000").is_err());
  }

  #[test]
  fn from_str_degree_invalid_cycle_number() {
    assert!(parse("5°0′0″0‴").is_ok());
    assert!(parse("6°0′0″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_epoch_offset() {
    assert!(parse("0°2099999′3359″0‴").is_ok());
    assert!(parse("0°2100000′3360″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_period_offset() {
    assert!(parse("0°20159′20159″0‴").is_ok());
    assert!(parse("0°20160′20160″0‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_block_offset() {
    assert!(parse("0°0′0″4999999999‴").is_ok());
    assert!(parse("0°0′0″5000000000‴").is_err());
    assert!(parse("0°2099999′3359″2499999999‴").is_ok());
    assert!(parse("0°0′3360″2499999999‴").is_err());
  }

  #[test]
  fn from_str_degree_invalid_period_block_relationship() {
    assert!(parse("0°20159′20159″0‴").is_ok());
    assert!(parse("0°20160′0″0‴").is_ok());
    assert!(parse("0°20160′1″0‴").is_err());
    assert!(parse("0°0′3360″0‴").is_ok());
  }

  #[test]
  fn from_str_degree_post_distribution() {
    assert!(parse("5°2099999′6719″0‴").is_ok());
    assert!(parse("5°0′6721″0‴").is_err());
  }

  #[test]
  fn from_str_name() {
    assert_eq!(Sat(20999999976899999).name(), "a");
    assert_eq!(parse("erssqpdkejvd").unwrap(), 0);
    assert_eq!(parse("a").unwrap(), 20999999976899999);
    assert!(parse("(").is_err());
    assert!(parse("").is_err());
    assert!(parse("erssqpdkejve").is_err());
    assert!(parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err());
  }

  #[test]
  fn cycle() {
    assert_eq!(
      Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL * CYCLE_EPOCHS % Epoch::FRACTAL_DIFFCHANGE_INTERVAL,
      0
    );

    for i in 1..CYCLE_EPOCHS {
      assert_ne!(i * Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL % Epoch::FRACTAL_DIFFCHANGE_INTERVAL, 0);
    }

    assert_eq!(
      CYCLE_EPOCHS * Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL % Epoch::FRACTAL_DIFFCHANGE_INTERVAL,
      0
    );

    assert_eq!(Sat(0).cycle(), 0);
    assert_eq!(Sat(20835937500000000 - 1).cycle(), 0);
    assert_eq!(Sat(20835937500000000).cycle(), 1);
    assert_eq!(Sat(20835937500000000 + 1).cycle(), 1);
  }

  #[test]
  fn third() {
    assert_eq!(Sat(0).third(), 0);
    assert_eq!(Sat(50 * COIN_VALUE - 1).third(), 4999999999);
    assert_eq!(Sat(50 * COIN_VALUE).third(), 0);
    assert_eq!(Sat(50 * COIN_VALUE + 1).third(), 1);
  }

  #[test]
  fn percentile() {
    assert_eq!(Sat(0).percentile(), "0%");
    assert_eq!(Sat(Sat::LAST.n() / 2).percentile(), "49.999999999999997619047616430%");
    assert_eq!(Sat::LAST.percentile(), "100%");
  }

  #[test]
  fn from_percentile() {
    "-1%".parse::<Sat>().unwrap_err();
    "101%".parse::<Sat>().unwrap_err();
  }

  #[test]
  fn percentile_round_trip() {
    #[track_caller]
    fn case(n: u64) {
      let expected = Sat(n);
      let actual = expected.percentile().parse::<Sat>().unwrap();
      assert_eq!(expected, actual);
    }

    for n in 0..1024 {
      case(n);
      case(Sat::LAST.n() / 2 + n);
      case(Sat::LAST.n() - n);
      case(Sat::LAST.n() / (n + 1));
    }
  }

  #[test]
  fn common() {
    #[track_caller]
    fn case(n: u64) {
      assert_eq!(Sat(n).common(), Sat(n).rarity() == Rarity::Common);
    }

    case(0);
    case(1);
    case(50 * COIN_VALUE - 1);
    case(50 * COIN_VALUE);
    case(50 * COIN_VALUE + 1);
    case(2067187500000000 - 1);
    case(2067187500000000);
    case(2067187500000000 + 1);
  }

  #[test]
  fn coin() {
    assert!(Sat(0).coin());
    assert!(!Sat(COIN_VALUE - 1).coin());
    assert!(Sat(COIN_VALUE).coin());
    assert!(!Sat(COIN_VALUE + 1).coin());
  }

  #[test]
  fn nineball() {
    for height in 0..10 {
      let sat = Height(height).starting_sat();
      assert_eq!(
        sat.nineball(),
        sat.height() == 9,
        "nineball: {} height: {}",
        sat.nineball(),
        sat.height()
      );
    }
  }

  #[test]
  fn error_display() {
    assert_eq!(
      Error {
        input: "foo".into(),
        kind: ErrorKind::Percentile
      }
      .to_string(),
      "failed to parse sat `foo`: invalid percentile",
    );
  }
}
