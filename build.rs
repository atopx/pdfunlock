fn main() {
    // 在 Windows 平台上，添加链接器标志以处理重复符号
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
    }
} 