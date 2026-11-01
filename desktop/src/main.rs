use kortex_desktop::tool_window;
fn main() {
    let tool = tool_window::requested_now();
    if let (Some(id), Ok(config)) = (&tool, kortex_desktop::config::Config::load())
        && tool_window::focus_open(&config, id)
    {
        return;
    }
    let size = if tool.is_some() {
        dioxus::desktop::LogicalSize::new(1000.0, 760.0)
    } else {
        dioxus::desktop::LogicalSize::new(1360.0, 900.0)
    };
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_menu(None).with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Kortex")
                    .with_window_icon(
                        dioxus::desktop::tao::window::Icon::from_rgba(
                            include_bytes!("../assets/icon.rgba").to_vec(),
                            32,
                            32,
                        )
                        .ok(),
                    )
                    .with_inner_size(size),
            ),
        )
        .launch(if tool.is_some() {
            tool_window::ToolWindow
        } else {
            kortex_desktop::app::App
        });
}
