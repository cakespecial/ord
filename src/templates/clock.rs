use super::*;

#[derive(Boilerplate)]
pub(crate) struct ClockSvg {
  height: Height,
  hour: f64,
  minute: f64,
  second: f64,
}

impl ClockSvg {
  pub(crate) fn new(height: Height) -> Self {
    let min = height.min(Epoch::FIRST_POST_SUBSIDY.starting_height());

    Self {
      height,
      hour: f64::from(min.n() % Epoch::FIRST_POST_SUBSIDY.starting_height().n())
        / f64::from(Epoch::FIRST_POST_SUBSIDY.starting_height().n())
        * 360.0,
      minute: f64::from(min.n() % Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL) / f64::from(Epoch::FRACTAL_SUBSIDY_HALVING_INTERVAL)
        * 360.0,
      second: f64::from(height.period_offset()) / f64::from(Epoch::FRACTAL_DIFFCHANGE_INTERVAL) * 360.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn second() {
    pretty_assert_eq!(ClockSvg::new(Height(0)).second, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(5040)).second, 90.0);
    pretty_assert_eq!(ClockSvg::new(Height(10080)).second, 180.0);
    pretty_assert_eq!(ClockSvg::new(Height(15120)).second, 270.0);
    pretty_assert_eq!(ClockSvg::new(Height(20160)).second, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(69300000)).second, 180.0);
    pretty_assert_eq!(ClockSvg::new(Height(69305040)).second, 270.0);
  }

  #[test]
  fn minute() {
    pretty_assert_eq!(ClockSvg::new(Height(0)).minute, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(525000)).minute, 90.0);
    pretty_assert_eq!(ClockSvg::new(Height(1050000)).minute, 180.0);
    pretty_assert_eq!(ClockSvg::new(Height(1575000)).minute, 270.0);
    pretty_assert_eq!(ClockSvg::new(Height(2100000)).minute, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(69300000)).minute, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(69300010)).minute, 0.0);
  }

  #[test]
  fn hour() {
    pretty_assert_eq!(ClockSvg::new(Height(0)).hour, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(16800000)).hour, 90.0);
    pretty_assert_eq!(ClockSvg::new(Height(33600000)).hour, 180.0);
    pretty_assert_eq!(ClockSvg::new(Height(50400000)).hour, 270.0);
    pretty_assert_eq!(ClockSvg::new(Height(67200000)).hour, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(67200001)).hour, 0.0);
  }

  #[test]
  fn final_subsidy_height() {
    pretty_assert_eq!(
      ClockSvg::new(Height(67199999)).second,
      6719.0 / 20160.0 * 360.0
    );
    pretty_assert_eq!(
      ClockSvg::new(Height(67199999)).minute,
      2_099_999.0 / 2_100_000.0 * 360.0
    );
    pretty_assert_eq!(
      ClockSvg::new(Height(67199999)).hour,
      67199999.0 / 67200000.0 * 360.0
    );
  }

  #[test]
  fn first_post_subsidy_height() {
    pretty_assert_eq!(ClockSvg::new(Height(6930000)).second, 180.0);
    pretty_assert_eq!(ClockSvg::new(Height(6930000)).minute, 0.0);
    pretty_assert_eq!(ClockSvg::new(Height(6930000)).hour, 0.0);
  }

  #[test]
  fn clock_svg() {
    assert_regex_match!(
      ClockSvg::new(Height(67199999)).to_string(),
      r##"<\?xml version="1.0" encoding="UTF-8"\?>
<svg.*>.*
  <text.*>67199999</text>.*
  <line y2="-9" transform="rotate\(359.9999480519481\)"><title>Subsidy</title></line>.*
  <line y2="-13" stroke-width="0.6" transform="rotate\(359.9982857142857\)"><title>Epoch</title></line>.*
  <line y2="-16" stroke="#d00505" stroke-width="0.2" transform="rotate\(119.98214285714285\)"><title>Period</title></line>.*
  <circle r="0.7" stroke="#d00505" stroke-width="0.3"/>.*
</svg>
"##,
    );
  }
}
