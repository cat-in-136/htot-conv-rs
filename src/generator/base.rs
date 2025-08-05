use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IntegrateCellsOption {
    Colspan,
    Rowspan,
    Both,
}
