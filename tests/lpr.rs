#![feature(bool_to_result)]

use std::path::PathBuf;

use anyhow::Error;
use chrono::NaiveDate;
use csv::Reader;
use intcalc::{calc::lpr::LprCalc, lpr::LprRates};
use rayon::prelude::*;

#[test]
fn lpr_calc() -> anyhow::Result<()> {
    let rates = LprRates::load_csv()?;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(manifest_dir).join("tests/ans/lpr_ans.csv");
    path.exists().ok_or(Error::msg("测试数据文件不存在"))?;

    let mut reader = Reader::from_path(&path)?;
    let records = reader.records().collect::<Result<Vec<_>, _>>()?;

    records.par_iter().try_for_each(|record| {
        let principal = record[0].parse::<f64>()?;
        let start = NaiveDate::parse_from_str(&record[1], "%Y-%m-%d")?;
        let end = NaiveDate::parse_from_str(&record[2], "%Y-%m-%d")?;
        let power = record[3].parse::<f64>()?;
        let day_basis = record[4].parse::<u64>()?;
        let use5y = record[5].parse::<bool>()?;
        let expected = record[6].parse::<f64>()?;

        let result = LprCalc::builder()
            .principal(principal)
            .start(start)
            .end(end)
            .power(power)
            .day_basis(day_basis)
            .use5y(use5y)
            .build()
            .to_calc_elements(rates.clone())
            .into_iter()
            .fold(0.0, |sum, ele| sum + ele.calc());

        let deviation = (expected - result).abs();
        assert!(
            deviation <= 0.02,
            "计算结果与预期不符: principal={}, start={}, end={}, power={}, day_basis={}, use5y={}, expected={:.2}, got={:.2}, deviation={:.4}",
            principal, start, end, power, day_basis, use5y, expected, result, deviation
        );

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}

#[cfg(test)]
mod generate {
    use chrono::{Duration, Local, NaiveDate};
    use csv::Writer;
    use intcalc::{calc::lpr::LprCalc, lpr::LprRates};
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use rayon::prelude::*;
    use std::path::PathBuf;

    const COUNT: usize = 100_000;

    #[tokio::test]
    #[ignore = "生成一次即可"]
    pub async fn gen_ans_csv() -> anyhow::Result<()> {
        let rates = LprRates::fetch_lpr().await?;

        let today = Local::now().date_naive();
        let lpr_start = NaiveDate::from_ymd_opt(2019, 8, 20).unwrap();
        let max_days = today.signed_duration_since(lpr_start).num_days();
        let path = PathBuf::from("tests/ans/lpr_ans.csv");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = Writer::from_path(path)?;
        writer.write_record(&[
            "principal",
            "start",
            "end",
            "power",
            "day_basis",
            "use5y",
            "ans",
        ])?;

        let records = (0..COUNT)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(i as u64);
                let principal = rng.gen_range(1_000.0..100_000_000.0);
                let start_offset = rng.gen_range(0..max_days);
                let start = lpr_start + Duration::days(start_offset);
                let remaining_days = today.signed_duration_since(start).num_days().max(1);
                let end_offset = rng.gen_range(1..=remaining_days);
                let end = start + Duration::days(end_offset);
                let power = (rng.gen_range(10..=50) as f64) / 10.0;
                let day_basis = if rng.gen_bool(0.5) { 360 } else { 365 };
                let use5y = rng.gen_bool(0.5);

                let result = LprCalc::builder()
                    .principal(principal)
                    .start(start)
                    .end(end)
                    .power(power)
                    .day_basis(day_basis)
                    .use5y(use5y)
                    .build()
                    .calc(rates.clone());
                [
                    format!("{:.2}", principal),
                    start.to_string(),
                    end.to_string(),
                    format!("{:.1}", power),
                    day_basis.to_string(),
                    use5y.to_string(),
                    format!("{:.2}", result),
                ]
            })
            .collect::<Vec<[String; 7]>>();

        for record in records {
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }
}
