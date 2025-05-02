use crate::analysis::MultiStringTable;

use super::{data::DataAddress, databuilder::StaticData, func::Function};

pub struct Program {
    static_data: StaticData,
    functions: Vec<Function>,
}

impl Program {
    pub fn new(static_data: StaticData, functions: Vec<Function>) -> Self {
        Self {
            static_data,
            functions,
        }
    }
}
