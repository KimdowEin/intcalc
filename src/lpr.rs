use std::{env, path::PathBuf, sync::LazyLock};

use anyhow::Error;
use chrono::NaiveDate;
use csv::StringRecord;
use derive_more::{Deref, DerefMut};
use directories_next::ProjectDirs;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const LPR_URL: &str = "https://www.boc.cn/fimarkets/lilv/fd32/201310/t20131031_2591219.html";

static LPR_TR_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("table tbody tr td").unwrap());

pub fn lpr_rate_path() -> PathBuf {
    ProjectDirs::from("com", "zhyimg", "apr_calc")
        .map(|dirs| dirs.data_local_dir().join("lpr_rates.csv"))
        .unwrap_or_else(|| {
            env::temp_dir()
                .join("com.zhyimg.apr_calc")
                .join("lpr_rates.csv")
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LprRateRecord {
    /// 利率发布日期
    pub date: NaiveDate,
    /// 一年期利率，已经对%进行处理
    pub rate1y: f64,
    /// 五年期利率，已经对%进行处理
    pub rate5y: f64,
}
impl LprRateRecord {
    pub fn new(date: NaiveDate, rate1y: f64, rate5y: f64) -> Self {
        Self {
            date,
            rate1y,
            rate5y,
        }
    }
    pub fn get_rate(&self, use_5y: bool) -> f64 {
        use_5y.then_some(self.rate5y).unwrap_or(self.rate1y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct LprRates {
    rates: Vec<LprRateRecord>,
}

impl LprRates {
    pub async fn fetch_lpr() -> Result<Self, Error> {
        let html = Client::new().get(LPR_URL).send().await?.text().await?;

        let tds = Html::parse_document(&html)
            .select(&LPR_TR_SELECTOR)
            .map(|td| td.text().collect::<String>())
            .map(|td| td.trim().trim_end_matches('%').to_string())
            .collect::<Vec<_>>();

        let records = tds
            .chunks(3)
            .map(StringRecord::from)
            .map(|record| record.deserialize::<LprRateRecord>(None))
            .collect::<Result<Vec<_>, csv::Error>>()?;

        let rates = records
            .into_iter()
            .map(|mut rate| {
                rate.rate1y /= 100.0;
                rate.rate5y /= 100.0;
                rate
            })
            .collect::<Vec<_>>();

        Ok(Self { rates })
    }

    pub fn load_csv() -> Result<Self, Error> {
        let mut reader = csv::Reader::from_path(lpr_rate_path())?;
        let rates = reader
            .deserialize::<LprRateRecord>()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rates })
    }

    pub fn save_csv(&self) -> Result<(), Error> {
        let mut writer = csv::Writer::from_path(lpr_rate_path())?;

        self.rates
            .iter()
            .try_for_each(|rate| writer.serialize(rate))?;

        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_lpr() -> Result<(), Error> {
        LprRates::fetch_lpr()
            .await
            .inspect(|record| assert!(!record.is_empty()))?
            .iter()
            .for_each(|record| {
                assert!(record.date.to_string().len() == 10);
                assert!(record.rate1y >= 0.0);
                assert!(record.rate5y >= 0.0);
            });
        Ok(())
    }
}
