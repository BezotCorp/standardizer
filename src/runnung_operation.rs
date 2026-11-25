use crate::tools::ToolOperation;
use std::sync::mpsc::Receiver;

pub(crate) struct RunningOperation {
    operation: ToolOperation,
    receiver: Receiver<Result<String, String>>,
}

impl RunningOperation {
    pub(crate) fn new(
        operation: ToolOperation,
        receiver: Receiver<Result<String, String>>,
    ) -> Self {
        Self {
            operation,
            receiver,
        }
    }

    pub(crate) fn operation(&self) -> &ToolOperation {
        &self.operation
    }

    pub(crate) fn receiver(&self) -> &Receiver<Result<String, String>> {
        &self.receiver
    }

    pub(crate) fn label(&self) -> &'static str {
        match self.operation {
            ToolOperation::Install => "Installing Biome…",
            ToolOperation::Reinstall => "Reinstalling Biome…",
            ToolOperation::Uninstall => "Uninstalling Biome…",
        }
    }
}
