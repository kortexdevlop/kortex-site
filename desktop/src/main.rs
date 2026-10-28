fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
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
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1360.0, 900.0)),
            ),
        )
        .launch(kortex_desktop::app::App);
}
