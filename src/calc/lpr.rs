use chrono::{Local, NaiveDate};
use clap::Parser;
use typed_builder::TypedBuilder;

use crate::{
    calc::ele::CalcElement,
    lpr::{LprRateRecord, LprRates},
};

#[derive(Parser, TypedBuilder)]
pub struct LprCalc {
    /// 本金
    principal: f64,
    /// 起始日期
    start: NaiveDate,
    /// 截至日期
    #[arg(default_value_t = Local::now().date_naive())]
    end: NaiveDate,
    /// 利率倍数
    #[arg(short, long, default_value_t = 1.0)]
    #[builder(default = 1.0)]
    power: f64,
    /// 年天数基数
    #[arg(short, long, default_value_t = 365)]
    #[builder(default = 365)]
    day_basis: u64,
    /// 是否采取5年期利率
    #[arg(short = 'f', long = "five", default_value_t = false)]
    use5y: bool,
}

impl LprCalc {
    pub fn insert_start_end_point(&self, rates: &mut LprRates) {
        rates.sort_by_key(|rate| rate.date);

        rates
            .iter()
            .find(|rate| rate.date.ge(&self.end))
            .map(|rate| LprRateRecord::new(self.end, rate.rate1y, rate.rate5y))
            .map(|rate| rates.push(rate));

        rates
            .iter()
            .rfind(|rate| rate.date.le(&self.start))
            .map(|rate| LprRateRecord::new(self.start, rate.rate1y, rate.rate5y))
            .map(|rate| rates.push(rate));

        rates.sort_by_key(|rate| rate.date);

        if let Some(first_rate) = rates.first()
            && first_rate.date > self.start.min(self.end)
        {
            eprintln!(
                "Warning: start {} before first LPR {}, truncating",
                self.start, first_rate.date
            )
        }
        if let Some(end_rate) = rates.last()
            && end_rate.date < self.end.max(self.start)
        {
            let end_rate = LprRateRecord::new(self.end, 0., 0.);
            rates.push(end_rate);
        }
    }

    pub fn to_calc_elements(&self, mut rates: LprRates) -> Vec<CalcElement> {
        self.insert_start_end_point(&mut rates);

        rates
            .iter()
            .filter(|rate| !rate.date.lt(&self.start))
            .filter(|rate| !rate.date.gt(&self.end))
            .collect::<Vec<_>>()
            .windows(2)
            .map(|window| (window[0], window[1]))
            .map(|(start, end)| {
                CalcElement::builder()
                    .day_basis(self.day_basis)
                    .power(self.power)
                    .principal(self.principal)
                    .rate(start.get_rate(self.use5y))
                    .start(start.date)
                    .end(end.date)
                    .build()
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use anyhow::Error;
    use chrono::NaiveDate;

    use crate::{calc::lpr::LprCalc, lpr::LprRates};

    #[tokio::test]
    pub async fn test_lpr_calc() -> Result<(), Error> {
        let principal = 2_000_000_f64;
        let start = NaiveDate::parse_from_str("2024-12-31", "%Y-%m-%d")?;
        let end = NaiveDate::parse_from_str("2026-04-16", "%Y-%m-%d")?;

        let ints = LprCalc::builder()
            .principal(principal)
            .day_basis(365)
            .end(end)
            .power(4.)
            .start(start)
            .use5y(false)
            .build()
            .to_calc_elements(LprRates::fetch_lpr().await?)
            .into_iter()
            .fold(0., |sum, ele| sum + ele.calc());

        let deviation = 312_767.13.sub(ints).abs();

        assert!(deviation <= 0.01);

        Ok(())
    }
}
