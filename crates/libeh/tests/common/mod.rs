//! 集成测试共享工具：夹具加载。
#![allow(dead_code)]

/// 读取 `tests/fixtures/parser/` 下的夹具文件。
///
/// # Panics
///
/// 夹具不存在时 panic（属于测试基础设施错误）。
pub fn fixture(name: &str) -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/parser/").to_string() + name;
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to load fixture {name}: {e}"))
}
