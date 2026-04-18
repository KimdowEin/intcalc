use std::ops::Mul;

use chrono::{Local, NaiveDate};
use clap::Parser;
use tap::Pipe;

use crate::{
    calc::ele::CalcElement,
    lpr::{LprRateRecord, LprRates},
};

#[derive(Parser)]
pub struct LprCalc {
    principal: f64,
    start: NaiveDate,
    #[arg(default_value_t = Local::now().date_naive())]
    end: NaiveDate,
    #[arg(short, long, default_value_t = 1.0)]
    power: f64,
    #[arg(short, long, default_value_t = 365)]
    day_basis: u64,
}

impl LprCalc {
    pub fn insert_start_end_point(&self, rates: &mut LprRates) {
        rates.sort_by_key(|rate| rate.date);

        rates
            .iter()
            .find(|rate| rate.date.ge(&self.end))
            .map(|rate| LprRateRecord::new(self.end, rate.rate_1y, rate.rate_5y))
            .and_then(|rate| rates.push(rate).pipe(Some));

        rates
            .iter()
            .rfind(|rate| rate.date.le(&self.start))
            .map(|rate| LprRateRecord::new(self.start, rate.rate_1y, rate.rate_5y))
            .and_then(|rate| rates.push(rate).pipe(Some));

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
        let use_5y = self
            .end
            .signed_duration_since(self.start)
            .num_days()
            .unsigned_abs()
            .ge(&self.day_basis.mul(5));

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
                    .rate(start.get_rate(use_5y))
                    .start(start.date)
                    .end(end.date)
                    .build()
            })
            .collect::<Vec<_>>()
    }
}
