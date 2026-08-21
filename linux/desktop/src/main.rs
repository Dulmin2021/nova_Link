pub mod ipc_client;

use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use ipc_client::IpcClient;
use ipc_client::IpcCommand;

#[derive(Clone)]
pub struct AppContext {
    pub client: Arc<IpcClient>,
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("NOVA-Link Desktop Application initializing (GTK4 + Libadwaita)");

    let ctx = AppContext {
        client: Arc::new(IpcClient::new()),
    };

    #[cfg(feature = "gui")]
    {
        use libadwaita::Application;
        use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};

        let app = Application::builder()
            .application_id("com.novalink.NovaLink")
            .build();

        app.connect_activate(move |app| {
            build_ui(app, &ctx);
        });

        app.run();
    }

    #[cfg(not(feature = "gui"))]
    {
        info!("Running NOVA-Link Desktop Client in CLI status mode (use '--features gui' for native GTK4 window)");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            match ctx.client.send_command(IpcCommand::ListDevices).await {
                Ok(resp) => info!("Connected to active background daemon! Device response: {}", resp),
                Err(e) => {
                    tracing::warn!("Could not connect to nova-daemon IPC socket ({})", e);
                    info!("TIP: Start the background daemon first with: 'cargo run --bin nova-daemon'");
                }
            }
        });
    }
}

#[cfg(feature = "gui")]
fn build_ui(app: &libadwaita::Application, ctx: &AppContext) {
    use gtk4::prelude::*;
    use libadwaita::prelude::*;
    use glib;
    use gtk4::{
        Align, Box as GtkBox, Button, Entry, Label, Orientation,
        ScrolledWindow, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk::Display,
    };
    use libadwaita::ApplicationWindow;

    // Load custom dashboard CSS theme
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("NOVA-Link")
        .default_width(1050)
        .default_height(700)
        .build();

    let root_box = GtkBox::new(Orientation::Vertical, 0);

    // ------------------------------------------
    // 0. TOP DAEMON WARNING BANNER (State-Aware)
    // ------------------------------------------
    let daemon_banner = GtkBox::new(Orientation::Horizontal, 12);
    daemon_banner.add_css_class("daemon-warning-banner");
    daemon_banner.set_visible(false); // Hidden by default, shown if daemon offline

    let warn_icon = Label::new(Some("⚠"));
    let warn_msg = Label::new(Some("Background daemon (nova-daemon) is offline. Start it in terminal: 'cargo run --bin nova-daemon'"));
    warn_msg.add_css_class("daemon-warning-text");
    warn_msg.set_hexpand(true);
    warn_msg.set_xalign(0.0);

    let retry_btn = Button::with_label("🔄 Retry");
    retry_btn.add_css_class("device-action-button");

    daemon_banner.append(&warn_icon);
    daemon_banner.append(&warn_msg);
    daemon_banner.append(&retry_btn);
    root_box.append(&daemon_banner);

    // ==========================================
    // MAIN 2-COLUMN BODY (Sidebar + Dashboard)
    // ==========================================
    let body_box = GtkBox::new(Orientation::Horizontal, 0);
    body_box.set_vexpand(true);

    // ------------------------------------------
    // 1. LEFT SIDEBAR
    // ------------------------------------------
    let sidebar = GtkBox::new(Orientation::Vertical, 12);
    sidebar.add_css_class("sidebar-panel");
    sidebar.set_size_request(230, -1);

    // Brand Header (Logo + Title + Subtitle)
    let brand_box = GtkBox::new(Orientation::Horizontal, 8);
    brand_box.set_margin_bottom(18);

    let logo_lbl = Label::new(Some("⚡"));
    logo_lbl.set_markup("<span size='x-large'><b>⚡</b></span>");
    brand_box.append(&logo_lbl);

    let title_vbox = GtkBox::new(Orientation::Vertical, 0);
    let brand_title = Label::new(Some("NOVA-Link"));
    brand_title.add_css_class("brand-title");
    brand_title.set_xalign(0.0);

    let brand_sub = Label::new(Some("Local Network"));
    brand_sub.add_css_class("brand-subtitle");
    brand_sub.set_xalign(0.0);

    title_vbox.append(&brand_title);
    title_vbox.append(&brand_sub);
    brand_box.append(&title_vbox);
    sidebar.append(&brand_box);

    // Navigation Menu Items
    let nav_devices = Button::builder()
        .label("💻  Devices")
        .halign(Align::Fill)
        .build();
    nav_devices.add_css_class("sidebar-btn");
    nav_devices.add_css_class("sidebar-btn-active");
    sidebar.append(&nav_devices);

    let nav_files = Button::builder()
        .label("📁  Shared Files")
        .halign(Align::Fill)
        .build();
    nav_files.add_css_class("sidebar-btn");
    sidebar.append(&nav_files);

    let nav_activity = Button::builder()
        .label("🕒  Activity")
        .halign(Align::Fill)
        .build();
    nav_activity.add_css_class("sidebar-btn");
    sidebar.append(&nav_activity);

    // Spacer between nav and bottom actions
    let sidebar_spacer = GtkBox::new(Orientation::Vertical, 0);
    sidebar_spacer.set_vexpand(true);
    sidebar.append(&sidebar_spacer);

    let nav_settings = Button::builder()
        .label("⚙  Settings")
        .halign(Align::Fill)
        .build();
    nav_settings.add_css_class("sidebar-btn");
    sidebar.append(&nav_settings);

    let pair_btn = Button::builder()
        .label("+ Pair New Device")
        .halign(Align::Fill)
        .build();
    pair_btn.add_css_class("pair-new-btn");
    sidebar.append(&pair_btn);

    body_box.append(&sidebar);

    // ------------------------------------------
    // 2. RIGHT MAIN DASHBOARD
    // ------------------------------------------
    let main_area = GtkBox::new(Orientation::Vertical, 0);
    main_area.set_hexpand(true);

    // Top Header Bar
    let top_header = GtkBox::new(Orientation::Horizontal, 12);
    top_header.set_margin_top(16);
    top_header.set_margin_bottom(16);
    top_header.set_margin_start(28);
    top_header.set_margin_end(28);

    let dashboard_lbl = Label::new(Some("Dashboard"));
    dashboard_lbl.add_css_class("dashboard-title");
    dashboard_lbl.set_xalign(0.0);
    dashboard_lbl.set_hexpand(true);
    top_header.append(&dashboard_lbl);

    let search_entry = Entry::builder()
        .placeholder_text("🔍  Search devices...")
        .width_chars(25)
        .build();
    search_entry.add_css_class("search-entry");
    top_header.append(&search_entry);

    let refresh_btn = Button::with_label("🔄");
    refresh_btn.add_css_class("sidebar-btn");
    top_header.append(&refresh_btn);

    let battery_btn = Button::with_label("🔋");
    battery_btn.add_css_class("sidebar-btn");
    top_header.append(&battery_btn);

    let menu_btn = Button::with_label("⋮");
    menu_btn.add_css_class("sidebar-btn");
    top_header.append(&menu_btn);

    main_area.append(&top_header);

    // Separator line
    let sep = gtk4::Separator::new(Orientation::Horizontal);
    main_area.append(&sep);

    // Dashboard Content Scrollable View
    let scrolled = ScrolledWindow::new();
    scrolled.set_vexpand(true);

    let content_box = GtkBox::new(Orientation::Vertical, 24);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(28);
    content_box.set_margin_end(28);

    // ------------------------------------------
    // Section A: Quick Actions
    // ------------------------------------------
    let qa_title = Label::new(Some("Quick Actions"));
    qa_title.add_css_class("section-title");
    qa_title.set_xalign(0.0);
    content_box.append(&qa_title);

    let qa_grid = GtkBox::new(Orientation::Horizontal, 16);
    qa_grid.set_homogeneous(true);

    // 1. Send File (Deep Blue Primary Card)
    let card_file = Button::new();
    card_file.add_css_class("quick-action-primary");
    let card_file_box = GtkBox::new(Orientation::Vertical, 6);
    card_file_box.set_valign(Align::Center);
    card_file_box.set_halign(Align::Center);

    let icon_file = Label::new(Some("📤"));
    icon_file.set_markup("<span size='xx-large'>📤</span>");
    let lbl_file = Label::new(Some("Send File"));
    lbl_file.add_css_class("action-text-primary");

    card_file_box.append(&icon_file);
    card_file_box.append(&lbl_file);
    card_file.set_child(Some(&card_file_box));
    qa_grid.append(&card_file);

    // 2. Send Text (Pale Blue Secondary Card)
    let card_text = Button::new();
    card_text.add_css_class("quick-action-secondary");
    let card_text_box = GtkBox::new(Orientation::Vertical, 6);
    card_text_box.set_valign(Align::Center);
    card_text_box.set_halign(Align::Center);

    let icon_text = Label::new(Some("💬"));
    icon_text.set_markup("<span size='xx-large'>💬</span>");
    let lbl_text = Label::new(Some("Send Text"));
    lbl_text.add_css_class("action-text-secondary");

    card_text_box.append(&icon_text);
    card_text_box.append(&lbl_text);
    card_text.set_child(Some(&card_text_box));
    qa_grid.append(&card_text);

    // 3. Send URL (Pale Wheat Secondary Card)
    let card_url = Button::new();
    card_url.add_css_class("quick-action-secondary");
    let card_url_box = GtkBox::new(Orientation::Vertical, 6);
    card_url_box.set_valign(Align::Center);
    card_url_box.set_halign(Align::Center);

    let icon_url = Label::new(Some("🔗"));
    icon_url.set_markup("<span size='xx-large'>🔗</span>");
    let lbl_url = Label::new(Some("Send URL"));
    lbl_url.add_css_class("action-text-secondary");

    card_url_box.append(&icon_url);
    card_url_box.append(&lbl_url);
    card_url.set_child(Some(&card_url_box));
    qa_grid.append(&card_url);

    content_box.append(&qa_grid);

    // ------------------------------------------
    // Section B: Nearby Devices
    // ------------------------------------------
    let dev_header_box = GtkBox::new(Orientation::Horizontal, 12);
    dev_header_box.set_margin_top(8);

    let dev_title = Label::new(Some("Nearby Devices"));
    dev_title.add_css_class("section-title");
    dev_title.set_xalign(0.0);
    dev_title.set_hexpand(true);
    dev_header_box.append(&dev_title);

    let scan_badge = Label::new(Some("● Scanning active"));
    scan_badge.add_css_class("scanning-badge");
    dev_header_box.append(&scan_badge);
    content_box.append(&dev_header_box);

    let dev_grid = GtkBox::new(Orientation::Horizontal, 16);
    dev_grid.set_homogeneous(true);

    // Device Card 1: Pixel 8 (Connected)
    let card_pixel = GtkBox::new(Orientation::Vertical, 14);
    card_pixel.add_css_class("device-card-active");

    let card_pixel_top = GtkBox::new(Orientation::Horizontal, 10);
    let icon_pixel = Label::new(Some("📱"));
    icon_pixel.set_markup("<span size='x-large'>📱</span>");
    card_pixel_top.append(&icon_pixel);

    let info_pixel = GtkBox::new(Orientation::Vertical, 2);
    info_pixel.set_hexpand(true);
    let name_pixel = Label::new(Some("Pixel 8"));
    name_pixel.add_css_class("device-name");
    name_pixel.set_xalign(0.0);

    let badge_row = GtkBox::new(Orientation::Horizontal, 6);
    let badge_connected = Label::new(Some("CONNECTED"));
    badge_connected.add_css_class("badge-connected");
    let battery_lbl = Label::new(Some("🔋 85%"));
    battery_lbl.add_css_class("device-subtext");
    badge_row.append(&badge_connected);
    badge_row.append(&battery_lbl);

    info_pixel.append(&name_pixel);
    info_pixel.append(&badge_row);
    card_pixel_top.append(&info_pixel);

    let menu_pixel = Button::with_label("⋮");
    menu_pixel.add_css_class("sidebar-btn");
    card_pixel_top.append(&menu_pixel);
    card_pixel.append(&card_pixel_top);

    // Action buttons inside card: Browse & Mirror
    let card_pixel_actions = GtkBox::new(Orientation::Horizontal, 8);
    card_pixel_actions.set_homogeneous(true);

    let btn_browse = Button::with_label("📁  Browse");
    btn_browse.add_css_class("device-action-button");
    let btn_mirror = Button::with_label("💻  Mirror");
    btn_mirror.add_css_class("device-action-button");

    card_pixel_actions.append(&btn_browse);
    card_pixel_actions.append(&btn_mirror);
    card_pixel.append(&card_pixel_actions);

    dev_grid.append(&card_pixel);

    // Device Card 2: Samsung Galaxy (Offline)
    let card_galaxy = GtkBox::new(Orientation::Vertical, 14);
    card_galaxy.add_css_class("device-card-offline");

    let card_galaxy_top = GtkBox::new(Orientation::Horizontal, 10);
    let icon_galaxy = Label::new(Some("📱"));
    icon_galaxy.set_markup("<span size='x-large'>📱</span>");
    card_galaxy_top.append(&icon_galaxy);

    let info_galaxy = GtkBox::new(Orientation::Vertical, 2);
    info_galaxy.set_hexpand(true);
    let name_galaxy = Label::new(Some("Samsung Galaxy"));
    name_galaxy.add_css_class("device-name");
    name_galaxy.set_xalign(0.0);

    let badge_row2 = GtkBox::new(Orientation::Horizontal, 6);
    let badge_offline = Label::new(Some("OFFLINE"));
    badge_offline.add_css_class("badge-offline");
    let seen_lbl = Label::new(Some("Last seen: 2 hrs ago"));
    seen_lbl.add_css_class("device-subtext");
    badge_row2.append(&badge_offline);
    badge_row2.append(&seen_lbl);

    info_galaxy.append(&name_galaxy);
    info_galaxy.append(&badge_row2);
    card_galaxy_top.append(&info_galaxy);

    let info_icon = Button::with_label("ⓘ");
    info_icon.add_css_class("sidebar-btn");
    card_galaxy_top.append(&info_icon);
    card_galaxy.append(&card_galaxy_top);

    dev_grid.append(&card_galaxy);
    content_box.append(&dev_grid);

    scrolled.set_child(Some(&content_box));
    main_area.append(&scrolled);

    body_box.append(&main_area);
    root_box.append(&body_box);

    // ------------------------------------------
    // 3. BOTTOM TECHNICAL STATUS BAR
    // ------------------------------------------
    let status_bar = GtkBox::new(Orientation::Horizontal, 12);
    status_bar.add_css_class("bottom-status-bar");

    let net_lbl = Label::new(Some("📶 Local Network: Connected"));
    net_lbl.add_css_class("status-network-active");
    net_lbl.set_xalign(0.0);
    net_lbl.set_hexpand(true);

    let meta_lbl = Label::new(Some("NOVA-Link v2.4.1  |  End-to-End Encrypted"));
    meta_lbl.set_xalign(1.0);

    status_bar.append(&net_lbl);
    status_bar.append(&meta_lbl);
    root_box.append(&status_bar);

    window.set_content(Some(&root_box));
    window.present();

    // ==========================================
    // BUTTON CLICK HANDLERS
    // ==========================================

    // Pair New Device button → show IP connect dialog and attempt IPC connection
    {
        let w = window.clone();
        let ipc = ctx.client.clone();
        pair_btn.connect_clicked(move |_| {
            let dialog = gtk4::Dialog::builder()
                .title("Pair New Device")
                .transient_for(&w)
                .modal(true)
                .build();

            let content = dialog.content_area();
            let vbox = GtkBox::new(Orientation::Vertical, 12);
            vbox.set_margin_top(16);
            vbox.set_margin_bottom(16);
            vbox.set_margin_start(20);
            vbox.set_margin_end(20);

            let label = Label::new(Some("Enter Android device IP or Tailscale IP:"));
            label.set_xalign(0.0);
            vbox.append(&label);

            let entry = Entry::builder()
                .placeholder_text("e.g. 100.x.x.x or 192.168.1.x")
                .build();
            vbox.append(&entry);

            let status_label = Label::new(Some(""));
            status_label.add_css_class("device-subtext");
            status_label.set_xalign(0.0);
            vbox.append(&status_label);

            let connect_btn = Button::builder()
                .label("Connect")
                .build();
            connect_btn.add_css_class("pair-new-btn");
            vbox.append(&connect_btn);

            content.append(&vbox);

            let entry_clone = entry.clone();
            let dialog_clone = dialog.clone();
            let status_clone = status_label.clone();
            let ipc_clone = ipc.clone();
            connect_btn.connect_clicked(move |_| {
                let ip = entry_clone.text().to_string();
                if !ip.is_empty() {
                    status_clone.set_text("⏳ Connecting...");
                    let ipc2 = ipc_clone.clone();
                    let status2 = status_clone.clone();
                    let dialog2 = dialog_clone.clone();
                    // Use glib::spawn_future to run async IPC without blocking the GTK main loop
                    glib::spawn_future_local(async move {
                        match ipc2.send_command(IpcCommand::GetStatus).await {
                            Ok(resp) => {
                                tracing::info!("IPC daemon status (for IP {}): {}", ip, resp);
                                status2.set_text("✓ Daemon reachable — use Android app to pair");
                                glib::timeout_add_seconds_local(2, move || {
                                    dialog2.close();
                                    glib::ControlFlow::Break
                                });
                            }
                            Err(e) => {
                                status2.set_text(&format!("⚠ Daemon not reachable: {}", e));
                                tracing::warn!("Pair dialog IPC error: {}", e);
                            }
                        }
                    });
                }
            });

            dialog.present();
        });
    }

    // Send File button
    {
        let w = window.clone();
        card_file.connect_clicked(move |_| {
            let chooser = gtk4::FileChooserDialog::new(
                Some("Select File to Send"),
                Some(&w),
                gtk4::FileChooserAction::Open,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Send", gtk4::ResponseType::Accept),
                ],
            );
            chooser.connect_response(move |d, resp| {
                if resp == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            tracing::info!("Sending file: {:?}", path);
                        }
                    }
                }
                d.close();
            });
            chooser.present();
        });
    }

    // Send Text button → text input dialog
    {
        let w = window.clone();
        card_text.connect_clicked(move |_| {
            let dialog = gtk4::Dialog::builder()
                .title("Send Text")
                .transient_for(&w)
                .modal(true)
                .build();

            let content = dialog.content_area();
            let vbox = GtkBox::new(Orientation::Vertical, 12);
            vbox.set_margin_top(16);
            vbox.set_margin_bottom(16);
            vbox.set_margin_start(20);
            vbox.set_margin_end(20);

            let label = Label::new(Some("Enter text to send to your Android device:"));
            label.set_xalign(0.0);
            vbox.append(&label);

            let entry = gtk4::TextView::new();
            entry.set_size_request(-1, 80);
            vbox.append(&entry);

            let send_btn = Button::builder().label("Send").build();
            send_btn.add_css_class("pair-new-btn");
            vbox.append(&send_btn);

            content.append(&vbox);

            let buf = entry.buffer();
            let dialog_clone = dialog.clone();
            send_btn.connect_clicked(move |_| {
                let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                tracing::info!("Sending text: {}", text);
                dialog_clone.close();
            });

            dialog.present();
        });
    }

    // Send URL button → URL input dialog
    {
        let w = window.clone();
        card_url.connect_clicked(move |_| {
            let dialog = gtk4::Dialog::builder()
                .title("Send URL")
                .transient_for(&w)
                .modal(true)
                .build();

            let content = dialog.content_area();
            let vbox = GtkBox::new(Orientation::Vertical, 12);
            vbox.set_margin_top(16);
            vbox.set_margin_bottom(16);
            vbox.set_margin_start(20);
            vbox.set_margin_end(20);

            let label = Label::new(Some("Paste URL to share with your Android device:"));
            label.set_xalign(0.0);
            vbox.append(&label);

            let entry = Entry::builder()
                .placeholder_text("https://...")
                .build();
            vbox.append(&entry);

            let send_btn = Button::builder().label("Share URL").build();
            send_btn.add_css_class("pair-new-btn");
            vbox.append(&send_btn);

            content.append(&vbox);

            let entry_clone = entry.clone();
            let dialog_clone = dialog.clone();
            send_btn.connect_clicked(move |_| {
                let url = entry_clone.text();
                tracing::info!("Sharing URL: {}", url);
                dialog_clone.close();
            });

            dialog.present();
        });
    }

    // Refresh button → query daemon for live device list
    {
        let badge = scan_badge.clone();
        let ipc = ctx.client.clone();
        refresh_btn.connect_clicked(move |_| {
            let badge2 = badge.clone();
            let ipc2 = ipc.clone();
            badge2.set_text("● Scanning...");
            glib::spawn_future_local(async move {
                match ipc2.send_command(IpcCommand::ListDevices).await {
                    Ok(resp) => {
                        tracing::info!("Device refresh: {}", resp);
                        // Count devices in JSON array response
                        let count = serde_json::from_str::<serde_json::Value>(&resp)
                            .ok()
                            .and_then(|v| v.as_array().map(|a| a.len()))
                            .unwrap_or(0);
                        if count > 0 {
                            badge2.set_text(&format!("● {} device(s) found", count));
                        } else {
                            badge2.set_text("● No devices — start nova-daemon");
                        }
                    }
                    Err(e) => {
                        badge2.set_text("⚠ Daemon offline");
                        tracing::warn!("Refresh IPC error: {}", e);
                    }
                }
            });
        });
    }

    // Browse button → open file chooser for browsing connected device
    {
        let w = window.clone();
        btn_browse.connect_clicked(move |_| {
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&w)
                .modal(true)
                .message_type(gtk4::MessageType::Info)
                .buttons(gtk4::ButtonsType::Close)
                .text("Browse Device Files")
                .secondary_text("File browser for connected Android device will open here once a pairing is established.")
                .build();
            dialog.connect_response(|d, _| d.close());
            dialog.present();
        });
    }

    // Mirror button
    {
        let w = window.clone();
        btn_mirror.connect_clicked(move |_| {
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&w)
                .modal(true)
                .message_type(gtk4::MessageType::Info)
                .buttons(gtk4::ButtonsType::Close)
                .text("Screen Mirror")
                .secondary_text("Screen mirroring session will be initiated once a paired Android device is connected.")
                .build();
            dialog.connect_response(|d, _| d.close());
            dialog.present();
        });
    }

    // Nav: Shared Files
    {
        let w = window.clone();
        nav_files.connect_clicked(move |_| {
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&w)
                .modal(true)
                .message_type(gtk4::MessageType::Info)
                .buttons(gtk4::ButtonsType::Close)
                .text("Shared Files")
                .secondary_text("All files transferred between your Android device and this Linux computer will appear here.")
                .build();
            dialog.connect_response(|d, _| d.close());
            dialog.present();
        });
    }

    // Nav: Activity
    {
        let w = window.clone();
        nav_activity.connect_clicked(move |_| {
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&w)
                .modal(true)
                .message_type(gtk4::MessageType::Info)
                .buttons(gtk4::ButtonsType::Close)
                .text("Activity Log")
                .secondary_text("Connection events, pairing history, and file transfer logs will appear here.")
                .build();
            dialog.connect_response(|d, _| d.close());
            dialog.present();
        });
    }

    // Nav: Settings
    {
        let w = window.clone();
        nav_settings.connect_clicked(move |_| {
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&w)
                .modal(true)
                .message_type(gtk4::MessageType::Info)
                .buttons(gtk4::ButtonsType::Close)
                .text("Settings")
                .secondary_text("Configure clipboard sync, file download path, pairing trust, and security options here.")
                .build();
            dialog.connect_response(|d, _| d.close());
            dialog.present();
        });
    }

    // ==========================================
    // BACKGROUND DAEMON STATUS POLLING (State-Aware)
    // ==========================================
    {
        let banner = daemon_banner.clone();
        let badge = scan_badge.clone();
        let net = net_lbl.clone();
        let ipc = ctx.client.clone();

        let check_status = move || {
            let banner2 = banner.clone();
            let badge2 = badge.clone();
            let net2 = net.clone();
            let ipc2 = ipc.clone();

            glib::spawn_future_local(async move {
                match ipc2.send_command(IpcCommand::GetStatus).await {
                    Ok(_) => {
                        banner2.set_visible(false);
                        net2.set_text("📶 Local Network: Daemon Active (Port 42424)");
                        // Check devices
                        if let Ok(dev_resp) = ipc2.send_command(IpcCommand::ListDevices).await {
                            let count = serde_json::from_str::<serde_json::Value>(&dev_resp)
                                .ok()
                                .and_then(|v| v.as_array().map(|a| a.len()))
                                .unwrap_or(0);
                            if count > 0 {
                                badge2.set_text(&format!("● {} device(s) connected", count));
                            } else {
                                badge2.set_text("● Ready for mobile connection");
                            }
                        }
                    }
                    Err(_) => {
                        banner2.set_visible(true);
                        badge2.set_text("⚠ Daemon offline");
                        net2.set_text("⚠ Daemon Offline — start with 'cargo run --bin nova-daemon'");
                    }
                }
            });
        };

        // Check immediately on startup
        check_status();

        // Retry button click
        retry_btn.connect_clicked(move |_| {
            check_status();
        });

        // Periodic check every 4 seconds
        let ipc_timer = ctx.client.clone();
        let banner_timer = daemon_banner.clone();
        let badge_timer = scan_badge.clone();
        let net_timer = net_lbl.clone();

        glib::timeout_add_seconds_local(4, move || {
            let banner2 = banner_timer.clone();
            let badge2 = badge_timer.clone();
            let net2 = net_timer.clone();
            let ipc2 = ipc_timer.clone();

            glib::spawn_future_local(async move {
                if let Ok(_) = ipc2.send_command(IpcCommand::GetStatus).await {
                    banner2.set_visible(false);
                    net2.set_text("📶 Local Network: Daemon Active (Port 42424)");
                } else {
                    banner2.set_visible(true);
                    badge2.set_text("⚠ Daemon offline");
                    net2.set_text("⚠ Daemon Offline — start with 'cargo run --bin nova-daemon'");
                }
            });

            glib::ControlFlow::Continue
        });
    }
}
