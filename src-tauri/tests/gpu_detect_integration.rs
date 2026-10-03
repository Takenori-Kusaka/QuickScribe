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

#[cfg(all(windows, feature = "vulkan"))]
#[test]
fn whisper_init_with_no_vulkan_device_does_not_abort() {
    std::env::set_var("VK_DRIVER_FILES", "Z:\\quickscribe_nonexistent_icd.json");
    std::env::set_var("VK_ICD_FILENAMES", "Z:\\quickscribe_nonexistent_icd.json");
    std::env::remove_var("GGML_VK_VISIBLE_DEVICES");

    let mut params = whisper_rs::WhisperContextParameters::default();
    params.use_gpu(false);
    // Passing a dummy buffer triggers whisper_init_with_params_no_state, which calls ggml_backend_dev_count()
    let res = whisper_rs::WhisperContext::new_from_buffer_with_params(&[0u8; 32], params);
    // It should safely fail with Err (model load error) rather than aborting / fast-failing the process!
    assert!(res.is_err());
}

#[cfg(all(windows, feature = "vulkan"))]
#[test]
fn gpu_can_be_disabled_via_env_var() {
    std::env::set_var("QUICKSCRIBE_DISABLE_GPU", "1");
    assert!(quickscribe_lib::is_gpu_disabled_by_cli_or_env());
    assert!(!quickscribe_lib::gpu_backend_available());
    std::env::remove_var("QUICKSCRIBE_DISABLE_GPU");

    std::env::set_var("QS_DISABLE_GPU", "1");
    assert!(quickscribe_lib::is_gpu_disabled_by_cli_or_env());
    assert!(!quickscribe_lib::gpu_backend_available());
    std::env::remove_var("QS_DISABLE_GPU");
}
