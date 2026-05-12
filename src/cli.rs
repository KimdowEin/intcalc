use std::sync::OnceLock;

use anyhow::Error;
use clap::{Parser, Subcommand};

use crate::{calc::lpr::LprCalc, lpr::LprRates};

pub static DEBUG: OnceLock<bool> = OnceLock::new();

#[derive(Parser)]
#[command(name = "intcalc")]
#[command(about = "Interest Calculator CLI")]
#[command(long_about = include_str!("../README.md"))]

pub struct Cli {
    /// 是否打印计算细则
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 更新LPR
    Update,
    /// 计算利息
    Lpr(LprCalc),
}

impl Cli {
    pub async fn run(&self) -> Result<(), Error> {
        DEBUG.set(self.debug).expect("");

        match &self.command {
            Commands::Update => {
                LprRates::fetch_lpr().await?.save_csv()?;
                eprintln!("LPR利率更新完成");
            }
            Commands::Lpr(lpr_calc) => {
                let lpr_rates = LprRates::load_csv()?;
                let interest = lpr_calc.calc(lpr_rates);
                println!("{:.2}", interest);
            }
        }

        Ok(())
    }
}
