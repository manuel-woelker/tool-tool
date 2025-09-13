pub type ToolToolError = anyhow::Error;

pub use anyhow::anyhow as err;
pub use anyhow::{Context, bail};
use std::fmt::{Debug, Display, Formatter};

pub type ToolToolResult<T> = Result<T, ToolToolError>;

pub struct MietteReportError {
    report: miette::Report,
}

impl MietteReportError {
    pub fn report(&self) -> &miette::Report {
        &self.report
    }
}

impl From<miette::Report> for MietteReportError {
    fn from(report: miette::Report) -> Self {
        Self { report }
    }
}

impl Debug for MietteReportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.report, f)
    }
}

impl Display for MietteReportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.report, f)
    }
}

impl std::error::Error for MietteReportError {}
