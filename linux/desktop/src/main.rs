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

    let dev_grid = GtkBox::new(Orientation::Vertical, 16);

    // Initial placeholder when waiting for connection
    let empty_box = GtkBox::new(Orientation::Vertical, 8);
    empty_box.add_css_class("connect-cta-box");
    empty_box.set_halign(Align::Fill);

    let empty_title = Label::new(Some("📡  Waiting for Mobile Connection"));
    empty_title.add_css_class("connect-cta-title");
    empty_title.set_xalign(0.0);

    let empty_desc = Label::new(Some("Open NOVA-Link on your Android phone, tap 'Direct IP', and enter this computer's Tailscale or local IP (Port 42424)."));
    empty_desc.add_css_class("connect-cta-desc");
    empty_desc.set_xalign(0.0);
    empty_desc.set_wrap(true);

    empty_box.append(&empty_title);
    empty_box.append(&empty_desc);
    dev_grid.append(&empty_box);
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

    // Refresh button → query daemon for live device list and update device cards
    {
        let badge = scan_badge.clone();
        let grid = dev_grid.clone();
        let w = window.clone();
        let ipc = ctx.client.clone();
        refresh_btn.connect_clicked(move |_| {
            let badge2 = badge.clone();
            let grid2 = grid.clone();
            let w2 = w.clone();
            let ipc2 = ipc.clone();
            badge2.set_text("● Scanning...");
            glib::spawn_future_local(async move {
                match ipc2.send_command(IpcCommand::ListDevices).await {
                    Ok(resp) => {
                        tracing::info!("Device refresh: {}", resp);
                        let devs_val = serde_json::from_str::<serde_json::Value>(&resp).ok();
                        let dev_array = devs_val.as_ref().and_then(|v| {
                            if let Some(arr) = v.as_array() {
                                Some(arr.clone())
                            } else if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                                Some(arr.clone())
                            } else {
                                None
                            }
                        }).unwrap_or_default();

                        // Update device count badge
                        if dev_array.is_empty() {
                            badge2.set_text("● Ready for mobile connection");
                        } else {
                            badge2.set_text(&format!("● {} device(s) active", dev_array.len()));
                        }

                        // Rebuild dev_grid with real devices
                        while let Some(child) = grid2.first_child() {
                            grid2.remove(&child);
                        }

                        if dev_array.is_empty() {
                            let empty_box = GtkBox::new(Orientation::Vertical, 8);
                            empty_box.add_css_class("connect-cta-box");
                            empty_box.set_halign(Align::Fill);

                            let empty_title = Label::new(Some("📡  Waiting for Mobile Connection"));
                            empty_title.add_css_class("connect-cta-title");
                            empty_title.set_xalign(0.0);

                            let empty_desc = Label::new(Some("Open NOVA-Link on your Android phone, tap 'Direct IP', and enter this computer's Tailscale or local IP (Port 42424)."));
                            empty_desc.add_css_class("connect-cta-desc");
                            empty_desc.set_xalign(0.0);
                            empty_desc.set_wrap(true);

                            empty_box.append(&empty_title);
                            empty_box.append(&empty_desc);
                            grid2.append(&empty_box);
                        } else {
                            let cards_box = GtkBox::new(Orientation::Horizontal, 16);
                            cards_box.set_homogeneous(true);

                            for dev in &dev_array {
                                let dev_name = dev.get("device_name").and_then(|v| v.as_str()).unwrap_or("Android Phone");
                                let dev_type = dev.get("device_type").and_then(|v| v.as_str()).unwrap_or("android");
                                let ip_str = dev.get("ip_addresses")
                                    .and_then(|v| v.as_array())
                                    .and_then(|arr| arr.first())
                                    .and_then(|ip| ip.as_str())
                                    .unwrap_or("Connected");

                                let card = GtkBox::new(Orientation::Vertical, 14);
                                card.add_css_class("device-card-active");

                                let card_top = GtkBox::new(Orientation::Horizontal, 10);
                                let icon = Label::new(Some(if dev_type == "android" { "📱" } else { "💻" }));
                                icon.set_markup(if dev_type == "android" { "<span size='x-large'>📱</span>" } else { "<span size='x-large'>💻</span>" });
                                card_top.append(&icon);

                                let info_box = GtkBox::new(Orientation::Vertical, 2);
                                info_box.set_hexpand(true);
                                let name_lbl = Label::new(Some(dev_name));
                                name_lbl.add_css_class("device-name");
                                name_lbl.set_xalign(0.0);

                                let badge_row = GtkBox::new(Orientation::Horizontal, 6);
                                let badge_connected = Label::new(Some("CONNECTED"));
                                badge_connected.add_css_class("badge-connected");
                                let ip_lbl = Label::new(Some(&format!("IP: {}", ip_str)));
                                ip_lbl.add_css_class("device-subtext");
                                badge_row.append(&badge_connected);
                                badge_row.append(&ip_lbl);

                                info_box.append(&name_lbl);
                                info_box.append(&badge_row);
                                card_top.append(&info_box);
                                card.append(&card_top);

                                let actions_box = GtkBox::new(Orientation::Horizontal, 8);
                                actions_box.set_homogeneous(true);

                                let btn_br = Button::with_label("📁  Browse");
                                btn_br.add_css_class("device-action-button");
                                let w_cl = w2.clone();
                                let dname = dev_name.to_string();
                                btn_br.connect_clicked(move |_| {
                                    let dialog = gtk4::MessageDialog::builder()
                                        .transient_for(&w_cl)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Info)
                                        .buttons(gtk4::ButtonsType::Close)
                                        .text(&format!("Browse {}", dname))
                                        .secondary_text("File transfer & browser ready.")
                                        .build();
                                    dialog.connect_response(|d, _| d.close());
                                    dialog.present();
                                });

                                let btn_mr = Button::with_label("💻  Mirror");
                                btn_mr.add_css_class("device-action-button");
                                let w_cl2 = w2.clone();
                                let dname2 = dev_name.to_string();
                                btn_mr.connect_clicked(move |_| {
                                    let dialog = gtk4::MessageDialog::builder()
                                        .transient_for(&w_cl2)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Info)
                                        .buttons(gtk4::ButtonsType::Close)
                                        .text(&format!("Screen Mirror - {}", dname2))
                                        .secondary_text("Screen mirroring session ready.")
                                        .build();
                                    dialog.connect_response(|d, _| d.close());
                                    dialog.present();
                                });

                                actions_box.append(&btn_br);
                                actions_box.append(&btn_mr);
                                card.append(&actions_box);

                                cards_box.append(&card);
                            }
                            grid2.append(&cards_box);
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
        let grid = dev_grid.clone();
        let w = window.clone();
        let ipc = ctx.client.clone();

        let check_status = move || {
            let banner2 = banner.clone();
            let badge2 = badge.clone();
            let net2 = net.clone();
            let grid2 = grid.clone();
            let w2 = w.clone();
            let ipc2 = ipc.clone();

            glib::spawn_future_local(async move {
                match ipc2.send_command(IpcCommand::GetStatus).await {
                    Ok(_) => {
                        banner2.set_visible(false);
                        net2.set_text("📶 Local Network: Daemon Active (Port 42424)");
                        // Query device list and update cards
                        if let Ok(dev_resp) = ipc2.send_command(IpcCommand::ListDevices).await {
                            let devs_val = serde_json::from_str::<serde_json::Value>(&dev_resp).ok();
                            let dev_array = devs_val.as_ref().and_then(|v| {
                                if let Some(arr) = v.as_array() {
                                    Some(arr.clone())
                                } else if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                                    Some(arr.clone())
                                } else {
                                    None
                                }
                            }).unwrap_or_default();

                            if dev_array.is_empty() {
                                badge2.set_text("● Ready for mobile connection");
                            } else {
                                badge2.set_text(&format!("● {} device(s) active", dev_array.len()));

                                while let Some(child) = grid2.first_child() {
                                    grid2.remove(&child);
                                }

                                let cards_box = GtkBox::new(Orientation::Horizontal, 16);
                                cards_box.set_homogeneous(true);

                                for dev in &dev_array {
                                    let dev_name = dev.get("device_name").and_then(|v| v.as_str()).unwrap_or("Android Phone");
                                    let dev_type = dev.get("device_type").and_then(|v| v.as_str()).unwrap_or("android");
                                    let ip_str = dev.get("ip_addresses")
                                        .and_then(|v| v.as_array())
                                        .and_then(|arr| arr.first())
                                        .and_then(|ip| ip.as_str())
                                        .unwrap_or("Connected");

                                    let card = GtkBox::new(Orientation::Vertical, 14);
                                    card.add_css_class("device-card-active");

                                    let card_top = GtkBox::new(Orientation::Horizontal, 10);
                                    let icon = Label::new(Some(if dev_type == "android" { "📱" } else { "💻" }));
                                    icon.set_markup(if dev_type == "android" { "<span size='x-large'>📱</span>" } else { "<span size='x-large'>💻</span>" });
                                    card_top.append(&icon);

                                    let info_box = GtkBox::new(Orientation::Vertical, 2);
                                    info_box.set_hexpand(true);
                                    let name_lbl = Label::new(Some(dev_name));
                                    name_lbl.add_css_class("device-name");
                                    name_lbl.set_xalign(0.0);

                                    let badge_row = GtkBox::new(Orientation::Horizontal, 6);
                                    let badge_connected = Label::new(Some("CONNECTED"));
                                    badge_connected.add_css_class("badge-connected");
                                    let ip_lbl = Label::new(Some(&format!("IP: {}", ip_str)));
                                    ip_lbl.add_css_class("device-subtext");
                                    badge_row.append(&badge_connected);
                                    badge_row.append(&ip_lbl);

                                    info_box.append(&name_lbl);
                                    info_box.append(&badge_row);
                                    card_top.append(&info_box);
                                    card.append(&card_top);

                                    let actions_box = GtkBox::new(Orientation::Horizontal, 8);
                                    actions_box.set_homogeneous(true);

                                    let btn_br = Button::with_label("📁  Browse");
                                    btn_br.add_css_class("device-action-button");
                                    let w_cl = w2.clone();
                                    let dname = dev_name.to_string();
                                    btn_br.connect_clicked(move |_| {
                                        let dialog = gtk4::MessageDialog::builder()
                                            .transient_for(&w_cl)
                                            .modal(true)
                                            .message_type(gtk4::MessageType::Info)
                                            .buttons(gtk4::ButtonsType::Close)
                                            .text(&format!("Browse {}", dname))
                                            .secondary_text("File transfer & browser ready.")
                                            .build();
                                        dialog.connect_response(|d, _| d.close());
                                        dialog.present();
                                    });

                                    let btn_mr = Button::with_label("💻  Mirror");
                                    btn_mr.add_css_class("device-action-button");
                                    let w_cl2 = w2.clone();
                                    let dname2 = dev_name.to_string();
                                    btn_mr.connect_clicked(move |_| {
                                        let dialog = gtk4::MessageDialog::builder()
                                            .transient_for(&w_cl2)
                                            .modal(true)
                                            .message_type(gtk4::MessageType::Info)
                                            .buttons(gtk4::ButtonsType::Close)
                                            .text(&format!("Screen Mirror - {}", dname2))
                                            .secondary_text("Screen mirroring session ready.")
                                            .build();
                                        dialog.connect_response(|d, _| d.close());
                                        dialog.present();
                                    });

                                    actions_box.append(&btn_br);
                                    actions_box.append(&btn_mr);
                                    card.append(&actions_box);

                                    cards_box.append(&card);
                                }
                                grid2.append(&cards_box);
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

        // Periodic check every 3 seconds
        let ipc_timer = ctx.client.clone();
        let banner_timer = daemon_banner.clone();
        let badge_timer = scan_badge.clone();
        let net_timer = net_lbl.clone();
        let grid_timer = dev_grid.clone();
        let w_timer = window.clone();

        glib::timeout_add_seconds_local(3, move || {
            let banner2 = banner_timer.clone();
            let badge2 = badge_timer.clone();
            let net2 = net_timer.clone();
            let grid2 = grid_timer.clone();
            let w2 = w_timer.clone();
            let ipc2 = ipc_timer.clone();

            glib::spawn_future_local(async move {
                if let Ok(_) = ipc2.send_command(IpcCommand::GetStatus).await {
                    banner2.set_visible(false);
                    net2.set_text("📶 Local Network: Daemon Active (Port 42424)");
                    if let Ok(dev_resp) = ipc2.send_command(IpcCommand::ListDevices).await {
                        let devs_val = serde_json::from_str::<serde_json::Value>(&dev_resp).ok();
                        let dev_array = devs_val.as_ref().and_then(|v| {
                            if let Some(arr) = v.as_array() {
                                Some(arr.clone())
                            } else if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
                                Some(arr.clone())
                            } else {
                                None
                            }
                        }).unwrap_or_default();

                        if dev_array.is_empty() {
                            badge2.set_text("● Ready for mobile connection");
                        } else {
                            badge2.set_text(&format!("● {} device(s) active", dev_array.len()));

                            while let Some(child) = grid2.first_child() {
                                grid2.remove(&child);
                            }

                            let cards_box = GtkBox::new(Orientation::Horizontal, 16);
                            cards_box.set_homogeneous(true);

                            for dev in &dev_array {
                                let dev_name = dev.get("device_name").and_then(|v| v.as_str()).unwrap_or("Android Phone");
                                let dev_type = dev.get("device_type").and_then(|v| v.as_str()).unwrap_or("android");
                                let ip_str = dev.get("ip_addresses")
                                    .and_then(|v| v.as_array())
                                    .and_then(|arr| arr.first())
                                    .and_then(|ip| ip.as_str())
                                    .unwrap_or("Connected");

                                let card = GtkBox::new(Orientation::Vertical, 14);
                                card.add_css_class("device-card-active");

                                let card_top = GtkBox::new(Orientation::Horizontal, 10);
                                let icon = Label::new(Some(if dev_type == "android" { "📱" } else { "💻" }));
                                icon.set_markup(if dev_type == "android" { "<span size='x-large'>📱</span>" } else { "<span size='x-large'>💻</span>" });
                                card_top.append(&icon);

                                let info_box = GtkBox::new(Orientation::Vertical, 2);
                                info_box.set_hexpand(true);
                                let name_lbl = Label::new(Some(dev_name));
                                name_lbl.add_css_class("device-name");
                                name_lbl.set_xalign(0.0);

                                let badge_row = GtkBox::new(Orientation::Horizontal, 6);
                                let badge_connected = Label::new(Some("CONNECTED"));
                                badge_connected.add_css_class("badge-connected");
                                let ip_lbl = Label::new(Some(&format!("IP: {}", ip_str)));
                                ip_lbl.add_css_class("device-subtext");
                                badge_row.append(&badge_connected);
                                badge_row.append(&ip_lbl);

                                info_box.append(&name_lbl);
                                info_box.append(&badge_row);
                                card_top.append(&info_box);
                                card.append(&card_top);

                                let actions_box = GtkBox::new(Orientation::Horizontal, 8);
                                actions_box.set_homogeneous(true);

                                let btn_br = Button::with_label("📁  Browse");
                                btn_br.add_css_class("device-action-button");
                                let w_cl = w2.clone();
                                let dname = dev_name.to_string();
                                btn_br.connect_clicked(move |_| {
                                    let dialog = gtk4::MessageDialog::builder()
                                        .transient_for(&w_cl)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Info)
                                        .buttons(gtk4::ButtonsType::Close)
                                        .text(&format!("Browse {}", dname))
                                        .secondary_text("File transfer & browser ready.")
                                        .build();
                                    dialog.connect_response(|d, _| d.close());
                                    dialog.present();
                                });

                                let btn_mr = Button::with_label("💻  Mirror");
                                btn_mr.add_css_class("device-action-button");
                                let w_cl2 = w2.clone();
                                let dname2 = dev_name.to_string();
                                btn_mr.connect_clicked(move |_| {
                                    let dialog = gtk4::MessageDialog::builder()
                                        .transient_for(&w_cl2)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Info)
                                        .buttons(gtk4::ButtonsType::Close)
                                        .text(&format!("Screen Mirror - {}", dname2))
                                        .secondary_text("Screen mirroring session ready.")
                                        .build();
                                    dialog.connect_response(|d, _| d.close());
                                    dialog.present();
                                });

                                actions_box.append(&btn_br);
                                actions_box.append(&btn_mr);
                                card.append(&actions_box);

                                cards_box.append(&card);
                            }
                            grid2.append(&cards_box);
                        }
                    }
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
