use std::sync::OnceLock;

use anyhow::Error;
use clap::{Parser, Subcommand};

use crate::{calc::lpr::LprCalc, lpr::LprRates};

pub static DEBUG: OnceLock<bool> = OnceLock::new();

#[derive(Parser)]
#[command(name = "apr_calc")]
#[command(about = "APR Calculator CLI", long_about = None)]
#[command(long_about = "
本工具仅供参考，不构成法律意见。
利率数据及计算逻辑可能存在延迟或误差，实际利息以合同约定、司法机关认定或官方数据为准。
使用者应自行复核结果，作者不对任何损失承担责任。
使用即视为同意本声明。
")]

pub struct Cli {
    /// 是否打印计算细则
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 更新lpr
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
                let interest = lpr_calc
                    .to_calc_elements(lpr_rates)
                    .into_iter()
                    .fold(0., |sum, ele| sum + ele.calc());
                println!("{:.2}", interest);
            }
        }

        Ok(())
    }
}
