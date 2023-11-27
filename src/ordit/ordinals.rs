use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OrdinalJson {
  pub number: u64,
  pub decimal: String,
  pub degree: String,
  pub name: String,
  pub height: u64,
  pub cycle: u64,
  pub epoch: u64,
  pub period: u64,
  pub offset: u64,
  pub rarity: Rarity,
  pub output: OutPoint,
  pub start: u64,
  pub end: u64,
  pub size: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub start: u64,
  pub end: u64,
  pub size: u64,
  pub offset: u64,
  pub rarity: Rarity,
  pub name: String,
}

pub(crate) fn get_ordinals(index: &Index, outpoint: OutPoint) -> Result<Vec<OrdinalJson>> {
  match index.list(outpoint)? {
    Some(crate::index::List::Unspent(ranges)) => {
      let mut ordinals = Vec::new();
      for Output {
        output,
        start,
        end,
        size,
        offset,
        rarity,
        name,
      } in list(outpoint, ranges)
      {
        let sat = Sat(start);
        ordinals.push(OrdinalJson {
          number: sat.n(),
          decimal: sat.decimal().to_string(),
          degree: sat.degree().to_string(),
          name,
          height: sat.height().0 as u64,
          cycle: sat.cycle() as u64,
          epoch: sat.epoch().0 as u64,
          period: sat.period() as u64,
          offset,
          rarity,
          output,
          start,
          end,
          size,
        });
      }
      Ok(ordinals)
    }
    Some(crate::index::List::Spent) => Ok(Vec::new()),
    None => Ok(Vec::new()),
  }
}

fn list(outpoint: OutPoint, ranges: Vec<(u64, u64)>) -> Vec<Output> {
  let mut offset = 0;
  ranges
    .into_iter()
    .map(|(start, end)| {
      let size = end - start;
      let output = Output {
        output: outpoint,
        start,
        end,
        size,
        offset,
        name: Sat(start).name(),
        rarity: Sat(start).rarity(),
      };

      offset += size;

      output
    })
    .collect()
}
