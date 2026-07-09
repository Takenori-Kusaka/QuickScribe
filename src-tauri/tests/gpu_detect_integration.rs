//! Vulkan 起動時デバイス検出の安全性検証（ADR-0027 Phase3）。
//!
//! whisper.cpp の GPU 初期化は、使えるデバイスが無い状態で呼ぶと **C++ 例外で abort** し
//! Rust では捕捉できない（実測: `Rust cannot catch foreign exceptions` / STATUS_STACK_BUFFER_OVERRUN）。
//! よって「GPUを試して失敗したらCPUへ」は不可能で、GPU を使う *前* に安全な Vulkan ローダ C API で
//! デバイス数を数える設計にした。本テストは **デバイス0（空ICD）でも gpu_backend_available() が
//! abort せず false を返す** ことを実プロセスで保証する（回帰したらここが STATUS_... で落ちる）。
//!
//! vulkan feature 時のみ意味を持つ（CPUビルドでは gpu_backend_available は常に false）。

#[cfg(all(windows, feature = "vulkan"))]
#[test]
fn no_vulkan_device_reports_unavailable_without_abort() {
    // 空ICD/ドライバファイルでローダに物理デバイスを一切見つけさせない（GPU無し機の擬似）。
    // VK_DRIVER_FILES は新しいローダの権威的オーバーライド、VK_ICD_FILENAMES は互換用。
    std::env::set_var("VK_DRIVER_FILES", "Z:\\quickscribe_nonexistent_icd.json");
    std::env::set_var("VK_ICD_FILENAMES", "Z:\\quickscribe_nonexistent_icd.json");
    // 無効インデックス指定(GGML_VK_VISIBLE_DEVICES=99)は別経路で abort するため必ず外す。
    std::env::remove_var("GGML_VK_VISIBLE_DEVICES");

    // ここで vkEnumeratePhysicalDevices が安全に0を返せば false。abort すればプロセスが落ちて失敗。
    let available = quickscribe_lib::gpu_backend_available();
    assert!(
        !available,
        "Vulkanデバイス0の環境ではGPU利用不可(false)を返すべき（whisperのGPU初期化abortを避けるため）"
    );
}
