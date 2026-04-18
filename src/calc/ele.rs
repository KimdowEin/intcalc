use std::ops::Mul;

use chrono::NaiveDate;
use typed_builder::TypedBuilder;

use crate::cli::DEBUG;

// 计算原子
#[derive(Debug, Default, TypedBuilder)]
pub struct CalcElement {
    /// 开始日期
    pub start: NaiveDate,
    /// 截至日期
    pub end: NaiveDate,
    /// 本金
    pub principal: f64,
    /// 基本利率
    pub rate: f64,
    /// 利率系数
    #[builder(default = 1.0)]
    pub power: f64,
    /// 年系数
    #[builder(default = 365)]
    pub day_basis: u64,
}

impl CalcElement {
    pub fn calc(&self) -> f64 {
        let duration = self
            .end
            .signed_duration_since(self.start)
            .num_days()
            .unsigned_abs();

        let ints = self
            .principal
            .mul(self.rate * self.power)
            .mul(duration as f64 / self.day_basis as f64);

        if DEBUG.get().unwrap_or(&false).to_owned() {
            eprintln!(
                "{}, {}, {}, {}, {:.2}",
                self.start, self.end, duration, self.rate, ints
            );
        }

        ints
    }
}
