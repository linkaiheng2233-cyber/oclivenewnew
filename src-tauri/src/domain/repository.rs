//! 数据访问端口由内核 crate 定义，此处 re-export 以保持 `crate::domain::repository` 路径稳定。

pub use oclive_kernel_runtime::domain::repository::{
    ExpertModelsRepository, FavorabilityRepository, MemoryRepository,
};
