use std::path::{Path, PathBuf};
use std::thread;

use fltk::dialog;
use fltk::{
    app,
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    prelude::*,
    window::Window,
};
use mupdf::pdf::{Encryption, document};

// PDF解密函数
fn unlock_pdf(input_file: &str, output_file: &str) -> Result<(), mupdf::Error> {
    // 加载PDF文档
    let doc = document::PdfDocument::open(input_file)?;
    let mut options = document::PdfWriteOptions::default();
    options.set_encryption(Encryption::None);
    doc.write_to_with_options(&mut std::fs::File::create(output_file)?, options)?;
    Ok(())
}

// 生成输出文件路径
fn generate_output_path(input_path: &Path) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default();
    let extension = input_path.extension().unwrap_or_default();
    let new_file_name = format!("{}_unlocked.{}", stem.to_string_lossy(), extension.to_string_lossy());
    input_path.with_file_name(new_file_name)
}

// 定义应用消息
#[derive(Clone)]
enum Message {
    SelectFile,
    Process,
    UpdateStatus(StatusType),
    Error(String),
    Success(String),
}

// 定义状态类型
#[derive(Clone, Copy)]
enum StatusType {
    FileSelected,
    Processing,
    Success,
    Error,
}

fn main() {
    // 初始化应用
    let (sender, receiver) = app::channel::<Message>();

    // 创建主窗口
    let mut window = Window::new(100, 100, 400, 170, "PDF解锁工具");

    // 创建按钮
    let mut select_btn = Button::new(10, 120, 180, 40, "选择PDF文件");
    let mut unlock_btn = Button::new(210, 120, 180, 40, "解锁PDF");

    // 创建状态框
    let mut status = Frame::new(
        10,
        10,
        380,
        100,
        "选择一个加密的PDF文件进行解锁\n备注：不支持密码破解，仅支持(复制/编辑/打印)权限解除",
    );
    status.set_frame(FrameType::EngravedBox);
    status.set_label_size(12);

    // 初始状态下禁用解锁按钮
    unlock_btn.deactivate();

    // 完成窗口设置
    window.end();
    window.show();

    // 设置按钮回调
    select_btn.emit(sender, Message::SelectFile);
    unlock_btn.emit(sender, Message::Process);

    // 保存当前选择的文件路径
    let mut selected_path: Option<PathBuf> = None;

    // 错误信息
    let mut error_message = String::new();

    // 输出路径
    let mut output_path = String::new();

    // 主事件循环
    while app::wait() {
        // 检查是否有新消息
        if let Some(msg) = receiver.recv() {
            match msg {
                Message::SelectFile => {
                    // 选择文件
                    let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
                    dialog.show();
                    dialog.set_filter("*.{pdf}");
                    dialog.set_title("选择pdf文件");
                    if dialog.filename().is_file() {
                        selected_path = Some(dialog.filename());
                        sender.send(Message::UpdateStatus(StatusType::FileSelected));
                    }
                }
                Message::Process => {
                    // 处理PDF文件
                    if let Some(path) = &selected_path {
                        // 设置处理中状态
                        sender.send(Message::UpdateStatus(StatusType::Processing));

                        // 生成输出路径
                        let out_path = generate_output_path(path);
                        let input_str = path.to_string_lossy().to_string();
                        let output_str = out_path.to_string_lossy().to_string();

                        // 在新线程中处理
                        thread::spawn(move || match unlock_pdf(&input_str, &output_str) {
                            Ok(_) => {
                                sender.send(Message::Success(output_str));
                            }
                            Err(e) => {
                                sender.send(Message::Error(e.to_string()));
                            }
                        });
                    }
                }
                Message::UpdateStatus(status_type) => {
                    // 更新状态
                    match status_type {
                        StatusType::FileSelected => {
                            if let Some(path) = &selected_path {
                                status.set_label(&format!(
                                    "已选择：{}\n点击\"解锁PDF\"按钮开始处理",
                                    path.to_string_lossy()
                                ));
                                status.set_label_color(Color::Black);
                                unlock_btn.activate();
                            }
                        }
                        StatusType::Processing => {
                            status.set_label("正在解锁中，请稍候...");
                            status.set_label_color(Color::Blue);
                            unlock_btn.deactivate();
                        }
                        StatusType::Success => {
                            status.set_label(&format!("解锁成功！\n输出文件: {}", output_path));
                            status.set_label_color(Color::from_rgb(0, 150, 0));
                            unlock_btn.deactivate();
                        }
                        StatusType::Error => {
                            status.set_label(&format!("解锁失败: {}", error_message));
                            status.set_label_color(Color::Red);
                            unlock_btn.activate();
                        }
                    }
                }
                Message::Error(e) => {
                    error_message = e;
                    sender.send(Message::UpdateStatus(StatusType::Error));
                }
                Message::Success(s) => {
                    output_path = s;
                    sender.send(Message::UpdateStatus(StatusType::Success));
                }
            }
        }
    }
}
