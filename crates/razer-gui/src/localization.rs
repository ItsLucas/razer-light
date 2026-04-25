use slint::SharedString;

pub const APP_NAME: &str = "RHelper";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn from_locale(locale: &str) -> Self {
        match locale {
            "zh" | "zh-CN" | "zh_CN" => Self::Chinese,
            _ => Self::English,
        }
    }
}

pub fn translate(locale: &str, key: &str) -> String {
    let value = match Language::from_locale(locale) {
        Language::English => english(key),
        Language::Chinese => chinese(key),
    };

    value.unwrap_or(key).to_string()
}

pub fn tr(locale: &str, key: &str) -> SharedString {
    translate(locale, key).into()
}

fn english(key: &str) -> Option<&'static str> {
    match key {
        "app.name" => Some(APP_NAME),
        "action.apply" => Some("Apply"),
        "action.connect" => Some("Connect"),
        "action.refresh" => Some("Refresh"),
        "action.exit" => Some("Exit"),
        "tab.performance" => Some("Performance"),
        "tab.lighting" => Some("Lighting"),
        "tab.power" => Some("Power"),
        "section.mode" => Some("Mode"),
        "section.fans" => Some("Fans"),
        "section.temperatures" => Some("Temperatures"),
        "section.boost" => Some("CPU / GPU Boost"),
        "section.logo" => Some("Logo"),
        "section.battery" => Some("Battery"),
        "value.monitoring" => Some("Monitoring"),
        "value.keyboard" => Some("Keyboard"),
        "value.charge_limit" => Some("Charge limit"),
        "value.custom_required" => Some("Custom required"),
        "detail.quiet" => Some("Quiet"),
        "detail.daily" => Some("Daily"),
        "detail.fast" => Some("Fast"),
        "detail.tuned" => Some("Tuned"),
        "detail.saver" => Some("Saver"),
        "detail.boost" => Some("Boost"),
        "detail.curve" => Some("Curve"),
        "detail.rpm" => Some("RPM"),
        "metric.cpu_fan" => Some("CPU Fan"),
        "metric.gpu_fan" => Some("GPU Fan"),
        "metric.battery_limit" => Some("Battery Charge Limit"),
        "metric.keyboard_brightness" => Some("Keyboard Brightness"),
        "status.connecting" => Some("Connecting to Razer device..."),
        "status.disconnected" => Some("Disconnected"),
        "status.ready" => Some("Ready"),
        "status.no_device" => Some("No device"),
        "error.no_device_connected" => Some("No Razer device is connected."),
        "error.state_read_failed" => Some("State read failed"),
        "label.unsupported" => Some("Unsupported"),
        "label.unavailable" => Some("Unavailable"),
        "version" => Some("Version 0.1.0"),
        "Silent" => Some("Silent"),
        "Balanced" => Some("Balanced"),
        "Performance" => Some("Performance"),
        "Custom" => Some("Custom"),
        "Battery Saver" => Some("Battery"),
        "Hyperboost" => Some("Hyper"),
        "Auto" => Some("Auto"),
        "Manual" => Some("Manual"),
        "Low" => Some("Low"),
        "Medium" => Some("Medium"),
        "High" => Some("High"),
        "Boost" => Some("Boost"),
        "Overclock" => Some("Overclock"),
        "Off" => Some("Off"),
        "Static" => Some("Static"),
        "Breathing" => Some("Breathe"),
        "Max" => Some("Max"),
        "CPU" => Some("CPU"),
        "GPU" => Some("GPU"),
        _ => None,
    }
}

fn chinese(key: &str) -> Option<&'static str> {
    match key {
        "app.name" => Some(APP_NAME),
        "action.apply" => Some("应用"),
        "action.connect" => Some("连接"),
        "action.refresh" => Some("刷新"),
        "action.exit" => Some("退出"),
        "tab.performance" => Some("性能"),
        "tab.lighting" => Some("灯光"),
        "tab.power" => Some("电源"),
        "section.mode" => Some("模式"),
        "section.fans" => Some("风扇"),
        "section.temperatures" => Some("温度"),
        "section.boost" => Some("CPU / GPU 加速"),
        "section.logo" => Some("Logo 灯"),
        "section.battery" => Some("电池"),
        "value.monitoring" => Some("监控中"),
        "value.keyboard" => Some("键盘"),
        "value.charge_limit" => Some("充电上限"),
        "value.custom_required" => Some("需要自定义模式"),
        "detail.quiet" => Some("安静"),
        "detail.daily" => Some("日常"),
        "detail.fast" => Some("高速"),
        "detail.tuned" => Some("调校"),
        "detail.saver" => Some("省电"),
        "detail.boost" => Some("增强"),
        "detail.curve" => Some("曲线"),
        "detail.rpm" => Some("转速"),
        "metric.cpu_fan" => Some("CPU 风扇"),
        "metric.gpu_fan" => Some("GPU 风扇"),
        "metric.battery_limit" => Some("电池充电上限"),
        "metric.keyboard_brightness" => Some("键盘亮度"),
        "status.connecting" => Some("正在连接 Razer 设备..."),
        "status.disconnected" => Some("未连接"),
        "status.ready" => Some("就绪"),
        "status.no_device" => Some("未发现设备"),
        "error.no_device_connected" => Some("未连接 Razer 设备。"),
        "error.state_read_failed" => Some("读取状态失败"),
        "label.unsupported" => Some("不支持"),
        "label.unavailable" => Some("不可用"),
        "version" => Some("版本 0.1.0"),
        "Silent" => Some("静音"),
        "Balanced" => Some("均衡"),
        "Performance" => Some("性能"),
        "Custom" => Some("自定义"),
        "Battery Saver" => Some("省电"),
        "Hyperboost" => Some("超频"),
        "Auto" => Some("自动"),
        "Manual" => Some("手动"),
        "Low" => Some("低"),
        "Medium" => Some("中"),
        "High" => Some("高"),
        "Boost" => Some("加速"),
        "Overclock" => Some("超频"),
        "Off" => Some("关闭"),
        "Static" => Some("常亮"),
        "Breathing" => Some("呼吸"),
        "Max" => Some("最大"),
        "CPU" => Some("CPU"),
        "GPU" => Some("GPU"),
        _ => None,
    }
}
