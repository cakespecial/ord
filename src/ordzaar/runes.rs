use super::*;

// Custom Ordzaar Rune Response
// one of the reasons to create a custom response is to
// convert some of the bigint values into string
// and also to make the response consistent
// (prevent broken responses when bumping to the latest Ord version)

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneOutpoint {
  pub rune: String,
  pub amount: String,
  pub divisibility: u8,
  pub symbol: Option<char>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneDetailMint {
  pub deadline: Option<u32>,
  pub end: Option<u32>,
  pub limit: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneDetail {
  pub rune_id: RuneId,
  pub burned: String,
  pub divisibility: u8,
  pub etching: Txid,
  pub mint: Option<RuneDetailMint>,
  pub mints: String,
  pub number: String,
  pub rune: Rune,
  pub spacers: u32,
  pub supply: String,
  pub symbol: Option<char>,
  pub timestamp: u32,
}

impl RuneOutpoint {
  pub fn from_spaced_rune_pile(spaced_rune_piled: (SpacedRune, Pile)) -> Self {
    Self {
      rune: format!("{}", spaced_rune_piled.0),
      amount: spaced_rune_piled.1.amount.to_string(),
      divisibility: spaced_rune_piled.1.divisibility,
      symbol: spaced_rune_piled.1.symbol,
    }
  }
}

impl RuneDetail {
  pub fn from_rune(rune_id: RuneId, entry: RuneEntry) -> Self {
    let mut mint: Option<RuneDetailMint> = None;
    if let Some(mint_value) = entry.mint {
      mint = Some(RuneDetailMint {
        deadline: mint_value.deadline,
        end: mint_value.end,
        limit: match mint_value.limit {
          Some(value) => Some(value.to_string()),
          None => None,
        },
      })
    }

    Self {
      rune_id,
      burned: entry.burned.to_string(),
      divisibility: entry.divisibility,
      etching: entry.etching,
      mint,
      mints: entry.mints.to_string(),
      number: entry.number.to_string(),
      rune: entry.rune,
      spacers: entry.spacers,
      supply: entry.supply.to_string(),
      symbol: entry.symbol,
      timestamp: entry.timestamp,
    }
  }
}

pub fn str_coma_to_array(str_coma: &str) -> Vec<String> {
  str_coma.split(',').map(|s| s.trim().to_string()).collect()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneOutputBulkQuery {
  pub outpoints: String,
}
